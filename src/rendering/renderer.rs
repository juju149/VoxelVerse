// engine renderer

use crate::math::Frustum;
use crate::rendering::lod_animation::LodAnimator;
use crate::rendering::types::{ChunkMesh, Vertex};
use crate::voxel::{ChunkKey, LodKey, VoxelCoord};
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
    pub params: [f32; 4], // x = opacity
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

    chunks: HashMap<ChunkKey, ChunkMesh>,
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
    load_queue: Vec<ChunkKey>,
    player_chunk_pos: Option<ChunkKey>,

    mesh_tx: Sender<(ChunkKey, Vec<Vertex>, Vec<u32>)>,
    mesh_rx: Receiver<(ChunkKey, Vec<Vertex>, Vec<u32>)>,
    pending_chunks: HashSet<ChunkKey>,

    lod_tx: Sender<(LodKey, Vec<Vertex>, Vec<u32>)>,
    lod_rx: Receiver<(LodKey, Vec<Vertex>, Vec<u32>)>,
    pending_lods: HashSet<LodKey>,

    // --- FPS ---
    last_fps_time: std::time::Instant,
    frame_count: u32,
    current_fps: u32,
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

mod debug_draw;
mod lod_selection;
mod pipelines;
mod render_passes;
mod setup;
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
}
