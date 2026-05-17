#![allow(dead_code)]

//! Runtime registry for render pipelines built from declarative descriptors.

use std::collections::HashMap;

use crate::pipeline::desc::{PipelineId, RenderPipelineDesc, PIPELINE_DESCS};
use crate::pipeline::factory::{
    create_render_pipeline, create_shader_modules, PipelineBindGroupLayouts,
};
use crate::pipeline::graph::{RenderPassId, ShaderPath};
use crate::shader::library::ShaderLibrary;

pub(crate) struct RenderPipelineRegistry {
    pipelines: HashMap<PipelineId, wgpu::RenderPipeline>,
    wireframe_supported: bool,
}

impl RenderPipelineRegistry {
    pub(crate) fn build(
        device: &wgpu::Device,
        shader_library: &ShaderLibrary,
        layouts: &PipelineBindGroupLayouts,
        swapchain_format: wgpu::TextureFormat,
        wireframe_supported: bool,
    ) -> Self {
        validate_pipeline_descriptors().expect("V1 pipeline descriptors are valid");

        let modules = create_shader_modules(device, shader_library);
        let mut pipelines = HashMap::with_capacity(PIPELINE_DESCS.len());
        for desc in PIPELINE_DESCS {
            if desc.id == PipelineId::TerrainWireDebug && !wireframe_supported {
                continue;
            }
            let pipeline =
                create_render_pipeline(device, layouts, &modules, desc, swapchain_format);
            pipelines.insert(desc.id, pipeline);
        }

        Self {
            pipelines,
            wireframe_supported,
        }
    }

    pub(crate) fn get(&self, id: PipelineId) -> &wgpu::RenderPipeline {
        self.pipelines
            .get(&id)
            .unwrap_or_else(|| panic!("render pipeline {id:?} was not built"))
    }

    pub(crate) fn terrain_wire_or_fill(&self) -> &wgpu::RenderPipeline {
        if self.wireframe_supported {
            self.get(PipelineId::TerrainWireDebug)
        } else {
            self.get(PipelineId::TerrainOpaque)
        }
    }

    pub(crate) fn wireframe_supported(&self) -> bool {
        self.wireframe_supported
    }
}

#[cfg(test)]
pub(crate) fn pipeline_descriptors() -> &'static [RenderPipelineDesc] {
    PIPELINE_DESCS
}

#[cfg(test)]
pub(crate) fn pipeline_descriptor(id: PipelineId) -> &'static RenderPipelineDesc {
    crate::pipeline::desc::pipeline_desc(id)
}

pub(crate) fn pipeline_descriptors_for_pass(
    pass: RenderPassId,
) -> impl Iterator<Item = &'static RenderPipelineDesc> {
    PIPELINE_DESCS.iter().filter(move |desc| desc.pass == pass)
}

pub(crate) fn first_pipeline_descriptor_for_pass(
    pass: RenderPassId,
) -> Option<&'static RenderPipelineDesc> {
    pipeline_descriptors_for_pass(pass).next()
}

pub(crate) fn validate_pipeline_descriptors() -> Result<(), String> {
    validate_unique_pipeline_ids()?;
    validate_required_pass_coverage()?;
    validate_pipeline_shader_coverage()?;
    Ok(())
}

fn validate_unique_pipeline_ids() -> Result<(), String> {
    for desc in PIPELINE_DESCS {
        let count = PIPELINE_DESCS
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

fn validate_required_pass_coverage() -> Result<(), String> {
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
        if first_pipeline_descriptor_for_pass(pass).is_none() {
            return Err(format!("render pass {pass:?} has no pipeline descriptor"));
        }
    }

    Ok(())
}

fn validate_pipeline_shader_coverage() -> Result<(), String> {
    for desc in PIPELINE_DESCS {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::desc::{
        BlendMode, DepthMode, PipelineKind, RenderTargetKind, VertexLayoutId,
    };

    #[test]
    fn registry_validates_v1_pipeline_table() {
        validate_pipeline_descriptors().expect("V1 pipeline registry is valid");
    }

    #[test]
    fn registry_can_lookup_pipeline_by_id() {
        let terrain = pipeline_descriptor(PipelineId::TerrainOpaque);
        assert_eq!(terrain.pass, RenderPassId::TerrainOpaque);
        assert_eq!(terrain.kind, PipelineKind::Mesh);
    }

    #[test]
    fn registry_can_lookup_pipeline_by_pass() {
        let sky =
            first_pipeline_descriptor_for_pass(RenderPassId::Sky).expect("sky pass has a pipeline");

        assert_eq!(sky.id, PipelineId::Sky);
        assert_eq!(sky.kind, PipelineKind::Fullscreen);
        assert_eq!(sky.vertex_layout, VertexLayoutId::None);
    }

    #[test]
    fn every_descriptor_uses_required_shader_paths() {
        for desc in pipeline_descriptors() {
            assert!(ShaderPath::REQUIRED.contains(&desc.vertex));

            if let Some(fragment) = desc.fragment {
                assert!(ShaderPath::REQUIRED.contains(&fragment));
            }
        }
    }

    #[test]
    fn fullscreen_pipelines_have_no_depth_write() {
        for desc in pipeline_descriptors() {
            if desc.kind == PipelineKind::Fullscreen {
                assert_ne!(desc.depth, DepthMode::WriteLess);
                assert_ne!(desc.depth, DepthMode::ShadowWriteLess);
            }
        }
    }

    #[test]
    fn opaque_terrain_contract_is_stable() {
        let terrain = pipeline_descriptor(PipelineId::TerrainOpaque);

        assert_eq!(terrain.target, RenderTargetKind::SceneHdr);
        assert_eq!(terrain.depth, DepthMode::WriteLess);
        assert_eq!(terrain.blend, BlendMode::Opaque);
        assert_eq!(terrain.vertex_layout, VertexLayoutId::Terrain);
    }

    #[test]
    fn final_composite_contract_is_stable() {
        let final_composite = pipeline_descriptor(PipelineId::FinalComposite);

        assert_eq!(final_composite.target, RenderTargetKind::Swapchain);
        assert_eq!(final_composite.depth, DepthMode::None);
        assert_eq!(final_composite.blend, BlendMode::Replace);
        assert_eq!(final_composite.vertex_layout, VertexLayoutId::None);
    }

    #[test]
    fn shadow_depth_has_no_fragment_shader() {
        let shadow = pipeline_descriptor(PipelineId::ShadowDepth);

        assert_eq!(shadow.pass, RenderPassId::ShadowDepth);
        assert!(shadow.fragment.is_none());
        assert_eq!(shadow.target, RenderTargetKind::ShadowDepth);
        assert_eq!(shadow.depth, DepthMode::ShadowWriteLess);
    }

    #[test]
    fn debug_pipelines_are_explicit_descriptors() {
        let wire = pipeline_descriptor(PipelineId::TerrainWireDebug);
        let line = pipeline_descriptor(PipelineId::DebugLine);

        assert_eq!(wire.pass, RenderPassId::TerrainOpaque);
        assert_eq!(line.pass, RenderPassId::TerrainOpaque);
    }
}
