use super::{GlobalUniform, LocalUniform, Renderer};
use vv_pack_compiler::TextureRegistry;
use vv_diagnostics::{FrameStats, SystemDiagnostics};
use vv_meshing::MeshGen;
use crate::lod_animation::LodAnimator;
use crate::perf_profile::{PerfProfile, PerfTier};
use crate::texture_atlas::TextureAtlas;
use crate::types::Vertex;
use vv_meshing::{MeshScheduler, SchedulerStats};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::channel;
use vv_pack_compiler::RenderRegistry;
use wgpu::util::DeviceExt;
use wgpu::PresentMode;
use winit::window::Window;

impl<'a> Renderer<'a> {
    pub async fn new(
        window: &'a Window,
        textures: &TextureRegistry,
        material_colors: &[[f32; 4]],
        render_registry: &RenderRegistry,
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

        // log GPU info
        let adapter_info = adapter.get_info();
        SystemDiagnostics::log_gpu(&adapter_info);

        // Detect hardware tier and derive every render-side knob from it.
        // Honours the VV_PERF env override.
        let perf_tier = PerfTier::resolve(&adapter_info);
        let perf = PerfProfile::for_tier(perf_tier);
        perf.print();

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

        let text_resources = Self::create_text_resources(&device, &queue, config.format);

        let shadow_map = Self::create_shadow_map(&device, perf.shadow_map_size);

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

        // --- MATERIAL TEXTURES BIND GROUP LAYOUT (group 2) ---
        let atlas_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("atlas_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
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

        let atlas = TextureAtlas::new(&device, &queue, textures);

        // Per-atlas-layer flat color buffer for the color-only debug toggle.
        // Always at least one entry so wgpu doesn't reject a zero-sized binding.
        let mut color_data: Vec<[f32; 4]> = if material_colors.is_empty() {
            vec![[1.0, 1.0, 1.0, 1.0]]
        } else {
            material_colors.to_vec()
        };
        // Pad to a multiple of vec4 stride (already is — kept explicit).
        if color_data.is_empty() {
            color_data.push([1.0, 1.0, 1.0, 1.0]);
        }
        let material_color_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Flat Colors"),
            contents: bytemuck::cast_slice(&color_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let atlas_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atlas Bind Group"),
            layout: &atlas_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas.albedo_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas.normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&atlas.roughness_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&atlas.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: material_color_buf.as_entire_binding(),
                },
            ],
        });

        // --- BUFFERS ---
        let global_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Global Uniform"),
            size: std::mem::size_of::<GlobalUniform>() as u64,
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
                    resource: wgpu::BindingResource::TextureView(&shadow_map.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&shadow_map.sampler),
                },
            ],
            label: None,
        });

        let shadow_pass =
            Self::create_shadow_pass_resources(&device, &global_layout, &shadow_map.sampler);

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
        let hotbar_v_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hotbar V"),
            size: 65536,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let hotbar_i_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hotbar I"),
            size: 65536,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Inventory modal needs more vertices than the hotbar (45 slots, 3
        // panels, scrim, buttons, drag ghost). 512 KiB / 32 B per vertex is
        // plenty for the modal frame even at 4K with the maxed-out grid.
        let inventory_v_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Inventory V"),
            size: 512 * 1024,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let inventory_i_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Inventory I"),
            size: 512 * 1024,
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
        let terrain_shaders = Self::create_shader_pair(
            &device,
            render_registry,
            "core:render/techniques/terrain/terrain_opaque",
        );
        let ui_shaders = Self::create_shader_pair(
            &device,
            render_registry,
            "core:render/techniques/ui/ui_flat",
        );
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&global_layout, &local_layout, &atlas_layout],
            push_constant_ranges: &[],
        });

        let pipeline_shadow = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &terrain_shaders.vertex,
                entry_point: "vs_main",
                buffers: &[super::pipelines::vertex_buffer_layout()],
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
            &terrain_shaders.vertex,
            &terrain_shaders.fragment,
            wgpu::PrimitiveTopology::TriangleList,
            false,
        );
        let pipeline_wire = Self::create_pipeline(
            &device,
            &config,
            &layout,
            &terrain_shaders.vertex,
            &terrain_shaders.fragment,
            wgpu::PrimitiveTopology::TriangleList,
            true,
        );
        let pipeline_line = Self::create_pipeline(
            &device,
            &config,
            &layout,
            &terrain_shaders.vertex,
            &terrain_shaders.fragment,
            wgpu::PrimitiveTopology::LineList,
            false,
        );
        let depth = Self::mk_depth(&device, &config);

        // --- UI PIPELINE ---
        let pipeline_ui = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &ui_shaders.vertex,
                entry_point: "vs_main",
                buffers: &[super::pipelines::vertex_buffer_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &ui_shaders.fragment,
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

        // --- SKY PIPELINE ---
        // Fullscreen sky rendered with a separate, simple pipeline that only needs
        // the global uniform (no shadow sampler, no atlas, no vertex buffer).
        let sky_global_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky_global_layout"),
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

        let sky_global_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sky Global Bind"),
            layout: &sky_global_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: global_buf.as_entire_binding(),
            }],
        });

        let sky_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sky Pipeline Layout"),
            bind_group_layouts: &[&sky_global_layout],
            push_constant_ranges: &[],
        });

        let sky_shaders = Self::create_shader_pair(
            &device,
            render_registry,
            "core:render/techniques/sky/sky_gradient",
        );

        let pipeline_sky = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sky Pipeline"),
            layout: Some(&sky_layout),
            vertex: wgpu::VertexState {
                module: &sky_shaders.vertex,
                entry_point: "vs_main",
                buffers: &[], // fullscreen triangle from vertex_index — no vertex buffer
            },
            fragment: Some(wgpu::FragmentState {
                module: &sky_shaders.fragment,
                entry_point: "fs_main",
                targets: &[Some(config.format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None, // sky writes no depth
            multisample: Default::default(),
            multiview: None,
        });

        // --- MESHES ---
        let player_mesh = MeshGen::generate_cylinder(0.4, 1.8, 16);
        let pv: Vec<Vertex> = player_mesh
            .vertices
            .iter()
            .copied()
            .map(Vertex::from)
            .collect();
        let pi = &player_mesh.indices;
        let player_v_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&pv),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let player_i_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(pi),
            usage: wgpu::BufferUsages::INDEX,
        });

        let cross_mesh = MeshGen::generate_crosshair();
        let cv: Vec<Vertex> = cross_mesh
            .vertices
            .iter()
            .copied()
            .map(Vertex::from)
            .collect();
        let ci = &cross_mesh.indices;
        let cross_v_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&cv),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let cross_i_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(ci),
            usage: wgpu::BufferUsages::INDEX,
        });

        let cursor_v_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cursor V"),
            size: 8192,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let cursor_i_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cursor I"),
            size: 2048,
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
            sky_horizon: [0.72, 0.84, 1.00, 0.5],
            sky_zenith: [0.12, 0.28, 0.76, 1.0],
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
                    resource: wgpu::BindingResource::TextureView(&shadow_map.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&shadow_map.sampler),
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
            pipeline_sky,
            sky_global_bind,
            chunks: HashMap::new(),
            lod_chunks: HashMap::new(),
            global_buf,
            global_bind,
            local_bind_identity,
            local_buf_player,
            local_bind_player,
            depth,

            font_system: text_resources.font_system,
            swash_cache: text_resources.swash_cache,
            text_atlas: text_resources.text_atlas,
            text_renderer: text_resources.text_renderer,
            shadow_view: shadow_map.view,
            pipeline_shadow,
            shadow_global_buf: shadow_pass.global_buf,
            shadow_global_bind: shadow_pass.global_bind,
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
            hotbar_v_buf,
            hotbar_i_buf,
            hotbar_inds: 0,
            inventory_v_buf,
            inventory_i_buf,
            inventory_inds: 0,
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
            load_queue_set: HashSet::new(),
            player_chunk_pos: None,
            required_voxels: HashSet::new(),
            required_lods: HashSet::new(),
            mesh_tx,
            mesh_rx,
            pending_chunks: HashSet::new(),
            dirty_chunks: HashSet::new(),
            pending_dirty: HashSet::new(),
            lod_tx,
            lod_rx,
            pending_lods: HashSet::new(),
            scheduler: MeshScheduler::new(perf.scheduler),
            scheduler_stats: SchedulerStats::default(),
            completed_mesh_time_sum_ms: 0.0,
            completed_mesh_time_max_ms: 0.0,
            completed_mesh_count: 0,
            update_view_ms: 0.0,
            last_render_ms: 0.0,
            last_draw_calls: 0,
            last_shadow_draw_calls: 0,

            frame_stats: FrameStats::new(),
            quality: perf.quality,
            shadow_map_size: perf.shadow_map_size,
            lod_distance_scale: perf.lod_distance_scale,
            atlas_bind,
            start_time: std::time::Instant::now(),
        }
    }

    fn create_shader_pair(
        device: &wgpu::Device,
        render_registry: &RenderRegistry,
        technique_key: &str,
    ) -> RenderShaderPair {
        let technique = render_registry
            .technique_by_key(technique_key)
            .unwrap_or_else(|| {
                panic!(
                    "Render technique '{}' is missing from RenderRegistry",
                    technique_key
                )
            });
        let vertex_module = render_registry
            .shader_module(technique.vertex_stage)
            .unwrap_or_else(|| {
                panic!(
                    "Render technique '{}' references a missing vertex shader",
                    technique_key
                )
            });
        let fragment_id = technique.fragment_stage.unwrap_or_else(|| {
            panic!(
                "Render technique '{}' must declare a fragment shader for this pipeline",
                technique_key
            )
        });
        let fragment_module = render_registry
            .shader_module(fragment_id)
            .unwrap_or_else(|| {
                panic!(
                    "Render technique '{}' references a missing fragment shader",
                    technique_key
                )
            });

        let vertex_label = format!("{} vertex {}", technique.label, vertex_module.source_path);
        let fragment_label = format!(
            "{} fragment {}",
            technique.label, fragment_module.source_path
        );
        RenderShaderPair {
            vertex: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&vertex_label),
                source: wgpu::ShaderSource::Wgsl(vertex_module.source.as_str().into()),
            }),
            fragment: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&fragment_label),
                source: wgpu::ShaderSource::Wgsl(fragment_module.source.as_str().into()),
            }),
        }
    }
}

struct RenderShaderPair {
    vertex: wgpu::ShaderModule,
    fragment: wgpu::ShaderModule,
}

