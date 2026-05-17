#![allow(dead_code)]

//! Declarative render pipeline descriptions.
//!
//! The goal is to move pipeline intent out of ad-hoc setup code. This file does
//! not create wgpu pipelines yet; it defines the stable vocabulary used by the
//! next phase to build a PipelineRegistry.

use crate::render_graph::{RenderPassId, ShaderPath};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum PipelineId {
    ShadowDepth,
    TerrainOpaque,
    TerrainWireDebug,
    DebugLine,
    Sky,
    Celestial,
    Clouds,
    VolumetricFog,
    Precipitation,
    FinalComposite,
    Ui,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PipelineKind {
    Mesh,
    Fullscreen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrimitiveTopology {
    TriangleList,
    LineList,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CullMode {
    None,
    Back,
    Front,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PolygonMode {
    Fill,
    Line,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VertexLayoutId {
    None,
    Terrain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BindGroupSlot {
    Global,
    Local,
    MaterialAtlas,
    PostProcessInput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DepthMode {
    None,
    Read,
    WriteLess,
    ShadowWriteLess,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BlendMode {
    Opaque,
    Alpha,
    Additive,
    Replace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderTargetKind {
    None,
    SceneHdr,
    Swapchain,
    ShadowDepth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderPipelineDesc {
    pub id: PipelineId,
    pub pass: RenderPassId,
    pub kind: PipelineKind,
    pub vertex: ShaderPath,
    pub fragment: Option<ShaderPath>,
    pub vertex_layout: VertexLayoutId,
    pub bind_groups: &'static [BindGroupSlot],
    pub target: RenderTargetKind,
    pub depth: DepthMode,
    pub blend: BlendMode,
    pub topology: PrimitiveTopology,
    pub cull: CullMode,
    pub polygon: PolygonMode,
}

const GLOBAL_ONLY: &[BindGroupSlot] = &[BindGroupSlot::Global];
const TERRAIN_BIND_GROUPS: &[BindGroupSlot] = &[
    BindGroupSlot::Global,
    BindGroupSlot::Local,
    BindGroupSlot::MaterialAtlas,
];
const POST_BIND_GROUPS: &[BindGroupSlot] =
    &[BindGroupSlot::Global, BindGroupSlot::PostProcessInput];

pub(crate) const PIPELINE_DESCS: &[RenderPipelineDesc] = &[
    RenderPipelineDesc {
        id: PipelineId::ShadowDepth,
        pass: RenderPassId::ShadowDepth,
        kind: PipelineKind::Mesh,
        vertex: ShaderPath::TerrainDepthVertex,
        fragment: None,
        vertex_layout: VertexLayoutId::Terrain,
        bind_groups: TERRAIN_BIND_GROUPS,
        target: RenderTargetKind::ShadowDepth,
        depth: DepthMode::ShadowWriteLess,
        blend: BlendMode::Opaque,
        topology: PrimitiveTopology::TriangleList,
        cull: CullMode::Front,
        polygon: PolygonMode::Fill,
    },
    RenderPipelineDesc {
        id: PipelineId::TerrainOpaque,
        pass: RenderPassId::TerrainOpaque,
        kind: PipelineKind::Mesh,
        vertex: ShaderPath::TerrainVertex,
        fragment: Some(ShaderPath::TerrainFragment),
        vertex_layout: VertexLayoutId::Terrain,
        bind_groups: TERRAIN_BIND_GROUPS,
        target: RenderTargetKind::SceneHdr,
        depth: DepthMode::WriteLess,
        blend: BlendMode::Opaque,
        topology: PrimitiveTopology::TriangleList,
        cull: CullMode::Back,
        polygon: PolygonMode::Fill,
    },
    RenderPipelineDesc {
        id: PipelineId::TerrainWireDebug,
        pass: RenderPassId::TerrainOpaque,
        kind: PipelineKind::Mesh,
        vertex: ShaderPath::TerrainVertex,
        fragment: Some(ShaderPath::TerrainFragment),
        vertex_layout: VertexLayoutId::Terrain,
        bind_groups: TERRAIN_BIND_GROUPS,
        target: RenderTargetKind::SceneHdr,
        depth: DepthMode::WriteLess,
        blend: BlendMode::Opaque,
        topology: PrimitiveTopology::TriangleList,
        cull: CullMode::Back,
        polygon: PolygonMode::Line,
    },
    RenderPipelineDesc {
        id: PipelineId::DebugLine,
        pass: RenderPassId::TerrainOpaque,
        kind: PipelineKind::Mesh,
        vertex: ShaderPath::TerrainVertex,
        fragment: Some(ShaderPath::TerrainFragment),
        vertex_layout: VertexLayoutId::Terrain,
        bind_groups: TERRAIN_BIND_GROUPS,
        target: RenderTargetKind::SceneHdr,
        depth: DepthMode::WriteLess,
        blend: BlendMode::Opaque,
        topology: PrimitiveTopology::LineList,
        cull: CullMode::None,
        polygon: PolygonMode::Fill,
    },
    RenderPipelineDesc {
        id: PipelineId::Sky,
        pass: RenderPassId::Sky,
        kind: PipelineKind::Fullscreen,
        vertex: ShaderPath::SkyVertex,
        fragment: Some(ShaderPath::SkyFragment),
        vertex_layout: VertexLayoutId::None,
        bind_groups: GLOBAL_ONLY,
        target: RenderTargetKind::SceneHdr,
        depth: DepthMode::None,
        blend: BlendMode::Replace,
        topology: PrimitiveTopology::TriangleList,
        cull: CullMode::None,
        polygon: PolygonMode::Fill,
    },
    RenderPipelineDesc {
        id: PipelineId::Celestial,
        pass: RenderPassId::Celestial,
        kind: PipelineKind::Fullscreen,
        vertex: ShaderPath::CelestialVertex,
        fragment: Some(ShaderPath::CelestialFragment),
        vertex_layout: VertexLayoutId::None,
        bind_groups: GLOBAL_ONLY,
        target: RenderTargetKind::SceneHdr,
        depth: DepthMode::None,
        blend: BlendMode::Additive,
        topology: PrimitiveTopology::TriangleList,
        cull: CullMode::None,
        polygon: PolygonMode::Fill,
    },
    RenderPipelineDesc {
        id: PipelineId::Clouds,
        pass: RenderPassId::Clouds,
        kind: PipelineKind::Fullscreen,
        vertex: ShaderPath::CloudsVertex,
        fragment: Some(ShaderPath::CloudsFragment),
        vertex_layout: VertexLayoutId::None,
        bind_groups: GLOBAL_ONLY,
        target: RenderTargetKind::SceneHdr,
        depth: DepthMode::None,
        blend: BlendMode::Alpha,
        topology: PrimitiveTopology::TriangleList,
        cull: CullMode::None,
        polygon: PolygonMode::Fill,
    },
    RenderPipelineDesc {
        id: PipelineId::VolumetricFog,
        pass: RenderPassId::VolumetricFog,
        kind: PipelineKind::Fullscreen,
        vertex: ShaderPath::VolumetricFogVertex,
        fragment: Some(ShaderPath::VolumetricFogFragment),
        vertex_layout: VertexLayoutId::None,
        bind_groups: GLOBAL_ONLY,
        target: RenderTargetKind::SceneHdr,
        depth: DepthMode::None,
        blend: BlendMode::Alpha,
        topology: PrimitiveTopology::TriangleList,
        cull: CullMode::None,
        polygon: PolygonMode::Fill,
    },
    RenderPipelineDesc {
        id: PipelineId::Precipitation,
        pass: RenderPassId::Precipitation,
        kind: PipelineKind::Fullscreen,
        vertex: ShaderPath::PrecipitationVertex,
        fragment: Some(ShaderPath::PrecipitationFragment),
        vertex_layout: VertexLayoutId::None,
        bind_groups: GLOBAL_ONLY,
        target: RenderTargetKind::SceneHdr,
        depth: DepthMode::None,
        blend: BlendMode::Alpha,
        topology: PrimitiveTopology::TriangleList,
        cull: CullMode::None,
        polygon: PolygonMode::Fill,
    },
    RenderPipelineDesc {
        id: PipelineId::FinalComposite,
        pass: RenderPassId::FinalComposite,
        kind: PipelineKind::Fullscreen,
        vertex: ShaderPath::FullscreenVertex,
        fragment: Some(ShaderPath::FinalCompositeFragment),
        vertex_layout: VertexLayoutId::None,
        bind_groups: POST_BIND_GROUPS,
        target: RenderTargetKind::Swapchain,
        depth: DepthMode::None,
        blend: BlendMode::Replace,
        topology: PrimitiveTopology::TriangleList,
        cull: CullMode::None,
        polygon: PolygonMode::Fill,
    },
    RenderPipelineDesc {
        id: PipelineId::Ui,
        pass: RenderPassId::Ui,
        kind: PipelineKind::Mesh,
        vertex: ShaderPath::UiVertex,
        fragment: Some(ShaderPath::UiFragment),
        vertex_layout: VertexLayoutId::Terrain,
        bind_groups: TERRAIN_BIND_GROUPS,
        target: RenderTargetKind::Swapchain,
        depth: DepthMode::None,
        blend: BlendMode::Alpha,
        topology: PrimitiveTopology::TriangleList,
        cull: CullMode::None,
        polygon: PolygonMode::Fill,
    },
];

pub(crate) fn pipeline_desc(id: PipelineId) -> &'static RenderPipelineDesc {
    PIPELINE_DESCS
        .iter()
        .find(|desc| desc.id == id)
        .unwrap_or_else(|| panic!("missing render pipeline descriptor for {id:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_pipeline_registry_has_one_descriptor_per_pipeline() {
        assert_eq!(PIPELINE_DESCS.len(), 11);

        for desc in PIPELINE_DESCS {
            let count = PIPELINE_DESCS
                .iter()
                .filter(|candidate| candidate.id == desc.id)
                .count();
            assert_eq!(count, 1, "duplicate pipeline descriptor: {:?}", desc.id);
        }
    }

    #[test]
    fn fullscreen_pipelines_do_not_use_vertex_buffers() {
        for desc in PIPELINE_DESCS {
            if desc.kind == PipelineKind::Fullscreen {
                assert_eq!(desc.vertex_layout, VertexLayoutId::None);
            }
        }
    }

    #[test]
    fn mesh_pipelines_use_terrain_vertex_layout() {
        for desc in PIPELINE_DESCS {
            if desc.kind == PipelineKind::Mesh {
                assert_eq!(desc.vertex_layout, VertexLayoutId::Terrain);
            }
        }
    }

    #[test]
    fn final_composite_targets_swapchain() {
        let desc = pipeline_desc(PipelineId::FinalComposite);
        assert_eq!(desc.target, RenderTargetKind::Swapchain);
        assert_eq!(desc.depth, DepthMode::None);
    }

    #[test]
    fn shadow_depth_uses_depth_only_shader() {
        let desc = pipeline_desc(PipelineId::ShadowDepth);
        assert_eq!(desc.vertex, ShaderPath::TerrainDepthVertex);
        assert!(desc.fragment.is_none());
    }
}
