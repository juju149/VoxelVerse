use glam::{Vec2, Vec3};
use glyphon::{FontSystem, SwashCache, TextAtlas, TextRenderer as GlyphRenderer};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender};
use wgpu::PresentMode;
use winit::window::Window;

use vv_config::{LodConfig, RenderConfig};
use vv_diagnostics::DiagnosticConfig;
use vv_gameplay::PlayerGameplayState;
use vv_input::Controller;
use vv_mesh::Vertex;
use vv_registry::{BlockContent, RuntimeBlockVisual};
use vv_voxel::{ChunkKey, LodKey};
use vv_world_runtime::PlanetData;

use crate::{sky_state::SkyState, AnyKey, ChunkMesh, Frustum, LodAnimator};

use self::types::{MeshJobResult, RendererFrameTelemetry};

mod debug_overlay;
mod diagnostics;
mod frame;
mod gpu_context;
mod init;
mod mesh_upload;
mod pipelines;
mod public_api;
mod shadow_pass;
mod sky_pass;
mod streaming;
mod terrain_pass;
mod types;
mod ui_bridge;
mod ui_frame;
mod visual_content;
mod world_overlay;

pub struct Renderer<'a> {
    pub window: &'a Window,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,

    // Config snapshots stored at construction time
    render_cfg: RenderConfig,
    lod_cfg: LodConfig,
    block_content: BlockContent,
    diagnostic_config: DiagnosticConfig,

    // Text engine
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_atlas: TextAtlas,
    text_renderer: GlyphRenderer,

    // Shadows
    shadow_texture: wgpu::Texture,
    shadow_view: wgpu::TextureView,
    shadow_sampler: wgpu::Sampler,
    pipeline_shadow: wgpu::RenderPipeline,
    shadow_global_buf: wgpu::Buffer,
    shadow_global_bind: wgpu::BindGroup,

    // UI
    ui_renderer: crate::ui::UiRenderer,

    // Core
    animator: LodAnimator,
    local_layout: wgpu::BindGroupLayout,

    pipeline_fill: wgpu::RenderPipeline,
    pipeline_wire: wgpu::RenderPipeline,
    pipeline_line: wgpu::RenderPipeline,
    pipeline_feedback: wgpu::RenderPipeline,
    pipeline_sky: wgpu::RenderPipeline,

    chunks: HashMap<ChunkKey, ChunkMesh>,
    lod_chunks: HashMap<LodKey, ChunkMesh>,

    global_buf: wgpu::Buffer,
    global_bind: wgpu::BindGroup,
    _block_visual_buf: wgpu::Buffer,
    _block_visual_palette_buf: wgpu::Buffer,

    local_buf_identity: wgpu::Buffer,
    local_bind_identity: wgpu::BindGroup,
    local_buf_player: wgpu::Buffer,
    local_bind_player: wgpu::BindGroup,
    local_buf_guide: wgpu::Buffer,
    local_bind_guide: wgpu::BindGroup,

    depth: wgpu::TextureView,
    global_bind_identity: wgpu::BindGroup,

    player_v_buf: wgpu::Buffer,
    player_i_buf: wgpu::Buffer,
    player_inds: u32,
    guide_v_buf: wgpu::Buffer,
    guide_i_buf: wgpu::Buffer,
    guide_inds: u32,
    cursor_v_buf: wgpu::Buffer,
    cursor_i_buf: wgpu::Buffer,
    cursor_inds: u32,
    break_v_buf: wgpu::Buffer,
    break_i_buf: wgpu::Buffer,
    break_inds: u32,
    drop_v_buf: wgpu::Buffer,
    drop_i_buf: wgpu::Buffer,
    drop_inds: u32,
    collision_v_buf: wgpu::Buffer,
    collision_i_buf: wgpu::Buffer,
    collision_inds: u32,

    frozen_frustum: Option<Frustum>,

    /// Day/night cycle clock. Owns the current time and computes atmosphere per frame.
    sky_state: SkyState,

    // Async mesh loading
    load_queue: Vec<ChunkKey>,
    player_chunk_pos: Option<ChunkKey>,
    mesh_tx: Sender<(MeshJobResult<ChunkKey>, Vec<Vertex>, Vec<u32>)>,
    mesh_rx: Receiver<(MeshJobResult<ChunkKey>, Vec<Vertex>, Vec<u32>)>,
    pending_chunks: HashSet<ChunkKey>,
    lod_tx: Sender<(MeshJobResult<LodKey>, Vec<Vertex>, Vec<u32>)>,
    lod_rx: Receiver<(MeshJobResult<LodKey>, Vec<Vertex>, Vec<u32>)>,
    pending_lods: HashSet<LodKey>,
    frame_telemetry: RendererFrameTelemetry,

    // FPS counter
    last_fps_time: std::time::Instant,
    frame_count: u32,
    current_fps: u32,
}
