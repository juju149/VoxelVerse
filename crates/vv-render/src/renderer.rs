// engine renderer

use crate::atmosphere::AtmosphereConfig;
use crate::lod_animation::LodAnimator;
use crate::quality::QualitySettings;
use crate::render_pipeline_desc::PipelineId;
use crate::render_pipeline_factory::{create_post_bind_group, PipelineBindGroupLayouts};
use crate::render_pipeline_registry::RenderPipelineRegistry;
use crate::types::ChunkMesh;
use crate::world_streaming::{StreamingView, WorldStreamingConfig};
use bytemuck::{Pod, Zeroable};
use glyphon::{FontSystem, SwashCache, TextAtlas, TextRenderer as GlyphRenderer};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender};
use vv_diagnostics::FrameStats;
use vv_math::Frustum;
use vv_meshing::{CpuMesh, VoxelMeshingConfig};
use vv_meshing::{MeshScheduler, SchedulerStats};
use vv_voxel::{LodKey, SurfaceChunkKey, VoxelCoord};
use vv_world::PlanetData;
use winit::window::Window;

use gpu_scene::GpuScene;

// --- UNIFORMS ---

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GlobalUniform {
    pub view_proj: [f32; 16],        // bytes   0–63
    pub light_view_proj: [f32; 16],  // bytes  64–127
    pub cam_pos: [f32; 4],           // bytes 128–143  (w = quality_bits)
    pub sun_dir: [f32; 4],           // bytes 144–159  (w = fog_density)
    pub sky_horizon: [f32; 4], // bytes 160–175  (rgb = horizon sky color, w = time_of_day 0-1)
    pub sky_zenith: [f32; 4],  // bytes 176–191  (rgb = zenith sky color, w = sun_intensity 0-1)
    pub render_params: [f32; 4], // bytes 192-207 (x=time, y=quality bits, zw=viewport)
    pub atmosphere_params: [f32; 4], // bytes 208-223 (x=fog, y=height fog, z=vol fog, w=exposure)
    pub cloud_params: [f32; 4], // bytes 224-239 (x=steps, y=density, z=speed, w=coverage)
    pub water_params: [f32; 4], // bytes 240-255 (x=fresnel, y=specular, z=alpha, w=reserved)
    pub weather_params: [f32; 4], // bytes 256-271 (x=precip_intensity, y=wind_dir_x, z=wind_dir_z, w=precip_kind)
    pub celestial_params: [f32; 4], // bytes 272-287 (x=eclipse, y=stars_visibility, z=aurora_intensity, w=sun_angular_radius)
    pub celestial_moon: [f32; 4], // bytes 288-303 (xyz=moon dir, w=moon angular radius; 0 if no moon)
}

/// Compile-time guard: buffer sizes in setup.rs / render_resources.rs use
/// `std::mem::size_of::<GlobalUniform>()` so they stay in sync automatically.
/// If you ever change GlobalUniform, this assertion documents the expected size.
const _: () = assert!(
    std::mem::size_of::<GlobalUniform>() == 304,
    "GlobalUniform size changed — verify all uniform buffer sizes stay consistent"
);

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct LocalUniform {
    pub model: [f32; 16],
    pub params: [f32; 4], // x = opacity, y = rounded edge radius in voxel UV
}

// --- RENDERER STRUCT ---

pub struct Renderer<'a> {
    pub window: &'a Window,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,

    // --- TEXT ENGINE ---
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_atlas: TextAtlas,
    text_renderer: GlyphRenderer,
    text_cache: text_cache::TextCache,

    // --- SHADOWS ---
    shadow_view: wgpu::TextureView,
    shadow_global_buf: wgpu::Buffer,
    shadow_global_bind: wgpu::BindGroup,

    // --- UI ---
    console_v_buf: wgpu::Buffer,
    console_i_buf: wgpu::Buffer,
    console_inds: u32,
    hotbar_v_buf: wgpu::Buffer,
    hotbar_i_buf: wgpu::Buffer,
    hotbar_inds: u32,
    inventory_v_buf: wgpu::Buffer,
    inventory_i_buf: wgpu::Buffer,
    inventory_inds: u32,

    // --- SKY ---
    post_bind: wgpu::BindGroup,
    scene: GpuScene,

    // --- CORE ---
    animator: LodAnimator,
    pipeline_layouts: PipelineBindGroupLayouts,
    pipelines: RenderPipelineRegistry,

    chunks: HashMap<SurfaceChunkKey, ChunkMesh>,
    lod_chunks: HashMap<LodKey, ChunkMesh>,

    // --- UNIFORMS ---
    global_buf: wgpu::Buffer,
    global_bind: wgpu::BindGroup,

    local_bind_identity: wgpu::BindGroup,

    local_buf_player: wgpu::Buffer,
    local_bind_player: wgpu::BindGroup,

    depth: wgpu::TextureView,
    global_bind_identity: wgpu::BindGroup, // For UI to access dummy shadows

    // --- MESHES ---
    player_v_buf: wgpu::Buffer,
    player_i_buf: wgpu::Buffer,
    player_inds: u32,

    cross_v_buf: wgpu::Buffer,
    cross_i_buf: wgpu::Buffer,
    cross_inds: u32,

    cursor_v_buf: wgpu::Buffer,
    cursor_i_buf: wgpu::Buffer,
    cursor_inds: u32,

    block_damage_v_buf: wgpu::Buffer,
    block_damage_i_buf: wgpu::Buffer,
    block_damage_inds: u32,

    first_person_v_buf: wgpu::Buffer,
    first_person_i_buf: wgpu::Buffer,
    first_person_inds: u32,
    first_person_animation: hand_animation::HandAnimation,

    collision_v_buf: wgpu::Buffer,
    collision_i_buf: wgpu::Buffer,
    collision_inds: u32,
    frozen_frustum: Option<Frustum>,

    // --- THREADING ---
    load_queue: Vec<SurfaceChunkKey>,
    load_queue_set: HashSet<SurfaceChunkKey>,
    player_chunk_pos: Option<SurfaceChunkKey>,
    required_voxels: HashSet<SurfaceChunkKey>,
    required_lods: HashSet<LodKey>,

    mesh_tx: Sender<MeshJobResult<SurfaceChunkKey>>,
    mesh_rx: Receiver<MeshJobResult<SurfaceChunkKey>>,
    pending_chunks: HashSet<SurfaceChunkKey>,

    /// Chunks invalidated by a player edit — dispatched before the normal load queue.
    dirty_chunks: HashSet<SurfaceChunkKey>,
    /// In-flight dirty rebuild jobs (subset of pending_chunks — lets us skip the stale guard on receipt).
    pending_dirty: HashSet<SurfaceChunkKey>,

    lod_tx: Sender<MeshJobResult<LodKey>>,
    lod_rx: Receiver<MeshJobResult<LodKey>>,
    pending_lods: HashSet<LodKey>,

    scheduler: MeshScheduler,
    scheduler_stats: SchedulerStats,
    completed_mesh_time_sum_ms: f32,
    completed_mesh_time_max_ms: f32,
    completed_mesh_count: usize,
    completed_voxel_mesh_time_sum_ms: f32,
    completed_voxel_mesh_time_max_ms: f32,
    completed_voxel_mesh_count: usize,
    completed_lod_mesh_time_sum_ms: f32,
    completed_lod_mesh_time_max_ms: f32,
    completed_lod_mesh_count: usize,
    update_view_ms: f32,
    lod_selection_ms: f32,
    gpu_upload_ms: f32,
    last_terrain_draw_ms: f32,
    last_render_ms: f32,
    last_draw_calls: usize,
    last_shadow_draw_calls: usize,

    frame_stats: FrameStats,
    engine_debug_page: bool,

    // --- QUALITY ---
    /// Cached signatures so we skip rebuilding meshes when their inputs are
    /// unchanged.  Each entry tracks `(revision, viewport, …)` of the last
    /// successful upload.
    hotbar_cache_signature: Option<HotbarCacheSignature>,
    block_damage_cache_signature: Option<BlockDamageCacheSignature>,

    pub quality: QualitySettings,
    /// Edge length of the shadow depth texture, in pixels.  Read by the main
    /// render pass for texel-snapping the sun view matrix.
    pub shadow_map_size: u32,
    pub world_streaming: WorldStreamingConfig,
    pub meshing: VoxelMeshingConfig,
    pub atmosphere: AtmosphereConfig,

    // --- ATLAS ---
    atlas_bind: wgpu::BindGroup,
}

#[derive(Clone, Copy)]
struct QuadNode {
    face: u8,
    x: u32,
    y: u32,
    size: u32,
}

struct QuadContext<'a> {
    view: StreamingView,
    planet: &'a PlanetData,
    player_id: Option<VoxelCoord>,
    previous_voxels: &'a HashSet<SurfaceChunkKey>,
    previous_lods: &'a HashSet<LodKey>,
}

struct MeshJobResult<K> {
    key: K,
    mesh: CpuMesh,
    elapsed_ms: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct HotbarCacheSignature {
    revision: u64,
    viewport: (u32, u32),
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct BlockDamageCacheSignature {
    revision: u64,
    focused: Option<VoxelCoord>,
}

mod block_damage_overlay;
mod celestial_renderer;
mod cloud_renderer;
mod collision_debug_renderer;
mod debug_draw;
mod first_person_item;
mod fog_renderer;
mod gpu_scene;
mod hand_animation;
mod inventory;
mod inventory_components;
mod inventory_geometry;
mod inventory_text;
mod loading_pass;
mod lod_selection;
mod metrics;
mod post_process_renderer;
mod precipitation_renderer;
mod prewarm;
mod render_passes;
mod render_resources;
mod setup;
mod sky_renderer;
mod terrain_renderer;
mod text_cache;
mod text_pass;
mod ui_pass;
mod ui_renderer;
mod world_streamer;

pub use first_person_item::PlayerActionFeedback;

impl<'a> Renderer<'a> {
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.depth = Self::mk_depth(&self.device, &self.config);
        self.scene = GpuScene::new(&self.device, self.config.width, self.config.height);
        self.post_bind = create_post_bind_group(
            &self.device,
            &self.pipeline_layouts.post_process_input,
            &self.scene.view,
            &self.scene.sampler,
        );
    }

    fn pipeline(&self, id: PipelineId) -> &wgpu::RenderPipeline {
        self.pipelines.get(id)
    }

    fn terrain_pipeline(&self, wireframe: bool) -> &wgpu::RenderPipeline {
        if wireframe && self.pipelines.wireframe_supported() {
            self.pipelines.terrain_wire_or_fill()
        } else {
            self.pipeline(PipelineId::TerrainOpaque)
        }
    }

    // QUADTREE LOGIC

    fn reset_streaming_frame_stats(&mut self) {
        self.scheduler_stats = SchedulerStats::default();
        self.completed_mesh_time_sum_ms = 0.0;
        self.completed_mesh_time_max_ms = 0.0;
        self.completed_mesh_count = 0;
        self.completed_voxel_mesh_time_sum_ms = 0.0;
        self.completed_voxel_mesh_time_max_ms = 0.0;
        self.completed_voxel_mesh_count = 0;
        self.completed_lod_mesh_time_sum_ms = 0.0;
        self.completed_lod_mesh_time_max_ms = 0.0;
        self.completed_lod_mesh_count = 0;
        self.update_view_ms = 0.0;
        self.lod_selection_ms = 0.0;
        self.gpu_upload_ms = 0.0;
    }

    fn record_mesh_time(&mut self, elapsed_ms: f32) {
        self.completed_mesh_time_sum_ms += elapsed_ms;
        self.completed_mesh_time_max_ms = self.completed_mesh_time_max_ms.max(elapsed_ms);
        self.completed_mesh_count += 1;
    }

    fn record_voxel_mesh_time(&mut self, elapsed_ms: f32) {
        self.record_mesh_time(elapsed_ms);
        self.completed_voxel_mesh_time_sum_ms += elapsed_ms;
        self.completed_voxel_mesh_time_max_ms =
            self.completed_voxel_mesh_time_max_ms.max(elapsed_ms);
        self.completed_voxel_mesh_count += 1;
    }

    fn record_lod_mesh_time(&mut self, elapsed_ms: f32) {
        self.record_mesh_time(elapsed_ms);
        self.completed_lod_mesh_time_sum_ms += elapsed_ms;
        self.completed_lod_mesh_time_max_ms = self.completed_lod_mesh_time_max_ms.max(elapsed_ms);
        self.completed_lod_mesh_count += 1;
    }
}
