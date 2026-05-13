use super::Renderer;
use crate::Vertex;

/// Shared vertex buffer layout matching `Vertex` (48 bytes).
/// pos(0,12) | uv(12,8) | normal(20,12) | color(32,12) | packed tex_index(44,4)
pub(super) fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as _,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0, // pos
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 12,
                shader_location: 1, // uv
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 20,
                shader_location: 2, // normal
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 32,
                shader_location: 3, // color
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32,
                offset: 44,
                shader_location: 4, // packed tex_index
            },
        ],
    }
}

impl<'a> Renderer<'a> {
    pub(super) fn create_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layout: &wgpu::PipelineLayout,
        vertex_shader: &wgpu::ShaderModule,
        fragment_shader: &wgpu::ShaderModule,
        topology: wgpu::PrimitiveTopology,
        wireframe: bool,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: vertex_shader,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: fragment_shader,
                entry_point: "fs_main",
                targets: &[Some(config.format.into())],
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

    pub(super) fn mk_depth(
        dev: &wgpu::Device,
        cfg: &wgpu::SurfaceConfiguration,
    ) -> wgpu::TextureView {
        dev.create_texture(&wgpu::TextureDescriptor {
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
            label: None,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
    }
}

