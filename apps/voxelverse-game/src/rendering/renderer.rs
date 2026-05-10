// engine renderer

use crate::diagnostics::FrameStats;
use crate::math::Frustum;
use crate::meshing::CpuMesh;
use crate::rendering::lod_animation::LodAnimator;
use crate::rendering::quality::QualitySettings;
use crate::rendering::types::ChunkMesh;
use crate::streaming::{MeshScheduler, SchedulerStats};
use crate::voxel::{LodKey, SurfaceChunkKey, VoxelCoord};
use crate::world::PlanetData;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use glyphon::{FontSystem, SwashCache, TextAtlas, TextRenderer as GlyphRenderer};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender};
use winit::window::Window;

// --- UNIFORMS ---

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GlobalUniform {
    pub view_proj: [f32; 16],
    pub light_view_proj: [f32; 16],
    pub cam_pos: [f32; 4],
    pub sun_dir: [f32; 4],
}

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

    // --- SHADOWS ---
    shadow_view: wgpu::TextureView,
    pipeline_shadow: wgpu::RenderPipeline,
    shadow_global_buf: wgpu::Buffer,
    shadow_global_bind: wgpu::BindGroup,

    // --- UI ---
    pipeline_ui: wgpu::RenderPipeline,
    console_v_buf: wgpu::Buffer,
    console_i_buf: wgpu::Buffer,
    console_inds: u32,

    // --- CORE ---
    animator: LodAnimator,
    local_layout: wgpu::BindGroupLayout,

    pipeline_fill: wgpu::RenderPipeline,
    pipeline_wire: wgpu::RenderPipeline,
    pipeline_line: wgpu::RenderPipeline,

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
    update_view_ms: f32,
    last_render_ms: f32,
    last_draw_calls: usize,
    last_shadow_draw_calls: usize,

    frame_stats: FrameStats,

    // --- QUALITY ---
    pub quality: QualitySettings,
    /// Edge length of the shadow depth texture, in pixels.  Read by the main
    /// render pass for texel-snapping the sun view matrix.
    pub shadow_map_size: u32,
    /// Multiplier applied to the LOD-split distance — < 1 reduces draw work.
    pub lod_distance_scale: f32,

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
    cam_pos: Vec3,
    planet: &'a PlanetData,
    player_id: Option<VoxelCoord>,
}

struct MeshJobResult<K> {
    key: K,
    mesh: CpuMesh,
    elapsed_ms: f32,
}

mod debug_draw;
mod lod_selection;
mod metrics;
mod pipelines;
mod render_passes;
mod setup;
mod setup_resources;
mod streaming;
mod ui;

impl<'a> Renderer<'a> {
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.depth = Self::mk_depth(&self.device, &self.config);
    }

    // QUADTREE LOGIC

    fn reset_streaming_frame_stats(&mut self) {
        self.scheduler_stats = SchedulerStats::default();
        self.completed_mesh_time_sum_ms = 0.0;
        self.completed_mesh_time_max_ms = 0.0;
        self.completed_mesh_count = 0;
        self.update_view_ms = 0.0;
    }

    fn record_mesh_time(&mut self, elapsed_ms: f32) {
        self.completed_mesh_time_sum_ms += elapsed_ms;
        self.completed_mesh_time_max_ms = self.completed_mesh_time_max_ms.max(elapsed_ms);
        self.completed_mesh_count += 1;
    }
}
