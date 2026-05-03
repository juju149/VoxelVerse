use glyphon::{FontSystem, SwashCache, TextAtlas, TextRenderer as GlyphRenderer};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::mpsc::channel;
use wgpu::util::DeviceExt;
use wgpu::PresentMode;
use winit::window::Window;

use vv_config::EngineConfig;
use vv_diagnostics::{emit, DiagnosticConfig, LogDomain, LogLevel};
use vv_mesh::{MeshGen, Vertex};
use vv_registry::CompiledContent;

use crate::{
    atmosphere::AtmosphereUniform, shader_source, sky_state::SkyState, AnyKey, ChunkMesh,
    LodAnimator,
};

use super::types::{GlobalUniform, LocalUniform, RendererFrameTelemetry};
use super::visual_content::{build_block_visual_palette, build_block_visuals};
use super::Renderer;

impl<'a> Renderer<'a> {
    pub async fn new(
        window: &'a Window,
        cfg: &EngineConfig,
        content: &CompiledContent,
        _assets_root: &Path,
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
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
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
        let block_visuals = build_block_visuals(content);
        let block_visual_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Visual Params"),
            contents: bytemuck::cast_slice(&block_visuals),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let block_visual_palette = build_block_visual_palette(content);
        let block_visual_palette_buf =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Block Visual Palette"),
                contents: bytemuck::cast_slice(&block_visual_palette),
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
                    resource: block_visual_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: block_visual_palette_buf.as_entire_binding(),
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
                    resource: block_visual_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: block_visual_palette_buf.as_entire_binding(),
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
            label: Some("voxelverse_main_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source::main_shader_source().into()),
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
        let pipeline_sky = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sky Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_sky",
                buffers: &[], // Fullscreen triangle generated in shader from vertex_index
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_sky",
                targets: &[Some(surf_cfg.format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false, // Sky must not occlude terrain
                depth_compare: wgpu::CompareFunction::Always, // Drawn first, always passes
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });
        let depth = Self::mk_depth(&device, &surf_cfg);

        let ui_renderer = crate::ui::UiRenderer::new(&device, surf_cfg.format);

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
        let drop_v_buf = mk_dyn_vbuf("Dropped Items V", 1_048_576);
        let drop_i_buf = mk_dyn_ibuf("Dropped Items I", 1_048_576);

        let identity_global = GlobalUniform {
            view_proj: identity_mat.to_cols_array(),
            light_view_proj: identity_mat.to_cols_array(),
            cam_pos: [0.0; 4],
            atmosphere: AtmosphereUniform::from_config(&cfg.render.atmosphere),
            inv_view_proj: identity_mat.to_cols_array(),
            planet: [1.0, 80_000.0, 0.0, 0.0],
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
                    resource: block_visual_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: block_visual_palette_buf.as_entire_binding(),
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
            ui_renderer,
            animator: LodAnimator::new(cfg.render.lod_fade_duration),
            local_layout,
            pipeline_fill,
            pipeline_wire,
            pipeline_line,
            pipeline_feedback,
            pipeline_sky,
            chunks: HashMap::new(),
            lod_chunks: HashMap::new(),
            global_buf,
            global_bind,
            _block_visual_buf: block_visual_buf,
            _block_visual_palette_buf: block_visual_palette_buf,
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
            sky_state: SkyState::new(cfg.day_cycle.clone()),
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
}
