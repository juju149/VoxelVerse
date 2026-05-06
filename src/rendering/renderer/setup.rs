use super::{GlobalUniform, LocalUniform, Renderer};
use crate::diagnostics::SystemDiagnostics;
use crate::meshing::MeshGen;
use crate::rendering::lod_animation::LodAnimator;
use crate::rendering::types::Vertex;
use glyphon::{FontSystem, SwashCache, TextAtlas, TextRenderer as GlyphRenderer};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::channel;
use wgpu::util::DeviceExt;
use wgpu::PresentMode;
use winit::window::Window;

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a Window) -> Self {
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

        // log GPU info
        SystemDiagnostics::log_gpu(&adapter.get_info());

        let target_buffer_size: u64 = 8 * 1024 * 1024 * 1024;
        let mut limits = adapter.limits();
        // we are requiring a maximum of 8gb but we take as much as the platform is capable of
        limits.max_buffer_size = target_buffer_size.min(limits.max_buffer_size);

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

        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();

        let available_present_modes = surface.get_capabilities(&adapter).present_modes;

        config.present_mode = [
            // presentation preference order.
            PresentMode::Immediate,
            PresentMode::Mailbox,
        ]
        .into_iter()
        .find(|&mode| available_present_modes.contains(&mode))
        .unwrap_or(PresentMode::Fifo);

        surface.configure(&device, &config);

        let font_system = FontSystem::new();

        let swash_cache = SwashCache::new();
        let mut text_atlas = TextAtlas::new(&device, &queue, config.format);
        let text_renderer = GlyphRenderer::new(
            &mut text_atlas,
            &device,
            wgpu::MultisampleState::default(),
            None,
        );

        let shadow_size = 4096;
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
                // 1: shadow Texture
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
                // 2: shadow Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
            label: Some("global_layout"),
        });

        let local_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("local_layout"),
        });

        // --- BUFFERS ---
        let global_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Global Uniform"),
            size: 160,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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
            ],
            label: None,
        });

        // --- SHADOW PASS RESOURCES ---
        // shadow uniform buffer
        let shadow_global_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Global Uniform"),
            size: 160,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // dummy depth tex (1x1)
        let dummy_depth_tex = device.create_texture(&wgpu::TextureDescriptor {
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
        let dummy_depth_view = dummy_depth_tex.create_view(&wgpu::TextureViewDescriptor::default());

        // shadow pass bind group
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
            ],
        });

        let identity_mat = glam::Mat4::IDENTITY;
        let default_local = LocalUniform {
            model: identity_mat.to_cols_array(),
            params: [1.0, 0.0, 1.0, 0.0],
        };

        // console buffers
        let console_v_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Console V"),
            size: 1024,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let console_i_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Console I"),
            size: 1024,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let local_buf_identity = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Identity Uniform"),
            contents: bytemuck::cast_slice(&[default_local]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let local_bind_identity = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &local_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: local_buf_identity.as_entire_binding(),
            }],
            label: None,
        });

        // player uniform
        let local_buf_player = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Player Uniform"),
            contents: bytemuck::cast_slice(&[default_local]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let local_bind_player = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &local_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: local_buf_player.as_entire_binding(),
            }],
            label: None,
        });

        // --- PIPELINES ---
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&global_layout, &local_layout],
            push_constant_ranges: &[],
        });

        let pipeline_shadow = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
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
                    ],
                }],
            },
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
            &config,
            &layout,
            &shader,
            wgpu::PrimitiveTopology::TriangleList,
            false,
        );
        let pipeline_wire = Self::create_pipeline(
            &device,
            &config,
            &layout,
            &shader,
            wgpu::PrimitiveTopology::TriangleList,
            true,
        );
        let pipeline_line = Self::create_pipeline(
            &device,
            &config,
            &layout,
            &shader,
            wgpu::PrimitiveTopology::LineList,
            false,
        );
        let depth = Self::mk_depth(&device, &config);

        // --- UI PIPELINE ---
        let pipeline_ui = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
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
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });

        // --- MESHES ---
        let (pv, pi) = MeshGen::generate_cylinder(0.4, 1.8, 16);
        let player_v_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&pv),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let player_i_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&pi),
            usage: wgpu::BufferUsages::INDEX,
        });

        let (cv, ci) = MeshGen::generate_crosshair();
        let cross_v_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&cv),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let cross_i_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&ci),
            usage: wgpu::BufferUsages::INDEX,
        });

        let cursor_v_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cursor V"),
            size: 4096,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let cursor_i_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cursor I"),
            size: 4096,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let collision_v_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Collision V"),
            size: 65536,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let collision_i_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Collision I"),
            size: 65536,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // global identity
        let identity_global_data = GlobalUniform {
            view_proj: identity_mat.to_cols_array(),
            light_view_proj: identity_mat.to_cols_array(),
            cam_pos: [0.0, 0.0, 0.0, 0.0],
            sun_dir: [0.0, 1.0, 0.0, 0.0],
        };

        let global_buf_identity = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Identity Buffer"),
            contents: bytemuck::cast_slice(&[identity_global_data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let global_bind_identity = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &global_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: global_buf_identity.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
            ],
            label: Some("Identity Bind Group"),
        });

        let (mesh_tx, mesh_rx) = channel();
        let (lod_tx, lod_rx) = channel();

        Self {
            window,
            surface,
            device,
            queue,
            config,
            pipeline_fill,
            pipeline_wire,
            pipeline_line,
            chunks: HashMap::new(),
            lod_chunks: HashMap::new(),
            global_buf,
            global_bind,
            local_bind_identity,
            local_buf_player,
            local_bind_player,
            depth,

            font_system,
            swash_cache,
            text_atlas,
            text_renderer,
            shadow_view,
            pipeline_shadow,
            shadow_global_buf,
            shadow_global_bind,
            collision_v_buf,
            collision_i_buf,
            collision_inds: 0,
            frozen_frustum: None,
            player_v_buf,
            player_i_buf,
            player_inds: pi.len() as u32,
            pipeline_ui,
            console_v_buf,
            console_i_buf,
            console_inds: 0,
            cross_v_buf,
            cross_i_buf,
            cross_inds: ci.len() as u32,
            global_bind_identity,
            cursor_v_buf,
            cursor_i_buf,
            cursor_inds: 0,
            animator: LodAnimator::new(),
            local_layout,
            load_queue: Vec::new(),
            player_chunk_pos: None,
            mesh_tx,
            mesh_rx,
            pending_chunks: HashSet::new(),
            lod_tx,
            lod_rx,
            pending_lods: HashSet::new(),

            last_fps_time: std::time::Instant::now(),
            frame_count: 0,
            current_fps: 0,
        }
    }
}
