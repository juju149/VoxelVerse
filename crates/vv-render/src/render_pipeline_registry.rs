#![allow(dead_code)]

//! Typed registry over declarative render pipeline descriptors.
//!
//! This is intentionally not a wgpu pipeline factory yet. It is the validation
//! and lookup layer between the render graph / schedule and the future concrete
//! PipelineRegistry that will own wgpu::RenderPipeline objects.

use crate::render_graph::{RenderPassId, ShaderPath};
use crate::render_pipeline_desc::{
    pipeline_desc, PipelineId, RenderPipelineDesc, PIPELINE_DESCS,
};

pub(crate) struct RenderPipelineRegistry {
    descriptors: &'static [RenderPipelineDesc],
}

impl RenderPipelineRegistry {
    pub(crate) const fn new() -> Self {
        Self {
            descriptors: PIPELINE_DESCS,
        }
    }

    pub(crate) fn descriptors(&self) -> &'static [RenderPipelineDesc] {
        self.descriptors
    }

    pub(crate) fn get(&self, id: PipelineId) -> &'static RenderPipelineDesc {
        pipeline_desc(id)
    }

    pub(crate) fn for_pass(&self, pass: RenderPassId) -> impl Iterator<Item = &'static RenderPipelineDesc> {
        self.descriptors.iter().filter(move |desc| desc.pass == pass)
    }

    pub(crate) fn first_for_pass(&self, pass: RenderPassId) -> Option<&'static RenderPipelineDesc> {
        self.for_pass(pass).next()
    }

    pub(crate) fn uses_shader(&self, shader: ShaderPath) -> bool {
        self.descriptors.iter().any(|desc| {
            desc.vertex == shader || desc.fragment.is_some_and(|fragment| fragment == shader)
        })
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        self.validate_unique_pipeline_ids()?;
        self.validate_required_pass_coverage()?;
        self.validate_pipeline_shader_coverage()?;
        Ok(())
    }

    fn validate_unique_pipeline_ids(&self) -> Result<(), String> {
        for desc in self.descriptors {
            let count = self
                .descriptors
                .iter()
                .filter(|candidate| candidate.id == desc.id)
                .count();

            if count != 1 {
                return Err(format!(
                    "pipeline id {:?} appears {count} times in PIPELINE_DESCS",
                    desc.id
                ));
            }
        }

        Ok(())
    }

    fn validate_required_pass_coverage(&self) -> Result<(), String> {
        let required_passes = [
            RenderPassId::ShadowDepth,
            RenderPassId::Sky,
            RenderPassId::Celestial,
            RenderPassId::Clouds,
            RenderPassId::TerrainOpaque,
            RenderPassId::VolumetricFog,
            RenderPassId::Precipitation,
            RenderPassId::FinalComposite,
            RenderPassId::Ui,
        ];

        for pass in required_passes {
            if self.first_for_pass(pass).is_none() {
                return Err(format!("render pass {pass:?} has no pipeline descriptor"));
            }
        }

        Ok(())
    }

    fn validate_pipeline_shader_coverage(&self) -> Result<(), String> {
        for desc in self.descriptors {
            if !ShaderPath::REQUIRED.contains(&desc.vertex) {
                return Err(format!(
                    "pipeline {:?} uses vertex shader {:?}, but it is not in ShaderPath::REQUIRED",
                    desc.id, desc.vertex
                ));
            }

            if let Some(fragment) = desc.fragment {
                if !ShaderPath::REQUIRED.contains(&fragment) {
                    return Err(format!(
                        "pipeline {:?} uses fragment shader {:?}, but it is not in ShaderPath::REQUIRED",
                        desc.id, fragment
                    ));
                }
            }
        }

        Ok(())
    }
}

pub(crate) const RENDER_PIPELINE_REGISTRY: RenderPipelineRegistry = RenderPipelineRegistry::new();

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_pipeline_desc::{
        BlendMode, DepthMode, PipelineKind, RenderTargetKind, VertexLayoutId,
    };

    #[test]
    fn registry_validates_v1_pipeline_table() {
        RENDER_PIPELINE_REGISTRY
            .validate()
            .expect("V1 pipeline registry is valid");
    }

    #[test]
    fn registry_can_lookup_pipeline_by_id() {
        let terrain = RENDER_PIPELINE_REGISTRY.get(PipelineId::TerrainOpaque);
        assert_eq!(terrain.pass, RenderPassId::TerrainOpaque);
        assert_eq!(terrain.kind, PipelineKind::Mesh);
    }

    #[test]
    fn registry_can_lookup_pipeline_by_pass() {
        let sky = RENDER_PIPELINE_REGISTRY
            .first_for_pass(RenderPassId::Sky)
            .expect("sky pass has a pipeline");

        assert_eq!(sky.id, PipelineId::Sky);
        assert_eq!(sky.kind, PipelineKind::Fullscreen);
        assert_eq!(sky.vertex_layout, VertexLayoutId::None);
    }

    #[test]
    fn every_descriptor_uses_required_shader_paths() {
        for desc in RENDER_PIPELINE_REGISTRY.descriptors() {
            assert!(ShaderPath::REQUIRED.contains(&desc.vertex));

            if let Some(fragment) = desc.fragment {
                assert!(ShaderPath::REQUIRED.contains(&fragment));
            }
        }
    }

    #[test]
    fn fullscreen_pipelines_have_no_depth_write() {
        for desc in RENDER_PIPELINE_REGISTRY.descriptors() {
            if desc.kind == PipelineKind::Fullscreen {
                assert_ne!(desc.depth, DepthMode::WriteLess);
                assert_ne!(desc.depth, DepthMode::ShadowWriteLess);
            }
        }
    }

    #[test]
    fn opaque_terrain_contract_is_stable() {
        let terrain = RENDER_PIPELINE_REGISTRY.get(PipelineId::TerrainOpaque);

        assert_eq!(terrain.target, RenderTargetKind::SceneHdr);
        assert_eq!(terrain.depth, DepthMode::WriteLess);
        assert_eq!(terrain.blend, BlendMode::Opaque);
        assert_eq!(terrain.vertex_layout, VertexLayoutId::Terrain);
    }

    #[test]
    fn final_composite_contract_is_stable() {
        let final_composite = RENDER_PIPELINE_REGISTRY.get(PipelineId::FinalComposite);

        assert_eq!(final_composite.target, RenderTargetKind::Swapchain);
        assert_eq!(final_composite.depth, DepthMode::None);
        assert_eq!(final_composite.blend, BlendMode::Replace);
        assert_eq!(final_composite.vertex_layout, VertexLayoutId::None);
    }

    #[test]
    fn shadow_depth_has_no_fragment_shader() {
        let shadow = RENDER_PIPELINE_REGISTRY.get(PipelineId::ShadowDepth);

        assert_eq!(shadow.pass, RenderPassId::ShadowDepth);
        assert!(shadow.fragment.is_none());
        assert_eq!(shadow.target, RenderTargetKind::ShadowDepth);
        assert_eq!(shadow.depth, DepthMode::ShadowWriteLess);
    }
}