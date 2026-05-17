#![allow(dead_code)]

//! Mapping layer from declarative pipeline descriptors to wgpu objects.

use std::collections::HashMap;

use crate::pipeline::graph::ShaderPath;
use crate::pipeline::desc::{
    BindGroupSlot, BlendMode, CullMode, DepthMode, PipelineKind, PolygonMode, PrimitiveTopology,
    RenderPipelineDesc, RenderTargetKind, VertexLayoutId,
};
use crate::shader::abi::vertex_location;
use crate::shader::library::ShaderLibrary;
use crate::types::Vertex;

pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub(crate) const SCENE_HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const TERRAIN_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 5] = [
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x3,
        offset: 0,
        shader_location: vertex_location::POSITION,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 12,
        shader_location: vertex_location::UV,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x3,
        offset: 20,
        shader_location: vertex_location::NORMAL,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x3,
        offset: 32,
        shader_location: vertex_location::COLOR,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Uint32,
        offset: 44,
        shader_location: vertex_location::TEX_INDEX,
    },
];
const TERRAIN_VERTEX_BUFFER_LAYOUTS: [wgpu::VertexBufferLayout<'static>; 1] =
    [wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &TERRAIN_VERTEX_ATTRIBUTES,
    }];

pub(crate) struct PipelineBindGroupLayouts {
    pub(crate) global: wgpu::BindGroupLayout,
    pub(crate) local: wgpu::BindGroupLayout,
    pub(crate) atlas: wgpu::BindGroupLayout,
    pub(crate) post_process_input: wgpu::BindGroupLayout,
}

impl PipelineBindGroupLayouts {
    pub(crate) fn create(device: &wgpu::Device) -> Self {
        let global = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            ],
            label: Some("global_layout"),
        });

        let local = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let atlas = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let post_process_input =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Post Bind Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        Self {
            global,
            local,
            atlas,
            post_process_input,
        }
    }

    fn ordered_layouts<'a>(&'a self, slots: &[BindGroupSlot]) -> Vec<&'a wgpu::BindGroupLayout> {
        slots
            .iter()
            .map(|slot| match slot {
                BindGroupSlot::Global => &self.global,
                BindGroupSlot::Local => &self.local,
                BindGroupSlot::MaterialAtlas => &self.atlas,
                BindGroupSlot::PostProcessInput => &self.post_process_input,
            })
            .collect()
    }
}

pub(crate) fn create_post_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Post Bind Group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

pub(crate) fn create_shader_modules(
    device: &wgpu::Device,
    shader_library: &ShaderLibrary,
) -> HashMap<ShaderPath, wgpu::ShaderModule> {
    let mut modules = HashMap::with_capacity(ShaderPath::REQUIRED.len());
    for &shader in ShaderPath::REQUIRED {
        let source = shader_library
            .source(shader)
            .unwrap_or_else(|error| panic!("missing shader {}: {}", shader.relative(), error));
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(shader.relative()),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        modules.insert(shader, module);
    }
    modules
}

pub(crate) fn vertex_buffer_layout_for(
    layout: VertexLayoutId,
) -> &'static [wgpu::VertexBufferLayout<'static>] {
    match layout {
        VertexLayoutId::None => &[],
        VertexLayoutId::Terrain => &TERRAIN_VERTEX_BUFFER_LAYOUTS,
    }
}

pub(crate) fn terrain_vertex_buffer_layout() -> &'static wgpu::VertexBufferLayout<'static> {
    &TERRAIN_VERTEX_BUFFER_LAYOUTS[0]
}

pub(crate) fn blend_state_for(mode: BlendMode) -> Option<wgpu::BlendState> {
    match mode {
        BlendMode::Opaque | BlendMode::Replace => None,
        BlendMode::Alpha => Some(wgpu::BlendState::ALPHA_BLENDING),
        BlendMode::Additive => Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::REPLACE,
        }),
    }
}

pub(crate) fn color_target_for(
    target: RenderTargetKind,
    swapchain_format: wgpu::TextureFormat,
    blend: BlendMode,
) -> Option<wgpu::ColorTargetState> {
    let format = match target {
        RenderTargetKind::None | RenderTargetKind::ShadowDepth => return None,
        RenderTargetKind::SceneHdr => SCENE_HDR_FORMAT,
        RenderTargetKind::Swapchain => swapchain_format,
    };

    Some(wgpu::ColorTargetState {
        format,
        blend: blend_state_for(blend),
        write_mask: wgpu::ColorWrites::ALL,
    })
}

pub(crate) fn depth_state_for(mode: DepthMode) -> Option<wgpu::DepthStencilState> {
    match mode {
        DepthMode::None => None,
        DepthMode::Read => Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        DepthMode::WriteLess => Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        DepthMode::ShadowWriteLess => Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: Default::default(),
            bias: wgpu::DepthBiasState {
                constant: 2,
                slope_scale: 2.0,
                clamp: 0.0,
            },
        }),
    }
}

pub(crate) fn primitive_state_for(desc: &RenderPipelineDesc) -> wgpu::PrimitiveState {
    let cull_mode = match desc.cull {
        CullMode::None => None,
        CullMode::Back => Some(wgpu::Face::Back),
        CullMode::Front => Some(wgpu::Face::Front),
    };

    wgpu::PrimitiveState {
        topology: match desc.topology {
            PrimitiveTopology::TriangleList => wgpu::PrimitiveTopology::TriangleList,
            PrimitiveTopology::LineList => wgpu::PrimitiveTopology::LineList,
        },
        cull_mode,
        polygon_mode: match desc.polygon {
            PolygonMode::Fill => wgpu::PolygonMode::Fill,
            PolygonMode::Line => wgpu::PolygonMode::Line,
        },
        ..Default::default()
    }
}

pub(crate) fn create_render_pipeline(
    device: &wgpu::Device,
    layouts: &PipelineBindGroupLayouts,
    modules: &HashMap<ShaderPath, wgpu::ShaderModule>,
    desc: &RenderPipelineDesc,
    swapchain_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    validate_factory_mapping(desc, swapchain_format)
        .unwrap_or_else(|error| panic!("invalid pipeline descriptor {:?}: {error}", desc.id));

    let bind_group_layouts = layouts.ordered_layouts(desc.bind_groups);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("{:?} Pipeline Layout", desc.id)),
        bind_group_layouts: &bind_group_layouts,
        push_constant_ranges: &[],
    });

    let vertex_module = modules
        .get(&desc.vertex)
        .unwrap_or_else(|| panic!("shader module missing for {:?}", desc.vertex));
    let vertex_buffers = vertex_buffer_layout_for(desc.vertex_layout);
    let color_target = color_target_for(desc.target, swapchain_format, desc.blend);
    let fragment_targets = [color_target];

    let fragment_state = desc.fragment.map(|fragment| {
        let fragment_module = modules
            .get(&fragment)
            .unwrap_or_else(|| panic!("shader module missing for {:?}", fragment));
        wgpu::FragmentState {
            module: fragment_module,
            entry_point: "fs_main",
            targets: &fragment_targets,
        }
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("{:?} Pipeline", desc.id)),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: vertex_module,
            entry_point: "vs_main",
            buffers: vertex_buffers,
        },
        fragment: fragment_state,
        primitive: primitive_state_for(desc),
        depth_stencil: depth_state_for(desc.depth),
        multisample: Default::default(),
        multiview: None,
    })
}

pub(crate) fn validate_factory_mapping(
    desc: &RenderPipelineDesc,
    swapchain_format: wgpu::TextureFormat,
) -> Result<(), String> {
    let vertex_layouts = vertex_buffer_layout_for(desc.vertex_layout);

    match desc.kind {
        PipelineKind::Fullscreen if !vertex_layouts.is_empty() => {
            return Err(format!(
                "fullscreen pipeline {:?} must not use vertex buffers",
                desc.id
            ));
        }
        PipelineKind::Mesh if vertex_layouts.is_empty() => {
            return Err(format!(
                "mesh pipeline {:?} must use a vertex buffer layout",
                desc.id
            ));
        }
        _ => {}
    }

    let color_target = color_target_for(desc.target, swapchain_format, desc.blend);
    match desc.target {
        RenderTargetKind::None | RenderTargetKind::ShadowDepth => {
            if color_target.is_some() {
                return Err(format!(
                    "pipeline {:?} must not create a color target for {:?}",
                    desc.id, desc.target
                ));
            }
        }
        RenderTargetKind::SceneHdr | RenderTargetKind::Swapchain => {
            if color_target.is_none() {
                return Err(format!(
                    "pipeline {:?} must create a color target for {:?}",
                    desc.id, desc.target
                ));
            }
        }
    }

    if desc.target == RenderTargetKind::ShadowDepth && desc.depth != DepthMode::ShadowWriteLess {
        return Err(format!(
            "shadow pipeline {:?} must use DepthMode::ShadowWriteLess",
            desc.id
        ));
    }

    if desc.blend == BlendMode::Opaque && blend_state_for(desc.blend).is_some() {
        return Err(format!("opaque pipeline {:?} unexpectedly blends", desc.id));
    }

    if desc.kind == PipelineKind::Fullscreen {
        if desc.topology != PrimitiveTopology::TriangleList {
            return Err(format!(
                "fullscreen pipeline {:?} must use TriangleList",
                desc.id
            ));
        }
        if desc.cull != CullMode::None {
            return Err(format!("fullscreen pipeline {:?} must not cull", desc.id));
        }
    }

    if desc.polygon == PolygonMode::Line && desc.topology != PrimitiveTopology::TriangleList {
        return Err(format!(
            "wire polygon pipeline {:?} must use TriangleList topology",
            desc.id
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::desc::{
        pipeline_desc, BlendMode, DepthMode, PipelineId, RenderTargetKind, VertexLayoutId,
        PIPELINE_DESCS,
    };

    const TEST_SWAPCHAIN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

    #[test]
    fn all_pipeline_descriptors_have_valid_factory_mappings() {
        for desc in PIPELINE_DESCS {
            validate_factory_mapping(desc, TEST_SWAPCHAIN_FORMAT).unwrap_or_else(|error| {
                panic!("invalid factory mapping for {:?}: {error}", desc.id)
            });
        }
    }

    #[test]
    fn terrain_vertex_layout_matches_vertex_contract() {
        let layout = terrain_vertex_buffer_layout();

        assert_eq!(layout.array_stride, 48);
        assert_eq!(layout.attributes.len(), 5);
        assert_eq!(layout.attributes[0].shader_location, 0);
        assert_eq!(layout.attributes[1].shader_location, 1);
        assert_eq!(layout.attributes[2].shader_location, 2);
        assert_eq!(layout.attributes[3].shader_location, 3);
        assert_eq!(layout.attributes[4].shader_location, 4);
    }

    #[test]
    fn fullscreen_layout_has_no_vertex_buffers() {
        assert!(vertex_buffer_layout_for(VertexLayoutId::None).is_empty());
    }

    #[test]
    fn opaque_and_replace_modes_do_not_blend() {
        assert_eq!(blend_state_for(BlendMode::Opaque), None);
        assert_eq!(blend_state_for(BlendMode::Replace), None);
    }

    #[test]
    fn alpha_and_additive_modes_blend() {
        assert!(blend_state_for(BlendMode::Alpha).is_some());
        assert!(blend_state_for(BlendMode::Additive).is_some());
    }

    #[test]
    fn scene_hdr_target_uses_hdr_format() {
        let target = color_target_for(
            RenderTargetKind::SceneHdr,
            TEST_SWAPCHAIN_FORMAT,
            BlendMode::Opaque,
        )
        .expect("scene hdr color target exists");

        assert_eq!(target.format, SCENE_HDR_FORMAT);
        assert!(target.blend.is_none());
    }

    #[test]
    fn swapchain_target_uses_surface_format() {
        let target = color_target_for(
            RenderTargetKind::Swapchain,
            TEST_SWAPCHAIN_FORMAT,
            BlendMode::Alpha,
        )
        .expect("swapchain color target exists");

        assert_eq!(target.format, TEST_SWAPCHAIN_FORMAT);
        assert!(target.blend.is_some());
    }

    #[test]
    fn shadow_depth_target_has_no_color_target() {
        assert!(color_target_for(
            RenderTargetKind::ShadowDepth,
            TEST_SWAPCHAIN_FORMAT,
            BlendMode::Opaque
        )
        .is_none());
    }

    #[test]
    fn depth_modes_map_to_expected_depth_write() {
        assert!(depth_state_for(DepthMode::None).is_none());

        let read = depth_state_for(DepthMode::Read).expect("read depth state");
        assert!(!read.depth_write_enabled);

        let write = depth_state_for(DepthMode::WriteLess).expect("write depth state");
        assert!(write.depth_write_enabled);

        let shadow = depth_state_for(DepthMode::ShadowWriteLess).expect("shadow depth state");
        assert!(shadow.depth_write_enabled);
        assert_eq!(shadow.bias.constant, 2);
    }

    #[test]
    fn terrain_opaque_factory_contract_is_stable() {
        let desc = pipeline_desc(PipelineId::TerrainOpaque);

        let color = color_target_for(desc.target, TEST_SWAPCHAIN_FORMAT, desc.blend)
            .expect("terrain color target");
        let depth = depth_state_for(desc.depth).expect("terrain depth state");
        let primitive = primitive_state_for(desc);

        assert_eq!(color.format, SCENE_HDR_FORMAT);
        assert!(color.blend.is_none());
        assert!(depth.depth_write_enabled);
        assert_eq!(primitive.cull_mode, Some(wgpu::Face::Back));
        assert_eq!(primitive.polygon_mode, wgpu::PolygonMode::Fill);
    }
}
