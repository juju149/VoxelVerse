/// Viewer renderer – standalone wgpu renderer for block preview.
/// Reuses the same GPU pipeline structure and Vertex format as vv-render,
/// but uses a much simpler shader designed for block inspection clarity.
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::path::Path;
use wgpu::util::DeviceExt;
use winit::window::Window;

use vv_mesh::Vertex;
use vv_registry::{BlockContent, CompiledContent, ContentKey, RuntimeBlockVisual};

use crate::args::ViewerState;
use crate::block_selector::BlockSelector;
use crate::camera::OrbitCamera;
use crate::viewer_screenshot;
use crate::viewer_ui;

// --- GPU uniform structs ---

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct ViewerGlobalUniform {
    view_proj: [f32; 16],
    camera_pos: [f32; 4],
    sun_direction: [f32; 4],
    sun_color: [f32; 4],
    sky_color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct ViewerLocalUniform {
    model: [f32; 16],
    // x=debug_mode, y=variation_scale, z=edge_strength_mult, w=exposure
    params: [f32; 4],
    // x=ao_mult, y=bevel_mult, z=macro_strength_mult, w=micro_strength_mult
    sliders: [f32; 4],
}

fn build_block_visual_uniforms(content: &CompiledContent) -> Vec<RuntimeBlockVisual> {
    let mut out = Vec::new();
    for &v in content.block_visuals.entries() {
        out.push(v);
    }
    if out.is_empty() {
        out.push(RuntimeBlockVisual::fallback());
    }
    out
}

const VERTEX_ATTRIBUTES: &[wgpu::VertexAttribute] = &[
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
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Uint32,
        offset: 52,
        shader_location: 6,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Uint32,
        offset: 56,
        shader_location: 7,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Sint32x3,
        offset: 60,
        shader_location: 8,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Uint32,
        offset: 72,
        shader_location: 9,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32,
        offset: 76,
        shader_location: 10,
    },
];

pub struct ViewerRenderer<'w> {
    pub window: &'w Window,
    surface: wgpu::Surface<'w>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,

    global_buf: wgpu::Buffer,
    global_layout: wgpu::BindGroupLayout,
    global_bind: wgpu::BindGroup,
    dummy_shadow_view: wgpu::TextureView,
    dummy_shadow_sampler: wgpu::Sampler,
    block_visual_buf: wgpu::Buffer,
    block_visual_palette_buf: wgpu::Buffer,
    local_layout: wgpu::BindGroupLayout,

    pipeline_fill: wgpu::RenderPipeline,
    pipeline_line: wgpu::RenderPipeline,

    depth: wgpu::TextureView,

    // Scene geometry
    scene_vbuf: wgpu::Buffer,
    scene_ibuf: wgpu::Buffer,
    scene_index_count: u32,
    scene_local_buf: wgpu::Buffer,
    scene_local_bind: wgpu::BindGroup,

    // Grid geometry
    grid_vbuf: wgpu::Buffer,
    grid_ibuf: wgpu::Buffer,
    grid_index_count: u32,
    grid_local_bind: wgpu::BindGroup,

    pub camera: OrbitCamera,
    pub block_content: BlockContent,
    pub scene_extent: u32,

    // egui
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,

    // Offscreen pipeline for screenshots (targets Rgba8UnormSrgb).
    screenshot_pipeline: wgpu::RenderPipeline,
}

impl<'w> ViewerRenderer<'w> {
    pub async fn new(
        window: &'w Window,
        content: &CompiledContent,
        _assets_root: &Path,
        scene_verts: &[Vertex],
        scene_inds: &[u32],
        scene_extent: u32,
        grid_verts: &[Vertex],
        grid_inds: &[u32],
    ) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("no suitable GPU adapter found");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("vv-viewer"),
                    required_features: wgpu::Features::empty(),
                    required_limits: adapter.limits(),
                },
                None,
            )
            .await
            .expect("failed to create wgpu device");

        let mut surf_cfg = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .unwrap();
        surf_cfg.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &surf_cfg);

        // --- Block visual buffers ---
        let block_visuals = build_block_visual_uniforms(content);
        let block_visual_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("viewer block visuals"),
            contents: bytemuck::cast_slice(&block_visuals),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let mut palette = content.block_visual_palettes.clone();
        if palette.is_empty() {
            palette.push([0.5, 0.0, 0.8, 1.0]);
        }
        let block_visual_palette_buf =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("viewer palette"),
                contents: bytemuck::cast_slice(&palette),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let dummy_shadow = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("viewer dummy shadow"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let dummy_shadow_view = dummy_shadow.create_view(&wgpu::TextureViewDescriptor::default());
        let dummy_shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("viewer dummy shadow sampler"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        // --- Bind group layouts ---
        let global_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewer global layout"),
            entries: &[
                // 0: global uniform
                bgl_entry(
                    0,
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
                // 1: dummy depth texture so the viewer can share the main block shader path
                bgl_entry(
                    1,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                // 2: comparison sampler for the dummy shadow binding
                bgl_entry(
                    2,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                ),
                // 3: block visuals storage
                bgl_entry(
                    3,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
                // 4: palette storage
                bgl_entry(
                    4,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
            ],
        });

        let local_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewer local layout"),
            entries: &[bgl_entry(
                0,
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            )],
        });

        // --- Global buffer and bind group ---
        let global_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewer global"),
            size: std::mem::size_of::<ViewerGlobalUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let global_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewer global bind"),
            layout: &global_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: global_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dummy_shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&dummy_shadow_sampler),
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

        // --- Shader ---
        let shader_src = include_str!("viewer.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("viewer shader"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });

        // --- Pipelines ---
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("viewer pipeline layout"),
            bind_group_layouts: &[&global_layout, &local_layout],
            push_constant_ranges: &[],
        });

        let vertex_buf_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: VERTEX_ATTRIBUTES,
        };

        let surf_format = surf_cfg.format;

        let pipeline_fill = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("viewer fill pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_buf_layout.clone()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surf_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let pipeline_line = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("viewer line pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_line",
                buffers: &[vertex_buf_layout.clone()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_line",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surf_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Screenshot pipeline targets Rgba8UnormSrgb (format-independent of swapchain).
        let screenshot_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("viewer screenshot pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_buf_layout.clone()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // --- Depth buffer ---
        let depth = create_depth_view(&device, size.width.max(1), size.height.max(1));

        // --- Scene geometry buffers ---
        let scene_vbuf = create_vertex_buffer(&device, scene_verts, "scene vbuf");
        let scene_ibuf = create_index_buffer(&device, scene_inds, "scene ibuf");

        let identity = ViewerLocalUniform {
            model: Mat4::IDENTITY.to_cols_array(),
            params: [0.0, 1.0, 1.0, 1.0],
            sliders: [1.0, 1.0, 1.0, 1.0],
        };
        let scene_local_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene local"),
            size: std::mem::size_of::<ViewerLocalUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&scene_local_buf, 0, bytemuck::bytes_of(&identity));
        let scene_local_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &local_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_local_buf.as_entire_binding(),
            }],
        });

        // --- Grid geometry buffers ---
        let grid_vbuf = create_vertex_buffer(&device, grid_verts, "grid vbuf");
        let grid_ibuf = create_index_buffer(&device, grid_inds, "grid ibuf");

        let grid_identity = ViewerLocalUniform {
            model: Mat4::IDENTITY.to_cols_array(),
            params: [0.0, 1.0, 1.0, 1.0],
            sliders: [1.0, 1.0, 1.0, 1.0],
        };
        let grid_local_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("grid local"),
            contents: bytemuck::bytes_of(&grid_identity),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let grid_local_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &local_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: grid_local_buf.as_entire_binding(),
            }],
        });

        let camera =
            OrbitCamera::new_for_scene(scene_extent.max(1), size.width as f32, size.height as f32);
        let block_content = content.to_block_content();

        // egui
        let egui_ctx = egui::Context::default();
        let egui_state =
            egui_winit::State::new(egui_ctx.clone(), egui::ViewportId::ROOT, window, None, None);
        let egui_renderer = egui_wgpu::Renderer::new(&device, surf_format, None, 1);

        Self {
            window,
            surface,
            device,
            queue,
            config: surf_cfg,
            global_buf,
            global_layout,
            global_bind,
            dummy_shadow_view,
            dummy_shadow_sampler,
            block_visual_buf,
            block_visual_palette_buf,
            local_layout,
            pipeline_fill,
            pipeline_line,
            depth,
            scene_vbuf,
            scene_ibuf,
            scene_index_count: scene_inds.len() as u32,
            scene_local_buf,
            scene_local_bind,
            grid_vbuf,
            grid_ibuf,
            grid_index_count: grid_inds.len() as u32,
            grid_local_bind,
            camera,
            block_content,
            scene_extent,
            egui_ctx,
            egui_state,
            egui_renderer,
            screenshot_pipeline,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let w = width.max(1);
        let h = height.max(1);
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
        self.depth = create_depth_view(&self.device, w, h);
        self.camera.width = w as f32;
        self.camera.height = h as f32;
    }

    pub fn update_scene(&mut self, verts: &[Vertex], inds: &[u32], extent: u32) {
        self.scene_extent = extent;
        self.scene_vbuf = create_vertex_buffer(&self.device, verts, "scene vbuf");
        self.scene_ibuf = create_index_buffer(&self.device, inds, "scene ibuf");
        self.scene_index_count = inds.len() as u32;

        let local_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("scene local"),
                contents: bytemuck::bytes_of(&ViewerLocalUniform {
                    model: Mat4::IDENTITY.to_cols_array(),
                    params: [0.0, 1.0, 1.0, 1.0],
                    sliders: [1.0, 1.0, 1.0, 1.0],
                }),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let new_scene_local_bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.local_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: local_buf.as_entire_binding(),
            }],
        });
        self.scene_local_buf = local_buf;
        self.scene_local_bind = new_scene_local_bind;
    }

    pub fn update_content(&mut self, content: &CompiledContent) {
        self.block_content = content.to_block_content();

        let visuals = build_block_visual_uniforms(content);
        self.block_visual_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("viewer block visuals"),
                contents: bytemuck::cast_slice(&visuals),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let mut palette = content.block_visual_palettes.clone();
        if palette.is_empty() {
            palette.push([0.5, 0.0, 0.8, 1.0]);
        }
        self.block_visual_palette_buf =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("viewer palette"),
                    contents: bytemuck::cast_slice(&palette),
                    usage: wgpu::BufferUsages::STORAGE,
                });

        self.global_bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewer global bind"),
            layout: &self.global_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.global_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.dummy_shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.dummy_shadow_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.block_visual_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.block_visual_palette_buf.as_entire_binding(),
                },
            ],
        });
    }

    pub fn render(
        &mut self,
        state: &mut ViewerState,
        selector: &mut BlockSelector,
        content: &CompiledContent,
    ) -> viewer_ui::UiActions {
        // Write debug/slider uniforms every frame.
        let local_data = ViewerLocalUniform {
            model: Mat4::IDENTITY.to_cols_array(),
            params: [
                state.debug_mode.as_u32() as f32,
                state.variation_scale,
                state.edge_strength_mult,
                state.exposure,
            ],
            sliders: [
                state.ao_mult,
                state.bevel_mult,
                state.macro_strength_mult,
                state.micro_strength_mult,
            ],
        };
        self.queue
            .write_buffer(&self.scene_local_buf, 0, bytemuck::bytes_of(&local_data));

        // Update global uniform
        let vp = self.camera.view_proj();
        let cam_pos = self.camera.position();
        // Fixed studio lighting: sun from upper-right front
        let sun_dir = Vec3::new(0.6, 0.9, 0.4).normalize();
        let global = ViewerGlobalUniform {
            view_proj: vp.to_cols_array(),
            camera_pos: [cam_pos.x, cam_pos.y, cam_pos.z, 0.0],
            sun_direction: [sun_dir.x, sun_dir.y, sun_dir.z, 0.0],
            sun_color: [1.15, 1.05, 0.90, 0.0],
            sky_color: [0.46, 0.62, 0.88, 0.0],
        };
        self.queue
            .write_buffer(&self.global_buf, 0, bytemuck::bytes_of(&global));

        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return viewer_ui::UiActions::default();
            }
            Err(e) => {
                eprintln!("[viewer] surface error: {e:?}");
                return viewer_ui::UiActions::default();
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("viewer pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.14,
                            g: 0.14,
                            b: 0.16,
                            a: 1.0,
                        }),
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
                ..Default::default()
            });

            // Draw scene blocks
            if self.scene_index_count > 0 {
                rpass.set_pipeline(&self.pipeline_fill);
                rpass.set_bind_group(0, &self.global_bind, &[]);
                rpass.set_bind_group(1, &self.scene_local_bind, &[]);
                rpass.set_vertex_buffer(0, self.scene_vbuf.slice(..));
                rpass.set_index_buffer(self.scene_ibuf.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..self.scene_index_count, 0, 0..1);
            }

            // Draw grid overlay
            if state.show_grid && self.grid_index_count > 0 {
                rpass.set_pipeline(&self.pipeline_line);
                rpass.set_bind_group(0, &self.global_bind, &[]);
                rpass.set_bind_group(1, &self.grid_local_bind, &[]);
                rpass.set_vertex_buffer(0, self.grid_vbuf.slice(..));
                rpass.set_index_buffer(self.grid_ibuf.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..self.grid_index_count, 0, 0..1);
            }
        } // 3D render pass ends here.

        // egui pass ─────────────────────────────────────────────────────────
        let raw_input = self.egui_state.take_egui_input(self.window);
        let mut actions = viewer_ui::UiActions::default();
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            viewer_ui::draw(ctx, state, selector, content, &mut actions);
        });
        self.egui_state
            .handle_platform_output(self.window, full_output.platform_output);

        let tris = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, delta);
        }
        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };
        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &tris,
            &screen_desc,
        );
        {
            let mut egui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // preserve 3D scene
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            self.egui_renderer
                .render(&mut egui_pass, &tris, &screen_desc);
        }
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        actions
    }

    pub fn on_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.egui_state.on_window_event(self.window, event).consumed
    }

    pub fn screenshot(&mut self, state: &ViewerState, key: &ContentKey) {
        let path =
            viewer_screenshot::screenshot_path(key, state.scene.label(), state.debug_mode.label());
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("[viewer] screenshot dir error: {e}");
                return;
            }
        }
        let img = viewer_screenshot::take_screenshot(
            &self.device,
            &self.queue,
            &self.screenshot_pipeline,
            &self.global_bind,
            &self.scene_local_bind,
            &self.scene_vbuf,
            &self.scene_ibuf,
            self.scene_index_count,
            self.config.width,
            self.config.height,
        );
        match img {
            Some(img) => match img.save(&path) {
                Ok(()) => println!("[viewer] screenshot -> {:?}", path),
                Err(e) => eprintln!("[viewer] screenshot save error: {e}"),
            },
            None => eprintln!("[viewer] screenshot failed (GPU readback error)"),
        }
    }
}

// --- Helpers -----------------------------------------------------------------

fn bgl_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
    ty: wgpu::BindingType,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty,
        count: None,
    }
}

fn create_depth_view(device: &wgpu::Device, w: u32, h: u32) -> wgpu::TextureView {
    let t = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("viewer depth"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    t.create_view(&wgpu::TextureViewDescriptor::default())
}

fn create_vertex_buffer(device: &wgpu::Device, data: &[Vertex], label: &str) -> wgpu::Buffer {
    if data.is_empty() {
        // Create a minimal 1-vertex dummy buffer so the pipeline doesn't fail
        let dummy = Vertex::untextured([0.0; 3], [0.0; 3], [0.0, 1.0, 0.0]);
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::bytes_of(&dummy),
            usage: wgpu::BufferUsages::VERTEX,
        })
    } else {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }
}

fn create_index_buffer(device: &wgpu::Device, data: &[u32], label: &str) -> wgpu::Buffer {
    if data.is_empty() {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: wgpu::BufferUsages::INDEX,
        })
    } else {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::INDEX,
        })
    }
}
