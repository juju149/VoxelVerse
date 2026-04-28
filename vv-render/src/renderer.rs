use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};
use glam::Vec3;
use wgpu::PresentMode;
use wgpu::util::DeviceExt;
use winit::window::Window;
use bytemuck::{Pod, Zeroable};
use glyphon::{
    FontSystem, SwashCache, TextAtlas, TextArea, TextRenderer as GlyphRenderer,
    TextBounds, Resolution, Buffer, Metrics, Shaping, Attrs, Family,
};

use vv_core::{BlockId, ChunkKey, LodKey, CHUNK_SIZE};
use vv_planet::CoordSystem;
use vv_world_runtime::PlanetData;
use vv_mesh::{Vertex, MeshGen};
use vv_config::{EngineConfig, RenderConfig};
use vv_input::Controller;
use vv_gameplay::{Player, Console};
use vv_physics::Physics;

use crate::{ChunkMesh, LodAnimator, AnyKey, Frustum};

// --- Uniform structs --------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GlobalUniform {
    pub view_proj:       [f32; 16],
    pub light_view_proj: [f32; 16],
    pub cam_pos:         [f32; 4],
    pub sun_dir:         [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LocalUniform {
    pub model:  [f32; 16],
    pub params: [f32; 4], // x = opacity
}

// --- Renderer ---------------------------------------------------------------

pub struct Renderer<'a> {
    pub window: &'a Window,
    surface:    wgpu::Surface<'a>,
    device:     wgpu::Device,
    queue:      wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,

    // Config snapshots stored at construction time
    render_cfg:    RenderConfig,
    lod_grid_res:  u32,

    // Text engine
    font_system:   FontSystem,
    swash_cache:   SwashCache,
    text_atlas:    TextAtlas,
    text_renderer: GlyphRenderer,

    // Shadows
    shadow_texture:     wgpu::Texture,
    shadow_view:        wgpu::TextureView,
    shadow_sampler:     wgpu::Sampler,
    pipeline_shadow:    wgpu::RenderPipeline,
    shadow_global_buf:  wgpu::Buffer,
    shadow_global_bind: wgpu::BindGroup,

    // UI
    pipeline_ui:   wgpu::RenderPipeline,
    console_v_buf: wgpu::Buffer,
    console_i_buf: wgpu::Buffer,
    console_inds:  u32,

    // Core
    animator:     LodAnimator,
    local_layout: wgpu::BindGroupLayout,

    pipeline_fill: wgpu::RenderPipeline,
    pipeline_wire: wgpu::RenderPipeline,
    pipeline_line: wgpu::RenderPipeline,

    chunks:     HashMap<ChunkKey, ChunkMesh>,
    lod_chunks: HashMap<LodKey,   ChunkMesh>,

    global_buf:   wgpu::Buffer,
    global_bind:  wgpu::BindGroup,

    local_buf_identity:  wgpu::Buffer,
    local_bind_identity: wgpu::BindGroup,
    local_buf_player:    wgpu::Buffer,
    local_bind_player:   wgpu::BindGroup,
    local_buf_guide:     wgpu::Buffer,
    local_bind_guide:    wgpu::BindGroup,

    depth:               wgpu::TextureView,
    global_bind_identity: wgpu::BindGroup,

    player_v_buf: wgpu::Buffer,
    player_i_buf: wgpu::Buffer,
    player_inds:  u32,
    guide_v_buf:  wgpu::Buffer,
    guide_i_buf:  wgpu::Buffer,
    guide_inds:   u32,
    cross_v_buf:  wgpu::Buffer,
    cross_i_buf:  wgpu::Buffer,
    cross_inds:   u32,
    cursor_v_buf: wgpu::Buffer,
    cursor_i_buf: wgpu::Buffer,
    cursor_inds:  u32,
    collision_v_buf: wgpu::Buffer,
    collision_i_buf: wgpu::Buffer,
    collision_inds:  u32,

    frozen_frustum: Option<Frustum>,

    // Async mesh loading
    load_queue:      Vec<ChunkKey>,
    player_chunk_pos: Option<ChunkKey>,
    mesh_tx:         Sender<(ChunkKey, Vec<Vertex>, Vec<u32>)>,
    mesh_rx:         Receiver<(ChunkKey, Vec<Vertex>, Vec<u32>)>,
    pending_chunks:  HashSet<ChunkKey>,
    lod_tx:          Sender<(LodKey, Vec<Vertex>, Vec<u32>)>,
    lod_rx:          Receiver<(LodKey, Vec<Vertex>, Vec<u32>)>,
    pending_lods:    HashSet<LodKey>,

    // FPS counter
    last_fps_time: std::time::Instant,
    frame_count:   u32,
    current_fps:   u32,
}

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a Window, cfg: &EngineConfig) -> Self {
        let instance = wgpu::Instance::default();
        let surface  = instance.create_surface(window).unwrap();
        let adapter  = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        // Log GPU info
        let info = adapter.get_info();
        println!("--- GPU ---");
        println!("Name   : {}", info.name);
        println!("Backend: {:?}", info.backend);
        println!("-----------");

        let mut limits = adapter.limits();
        limits.max_buffer_size = (8u64 * 1024 * 1024 * 1024).min(limits.max_buffer_size);

        let mut features = wgpu::Features::empty();
        if adapter.features().contains(wgpu::Features::POLYGON_MODE_LINE) {
            features |= wgpu::Features::POLYGON_MODE_LINE;
        }
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor { label: None, required_features: features, required_limits: limits },
            None,
        ).await.unwrap();

        let size = window.inner_size();
        let mut surf_cfg = surface.get_default_config(&adapter, size.width, size.height).unwrap();
        let present_modes = surface.get_capabilities(&adapter).present_modes;
        surf_cfg.present_mode = [PresentMode::Immediate, PresentMode::Mailbox]
            .iter().copied()
            .find(|m| present_modes.contains(m))
            .unwrap_or(PresentMode::Fifo);
        surface.configure(&device, &surf_cfg);

        let shadow_size = cfg.render.shadow_map_size;
        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map"),
            size: wgpu::Extent3d { width: shadow_size, height: shadow_size, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_view    = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let global_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("global_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison), count: None },
            ],
        });
        let local_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("local_layout"),
            entries: &[wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None }],
        });

        let global_buf = device.create_buffer(&wgpu::BufferDescriptor { label: Some("Global Uniform"), size: 160, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let global_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &global_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: global_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&shadow_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&shadow_sampler) },
            ],
            label: None,
        });

        let shadow_global_buf = device.create_buffer(&wgpu::BufferDescriptor { label: Some("Shadow Global"), size: 160, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let dummy_depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Dummy Depth"), size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float, usage: wgpu::TextureUsages::TEXTURE_BINDING, view_formats: &[],
        });
        let dummy_depth_view = dummy_depth.create_view(&wgpu::TextureViewDescriptor::default());
        let shadow_global_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Pass Bind Group"), layout: &global_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: shadow_global_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&dummy_depth_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&shadow_sampler) },
            ],
        });

        let identity_mat = glam::Mat4::IDENTITY;
        let default_local = LocalUniform { model: identity_mat.to_cols_array(), params: [1.0, 0.0, 1.0, 0.0] };

        let make_local_buf = |label: &str| device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label), contents: bytemuck::cast_slice(&[default_local]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let local_buf_identity = make_local_buf("Identity Uniform");
        let local_buf_player   = make_local_buf("Player Uniform");
        let local_buf_guide    = make_local_buf("Guide Uniform");

        let make_local_bind = |buf: &wgpu::Buffer| device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &local_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.as_entire_binding() }],
            label: None,
        });
        let local_bind_identity = make_local_bind(&local_buf_identity);
        let local_bind_player   = make_local_bind(&local_buf_player);
        let local_bind_guide    = make_local_bind(&local_buf_guide);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None, source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None, bind_group_layouts: &[&global_layout, &local_layout], push_constant_ranges: &[],
        });

        let pipeline_shadow = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Pipeline"), layout: Some(&layout),
            vertex: Self::vertex_state(&shader),
            fragment: None,
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: Some(wgpu::Face::Front), ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: true, depth_compare: wgpu::CompareFunction::Less, stencil: Default::default(), bias: wgpu::DepthBiasState { constant: 2, slope_scale: 2.0, clamp: 0.0 } }),
            multisample: Default::default(), multiview: None,
        });
        let pipeline_fill = Self::create_pipeline(&device, &surf_cfg, &layout, &shader, wgpu::PrimitiveTopology::TriangleList, false);
        let pipeline_wire = Self::create_pipeline(&device, &surf_cfg, &layout, &shader, wgpu::PrimitiveTopology::TriangleList, true);
        let pipeline_line = Self::create_pipeline(&device, &surf_cfg, &layout, &shader, wgpu::PrimitiveTopology::LineList, false);
        let depth = Self::mk_depth(&device, &surf_cfg);

        let pipeline_ui = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"), layout: Some(&layout),
            vertex: Self::vertex_state(&shader),
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: "fs_main", targets: &[Some(wgpu::ColorTargetState { format: surf_cfg.format, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: false, depth_compare: wgpu::CompareFunction::Always, stencil: Default::default(), bias: Default::default() }),
            multisample: Default::default(), multiview: None,
        });

        let font_system   = FontSystem::new();
        let swash_cache   = SwashCache::new();
        let mut text_atlas    = TextAtlas::new(&device, &queue, surf_cfg.format);
        let text_renderer = GlyphRenderer::new(&mut text_atlas, &device, wgpu::MultisampleState::default(), None);

        let (pv, pi) = MeshGen::generate_cylinder(0.4, 1.8, 16);
        let (gv, gi) = MeshGen::generate_sphere_guide(1.0, 64);
        let (cv, ci) = MeshGen::generate_crosshair();

        let mk_vbuf = |v: &[Vertex]| device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(v), usage: wgpu::BufferUsages::VERTEX });
        let mk_ibuf = |i: &[u32]|   device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(i), usage: wgpu::BufferUsages::INDEX });

        let player_v_buf = mk_vbuf(&pv); let player_i_buf = mk_ibuf(&pi);
        let guide_v_buf  = mk_vbuf(&gv); let guide_i_buf  = mk_ibuf(&gi);
        let cross_v_buf  = mk_vbuf(&cv); let cross_i_buf  = mk_ibuf(&ci);

        let mk_dyn_vbuf = |label, size| device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let mk_dyn_ibuf = |label, size| device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size, usage: wgpu::BufferUsages::INDEX  | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });

        let cursor_v_buf    = mk_dyn_vbuf("Cursor V",    4096);
        let cursor_i_buf    = mk_dyn_ibuf("Cursor I",    4096);
        let collision_v_buf = mk_dyn_vbuf("Collision V", 65536);
        let collision_i_buf = mk_dyn_ibuf("Collision I", 65536);
        let console_v_buf   = mk_dyn_vbuf("Console V",   1024);
        let console_i_buf   = mk_dyn_ibuf("Console I",   1024);

        let identity_global = GlobalUniform {
            view_proj: identity_mat.to_cols_array(), light_view_proj: identity_mat.to_cols_array(),
            cam_pos: [0.0; 4], sun_dir: [0.0, 1.0, 0.0, 0.0],
        };
        let global_buf_id = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Identity"), contents: bytemuck::cast_slice(&[identity_global]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let global_bind_identity = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &global_layout, label: Some("Identity Bind Group"),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: global_buf_id.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&shadow_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&shadow_sampler) },
            ],
        });

        let (mesh_tx, mesh_rx) = channel();
        let (lod_tx,  lod_rx)  = channel();

        Self {
            window, surface, device, queue, config: surf_cfg,
            render_cfg: cfg.render.clone(),
            lod_grid_res: cfg.lod.tile_grid_res,
            font_system, swash_cache, text_atlas, text_renderer,
            shadow_texture, shadow_view, shadow_sampler, pipeline_shadow, shadow_global_buf, shadow_global_bind,
            pipeline_ui, console_v_buf, console_i_buf, console_inds: 0,
            animator: LodAnimator::new(cfg.render.lod_fade_duration),
            local_layout,
            pipeline_fill, pipeline_wire, pipeline_line,
            chunks: HashMap::new(), lod_chunks: HashMap::new(),
            global_buf, global_bind,
            local_buf_identity, local_bind_identity,
            local_buf_player, local_bind_player,
            local_buf_guide, local_bind_guide,
            depth, global_bind_identity,
            player_v_buf, player_i_buf, player_inds: pi.len() as u32,
            guide_v_buf,  guide_i_buf,  guide_inds:  gi.len() as u32,
            cross_v_buf,  cross_i_buf,  cross_inds:  ci.len() as u32,
            cursor_v_buf, cursor_i_buf, cursor_inds: 0,
            collision_v_buf, collision_i_buf, collision_inds: 0,
            frozen_frustum: None,
            load_queue: Vec::new(), player_chunk_pos: None,
            mesh_tx, mesh_rx, pending_chunks: HashSet::new(),
            lod_tx,  lod_rx,  pending_lods:   HashSet::new(),
            last_fps_time: std::time::Instant::now(), frame_count: 0, current_fps: 0,
        }
    }

    // --- Pipeline helpers ---------------------------------------------------

    fn vertex_state(shader: &wgpu::ShaderModule) -> wgpu::VertexState {
        wgpu::VertexState {
            module: shader, entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as _,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0,  shader_location: 0 },
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 12, shader_location: 1 },
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 24, shader_location: 2 },
                ],
            }],
        }
    }

    fn create_pipeline(device: &wgpu::Device, cfg: &wgpu::SurfaceConfiguration, layout: &wgpu::PipelineLayout, shader: &wgpu::ShaderModule, topology: wgpu::PrimitiveTopology, wireframe: bool) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None, layout: Some(layout),
            vertex: Self::vertex_state(shader),
            fragment: Some(wgpu::FragmentState { module: shader, entry_point: "fs_main", targets: &[Some(cfg.format.into())] }),
            primitive: wgpu::PrimitiveState {
                topology, cull_mode: None,
                polygon_mode: if wireframe { wgpu::PolygonMode::Line } else { wgpu::PolygonMode::Fill },
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: true, depth_compare: wgpu::CompareFunction::Less, stencil: Default::default(), bias: Default::default() }),
            multisample: Default::default(), multiview: None,
        })
    }

    fn mk_depth(dev: &wgpu::Device, cfg: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
        dev.create_texture(&wgpu::TextureDescriptor {
            label: None, size: wgpu::Extent3d { width: cfg.width, height: cfg.height, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float, usage: wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[],
        }).create_view(&wgpu::TextureViewDescriptor::default())
    }

    // --- Public interface ---------------------------------------------------

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width  = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.depth = Self::mk_depth(&self.device, &self.config);
    }

    pub fn update_view(&mut self, player_pos: Vec3, planet: &PlanetData) {
        let res = planet.resolution;
        let player_id = CoordSystem::pos_to_id(player_pos, res);

        let mut upload_count = 0;
        while let Ok((key, v, i)) = self.lod_rx.try_recv() {
            self.pending_lods.remove(&key);
            self.upload_lod_buffer(key, v, i);
            upload_count += 1;
            if upload_count > 20 { break; }
        }

        let mut required_voxels: HashSet<ChunkKey> = HashSet::new();
        let mut required_lods:   HashSet<LodKey>   = HashSet::new();
        let logical_size = res.next_power_of_two();

        for face in 0u8..6 {
            self.process_quadtree(face, 0, 0, logical_size, player_pos, planet, player_id, &mut required_voxels, &mut required_lods);
        }

        let missing_voxels: Vec<ChunkKey> = required_voxels.iter()
            .filter(|k| !self.chunks.contains_key(k)).cloned().collect();

        let current_lods: Vec<LodKey> = self.lod_chunks.keys().cloned().collect();
        for k in current_lods {
            if required_lods.contains(&k) { continue; }
            let mut children_missing = false;
            for v_key in &missing_voxels {
                if v_key.face != k.face { continue; }
                let vx = v_key.u_idx * CHUNK_SIZE; let vy = v_key.v_idx * CHUNK_SIZE; let vs = CHUNK_SIZE;
                if k.x < vx + vs && k.x + k.size > vx && k.y < vy + vs && k.y + k.size > vy {
                    children_missing = true; break;
                }
            }
            if children_missing { required_lods.insert(k); }
            else if let Some(mesh) = self.lod_chunks.remove(&k) {
                self.animator.retire(AnyKey::Lod(k), mesh);
            }
        }

        let mut spawn_count = 0;
        let grid_res = self.lod_grid_res;
        for key in required_lods {
            if !self.lod_chunks.contains_key(&key) && !self.pending_lods.contains(&key) {
                if spawn_count >= 8 { break; }
                self.pending_lods.insert(key);
                let tx = self.lod_tx.clone();
                let p  = planet.clone();
                std::thread::spawn(move || {
                    let (v, i) = MeshGen::generate_lod_mesh(key, &p, grid_res);
                    let _ = tx.send((key, v, i));
                });
                spawn_count += 1;
            }
        }

        let current_voxels: Vec<ChunkKey> = self.chunks.keys().cloned().collect();
        for k in current_voxels {
            if !required_voxels.contains(&k) {
                if let Some(mesh) = self.chunks.remove(&k) {
                    self.animator.retire(AnyKey::Voxel(k), mesh);
                }
            }
        }

        self.load_queue.retain(|k| required_voxels.contains(k));
        for k in required_voxels {
            if !self.chunks.contains_key(&k) && !self.load_queue.contains(&k) {
                self.load_queue.push(k);
            }
        }
        self.load_queue.sort_by(|a, b| {
            let center = |k: &ChunkKey| CoordSystem::get_vertex_pos(k.face, k.u_idx * CHUNK_SIZE + CHUNK_SIZE / 2, k.v_idx * CHUNK_SIZE + CHUNK_SIZE / 2, planet.resolution / 2, planet.resolution);
            let da = center(a).distance_squared(player_pos);
            let db = center(b).distance_squared(player_pos);
            db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
        });
        self.process_load_queue(player_pos, planet);
    }

    pub fn update_cursor(&mut self, planet: &PlanetData, id: Option<BlockId>) {
        if let Some(id) = id {
            let res = planet.resolution;
            let p   = |u, v, l| CoordSystem::get_vertex_pos(id.face, id.u + u, id.v + v, id.layer + l, res);
            let corners = [p(0,0,0),p(1,0,0),p(0,1,0),p(1,1,0),p(0,0,1),p(1,0,1),p(0,1,1),p(1,1,1)];
            let edges = [(0,1),(1,3),(3,2),(2,0),(4,5),(5,7),(7,6),(6,4),(0,4),(1,5),(2,6),(3,7)];
            let mut verts = Vec::new(); let mut inds = Vec::new();
            let thickness = 0.025; let color = [1.0, 1.0, 0.0]; let mut idx_base = 0u32;
            for (start, end) in edges {
                let a = corners[start]; let b = corners[end];
                let dir = (b - a).normalize();
                let ref_up = if dir.dot(Vec3::Y).abs() > 0.9 { Vec3::X } else { Vec3::Y };
                let right = dir.cross(ref_up).normalize() * thickness;
                let up    = dir.cross(right).normalize() * thickness;
                for off in [(-right - up), (right - up), (right + up), (-right + up)] {
                    verts.push(Vertex { pos: (a + off).to_array(), color, normal: [0.0; 3] });
                    verts.push(Vertex { pos: (b + off).to_array(), color, normal: [0.0; 3] });
                }
                for (i0,i1,i2,i3) in [(0u32,1,3,2),(2,3,5,4),(4,5,7,6),(6,7,1,0)] {
                    inds.push(idx_base+i0); inds.push(idx_base+i1); inds.push(idx_base+i2);
                    inds.push(idx_base+i2); inds.push(idx_base+i3); inds.push(idx_base+i0);
                }
                idx_base += 8;
            }
            self.queue.write_buffer(&self.cursor_v_buf, 0, bytemuck::cast_slice(&verts));
            self.queue.write_buffer(&self.cursor_i_buf, 0, bytemuck::cast_slice(&inds));
            self.cursor_inds = inds.len() as u32;
        } else {
            self.cursor_inds = 0;
        }
    }

    pub fn refresh_neighbors(&mut self, id: BlockId, planet: &PlanetData) {
        let u_c = id.u / CHUNK_SIZE;
        let v_c = id.v / CHUNK_SIZE;
        for key in [
            ChunkKey { face: id.face, u_idx: u_c,                    v_idx: v_c },
            ChunkKey { face: id.face, u_idx: u_c.saturating_sub(1),  v_idx: v_c },
            ChunkKey { face: id.face, u_idx: u_c + 1,                v_idx: v_c },
            ChunkKey { face: id.face, u_idx: u_c,                    v_idx: v_c.saturating_sub(1) },
            ChunkKey { face: id.face, u_idx: u_c,                    v_idx: v_c + 1 },
        ] {
            if self.chunks.contains_key(&key) {
                let (v, i) = MeshGen::build_chunk(key, planet);
                if v.is_empty() { self.chunks.remove(&key); }
                else { self.upload_chunk_buffers(key, v, i); }
            }
        }
    }

    pub fn force_reload_all(&mut self, planet: &PlanetData, player_pos: Vec3) {
        self.chunks.clear(); self.lod_chunks.clear();
        self.load_queue.clear(); self.pending_chunks.clear(); self.pending_lods.clear();
        self.player_chunk_pos = None;
        self.update_view(player_pos, planet);
    }

    pub fn log_memory(&self, planet: &PlanetData) {
        let (tv, ti) = self.chunks.values().fold((0usize, 0usize), |(v, i), c| (v + c.num_verts, i + c.num_inds as usize));
        let mb = ((tv * 36) + (ti * 4)) as f32 / (1024.0 * 1024.0);
        println!("Resolution: {} | Chunks: {} | GPU: {:.2} MB", planet.resolution, self.chunks.len(), mb);
    }

    // --- Render -------------------------------------------------------------

    pub fn render(
        &mut self,
        controller: &Controller,
        player: &Player,
        physics: &Physics,
        planet: &PlanetData,
        console: &Console,
    ) {
        self.update_console_mesh(console.height_fraction);

        if controller.show_collisions {
            let (v, i) = MeshGen::generate_collision_debug(player.position, planet);
            self.queue.write_buffer(&self.collision_v_buf, 0, bytemuck::cast_slice(&v));
            self.queue.write_buffer(&self.collision_i_buf, 0, bytemuck::cast_slice(&i));
            self.collision_inds = i.len() as u32;
        } else {
            self.collision_inds = 0;
        }

        let out  = match self.surface.get_current_texture() { Ok(o) => o, _ => return };
        let view = out.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let w = self.config.width  as f32;
        let h = self.config.height as f32;
        let mvp = controller.get_matrix(player, physics, w, h, &self.render_cfg);

        let sun_dir    = Vec3::new(0.5, 0.8, 0.4).normalize();
        let shadow_dist = 200.0f32;
        let proj_size   = 60.0f32;
        let center      = player.position;
        let mut sun_view = glam::Mat4::look_at_rh(center + sun_dir * shadow_dist, center, Vec3::Y);

        let shadow_map_size_f = self.render_cfg.shadow_map_size as f32;
        let texel_size = (2.0 * proj_size) / shadow_map_size_f;
        let shadow_origin = sun_view.transform_point3(center);
        let snap_x = (shadow_origin.x / texel_size).round() * texel_size - shadow_origin.x;
        let snap_y = (shadow_origin.y / texel_size).round() * texel_size - shadow_origin.y;
        sun_view = glam::Mat4::from_translation(Vec3::new(snap_x, snap_y, 0.0)) * sun_view;

        let sun_proj = glam::Mat4::orthographic_rh(-proj_size, proj_size, -proj_size, proj_size, -200.0, 500.0);
        let light_vp = sun_proj * sun_view;

        let cam_pos = controller.get_camera_pos(player, physics);
        let frustum = Frustum::from_matrix(mvp);

        let cull_frustum_val;
        let cull_frustum: &Frustum = if controller.freeze_culling {
            if self.frozen_frustum.is_none() { self.frozen_frustum = Some(Frustum::from_matrix(mvp)); }
            self.frozen_frustum.as_ref().unwrap()
        } else {
            self.frozen_frustum = None;
            cull_frustum_val = Frustum::from_matrix(mvp);
            &cull_frustum_val
        };

        let global_data = GlobalUniform {
            view_proj: mvp.to_cols_array(), light_view_proj: light_vp.to_cols_array(),
            cam_pos: [cam_pos.x, cam_pos.y, cam_pos.z, 1.0],
            sun_dir: [sun_dir.x, sun_dir.y, sun_dir.z, 0.0],
        };
        self.queue.write_buffer(&self.global_buf,        0, bytemuck::cast_slice(&[global_data]));
        self.queue.write_buffer(&self.shadow_global_buf, 0, bytemuck::cast_slice(&[GlobalUniform {
            view_proj: light_vp.to_cols_array(), ..global_data
        }]));

        let model_mat = player.get_model_matrix();
        self.queue.write_buffer(&self.local_buf_player, 0, bytemuck::cast_slice(model_mat.as_ref()));
        let r = planet.resolution as f32 / 2.0;
        self.queue.write_buffer(&self.local_buf_guide, 0, bytemuck::cast_slice(glam::Mat4::from_scale(Vec3::splat(r)).as_ref()));

        let now = std::time::Instant::now();
        let dying = self.animator.update_dying(now);
        for (key, alpha) in dying {
            if let Some(state) = self.animator.dying_chunks.get(&key) {
                let d = LocalUniform { model: glam::Mat4::IDENTITY.to_cols_array(), params: [alpha, 1.0, 0.0, 0.0] };
                self.queue.write_buffer(&state.mesh.uniform_buf, 0, bytemuck::cast_slice(&[d]));
            }
        }

        let queue     = &self.queue;
        let animator  = &mut self.animator;
        for (key, mesh) in &self.lod_chunks {
            let alpha = animator.get_opacity(AnyKey::Lod(*key), now);
            if alpha < 1.0 || animator.spawning_chunks.contains_key(&AnyKey::Lod(*key)) {
                let d = LocalUniform { model: glam::Mat4::IDENTITY.to_cols_array(), params: [alpha.min(1.0), 0.0, 0.0, 0.0] };
                queue.write_buffer(&mesh.uniform_buf, 0, bytemuck::cast_slice(&[d]));
                if alpha >= 1.0 { animator.spawning_chunks.remove(&AnyKey::Lod(*key)); }
            }
        }
        for (key, mesh) in &self.chunks {
            let alpha = animator.get_opacity(AnyKey::Voxel(*key), now);
            if alpha < 1.0 || animator.spawning_chunks.contains_key(&AnyKey::Voxel(*key)) {
                let d = LocalUniform { model: glam::Mat4::IDENTITY.to_cols_array(), params: [alpha.min(1.0), 0.0, 0.0, 0.0] };
                queue.write_buffer(&mesh.uniform_buf, 0, bytemuck::cast_slice(&[d]));
                if alpha >= 1.0 { animator.spawning_chunks.remove(&AnyKey::Voxel(*key)); }
            }
        }

        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // Shadow pass
        {
            let mut sp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shadow"), color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None, occlusion_query_set: None,
            });
            sp.set_pipeline(&self.pipeline_shadow);
            sp.set_bind_group(0, &self.shadow_global_bind, &[]);
            for mesh in self.chunks.values() {
                if frustum.intersects_sphere(mesh.center, mesh.radius) {
                    sp.set_bind_group(1, &mesh.bind_group, &[]);
                    sp.set_vertex_buffer(0, mesh.v_buf.slice(..));
                    sp.set_index_buffer(mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    sp.draw_indexed(0..mesh.num_inds, 0, 0..1);
                }
            }
            for mesh in self.lod_chunks.values() {
                if frustum.intersects_sphere(mesh.center, mesh.radius) {
                    sp.set_bind_group(1, &mesh.bind_group, &[]);
                    sp.set_vertex_buffer(0, mesh.v_buf.slice(..));
                    sp.set_index_buffer(mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    sp.draw_indexed(0..mesh.num_inds, 0, 0..1);
                }
            }
        }

        // Main pass
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.02, g: 0.03, b: 0.05, a: 1.0 }), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None, occlusion_query_set: None,
            });

            let terrain_pipeline = if controller.is_wireframe { &self.pipeline_wire } else { &self.pipeline_fill };
            pass.set_pipeline(terrain_pipeline);
            pass.set_bind_group(0, &self.global_bind, &[]);

            for mesh in self.lod_chunks.values() {
                if cull_frustum.intersects_sphere(mesh.center, mesh.radius) {
                    pass.set_bind_group(1, &mesh.bind_group, &[]);
                    pass.set_vertex_buffer(0, mesh.v_buf.slice(..));
                    pass.set_index_buffer(mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..mesh.num_inds, 0, 0..1);
                }
            }
            for mesh in self.chunks.values() {
                if cull_frustum.intersects_sphere(mesh.center, mesh.radius) {
                    pass.set_bind_group(1, &mesh.bind_group, &[]);
                    pass.set_vertex_buffer(0, mesh.v_buf.slice(..));
                    pass.set_index_buffer(mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..mesh.num_inds, 0, 0..1);
                }
            }
            for state in self.animator.dying_chunks.values() {
                if frustum.intersects_sphere(state.mesh.center, state.mesh.radius) {
                    pass.set_bind_group(1, &state.mesh.bind_group, &[]);
                    pass.set_vertex_buffer(0, state.mesh.v_buf.slice(..));
                    pass.set_index_buffer(state.mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..state.mesh.num_inds, 0, 0..1);
                }
            }

            if !controller.first_person {
                pass.set_pipeline(terrain_pipeline);
                pass.set_bind_group(1, &self.local_bind_player, &[]);
                pass.set_vertex_buffer(0, self.player_v_buf.slice(..));
                pass.set_index_buffer(self.player_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.player_inds, 0, 0..1);
            }
            if self.collision_inds > 0 {
                pass.set_pipeline(&self.pipeline_line);
                pass.set_bind_group(0, &self.global_bind, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.collision_v_buf.slice(..));
                pass.set_index_buffer(self.collision_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.collision_inds, 0, 0..1);
            }
            if self.cursor_inds > 0 {
                pass.set_pipeline(&self.pipeline_fill);
                pass.set_bind_group(0, &self.global_bind, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.cursor_v_buf.slice(..));
                pass.set_index_buffer(self.cursor_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.cursor_inds, 0, 0..1);
            }
            if controller.first_person {
                pass.set_pipeline(&self.pipeline_line);
                pass.set_bind_group(0, &self.global_bind_identity, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.cross_v_buf.slice(..));
                pass.set_index_buffer(self.cross_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.cross_inds, 0, 0..1);
            }
            if self.console_inds > 0 {
                pass.set_pipeline(&self.pipeline_ui);
                pass.set_bind_group(0, &self.global_bind_identity, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.console_v_buf.slice(..));
                pass.set_index_buffer(self.console_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.console_inds, 0, 0..1);
            }
        }

        self.frame_count += 1;
        let now2 = std::time::Instant::now();
        if now2.duration_since(self.last_fps_time).as_secs_f32() >= 1.0 {
            self.current_fps = self.frame_count;
            self.frame_count = 0;
            self.last_fps_time = now2;
        }

        // Text pass
        {
            let mut text_areas: Vec<TextArea> = Vec::new();
            let mut text_buffers = Vec::new();

            if console.height_fraction > 0.0 {
                let console_h = (self.config.height as f32 / 2.0) * console.height_fraction;
                let start_y   = console_h - 40.0;
                let line_h    = 20.0;
                for (i, (line, color)) in console.history.iter().rev().enumerate() {
                    let y = start_y - i as f32 * line_h;
                    if y < 0.0 { break; }
                    let mut buf = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
                    buf.set_size(&mut self.font_system, w, h);
                    buf.set_text(&mut self.font_system, line,
                        Attrs::new().family(Family::Monospace).color(glyphon::Color::rgb(
                            (color[0] * 255.0) as u8, (color[1] * 255.0) as u8, (color[2] * 255.0) as u8,
                        )), Shaping::Advanced);
                    text_buffers.push((buf, y, false));
                }
                let ms  = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
                let cur = if (ms / 500) % 2 == 0 { "_" } else { " " };
                let mut ibuf = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
                ibuf.set_size(&mut self.font_system, w, h);
                ibuf.set_text(&mut self.font_system, &format!("> {}{}", console.input_buffer, cur),
                    Attrs::new().family(Family::Monospace).color(glyphon::Color::rgb(255, 255, 0)), Shaping::Advanced);
                text_buffers.push((ibuf, console_h - 20.0, false));
            }

            let fps_text = format!("FPS: {}", self.current_fps);
            let mut fps_buf = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
            fps_buf.set_size(&mut self.font_system, w, h);
            fps_buf.set_text(&mut self.font_system, &fps_text, Attrs::new().family(Family::Monospace).color(glyphon::Color::rgb(255, 255, 255)), Shaping::Advanced);
            text_buffers.push((fps_buf, 5.0, true));

            for (buf, y, _) in &text_buffers {
                text_areas.push(TextArea {
                    buffer: buf, left: 10.0, top: *y,
                    scale: 1.0, bounds: TextBounds { left: 0, top: 0, right: w as i32, bottom: h as i32 },
                    default_color: glyphon::Color::rgb(255, 255, 255),
                });
            }

            if !text_areas.is_empty() {
                let _ = self.text_renderer.prepare(
                    &self.device, &self.queue,
                    &mut self.font_system, &mut self.text_atlas,
                    Resolution { width: self.config.width, height: self.config.height },
                    text_areas, &mut self.swash_cache,
                );
                let mut text_pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Text"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view, resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
                });
                let _ = self.text_renderer.render(&self.text_atlas, &mut text_pass);
            }
        }

        self.queue.submit(std::iter::once(enc.finish()));
        out.present();
        self.text_atlas.trim();
    }

    // --- Private internals --------------------------------------------------

    fn update_console_mesh(&mut self, t: f32) {
        if t <= 0.001 { self.console_inds = 0; return; }
        let bottom_y = 1.0 - t;
        let color = [0.1, 0.1, 0.15]; let normal = [0.0, 0.0, 1.0];
        let verts = vec![
            Vertex { pos: [-1.0, 1.0,      0.0], color, normal },
            Vertex { pos: [ 1.0, 1.0,      0.0], color, normal },
            Vertex { pos: [-1.0, bottom_y, 0.0], color, normal },
            Vertex { pos: [ 1.0, bottom_y, 0.0], color, normal },
        ];
        let inds = vec![0u32, 2, 1, 1, 2, 3];
        self.queue.write_buffer(&self.console_v_buf, 0, bytemuck::cast_slice(&verts));
        self.queue.write_buffer(&self.console_i_buf, 0, bytemuck::cast_slice(&inds));
        self.console_inds = 6;
    }

    fn process_quadtree(
        &self, face: u8, x: u32, y: u32, size: u32,
        cam_pos: Vec3, planet: &PlanetData, player_id: Option<BlockId>,
        voxels: &mut HashSet<ChunkKey>, lods: &mut HashSet<LodKey>,
    ) {
        if x >= planet.resolution || y >= planet.resolution { return; }
        let cu = (x + size / 2).min(planet.resolution - 1);
        let cv = (y + size / 2).min(planet.resolution - 1);
        let h  = planet.resolution / 2;
        let world_pos = CoordSystem::get_vertex_pos(face, cu, cv, h, planet.resolution);

        let mut dist = world_pos.distance(cam_pos);
        if let Some(pid) = player_id {
            if pid.face == face && pid.u >= x && pid.u < x + size && pid.v >= y && pid.v < y + size {
                dist = 0.0;
            }
        }

        let node_r = (size as f32 * CoordSystem::get_layer_radius(h, planet.resolution)) / planet.resolution as f32;
        let lod_factor = if size <= CHUNK_SIZE      { 18.0 }
                         else if size <= CHUNK_SIZE * 2 { 12.0 }
                         else if size <= CHUNK_SIZE * 4 { 7.0 }
                         else if size <= CHUNK_SIZE * 8 { 5.0 }
                         else { 4.0 };

        let is_smallest = size <= CHUNK_SIZE;
        if dist < node_r * lod_factor && !is_smallest {
            let half = size / 2;
            self.process_quadtree(face, x,        y,        half, cam_pos, planet, player_id, voxels, lods);
            self.process_quadtree(face, x + half, y,        half, cam_pos, planet, player_id, voxels, lods);
            self.process_quadtree(face, x,        y + half, half, cam_pos, planet, player_id, voxels, lods);
            self.process_quadtree(face, x + half, y + half, half, cam_pos, planet, player_id, voxels, lods);
        } else if size <= CHUNK_SIZE {
            let key = ChunkKey { face, u_idx: x / CHUNK_SIZE, v_idx: y / CHUNK_SIZE };
            if key.u_idx * CHUNK_SIZE < planet.resolution && key.v_idx * CHUNK_SIZE < planet.resolution {
                voxels.insert(key);
            }
        } else {
            lods.insert(LodKey { face, x, y, size });
        }
    }

    fn upload_lod_buffer(&mut self, key: LodKey, v: Vec<Vertex>, i: Vec<u32>) {
        let v_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&v), usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST });
        let i_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&i), usage: wgpu::BufferUsages::INDEX  | wgpu::BufferUsages::COPY_DST });
        let uniform_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("LOD Uniform"),
            contents: bytemuck::cast_slice(&[LocalUniform { model: glam::Mat4::IDENTITY.to_cols_array(), params: [0.0; 4] }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.local_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform_buf.as_entire_binding() }],
            label: None,
        });
        let (center, radius) = Self::bounds_from_verts(&v);
        self.lod_chunks.insert(key, ChunkMesh { v_buf, i_buf, num_inds: i.len() as u32, num_verts: v.len(), uniform_buf, bind_group, center, radius });
        self.animator.start_spawn(AnyKey::Lod(key));
    }

    fn upload_chunk_buffers(&mut self, key: ChunkKey, v: Vec<Vertex>, i: Vec<u32>) {
        let v_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&v), usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST });
        let i_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&i), usage: wgpu::BufferUsages::INDEX  | wgpu::BufferUsages::COPY_DST });
        let is_update = self.chunks.contains_key(&key);
        let start_opacity = if is_update { 1.0f32 } else { 0.0 };
        let uniform_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Uniform"),
            contents: bytemuck::cast_slice(&[LocalUniform { model: glam::Mat4::IDENTITY.to_cols_array(), params: [start_opacity, 0.0, 0.0, 0.0] }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.local_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform_buf.as_entire_binding() }],
            label: None,
        });
        let (center, radius) = Self::bounds_from_verts(&v);
        self.chunks.insert(key, ChunkMesh { v_buf, i_buf, num_inds: i.len() as u32, num_verts: v.len(), uniform_buf, bind_group, center, radius });
        if !is_update { self.animator.start_spawn(AnyKey::Voxel(key)); }
    }

    fn bounds_from_verts(v: &[Vertex]) -> (Vec3, f32) {
        if v.is_empty() { return (Vec3::ZERO, 0.0); }
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for vert in v { let p = Vec3::from_array(vert.pos); min = min.min(p); max = max.max(p); }
        let center = (min + max) * 0.5;
        (center, min.distance(max) * 0.5)
    }

    fn process_load_queue(&mut self, _player_pos: Vec3, planet: &PlanetData) {
        let mut budget = 4i32;
        while let Ok((key, v, i)) = self.mesh_rx.try_recv() {
            self.pending_chunks.remove(&key);
            if !v.is_empty() { self.upload_chunk_buffers(key, v, i); budget -= 1; }
            if budget <= 0 { break; }
        }
        if budget <= 0 || self.load_queue.is_empty() || self.pending_chunks.len() >= 12 { return; }
        for _ in 0..4 {
            if let Some(key) = self.load_queue.pop() {
                if self.chunks.contains_key(&key) || self.pending_chunks.contains(&key) { continue; }
                self.pending_chunks.insert(key);
                let p  = planet.clone();
                let tx = self.mesh_tx.clone();
                std::thread::spawn(move || { let (v, i) = MeshGen::build_chunk(key, &p); let _ = tx.send((key, v, i)); });
            } else { break; }
        }
    }
}
