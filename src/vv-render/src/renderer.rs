use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use glyphon::{
    Attrs, Buffer, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer as GlyphRenderer,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;
use wgpu::PresentMode;
use winit::window::Window;

use vv_config::{EngineConfig, LodConfig, RenderConfig};
use vv_core::{BlockId, ChunkKey, LodKey, CHUNK_SIZE};
use vv_diagnostics::{
    emit, DiagnosticConfig, GpuCounters, LodCounters, LogDomain, LogLevel, MeshCounters,
    RuntimeSnapshot, StreamingCounters, WorldCounters, WorldgenCounters,
};
use vv_gameplay::{can_craft_hand_recipe, Console, Player, PlayerGameplayState};
use vv_input::Controller;
use vv_mesh::{MeshGen, Vertex};
use vv_physics::Physics;
use vv_planet::CoordSystem;
use vv_registry::{
    BlockContent, BlockRenderSource, CompiledContent, CompiledItemKind, CompiledTintMode,
    CompiledVisualMaterialType, ItemId, RecipeId,
};
use vv_world_runtime::PlanetData;

use crate::{
    atmosphere::AtmosphereUniform,
    block_atlas::BlockTextureAtlas,
    block_feedback::{
        block_break_mesh, selection_outline_mesh, BlockBreakStyle, SelectionOutlineStyle,
    },
    gameplay_ui::{GameplayUiLayout, RectPx},
    AnyKey, ChunkMesh, Frustum, LodAnimator,
};

// --- Uniform structs --------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct GlobalUniform {
    view_proj: [f32; 16],
    light_view_proj: [f32; 16],
    cam_pos: [f32; 4],
    atmosphere: AtmosphereUniform,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct LocalUniform {
    model: [f32; 16],
    params: [f32; 4], // x = opacity
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct BlockMaterialUniform {
    secondary_color_texture: [f32; 4],
    variation: [f32; 4],
    flags: [f32; 4],
}

#[derive(Clone, Copy, Debug)]
struct MeshJobResult<K> {
    key: K,
    vertices: usize,
    indices: usize,
    duration: Duration,
}

#[derive(Clone, Debug, Default)]
struct RendererFrameTelemetry {
    streaming: StreamingCounters,
    lod: LodCounters,
    mesh: MeshCounters,
    gpu: GpuCounters,
    render_prep_time: Duration,
    lod_coverage_time: Duration,
    chunk_streaming_time: Duration,
}

#[derive(Clone, Copy, Debug)]
enum MeshJobKind {
    Chunk,
    Lod,
    Remesh,
}

fn build_block_materials(content: &CompiledContent) -> Vec<BlockMaterialUniform> {
    let mut materials = Vec::with_capacity(content.blocks.len().max(1));
    for block in content.blocks.entries() {
        let material = block.render.material;
        materials.push(BlockMaterialUniform {
            secondary_color_texture: [
                material.secondary_color[0],
                material.secondary_color[1],
                material.secondary_color[2],
                material.texture_influence,
            ],
            variation: [
                material.block_variation,
                material.face_variation,
                material.macro_variation,
                material.detail_strength,
            ],
            flags: [
                visual_material_code(material.visual_type),
                block.render.roughness,
                tint_mode_code(block.render.tint),
                if block.render.translucent { 1.0 } else { 0.0 },
            ],
        });
    }
    if materials.is_empty() {
        materials.push(BlockMaterialUniform {
            secondary_color_texture: [1.0, 1.0, 1.0, 1.0],
            variation: [0.05, 0.03, 0.03, 0.02],
            flags: [0.0, 0.7, 0.0, 0.0],
        });
    }
    materials
}

fn visual_material_code(kind: CompiledVisualMaterialType) -> f32 {
    match kind {
        CompiledVisualMaterialType::Generic => 0.0,
        CompiledVisualMaterialType::Grass => 1.0,
        CompiledVisualMaterialType::Dirt => 2.0,
        CompiledVisualMaterialType::Stone => 3.0,
        CompiledVisualMaterialType::Sand => 4.0,
        CompiledVisualMaterialType::Wood => 5.0,
        CompiledVisualMaterialType::Leaves => 6.0,
        CompiledVisualMaterialType::CutStone => 7.0,
        CompiledVisualMaterialType::Planks => 8.0,
        CompiledVisualMaterialType::Ore => 9.0,
        CompiledVisualMaterialType::Water => 10.0,
    }
}

fn tint_mode_code(mode: CompiledTintMode) -> f32 {
    match mode {
        CompiledTintMode::None => 0.0,
        CompiledTintMode::GrassColor => 1.0,
        CompiledTintMode::FoliageColor => 2.0,
        CompiledTintMode::WaterColor => 3.0,
    }
}

// --- Renderer ---------------------------------------------------------------

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
    _block_atlas: BlockTextureAtlas,
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
    pipeline_ui: wgpu::RenderPipeline,
    console_v_buf: wgpu::Buffer,
    console_i_buf: wgpu::Buffer,
    console_inds: u32,
    ui_v_buf: wgpu::Buffer,
    ui_i_buf: wgpu::Buffer,
    ui_inds: u32,

    // Core
    animator: LodAnimator,
    local_layout: wgpu::BindGroupLayout,

    pipeline_fill: wgpu::RenderPipeline,
    pipeline_wire: wgpu::RenderPipeline,
    pipeline_line: wgpu::RenderPipeline,
    pipeline_feedback: wgpu::RenderPipeline,

    chunks: HashMap<ChunkKey, ChunkMesh>,
    lod_chunks: HashMap<LodKey, ChunkMesh>,

    global_buf: wgpu::Buffer,
    global_bind: wgpu::BindGroup,
    _block_material_buf: wgpu::Buffer,

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

impl<'a> Renderer<'a> {
    pub async fn new(
        window: &'a Window,
        cfg: &EngineConfig,
        content: &CompiledContent,
        assets_root: &Path,
        diagnostic_config: DiagnosticConfig,
    ) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let info = adapter.get_info();
        emit(
            diagnostic_config,
            LogLevel::Info,
            LogDomain::Gpu,
            format!("adapter name=\"{}\" backend={:?}", info.name, info.backend),
        );

        let mut limits = adapter.limits();
        limits.max_buffer_size = (8u64 * 1024 * 1024 * 1024).min(limits.max_buffer_size);

        let mut features = wgpu::Features::empty();
        if adapter
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
        {
            features |= wgpu::Features::POLYGON_MODE_LINE;
        }
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: limits,
                },
                None,
            )
            .await
            .unwrap();
        let block_content = content.to_block_content();
        let block_atlas = BlockTextureAtlas::build(&device, &queue, assets_root, &content.textures);

        let size = window.inner_size();
        let mut surf_cfg = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        let present_modes = surface.get_capabilities(&adapter).present_modes;
        surf_cfg.present_mode = [PresentMode::Immediate, PresentMode::Mailbox]
            .iter()
            .copied()
            .find(|m| present_modes.contains(m))
            .unwrap_or(PresentMode::Fifo);
        surface.configure(&device, &surf_cfg);

        let shadow_size = cfg.render.shadow_map_size;
        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map"),
            size: wgpu::Extent3d {
                width: shadow_size,
                height: shadow_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());
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
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let local_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("local_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let global_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Global Uniform"),
            size: std::mem::size_of::<GlobalUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let block_materials = build_block_materials(content);
        let block_material_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Material Params"),
            contents: bytemuck::cast_slice(&block_materials),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let global_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &global_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: global_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&block_atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&block_atlas.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: block_atlas.rect_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: block_material_buf.as_entire_binding(),
                },
            ],
            label: None,
        });

        let shadow_global_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Global"),
            size: std::mem::size_of::<GlobalUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let dummy_depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Dummy Depth"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let dummy_depth_view = dummy_depth.create_view(&wgpu::TextureViewDescriptor::default());
        let shadow_global_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Pass Bind Group"),
            layout: &global_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: shadow_global_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dummy_depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&block_atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&block_atlas.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: block_atlas.rect_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: block_material_buf.as_entire_binding(),
                },
            ],
        });

        let identity_mat = glam::Mat4::IDENTITY;
        let default_local = LocalUniform {
            model: identity_mat.to_cols_array(),
            params: [1.0, 0.0, 1.0, 0.0],
        };

        let make_local_buf = |label: &str| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(&[default_local]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        };
        let local_buf_identity = make_local_buf("Identity Uniform");
        let local_buf_player = make_local_buf("Player Uniform");
        let local_buf_guide = make_local_buf("Guide Uniform");

        let make_local_bind = |buf: &wgpu::Buffer| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &local_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buf.as_entire_binding(),
                }],
                label: None,
            })
        };
        let local_bind_identity = make_local_bind(&local_buf_identity);
        let local_bind_player = make_local_bind(&local_buf_player);
        let local_bind_guide = make_local_bind(&local_buf_guide);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&global_layout, &local_layout],
            push_constant_ranges: &[],
        });

        let pipeline_shadow = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Pipeline"),
            layout: Some(&layout),
            vertex: Self::vertex_state(&shader),
            fragment: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
            multisample: Default::default(),
            multiview: None,
        });
        let pipeline_fill = Self::create_pipeline(
            &device,
            &surf_cfg,
            &layout,
            &shader,
            wgpu::PrimitiveTopology::TriangleList,
            false,
        );
        let pipeline_wire = Self::create_pipeline(
            &device,
            &surf_cfg,
            &layout,
            &shader,
            wgpu::PrimitiveTopology::TriangleList,
            true,
        );
        let pipeline_line = Self::create_pipeline(
            &device,
            &surf_cfg,
            &layout,
            &shader,
            wgpu::PrimitiveTopology::LineList,
            false,
        );
        let pipeline_feedback =
            Self::create_feedback_pipeline(&device, &surf_cfg, &layout, &shader);
        let depth = Self::mk_depth(&device, &surf_cfg);

        let pipeline_ui = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&layout),
            vertex: Self::vertex_state(&shader),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surf_cfg.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });

        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let mut text_atlas = TextAtlas::new(&device, &queue, surf_cfg.format);
        let text_renderer = GlyphRenderer::new(
            &mut text_atlas,
            &device,
            wgpu::MultisampleState::default(),
            None,
        );

        let (pv, pi) = MeshGen::generate_cylinder(0.4, 1.8, 16);
        let (gv, gi) = MeshGen::generate_sphere_guide(1.0, 64);
        let mk_vbuf = |v: &[Vertex]| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(v),
                usage: wgpu::BufferUsages::VERTEX,
            })
        };
        let mk_ibuf = |i: &[u32]| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(i),
                usage: wgpu::BufferUsages::INDEX,
            })
        };

        let player_v_buf = mk_vbuf(&pv);
        let player_i_buf = mk_ibuf(&pi);
        let guide_v_buf = mk_vbuf(&gv);
        let guide_i_buf = mk_ibuf(&gi);
        let mk_dyn_vbuf = |label, size| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let mk_dyn_ibuf = |label, size| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };

        let cursor_v_buf = mk_dyn_vbuf("Cursor V", 8192);
        let cursor_i_buf = mk_dyn_ibuf("Cursor I", 4096);
        let break_v_buf = mk_dyn_vbuf("Block Break V", 65536);
        let break_i_buf = mk_dyn_ibuf("Block Break I", 65536);
        let collision_v_buf = mk_dyn_vbuf("Collision V", 65536);
        let collision_i_buf = mk_dyn_ibuf("Collision I", 65536);
        let console_v_buf = mk_dyn_vbuf("Console V", 1024);
        let console_i_buf = mk_dyn_ibuf("Console I", 1024);
        let ui_v_buf = mk_dyn_vbuf("Gameplay UI V", 1_048_576);
        let ui_i_buf = mk_dyn_ibuf("Gameplay UI I", 1_048_576);
        let drop_v_buf = mk_dyn_vbuf("Dropped Items V", 1_048_576);
        let drop_i_buf = mk_dyn_ibuf("Dropped Items I", 1_048_576);

        let identity_global = GlobalUniform {
            view_proj: identity_mat.to_cols_array(),
            light_view_proj: identity_mat.to_cols_array(),
            cam_pos: [0.0; 4],
            atmosphere: AtmosphereUniform::from_config(&cfg.render.atmosphere),
        };
        let global_buf_id = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Identity"),
            contents: bytemuck::cast_slice(&[identity_global]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let global_bind_identity = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &global_layout,
            label: Some("Identity Bind Group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: global_buf_id.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&block_atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&block_atlas.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: block_atlas.rect_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: block_material_buf.as_entire_binding(),
                },
            ],
        });

        let (mesh_tx, mesh_rx) = channel();
        let (lod_tx, lod_rx) = channel();

        Self {
            window,
            surface,
            device,
            queue,
            config: surf_cfg,
            render_cfg: cfg.render.clone(),
            lod_cfg: cfg.lod.clone(),
            block_content,
            _block_atlas: block_atlas,
            diagnostic_config,
            font_system,
            swash_cache,
            text_atlas,
            text_renderer,
            shadow_texture,
            shadow_view,
            shadow_sampler,
            pipeline_shadow,
            shadow_global_buf,
            shadow_global_bind,
            pipeline_ui,
            console_v_buf,
            console_i_buf,
            console_inds: 0,
            ui_v_buf,
            ui_i_buf,
            ui_inds: 0,
            animator: LodAnimator::new(cfg.render.lod_fade_duration),
            local_layout,
            pipeline_fill,
            pipeline_wire,
            pipeline_line,
            pipeline_feedback,
            chunks: HashMap::new(),
            lod_chunks: HashMap::new(),
            global_buf,
            global_bind,
            _block_material_buf: block_material_buf,
            local_buf_identity,
            local_bind_identity,
            local_buf_player,
            local_bind_player,
            local_buf_guide,
            local_bind_guide,
            depth,
            global_bind_identity,
            player_v_buf,
            player_i_buf,
            player_inds: pi.len() as u32,
            guide_v_buf,
            guide_i_buf,
            guide_inds: gi.len() as u32,
            cursor_v_buf,
            cursor_i_buf,
            cursor_inds: 0,
            break_v_buf,
            break_i_buf,
            break_inds: 0,
            drop_v_buf,
            drop_i_buf,
            drop_inds: 0,
            collision_v_buf,
            collision_i_buf,
            collision_inds: 0,
            frozen_frustum: None,
            load_queue: Vec::new(),
            player_chunk_pos: None,
            mesh_tx,
            mesh_rx,
            pending_chunks: HashSet::new(),
            lod_tx,
            lod_rx,
            pending_lods: HashSet::new(),
            frame_telemetry: RendererFrameTelemetry::default(),
            last_fps_time: std::time::Instant::now(),
            frame_count: 0,
            current_fps: 0,
        }
    }

    // --- Pipeline helpers ---------------------------------------------------

    fn vertex_state(shader: &wgpu::ShaderModule) -> wgpu::VertexState<'_> {
        wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as _,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 12,
                        shader_location: 1,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 24,
                        shader_location: 2,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 36,
                        shader_location: 3,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Sint32,
                        offset: 44,
                        shader_location: 4,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Sint32,
                        offset: 48,
                        shader_location: 5,
                    },
                ],
            }],
        }
    }

    fn create_pipeline(
        device: &wgpu::Device,
        cfg: &wgpu::SurfaceConfiguration,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        topology: wgpu::PrimitiveTopology,
        wireframe: bool,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(layout),
            vertex: Self::vertex_state(shader),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[Some(cfg.format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology,
                cull_mode: None,
                polygon_mode: if wireframe {
                    wgpu::PolygonMode::Line
                } else {
                    wgpu::PolygonMode::Fill
                },
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        })
    }

    fn create_feedback_pipeline(
        device: &wgpu::Device,
        cfg: &wgpu::SurfaceConfiguration,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Block Feedback Pipeline"),
            layout: Some(layout),
            vertex: Self::vertex_state(shader),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_feedback",
                targets: &[Some(wgpu::ColorTargetState {
                    format: cfg.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        })
    }

    fn mk_depth(dev: &wgpu::Device, cfg: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
        dev.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: cfg.width,
                height: cfg.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
    }

    // --- Public interface ---------------------------------------------------

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.depth = Self::mk_depth(&self.device, &self.config);
        emit(
            self.diagnostic_config,
            LogLevel::Info,
            LogDomain::Render,
            format!("surface resized width={} height={}", width, height),
        );
    }

    pub fn begin_diagnostic_frame(&mut self) {
        self.frame_telemetry = RendererFrameTelemetry::default();
    }

    pub fn update_view(&mut self, player_pos: Vec3, planet: &PlanetData) {
        let lod_start = Instant::now();
        let res = planet.resolution;
        let player_id = CoordSystem::pos_to_id(player_pos, planet.geometry);

        let mut upload_count = 0;
        while upload_count < self.lod_cfg.lod_uploads_per_frame
            && self.frame_telemetry.gpu.uploads < self.lod_cfg.max_gpu_uploads_per_frame
        {
            let Ok((job, v, i)) = self.lod_rx.try_recv() else {
                break;
            };
            self.pending_lods.remove(&job.key);
            self.record_mesh_job(job.duration, job.vertices, job.indices, MeshJobKind::Lod);
            if !v.is_empty() {
                self.upload_lod_buffer(job.key, v, i);
                self.frame_telemetry.lod.lods_uploaded += 1;
                upload_count += 1;
            }
        }

        let mut raw_required_voxels: HashSet<ChunkKey> = HashSet::new();
        let mut raw_required_lods: HashSet<LodKey> = HashSet::new();
        let logical_size = res.next_power_of_two();

        for face in 0u8..6 {
            self.process_quadtree(
                face,
                0,
                0,
                logical_size,
                player_pos,
                planet,
                player_id,
                &mut raw_required_voxels,
                &mut raw_required_lods,
            );
        }

        let (required_voxels, dropped_voxels) = self.prioritized_chunk_split(
            raw_required_voxels,
            player_pos,
            planet,
            self.lod_cfg.max_required_chunks,
        );
        let missing_voxels: Vec<ChunkKey> = required_voxels
            .iter()
            .filter(|k| !self.chunks.contains_key(k))
            .cloned()
            .collect();
        let required_chunk_count = required_voxels.len();
        let missing_chunk_count = missing_voxels.len();

        let mut required_lods = self.prioritized_lods(
            raw_required_lods,
            player_pos,
            planet,
            self.lod_cfg.max_required_lods,
        );
        self.add_chunk_fallback_lods(
            missing_voxels
                .iter()
                .copied()
                .chain(dropped_voxels.iter().copied()),
            &mut required_lods,
            player_pos,
            planet,
        );
        let current_lods: Vec<LodKey> = self.lod_chunks.keys().cloned().collect();
        for k in current_lods {
            if required_lods.contains(&k) {
                continue;
            }
            let mut children_missing = false;
            for v_key in &missing_voxels {
                if v_key.face != k.face {
                    continue;
                }
                let vx = v_key.u_idx * CHUNK_SIZE;
                let vy = v_key.v_idx * CHUNK_SIZE;
                let vs = CHUNK_SIZE;
                if k.x < vx + vs && k.x + k.size > vx && k.y < vy + vs && k.y + k.size > vy {
                    children_missing = true;
                    break;
                }
            }
            if children_missing {
                required_lods.insert(k);
            } else if let Some(mesh) = self.lod_chunks.remove(&k) {
                self.animator.retire(AnyKey::Lod(k), mesh);
            }
        }
        self.limit_lod_pressure(player_pos, planet);
        self.animator.limit_retained(
            self.lod_cfg.max_retiring_meshes,
            self.lod_cfg.max_retiring_meshes,
        );

        let required_lod_count = required_lods.len();
        let covered_lod_count = required_lods
            .iter()
            .filter(|key| self.lod_chunks.contains_key(key))
            .count();
        let missing_lod_count = required_lod_count.saturating_sub(covered_lod_count);

        let mut spawn_count = 0;
        let grid_res = self.lod_cfg.tile_grid_res;
        let mut missing_lods: Vec<LodKey> = required_lods
            .iter()
            .copied()
            .filter(|key| !self.lod_chunks.contains_key(key) && !self.pending_lods.contains(key))
            .collect();
        missing_lods.sort_by(|a, b| {
            Self::lod_distance_squared(a, player_pos, planet)
                .partial_cmp(&Self::lod_distance_squared(b, player_pos, planet))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for key in missing_lods {
            if !self.lod_chunks.contains_key(&key) && !self.pending_lods.contains(&key) {
                if spawn_count >= self.lod_cfg.lod_jobs_per_frame
                    || self.pending_lods.len() >= self.lod_cfg.max_pending_lod_jobs
                    || self.lod_chunks.len() + self.pending_lods.len()
                        >= self.lod_cfg.max_active_lods
                {
                    break;
                }
                self.pending_lods.insert(key);
                let tx = self.lod_tx.clone();
                let p = planet.clone();
                let blocks = self.block_content.clone();
                std::thread::spawn(move || {
                    let start = Instant::now();
                    let (v, i) = MeshGen::generate_lod_mesh(key, &p, grid_res, &blocks);
                    let job = MeshJobResult {
                        key,
                        vertices: v.len(),
                        indices: i.len(),
                        duration: start.elapsed(),
                    };
                    let _ = tx.send((job, v, i));
                });
                spawn_count += 1;
            }
        }
        self.frame_telemetry.lod.lod_jobs_started += spawn_count as u32;

        let current_voxels: Vec<ChunkKey> = self.chunks.keys().cloned().collect();
        for k in current_voxels {
            if !required_voxels.contains(&k) {
                if let Some(mesh) = self.chunks.remove(&k) {
                    self.animator.retire(AnyKey::Voxel(k), mesh);
                }
            }
        }
        self.frame_telemetry.lod_coverage_time += lod_start.elapsed();

        let streaming_start = Instant::now();
        let mut queued: Vec<(ChunkKey, f32)> = required_voxels
            .iter()
            .copied()
            .filter(|k| !self.chunks.contains_key(k) && !self.pending_chunks.contains(k))
            .map(|k| (k, Self::chunk_distance_squared(&k, player_pos, planet)))
            .collect();
        queued.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        queued.truncate(self.lod_cfg.max_chunk_queue);
        self.load_queue = queued.into_iter().rev().map(|(key, _)| key).collect();
        self.process_load_queue(player_pos, planet);
        self.frame_telemetry.chunk_streaming_time += streaming_start.elapsed();

        self.frame_telemetry.streaming.active_chunks = self.chunks.len();
        self.frame_telemetry.streaming.required_chunks = required_chunk_count;
        self.frame_telemetry.streaming.missing_chunks = missing_chunk_count;
        self.frame_telemetry.streaming.load_queue = self.load_queue.len();
        self.frame_telemetry.streaming.pending_chunk_jobs = self.pending_chunks.len();
        self.frame_telemetry.lod.active_lods = self.lod_chunks.len();
        self.frame_telemetry.lod.required_lods = required_lod_count;
        self.frame_telemetry.lod.covered_lods = covered_lod_count;
        self.frame_telemetry.lod.missing_lods = missing_lod_count;
        self.frame_telemetry.lod.pending_lod_jobs = self.pending_lods.len();
        self.frame_telemetry.lod.coverage_percent = if required_lod_count == 0 {
            100.0
        } else {
            covered_lod_count as f32 * 100.0 / required_lod_count as f32
        };
    }

    pub fn update_cursor(&mut self, planet: &PlanetData, id: Option<BlockId>) {
        if let Some(id) = id {
            let mesh = selection_outline_mesh(planet, id, SelectionOutlineStyle::default());
            self.queue
                .write_buffer(&self.cursor_v_buf, 0, bytemuck::cast_slice(&mesh.vertices));
            self.queue
                .write_buffer(&self.cursor_i_buf, 0, bytemuck::cast_slice(&mesh.indices));
            self.cursor_inds = mesh.indices.len() as u32;
        } else {
            self.cursor_inds = 0;
        }
    }

    pub fn inventory_slot_at(
        &self,
        gameplay: &PlayerGameplayState,
        mouse_pos: Vec2,
    ) -> Option<usize> {
        GameplayUiLayout::new(
            self.config.width as f32,
            self.config.height as f32,
            &gameplay.inventory,
            gameplay.inventory_open,
        )
        .inventory_slot_at(mouse_pos)
    }

    pub fn inventory_recipe_at(
        &self,
        gameplay: &PlayerGameplayState,
        content: &CompiledContent,
        mouse_pos: Vec2,
    ) -> Option<RecipeId> {
        let mut layout = GameplayUiLayout::new(
            self.config.width as f32,
            self.config.height as f32,
            &gameplay.inventory,
            gameplay.inventory_open,
        );
        layout.add_hand_recipes(content.recipes.recipes_for_station(None));
        layout.recipe_at(mouse_pos)
    }

    pub fn refresh_neighbors(&mut self, id: BlockId, planet: &PlanetData) {
        let u_c = id.u / CHUNK_SIZE;
        let v_c = id.v / CHUNK_SIZE;
        for key in [
            ChunkKey {
                face: id.face,
                u_idx: u_c,
                v_idx: v_c,
            },
            ChunkKey {
                face: id.face,
                u_idx: u_c.saturating_sub(1),
                v_idx: v_c,
            },
            ChunkKey {
                face: id.face,
                u_idx: u_c + 1,
                v_idx: v_c,
            },
            ChunkKey {
                face: id.face,
                u_idx: u_c,
                v_idx: v_c.saturating_sub(1),
            },
            ChunkKey {
                face: id.face,
                u_idx: u_c,
                v_idx: v_c + 1,
            },
        ] {
            if self.chunks.contains_key(&key) {
                let start = Instant::now();
                let (v, i) = MeshGen::build_chunk(key, planet, &self.block_content);
                self.record_mesh_job(start.elapsed(), v.len(), i.len(), MeshJobKind::Remesh);
                if v.is_empty() {
                    self.chunks.remove(&key);
                    self.frame_telemetry.streaming.empty_chunks += 1;
                } else {
                    self.upload_chunk_buffers(key, v, i);
                }
                self.frame_telemetry.streaming.chunks_invalidated += 1;
            }
        }
    }

    pub fn force_reload_all(&mut self, planet: &PlanetData, player_pos: Vec3) {
        self.chunks.clear();
        self.lod_chunks.clear();
        self.load_queue.clear();
        self.pending_chunks.clear();
        self.pending_lods.clear();
        self.player_chunk_pos = None;
        self.update_view(player_pos, planet);
    }

    pub fn log_memory(&self, planet: &PlanetData) {
        let (tv, ti) = self.chunks.values().fold((0usize, 0usize), |(v, i), c| {
            (v + c.num_verts, i + c.num_inds as usize)
        });
        let mb = ((tv * std::mem::size_of::<Vertex>()) + (ti * 4)) as f32 / (1024.0 * 1024.0);
        emit(
            self.diagnostic_config,
            LogLevel::Info,
            LogDomain::Memory,
            format!(
                "resolution={} chunks={} mesh_cpu={:.2}MB",
                planet.resolution,
                self.chunks.len(),
                mb
            ),
        );
    }

    pub fn render_prep_time(&self) -> Duration {
        self.frame_telemetry.render_prep_time
    }

    pub fn lod_coverage_time(&self) -> Duration {
        self.frame_telemetry.lod_coverage_time
    }

    pub fn chunk_streaming_time(&self) -> Duration {
        self.frame_telemetry.chunk_streaming_time
    }

    pub fn diagnostic_snapshot(
        &self,
        planet: &PlanetData,
        gameplay: &PlayerGameplayState,
    ) -> RuntimeSnapshot {
        let planet_stats = planet.runtime_stats();
        let terrain_stats = planet.terrain.cache_stats();
        let mut snapshot = RuntimeSnapshot {
            world: WorldCounters {
                edited_chunks: planet_stats.edited_chunks,
                mined_blocks: planet_stats.mined_blocks,
                placed_blocks: planet_stats.placed_blocks,
                dirty_chunks: planet_stats.dirty_chunks,
            },
            worldgen: WorldgenCounters {
                cached_columns: terrain_stats.cached_columns,
                cache_hits: terrain_stats.cache_hits,
                cache_misses: terrain_stats.cache_misses,
                compute_time: Duration::from_micros(terrain_stats.compute_micros),
            },
            streaming: self.frame_telemetry.streaming.clone(),
            lod: self.frame_telemetry.lod.clone(),
            mesh: self.frame_telemetry.mesh.clone(),
            gpu: self.frame_telemetry.gpu.clone(),
            dropped_items: gameplay.dropped_items.len(),
            inventory_open: gameplay.inventory_open,
        };

        snapshot.streaming.active_chunks = self.chunks.len();
        snapshot.streaming.load_queue = self.load_queue.len();
        snapshot.streaming.pending_chunk_jobs = self.pending_chunks.len();
        snapshot.lod.active_lods = self.lod_chunks.len();
        snapshot.lod.pending_lod_jobs = self.pending_lods.len();

        let mesh_totals = self.mesh_totals();
        snapshot.mesh.active_meshes = self.chunks.len() + self.lod_chunks.len();
        snapshot.mesh.vertices = mesh_totals.0;
        snapshot.mesh.indices = mesh_totals.1;
        snapshot.mesh.mesh_cpu_mb = mesh_totals.2;
        snapshot.gpu.active_buffers =
            (self.chunks.len() + self.lod_chunks.len() + self.animator.dying_chunks.len()) * 3;
        snapshot
    }

    fn mesh_totals(&self) -> (usize, usize, f32) {
        let mut vertices = 0usize;
        let mut indices = 0usize;
        for mesh in self
            .chunks
            .values()
            .chain(self.lod_chunks.values())
            .chain(self.animator.dying_chunks.values().map(|state| &state.mesh))
        {
            vertices += mesh.num_verts;
            indices += mesh.num_inds as usize;
        }
        let mb =
            ((vertices * std::mem::size_of::<Vertex>()) + (indices * 4)) as f32 / (1024.0 * 1024.0);
        (vertices, indices, mb)
    }

    // --- Render -------------------------------------------------------------

    pub fn render(
        &mut self,
        controller: &Controller,
        player: &Player,
        physics: &Physics,
        planet: &PlanetData,
        console: &Console,
        gameplay: &PlayerGameplayState,
        content: &CompiledContent,
    ) {
        let prep_start = Instant::now();
        self.update_console_mesh(console.height_fraction);
        self.update_gameplay_ui_mesh(controller, gameplay, content);
        self.update_block_break_feedback(planet, gameplay);
        self.update_dropped_item_mesh(gameplay, content);

        if controller.show_collisions {
            let (v, i) = MeshGen::generate_collision_debug(player.position, planet);
            self.queue
                .write_buffer(&self.collision_v_buf, 0, bytemuck::cast_slice(&v));
            self.queue
                .write_buffer(&self.collision_i_buf, 0, bytemuck::cast_slice(&i));
            self.collision_inds = i.len() as u32;
        } else {
            self.collision_inds = 0;
        }

        let out = match self.surface.get_current_texture() {
            Ok(o) => o,
            _ => return,
        };
        let view = out
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let mvp = controller.get_matrix(player, physics, w, h, &self.render_cfg);

        let atmosphere = AtmosphereUniform::from_config(&self.render_cfg.atmosphere);
        let sun_dir = atmosphere.sun_direction_vec3();
        let shadow_dist = 200.0f32;
        let proj_size = 60.0f32;
        let center = player.position;
        let mut sun_view = glam::Mat4::look_at_rh(center + sun_dir * shadow_dist, center, Vec3::Y);

        let shadow_map_size_f = self.render_cfg.shadow_map_size as f32;
        let texel_size = (2.0 * proj_size) / shadow_map_size_f;
        let shadow_origin = sun_view.transform_point3(center);
        let snap_x = (shadow_origin.x / texel_size).round() * texel_size - shadow_origin.x;
        let snap_y = (shadow_origin.y / texel_size).round() * texel_size - shadow_origin.y;
        sun_view = glam::Mat4::from_translation(Vec3::new(snap_x, snap_y, 0.0)) * sun_view;

        let sun_proj = glam::Mat4::orthographic_rh(
            -proj_size, proj_size, -proj_size, proj_size, -200.0, 500.0,
        );
        let light_vp = sun_proj * sun_view;

        let cam_pos = controller.get_camera_pos(player, physics);
        let frustum = Frustum::from_matrix(mvp);

        let cull_frustum_val;
        let cull_frustum: &Frustum = if controller.freeze_culling {
            if self.frozen_frustum.is_none() {
                self.frozen_frustum = Some(Frustum::from_matrix(mvp));
            }
            self.frozen_frustum.as_ref().unwrap()
        } else {
            self.frozen_frustum = None;
            cull_frustum_val = Frustum::from_matrix(mvp);
            &cull_frustum_val
        };

        let global_data = GlobalUniform {
            view_proj: mvp.to_cols_array(),
            light_view_proj: light_vp.to_cols_array(),
            cam_pos: [cam_pos.x, cam_pos.y, cam_pos.z, 1.0],
            atmosphere,
        };
        self.queue
            .write_buffer(&self.global_buf, 0, bytemuck::cast_slice(&[global_data]));
        self.queue.write_buffer(
            &self.shadow_global_buf,
            0,
            bytemuck::cast_slice(&[GlobalUniform {
                view_proj: light_vp.to_cols_array(),
                ..global_data
            }]),
        );

        let model_mat = player.get_model_matrix();
        self.queue.write_buffer(
            &self.local_buf_player,
            0,
            bytemuck::cast_slice(model_mat.as_ref()),
        );
        let r = planet.resolution as f32 / 2.0;
        self.queue.write_buffer(
            &self.local_buf_guide,
            0,
            bytemuck::cast_slice(glam::Mat4::from_scale(Vec3::splat(r)).as_ref()),
        );

        let now = std::time::Instant::now();
        let dying = self.animator.update_dying(now);
        for (key, alpha) in dying {
            if let Some(state) = self.animator.dying_chunks.get(&key) {
                let d = LocalUniform {
                    model: glam::Mat4::IDENTITY.to_cols_array(),
                    params: [alpha, 1.0, 0.0, 0.0],
                };
                self.queue
                    .write_buffer(&state.mesh.uniform_buf, 0, bytemuck::cast_slice(&[d]));
            }
        }

        let queue = &self.queue;
        let animator = &mut self.animator;
        for (key, mesh) in &self.lod_chunks {
            let alpha = animator.get_opacity(AnyKey::Lod(*key), now);
            if alpha < 1.0 || animator.spawning_chunks.contains_key(&AnyKey::Lod(*key)) {
                let d = LocalUniform {
                    model: glam::Mat4::IDENTITY.to_cols_array(),
                    params: [alpha.min(1.0), 0.0, 0.0, 0.0],
                };
                queue.write_buffer(&mesh.uniform_buf, 0, bytemuck::cast_slice(&[d]));
                if alpha >= 1.0 {
                    animator.spawning_chunks.remove(&AnyKey::Lod(*key));
                }
            }
        }
        for (key, mesh) in &self.chunks {
            let alpha = animator.get_opacity(AnyKey::Voxel(*key), now);
            if alpha < 1.0 || animator.spawning_chunks.contains_key(&AnyKey::Voxel(*key)) {
                let d = LocalUniform {
                    model: glam::Mat4::IDENTITY.to_cols_array(),
                    params: [alpha.min(1.0), 0.0, 0.0, 0.0],
                };
                queue.write_buffer(&mesh.uniform_buf, 0, bytemuck::cast_slice(&[d]));
                if alpha >= 1.0 {
                    animator.spawning_chunks.remove(&AnyKey::Voxel(*key));
                }
            }
        }
        self.frame_telemetry.render_prep_time += prep_start.elapsed();

        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let shadow_visible_chunks = self
            .chunks
            .values()
            .filter(|mesh| frustum.intersects_sphere(mesh.center, mesh.radius))
            .count();
        let shadow_visible_lods = self
            .lod_chunks
            .values()
            .filter(|mesh| frustum.intersects_sphere(mesh.center, mesh.radius))
            .count();
        let main_visible_chunks = self
            .chunks
            .values()
            .filter(|mesh| cull_frustum.intersects_sphere(mesh.center, mesh.radius))
            .count();
        let main_visible_lods = self
            .lod_chunks
            .values()
            .filter(|mesh| cull_frustum.intersects_sphere(mesh.center, mesh.radius))
            .count();
        let dying_visible = self
            .animator
            .dying_chunks
            .values()
            .filter(|state| frustum.intersects_sphere(state.mesh.center, state.mesh.radius))
            .count();

        // Shadow pass
        {
            let mut sp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shadow"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
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
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(atmosphere.clear_color()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let terrain_pipeline = if controller.is_wireframe {
                &self.pipeline_wire
            } else {
                &self.pipeline_fill
            };
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
                pass.set_pipeline(&self.pipeline_feedback);
                pass.set_bind_group(0, &self.global_bind, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.cursor_v_buf.slice(..));
                pass.set_index_buffer(self.cursor_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.cursor_inds, 0, 0..1);
            }
            if self.break_inds > 0 {
                pass.set_pipeline(&self.pipeline_feedback);
                pass.set_bind_group(0, &self.global_bind, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.break_v_buf.slice(..));
                pass.set_index_buffer(self.break_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.break_inds, 0, 0..1);
            }
            if self.drop_inds > 0 {
                pass.set_pipeline(&self.pipeline_fill);
                pass.set_bind_group(0, &self.global_bind, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.drop_v_buf.slice(..));
                pass.set_index_buffer(self.drop_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.drop_inds, 0, 0..1);
            }
            if self.console_inds > 0 {
                pass.set_pipeline(&self.pipeline_ui);
                pass.set_bind_group(0, &self.global_bind_identity, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.console_v_buf.slice(..));
                pass.set_index_buffer(self.console_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.console_inds, 0, 0..1);
            }
            if self.ui_inds > 0 {
                pass.set_pipeline(&self.pipeline_ui);
                pass.set_bind_group(0, &self.global_bind_identity, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.ui_v_buf.slice(..));
                pass.set_index_buffer(self.ui_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.ui_inds, 0, 0..1);
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
                let start_y = console_h - 40.0;
                let line_h = 20.0;
                for (i, (line, color)) in console.history.iter().rev().enumerate() {
                    let y = start_y - i as f32 * line_h;
                    if y < 0.0 {
                        break;
                    }
                    let mut buf = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
                    buf.set_size(&mut self.font_system, w, h);
                    buf.set_text(
                        &mut self.font_system,
                        line,
                        Attrs::new()
                            .family(Family::Monospace)
                            .color(glyphon::Color::rgb(
                                (color[0] * 255.0) as u8,
                                (color[1] * 255.0) as u8,
                                (color[2] * 255.0) as u8,
                            )),
                        Shaping::Advanced,
                    );
                    text_buffers.push((buf, 10.0, y));
                }
                let ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let cur = if (ms / 500) % 2 == 0 { "_" } else { " " };
                let mut ibuf = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
                ibuf.set_size(&mut self.font_system, w, h);
                ibuf.set_text(
                    &mut self.font_system,
                    &format!("> {}{}", console.input_buffer, cur),
                    Attrs::new()
                        .family(Family::Monospace)
                        .color(glyphon::Color::rgb(255, 255, 0)),
                    Shaping::Advanced,
                );
                text_buffers.push((ibuf, 10.0, console_h - 20.0));
            }

            let fps_text = format!("FPS: {}", self.current_fps);
            let mut fps_buf = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
            fps_buf.set_size(&mut self.font_system, w, h);
            fps_buf.set_text(
                &mut self.font_system,
                &fps_text,
                Attrs::new()
                    .family(Family::Monospace)
                    .color(glyphon::Color::rgb(255, 255, 255)),
                Shaping::Advanced,
            );
            text_buffers.push((fps_buf, 10.0, 5.0));

            self.push_gameplay_text(controller, gameplay, &mut text_buffers);

            for (buf, x, y) in &text_buffers {
                text_areas.push(TextArea {
                    buffer: buf,
                    left: *x,
                    top: *y,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: w as i32,
                        bottom: h as i32,
                    },
                    default_color: glyphon::Color::rgb(255, 255, 255),
                });
            }

            if !text_areas.is_empty() {
                let _ = self.text_renderer.prepare(
                    &self.device,
                    &self.queue,
                    &mut self.font_system,
                    &mut self.text_atlas,
                    Resolution {
                        width: self.config.width,
                        height: self.config.height,
                    },
                    text_areas,
                    &mut self.swash_cache,
                );
                let mut text_pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Text"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                let _ = self.text_renderer.render(&self.text_atlas, &mut text_pass);
            }
        }

        self.queue.submit(std::iter::once(enc.finish()));
        out.present();
        self.text_atlas.trim();
        let overlay_draws = (!controller.first_person) as u32
            + (self.collision_inds > 0) as u32
            + (self.cursor_inds > 0) as u32
            + (self.break_inds > 0) as u32
            + (self.drop_inds > 0) as u32
            + (self.console_inds > 0) as u32
            + (self.ui_inds > 0) as u32;
        self.frame_telemetry.gpu.draw_calls = (shadow_visible_chunks
            + shadow_visible_lods
            + main_visible_chunks
            + main_visible_lods
            + dying_visible) as u32
            + overlay_draws;
        self.frame_telemetry.gpu.visible_chunks = main_visible_chunks;
        self.frame_telemetry.gpu.visible_lods = main_visible_lods;
        self.frame_telemetry.gpu.active_buffers =
            (self.chunks.len() + self.lod_chunks.len() + self.animator.dying_chunks.len()) * 3;
    }

    // --- Private internals --------------------------------------------------

    fn update_gameplay_ui_mesh(
        &mut self,
        controller: &Controller,
        gameplay: &PlayerGameplayState,
        content: &CompiledContent,
    ) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let mut idx = 0u32;

        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let mut layout = GameplayUiLayout::new(w, h, &gameplay.inventory, gameplay.inventory_open);
        if gameplay.inventory_open {
            layout.add_hand_recipes(content.recipes.recipes_for_station(None));
        }
        self.push_crosshair(controller, gameplay, &mut verts, &mut inds, &mut idx);

        if !gameplay.inventory_open {
            for slot in &layout.hotbar_slots {
                let selected = slot.index == gameplay.selected_hotbar_slot;
                self.push_slot(
                    &mut verts,
                    &mut inds,
                    &mut idx,
                    slot.rect,
                    selected,
                    gameplay.inventory.slots()[slot.index].stack.map(|stack| {
                        (
                            self.item_color(stack.item, content),
                            gameplay.inventory_drag.source_slot == Some(slot.index),
                        )
                    }),
                );
            }
        }

        if gameplay.inventory_open {
            if let Some(panel) = layout.inventory_panel {
                self.push_panel(&mut verts, &mut inds, &mut idx, panel);
            }

            for slot in &layout.inventory_slots {
                self.push_slot(
                    &mut verts,
                    &mut inds,
                    &mut idx,
                    slot.rect,
                    slot.index == gameplay.selected_hotbar_slot,
                    gameplay.inventory.slots()[slot.index].stack.map(|stack| {
                        (
                            self.item_color(stack.item, content),
                            gameplay.inventory_drag.source_slot == Some(slot.index),
                        )
                    }),
                );
            }

            for slot in &layout.recipe_slots {
                let enabled = can_craft_hand_recipe(&gameplay.inventory, slot.recipe, content);
                let color = content
                    .recipes
                    .get(slot.recipe)
                    .map(|recipe| self.item_color(recipe.result_item, content))
                    .unwrap_or([0.45, 0.45, 0.45]);
                Self::push_rect_px(
                    &mut verts,
                    &mut inds,
                    &mut idx,
                    w,
                    h,
                    slot.rect.x - 2.0,
                    slot.rect.y - 2.0,
                    slot.rect.w + 4.0,
                    slot.rect.h + 4.0,
                    if enabled {
                        [0.36, 0.48, 0.28]
                    } else {
                        [0.16, 0.17, 0.18]
                    },
                );
                Self::push_rect_px(
                    &mut verts,
                    &mut inds,
                    &mut idx,
                    w,
                    h,
                    slot.rect.x,
                    slot.rect.y,
                    slot.rect.w,
                    slot.rect.h,
                    [0.075, 0.08, 0.085],
                );
                Self::push_rect_px(
                    &mut verts,
                    &mut inds,
                    &mut idx,
                    w,
                    h,
                    slot.rect.x + slot.rect.w * 0.28,
                    slot.rect.y + slot.rect.h * 0.22,
                    slot.rect.w * 0.44,
                    slot.rect.h * 0.56,
                    if enabled {
                        color
                    } else {
                        [color[0] * 0.35, color[1] * 0.35, color[2] * 0.35]
                    },
                );
            }
        }

        if let Some(stack) = gameplay.inventory_drag.stack {
            let color = self.item_color(stack.item, content);
            let size = layout.slot * 0.78;
            let rect = RectPx {
                x: controller.mouse_pos.x - size * 0.5,
                y: controller.mouse_pos.y - size * 0.5,
                w: size,
                h: size,
            };
            Self::push_rect_px(
                &mut verts,
                &mut inds,
                &mut idx,
                w,
                h,
                rect.x - 3.0 * layout.scale,
                rect.y - 3.0 * layout.scale,
                rect.w + 6.0 * layout.scale,
                rect.h + 6.0 * layout.scale,
                [0.02, 0.02, 0.02],
            );
            Self::push_rect_px(
                &mut verts, &mut inds, &mut idx, w, h, rect.x, rect.y, rect.w, rect.h, color,
            );
        }

        if !verts.is_empty() {
            self.queue
                .write_buffer(&self.ui_v_buf, 0, bytemuck::cast_slice(&verts));
            self.queue
                .write_buffer(&self.ui_i_buf, 0, bytemuck::cast_slice(&inds));
        }
        self.ui_inds = inds.len() as u32;
    }

    fn push_crosshair(
        &self,
        controller: &Controller,
        gameplay: &PlayerGameplayState,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
    ) {
        if !controller.first_person || gameplay.inventory_open {
            return;
        }
        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let scale = (w.min(h) / 720.0).clamp(0.75, 1.35);
        let cx = w * 0.5;
        let cy = h * 0.5;
        let thickness = (2.0 * scale).max(1.5);
        let gap = 6.0 * scale;
        let arm = 10.0 * scale;
        let active = gameplay.target.is_some();
        let mining = gameplay.mining.progress > 0.0;
        let color = if mining {
            [0.95, 0.78, 0.35]
        } else if active {
            [0.92, 0.9, 0.78]
        } else {
            [0.82, 0.86, 0.88]
        };
        let shadow = [0.015, 0.018, 0.02];
        for (dx, dy, ww, hh) in [
            (-gap - arm, -thickness * 0.5, arm, thickness),
            (gap, -thickness * 0.5, arm, thickness),
            (-thickness * 0.5, -gap - arm, thickness, arm),
            (-thickness * 0.5, gap, thickness, arm),
        ] {
            Self::push_rect_px(
                verts,
                inds,
                idx,
                w,
                h,
                cx + dx + scale,
                cy + dy + scale,
                ww,
                hh,
                shadow,
            );
            Self::push_rect_px(verts, inds, idx, w, h, cx + dx, cy + dy, ww, hh, color);
        }
    }

    fn push_panel(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        rect: RectPx,
    ) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;
        Self::push_rect_px(
            verts,
            inds,
            idx,
            w,
            h,
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            [0.055, 0.06, 0.065],
        );
        Self::push_rect_px(
            verts,
            inds,
            idx,
            w,
            h,
            rect.x + 4.0,
            rect.y + 4.0,
            rect.w - 8.0,
            rect.h - 8.0,
            [0.115, 0.12, 0.125],
        );
    }

    fn push_slot(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        rect: RectPx,
        selected: bool,
        item: Option<([f32; 3], bool)>,
    ) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let border = if selected {
            [0.95, 0.88, 0.52]
        } else {
            [0.19, 0.2, 0.21]
        };
        let inset = (rect.w * 0.1).max(3.0);
        Self::push_rect_px(
            verts,
            inds,
            idx,
            w,
            h,
            rect.x - 2.0,
            rect.y - 2.0,
            rect.w + 4.0,
            rect.h + 4.0,
            border,
        );
        Self::push_rect_px(
            verts,
            inds,
            idx,
            w,
            h,
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            [0.07, 0.075, 0.08],
        );
        Self::push_rect_px(
            verts,
            inds,
            idx,
            w,
            h,
            rect.x + inset,
            rect.y + inset,
            rect.w - inset * 2.0,
            rect.h - inset * 2.0,
            [0.135, 0.14, 0.145],
        );
        if let Some((color, hidden_by_drag)) = item {
            if hidden_by_drag {
                return;
            }
            let item_inset = rect.w * 0.26;
            Self::push_rect_px(
                verts,
                inds,
                idx,
                w,
                h,
                rect.x + item_inset,
                rect.y + item_inset * 0.85,
                rect.w - item_inset * 2.0,
                rect.h - item_inset * 1.75,
                color,
            );
        }
    }

    fn update_block_break_feedback(&mut self, planet: &PlanetData, gameplay: &PlayerGameplayState) {
        let progress = gameplay.mining.progress;
        let Some(id) = gameplay.mining.target else {
            self.break_inds = 0;
            return;
        };

        let mesh = block_break_mesh(planet, id, progress, BlockBreakStyle::default());
        if mesh.indices.is_empty() {
            self.break_inds = 0;
            return;
        }

        self.queue
            .write_buffer(&self.break_v_buf, 0, bytemuck::cast_slice(&mesh.vertices));
        self.queue
            .write_buffer(&self.break_i_buf, 0, bytemuck::cast_slice(&mesh.indices));
        self.break_inds = mesh.indices.len() as u32;
    }

    fn update_dropped_item_mesh(
        &mut self,
        gameplay: &PlayerGameplayState,
        content: &CompiledContent,
    ) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let mut idx = 0u32;
        for drop in gameplay.dropped_items.iter().take(128) {
            let color = self.item_color(drop.stack.item, content);
            Self::push_cube(&mut verts, &mut inds, &mut idx, drop.position, 0.28, color);
        }
        if !verts.is_empty() {
            self.queue
                .write_buffer(&self.drop_v_buf, 0, bytemuck::cast_slice(&verts));
            self.queue
                .write_buffer(&self.drop_i_buf, 0, bytemuck::cast_slice(&inds));
        }
        self.drop_inds = inds.len() as u32;
    }

    fn push_gameplay_text(
        &mut self,
        controller: &Controller,
        gameplay: &PlayerGameplayState,
        text_buffers: &mut Vec<(Buffer, f32, f32)>,
    ) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let layout = GameplayUiLayout::new(w, h, &gameplay.inventory, gameplay.inventory_open);

        if !gameplay.inventory_open {
            for slot in &layout.hotbar_slots {
                if gameplay.inventory_drag.source_slot == Some(slot.index) {
                    continue;
                }
                if let Some(stack) = gameplay.inventory.slots()[slot.index].stack {
                    if stack.count > 1 {
                        let x = slot.rect.x + slot.rect.w - 18.0 * layout.scale;
                        let y = slot.rect.y + slot.rect.h - 20.0 * layout.scale;
                        self.push_text(
                            text_buffers,
                            &stack.count.to_string(),
                            x,
                            y,
                            15.0 * layout.scale,
                        );
                    }
                }
            }
        }

        if gameplay.inventory_open {
            if let Some(panel) = layout.inventory_panel {
                self.push_text(
                    text_buffers,
                    "Backpack",
                    panel.x + 16.0 * layout.scale,
                    panel.y + 9.0 * layout.scale,
                    16.0 * layout.scale,
                );
                self.push_text(
                    text_buffers,
                    "Hotbar",
                    panel.x + 16.0 * layout.scale,
                    panel.y + panel.h - layout.slot - 18.0 * layout.scale,
                    13.0 * layout.scale,
                );
            }
            for slot in &layout.inventory_slots {
                if gameplay.inventory_drag.source_slot == Some(slot.index) {
                    continue;
                }
                let Some(stack) = gameplay.inventory.slots()[slot.index].stack else {
                    continue;
                };
                if stack.count <= 1 {
                    continue;
                }
                let x = slot.rect.x + slot.rect.w - 18.0 * layout.scale;
                let y = slot.rect.y + slot.rect.h - 20.0 * layout.scale;
                self.push_text(
                    text_buffers,
                    &stack.count.to_string(),
                    x,
                    y,
                    15.0 * layout.scale,
                );
            }
        }

        if let Some(stack) = gameplay.inventory_drag.stack {
            if stack.count > 1 {
                self.push_text(
                    text_buffers,
                    &stack.count.to_string(),
                    controller.mouse_pos.x + 8.0 * layout.scale,
                    controller.mouse_pos.y + 8.0 * layout.scale,
                    15.0 * layout.scale,
                );
            }
        }

        if gameplay.pickup_notice_timer > 0.0 {
            self.push_text(text_buffers, "Picked up", w * 0.5 - 42.0, h - 92.0, 16.0);
        }
        if gameplay.placement_blocked_timer > 0.0 {
            self.push_text(text_buffers, "Cannot place", w * 0.5 - 48.0, h * 0.58, 16.0);
        }
    }

    fn push_text(
        &mut self,
        text_buffers: &mut Vec<(Buffer, f32, f32)>,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
    ) {
        let mut buf = Buffer::new(&mut self.font_system, Metrics::new(size, size + 4.0));
        buf.set_size(
            &mut self.font_system,
            self.config.width as f32,
            self.config.height as f32,
        );
        buf.set_text(
            &mut self.font_system,
            text,
            Attrs::new()
                .family(Family::Monospace)
                .color(glyphon::Color::rgb(255, 255, 255)),
            Shaping::Advanced,
        );
        text_buffers.push((buf, x, y));
    }

    fn item_color(&self, item: ItemId, content: &CompiledContent) -> [f32; 3] {
        let Some(item) = content.items.get(item) else {
            return [0.75, 0.75, 0.75];
        };
        match item.kind {
            CompiledItemKind::Block { block } => self
                .block_content
                .block_render(block)
                .map(|render| render.color)
                .unwrap_or([0.75, 0.75, 0.75]),
            CompiledItemKind::Placeable { .. } => [0.95, 0.72, 0.35],
            CompiledItemKind::Tool { .. } => [0.72, 0.78, 0.85],
            CompiledItemKind::Armor => [0.62, 0.72, 0.9],
            CompiledItemKind::Food => [0.72, 0.9, 0.48],
            CompiledItemKind::Resource => [0.72, 0.68, 0.58],
        }
    }

    fn push_rect_px(
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        screen_w: f32,
        screen_h: f32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 3],
    ) {
        let x0 = x / screen_w * 2.0 - 1.0;
        let x1 = (x + width) / screen_w * 2.0 - 1.0;
        let y0 = 1.0 - y / screen_h * 2.0;
        let y1 = 1.0 - (y + height) / screen_h * 2.0;
        let normal = [0.0, 0.0, 1.0];
        let base = *idx;
        verts.extend_from_slice(&[
            Vertex::untextured([x0, y0, 0.0], color, normal),
            Vertex::untextured([x1, y0, 0.0], color, normal),
            Vertex::untextured([x1, y1, 0.0], color, normal),
            Vertex::untextured([x0, y1, 0.0], color, normal),
        ]);
        inds.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
        *idx += 4;
    }

    fn push_cube(
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        center: Vec3,
        size: f32,
        color: [f32; 3],
    ) {
        let h = size * 0.5;
        let p = [
            center + Vec3::new(-h, -h, -h),
            center + Vec3::new(h, -h, -h),
            center + Vec3::new(h, h, -h),
            center + Vec3::new(-h, h, -h),
            center + Vec3::new(-h, -h, h),
            center + Vec3::new(h, -h, h),
            center + Vec3::new(h, h, h),
            center + Vec3::new(-h, h, h),
        ];
        let faces = [
            ([0, 1, 2, 3], [0.0, 0.0, -1.0]),
            ([5, 4, 7, 6], [0.0, 0.0, 1.0]),
            ([4, 0, 3, 7], [-1.0, 0.0, 0.0]),
            ([1, 5, 6, 2], [1.0, 0.0, 0.0]),
            ([3, 2, 6, 7], [0.0, 1.0, 0.0]),
            ([4, 5, 1, 0], [0.0, -1.0, 0.0]),
        ];
        for (face, normal) in faces {
            let base = *idx;
            for i in face {
                verts.push(Vertex::untextured(p[i].to_array(), color, normal));
            }
            inds.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
            *idx += 4;
        }
    }

    fn update_console_mesh(&mut self, t: f32) {
        if t <= 0.001 {
            self.console_inds = 0;
            return;
        }
        let bottom_y = 1.0 - t;
        let color = [0.1, 0.1, 0.15];
        let normal = [0.0, 0.0, 1.0];
        let verts = vec![
            Vertex::untextured([-1.0, 1.0, 0.0], color, normal),
            Vertex::untextured([1.0, 1.0, 0.0], color, normal),
            Vertex::untextured([-1.0, bottom_y, 0.0], color, normal),
            Vertex::untextured([1.0, bottom_y, 0.0], color, normal),
        ];
        let inds = vec![0u32, 2, 1, 1, 2, 3];
        self.queue
            .write_buffer(&self.console_v_buf, 0, bytemuck::cast_slice(&verts));
        self.queue
            .write_buffer(&self.console_i_buf, 0, bytemuck::cast_slice(&inds));
        self.console_inds = 6;
    }

    fn process_quadtree(
        &self,
        face: u8,
        x: u32,
        y: u32,
        size: u32,
        cam_pos: Vec3,
        planet: &PlanetData,
        player_id: Option<BlockId>,
        voxels: &mut HashSet<ChunkKey>,
        lods: &mut HashSet<LodKey>,
    ) {
        if x >= planet.resolution || y >= planet.resolution {
            return;
        }
        let cu = (x + size / 2).min(planet.resolution - 1);
        let cv = (y + size / 2).min(planet.resolution - 1);
        let h = planet.geometry.surface_layer();
        let world_pos = CoordSystem::get_vertex_pos(face, cu, cv, h, planet.geometry);

        let mut dist = world_pos.distance(cam_pos);
        if let Some(pid) = player_id {
            if pid.face == face && pid.u >= x && pid.u < x + size && pid.v >= y && pid.v < y + size
            {
                dist = 0.0;
            }
        }

        let node_r = (size as f32 * CoordSystem::get_layer_radius(h, planet.geometry))
            / planet.resolution as f32;
        let lod_factor = if size <= CHUNK_SIZE {
            18.0
        } else if size <= CHUNK_SIZE * 2 {
            12.0
        } else if size <= CHUNK_SIZE * 4 {
            7.0
        } else if size <= CHUNK_SIZE * 8 {
            5.0
        } else {
            4.0
        };

        let is_smallest = size <= CHUNK_SIZE;
        if dist < node_r * lod_factor && !is_smallest {
            let half = size / 2;
            self.process_quadtree(face, x, y, half, cam_pos, planet, player_id, voxels, lods);
            self.process_quadtree(
                face,
                x + half,
                y,
                half,
                cam_pos,
                planet,
                player_id,
                voxels,
                lods,
            );
            self.process_quadtree(
                face,
                x,
                y + half,
                half,
                cam_pos,
                planet,
                player_id,
                voxels,
                lods,
            );
            self.process_quadtree(
                face,
                x + half,
                y + half,
                half,
                cam_pos,
                planet,
                player_id,
                voxels,
                lods,
            );
        } else if size <= CHUNK_SIZE {
            let key = ChunkKey {
                face,
                u_idx: x / CHUNK_SIZE,
                v_idx: y / CHUNK_SIZE,
            };
            if key.u_idx * CHUNK_SIZE < planet.resolution
                && key.v_idx * CHUNK_SIZE < planet.resolution
            {
                voxels.insert(key);
            }
        } else {
            lods.insert(LodKey { face, x, y, size });
        }
    }

    fn prioritized_chunk_split(
        &self,
        keys: HashSet<ChunkKey>,
        player_pos: Vec3,
        planet: &PlanetData,
        limit: usize,
    ) -> (HashSet<ChunkKey>, Vec<ChunkKey>) {
        let mut ranked: Vec<(ChunkKey, f32)> = keys
            .into_iter()
            .map(|key| (key, Self::chunk_distance_squared(&key, player_pos, planet)))
            .collect();
        ranked.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let keep = limit.min(self.lod_cfg.max_active_chunks).min(ranked.len());
        let required = ranked[..keep].iter().map(|(key, _)| *key).collect();
        let dropped = ranked[keep..].iter().map(|(key, _)| *key).collect();
        (required, dropped)
    }

    fn prioritized_lods(
        &self,
        keys: HashSet<LodKey>,
        player_pos: Vec3,
        planet: &PlanetData,
        limit: usize,
    ) -> HashSet<LodKey> {
        let mut ranked: Vec<(LodKey, f32)> = keys
            .into_iter()
            .map(|key| (key, Self::lod_distance_squared(&key, player_pos, planet)))
            .collect();
        ranked.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(limit.min(self.lod_cfg.max_active_lods));
        ranked.into_iter().map(|(key, _)| key).collect()
    }

    fn add_chunk_fallback_lods(
        &self,
        chunks: impl Iterator<Item = ChunkKey>,
        lods: &mut HashSet<LodKey>,
        player_pos: Vec3,
        planet: &PlanetData,
    ) {
        let mut ranked: Vec<(LodKey, f32)> = chunks
            .filter_map(|chunk| Self::chunk_fallback_lod(chunk, planet))
            .map(|key| (key, Self::lod_distance_squared(&key, player_pos, planet)))
            .collect();
        ranked.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        for (key, _) in ranked {
            lods.insert(key);
        }
        if lods.len() > self.lod_cfg.max_required_lods {
            let mut ranked_lods: Vec<(LodKey, f32)> = lods
                .iter()
                .copied()
                .map(|key| (key, Self::lod_distance_squared(&key, player_pos, planet)))
                .collect();
            ranked_lods.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            for (key, _) in ranked_lods
                .into_iter()
                .take(lods.len() - self.lod_cfg.max_required_lods)
            {
                lods.remove(&key);
            }
        }
    }

    fn chunk_fallback_lod(chunk: ChunkKey, planet: &PlanetData) -> Option<LodKey> {
        let size = (CHUNK_SIZE * 4).min(planet.resolution.next_power_of_two());
        if size <= CHUNK_SIZE {
            return None;
        }
        let x = (chunk.u_idx * CHUNK_SIZE / size) * size;
        let y = (chunk.v_idx * CHUNK_SIZE / size) * size;
        if x >= planet.resolution || y >= planet.resolution {
            return None;
        }
        Some(LodKey {
            face: chunk.face,
            x,
            y,
            size,
        })
    }

    fn limit_lod_pressure(&mut self, player_pos: Vec3, planet: &PlanetData) {
        if self.lod_chunks.len() <= self.lod_cfg.max_active_lods {
            return;
        }
        let mut ranked: Vec<(LodKey, f32)> = self
            .lod_chunks
            .keys()
            .copied()
            .map(|key| (key, Self::lod_distance_squared(&key, player_pos, planet)))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let remove_count = self.lod_chunks.len() - self.lod_cfg.max_active_lods;
        for (key, _) in ranked.into_iter().take(remove_count) {
            if let Some(mesh) = self.lod_chunks.remove(&key) {
                self.animator.retire(AnyKey::Lod(key), mesh);
            }
        }
    }

    fn chunk_distance_squared(key: &ChunkKey, player_pos: Vec3, planet: &PlanetData) -> f32 {
        CoordSystem::get_vertex_pos(
            key.face,
            key.u_idx * CHUNK_SIZE + CHUNK_SIZE / 2,
            key.v_idx * CHUNK_SIZE + CHUNK_SIZE / 2,
            planet.geometry.surface_layer(),
            planet.geometry,
        )
        .distance_squared(player_pos)
    }

    fn lod_distance_squared(key: &LodKey, player_pos: Vec3, planet: &PlanetData) -> f32 {
        CoordSystem::get_vertex_pos(
            key.face,
            key.x
                .saturating_add(key.size / 2)
                .min(planet.resolution - 1),
            key.y
                .saturating_add(key.size / 2)
                .min(planet.resolution - 1),
            planet.geometry.surface_layer(),
            planet.geometry,
        )
        .distance_squared(player_pos)
    }

    fn upload_lod_buffer(&mut self, key: LodKey, v: Vec<Vertex>, i: Vec<u32>) {
        let upload_start = Instant::now();
        self.record_upload(v.len(), i.len());
        let v_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&v),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        let i_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&i),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });
        let uniform_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("LOD Uniform"),
                contents: bytemuck::cast_slice(&[LocalUniform {
                    model: glam::Mat4::IDENTITY.to_cols_array(),
                    params: [1.0, 0.0, 0.0, 0.0],
                }]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.local_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
            label: None,
        });
        let (center, radius) = Self::bounds_from_verts(&v);
        self.lod_chunks.insert(
            key,
            ChunkMesh {
                v_buf,
                i_buf,
                num_inds: i.len() as u32,
                num_verts: v.len(),
                uniform_buf,
                bind_group,
                center,
                radius,
            },
        );
        self.frame_telemetry.gpu.upload_time += upload_start.elapsed();
    }

    fn upload_chunk_buffers(&mut self, key: ChunkKey, v: Vec<Vertex>, i: Vec<u32>) {
        let upload_start = Instant::now();
        self.record_upload(v.len(), i.len());
        let v_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&v),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        let i_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&i),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });
        let uniform_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Uniform"),
                contents: bytemuck::cast_slice(&[LocalUniform {
                    model: glam::Mat4::IDENTITY.to_cols_array(),
                    params: [1.0, 0.0, 0.0, 0.0],
                }]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.local_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
            label: None,
        });
        let (center, radius) = Self::bounds_from_verts(&v);
        self.chunks.insert(
            key,
            ChunkMesh {
                v_buf,
                i_buf,
                num_inds: i.len() as u32,
                num_verts: v.len(),
                uniform_buf,
                bind_group,
                center,
                radius,
            },
        );
        self.frame_telemetry.gpu.upload_time += upload_start.elapsed();
    }

    fn bounds_from_verts(v: &[Vertex]) -> (Vec3, f32) {
        if v.is_empty() {
            return (Vec3::ZERO, 0.0);
        }
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for vert in v {
            let p = Vec3::from_array(vert.pos);
            min = min.min(p);
            max = max.max(p);
        }
        let center = (min + max) * 0.5;
        (center, min.distance(max) * 0.5)
    }

    fn process_load_queue(&mut self, _player_pos: Vec3, planet: &PlanetData) {
        let mut uploads = 0usize;
        while uploads < self.lod_cfg.chunk_uploads_per_frame
            && self.frame_telemetry.gpu.uploads < self.lod_cfg.max_gpu_uploads_per_frame
        {
            let Ok((job, v, i)) = self.mesh_rx.try_recv() else {
                break;
            };
            self.pending_chunks.remove(&job.key);
            self.record_mesh_job(job.duration, job.vertices, job.indices, MeshJobKind::Chunk);
            if !v.is_empty() {
                self.upload_chunk_buffers(job.key, v, i);
                self.frame_telemetry.streaming.chunks_uploaded += 1;
                uploads += 1;
            } else {
                self.frame_telemetry.streaming.empty_chunks += 1;
            }
        }
        if self.load_queue.is_empty()
            || self.pending_chunks.len() >= self.lod_cfg.max_pending_chunk_jobs
        {
            return;
        }
        for _ in 0..self.lod_cfg.chunk_jobs_per_frame {
            if self.pending_chunks.len() >= self.lod_cfg.max_pending_chunk_jobs {
                break;
            }
            if let Some(key) = self.load_queue.pop() {
                if self.chunks.contains_key(&key) || self.pending_chunks.contains(&key) {
                    continue;
                }
                self.pending_chunks.insert(key);
                let p = planet.clone();
                let tx = self.mesh_tx.clone();
                let blocks = self.block_content.clone();
                std::thread::spawn(move || {
                    let start = Instant::now();
                    let (v, i) = MeshGen::build_chunk(key, &p, &blocks);
                    let job = MeshJobResult {
                        key,
                        vertices: v.len(),
                        indices: i.len(),
                        duration: start.elapsed(),
                    };
                    let _ = tx.send((job, v, i));
                });
                self.frame_telemetry.streaming.chunk_jobs_started += 1;
            } else {
                break;
            }
        }
    }

    fn record_upload(&mut self, vertices: usize, indices: usize) {
        self.frame_telemetry.gpu.uploads += 1;
        self.frame_telemetry.gpu.upload_vertices += vertices;
        self.frame_telemetry.gpu.upload_indices += indices;
    }

    fn record_mesh_job(
        &mut self,
        duration: Duration,
        vertices: usize,
        indices: usize,
        kind: MeshJobKind,
    ) {
        match kind {
            MeshJobKind::Chunk => self.frame_telemetry.mesh.chunk_jobs_completed += 1,
            MeshJobKind::Lod => self.frame_telemetry.mesh.lod_jobs_completed += 1,
            MeshJobKind::Remesh => self.frame_telemetry.mesh.remeshes += 1,
        }
        self.frame_telemetry.mesh.total_job_time += duration;
        self.frame_telemetry.mesh.max_job_time =
            self.frame_telemetry.mesh.max_job_time.max(duration);
        self.frame_telemetry.mesh.vertices += vertices;
        self.frame_telemetry.mesh.indices += indices;
    }
}
