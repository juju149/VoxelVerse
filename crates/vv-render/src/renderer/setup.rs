use super::{GlobalUniform, GpuScene, LocalUniform, Renderer};
use crate::atmosphere::AtmosphereConfig;
use crate::lod_animation::LodAnimator;
use crate::perf_profile::{PerfProfile, PerfTier};
use crate::pipeline::factory::{create_post_bind_group, PipelineBindGroupLayouts};
use crate::pipeline::registry::RenderPipelineRegistry;
use crate::shader::library::ShaderLibrary;
use crate::texture_atlas::TextureAtlas;
use crate::types::Vertex;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::channel;
use vv_diagnostics::{FrameStats, SystemDiagnostics};
use vv_meshing::MeshGen;
use vv_meshing::{MeshScheduler, SchedulerStats};
use vv_pack_compiler::shader::PackShaderRoot;
use vv_pack_compiler::TextureRegistry;
use wgpu::util::DeviceExt;
use wgpu::PresentMode;
use winit::window::Window;

impl<'a> Renderer<'a> {
    pub async fn new(
        window: &'a Window,
        textures: &TextureRegistry,
        pack_stack: &[PackShaderRoot],
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
        // Force swapchain to a widely-supported LDR format to avoid validation
        // errors when third-party pipelines (glyphon) expect an sRGB target.
        // This sacrifices HDR swapchain, but keeps runtime behavior stable
        // across diverse GPUs. If HDR is required later, glyphon pipeline
        // creation must target the surface format.
        config.format = wgpu::TextureFormat::Bgra8UnormSrgb;

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
        let (shader_library, override_report) = ShaderLibrary::load_stack(pack_stack)
            .unwrap_or_else(|e| {
                let stack = pack_stack
                    .iter()
                    .map(|p| format!("{}@{}", p.name, p.root.display()))
                    .collect::<Vec<_>>()
                    .join(" | ");
                panic!("Failed to load render shader library from stack [{stack}]: {e}")
            });
        if !override_report.is_empty() {
            println!("[render/shader] pack overrides:");
            for o in &override_report.overrides {
                println!(
                    "  {} -> winner={} shadowed=[{}]",
                    o.relative_path.display(),
                    o.winner,
                    o.shadowed.join(", ")
                );
            }
        }

        let shadow_map = Self::create_shadow_map(&device, perf.shadow_map_size);
        let pipeline_layouts = PipelineBindGroupLayouts::create(&device);
        let global_layout = &pipeline_layouts.global;
        let local_layout = &pipeline_layouts.local;
        let atlas_layout = &pipeline_layouts.atlas;
        let post_bind_layout = &pipeline_layouts.post_process_input;

        let atlas = TextureAtlas::new(&device, &queue, textures);

        let color_data: Vec<[f32; 4]> = (0..textures.materials().len())
            .map(|layer| {
                let color = textures.average_albedo_color(layer as u32);
                [color[0], color[1], color[2], 1.0]
            })
            .collect();
        let material_color_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Flat Colors"),
            contents: bytemuck::cast_slice(&color_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let atlas_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atlas Bind Group"),
            layout: atlas_layout,
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
            layout: global_layout,
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
            Self::create_shadow_pass_resources(&device, global_layout, &shadow_map.sampler);

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
            layout: local_layout,
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
            layout: local_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: local_buf_player.as_entire_binding(),
            }],
            label: None,
        });

        // --- PIPELINES ---
        let pipelines = RenderPipelineRegistry::build(
            &device,
            &shader_library,
            &pipeline_layouts,
            config.format,
            features.contains(wgpu::Features::POLYGON_MODE_LINE),
        );
        let depth = Self::mk_depth(&device, &config);

        let scene = GpuScene::new(&device, config.width, config.height);
        let post_bind =
            create_post_bind_group(&device, post_bind_layout, &scene.view, &scene.sampler);

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

        let block_damage_v_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Block Damage Overlay V"),
            size: 256 * 1024,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let block_damage_i_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Block Damage Overlay I"),
            size: 64 * 1024,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let first_person_v_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("First Person Item V"),
            size: 16 * 1024,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let first_person_i_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("First Person Item I"),
            size: 8 * 1024,
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
            render_params: [0.0, 0.0, config.width as f32, config.height as f32],
            atmosphere_params: [0.0, 0.0, 0.0, 1.0],
            cloud_params: [0.0, 0.0, 0.0, 0.0],
            water_params: [0.55, 0.90, 0.72, 0.0],
            weather_params: [0.0, 1.0, 0.0, 0.0],
            celestial_params: [0.0, 0.0, 0.0, 0.0],
            celestial_moon: [0.0, 0.0, 0.0, 0.0],
        };

        let global_buf_identity = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Identity Buffer"),
            contents: bytemuck::cast_slice(&[identity_global_data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let global_bind_identity = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: global_layout,
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
            post_bind,
            scene,
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
            text_cache: super::text_cache::TextCache::default(),
            shadow_view: shadow_map.view,
            shadow_global_buf: shadow_pass.global_buf,
            shadow_global_bind: shadow_pass.global_bind,
            collision_v_buf,
            collision_i_buf,
            collision_inds: 0,
            frozen_frustum: None,
            player_v_buf,
            player_i_buf,
            player_inds: pi.len() as u32,
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
            block_damage_v_buf,
            block_damage_i_buf,
            block_damage_inds: 0,
            first_person_v_buf,
            first_person_i_buf,
            first_person_inds: 0,
            first_person_animation: super::hand_animation::HandAnimation::new(),
            animator: LodAnimator::new(),
            pipeline_layouts,
            pipelines,
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
            scheduler: MeshScheduler::new(perf.render_budget.mesh_scheduler),
            scheduler_stats: SchedulerStats::default(),
            hotbar_cache_signature: None,
            block_damage_cache_signature: None,
            completed_mesh_time_sum_ms: 0.0,
            completed_mesh_time_max_ms: 0.0,
            completed_mesh_count: 0,
            completed_voxel_mesh_time_sum_ms: 0.0,
            completed_voxel_mesh_time_max_ms: 0.0,
            completed_voxel_mesh_count: 0,
            completed_lod_mesh_time_sum_ms: 0.0,
            completed_lod_mesh_time_max_ms: 0.0,
            completed_lod_mesh_count: 0,
            update_view_ms: 0.0,
            lod_selection_ms: 0.0,
            gpu_upload_ms: 0.0,
            last_terrain_draw_ms: 0.0,
            last_render_ms: 0.0,
            last_draw_calls: 0,
            last_shadow_draw_calls: 0,

            frame_stats: FrameStats::new(),
            engine_debug_page: false,
            quality: perf.quality,
            shadow_map_size: perf.shadow_map_size,
            world_streaming: perf.world_streaming,
            meshing: perf.meshing,
            atmosphere: AtmosphereConfig::default(),
            atlas_bind,
        }
    }
}
