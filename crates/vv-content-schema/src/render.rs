use crate::ContentRef;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawShaderLanguage {
    Wgsl,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawShaderModule {
    pub language: RawShaderLanguage,
    #[serde(default)]
    pub imports: Vec<ContentRef>,
    pub feature_class: String,
    #[serde(default)]
    pub contracts: Vec<ContentRef>,
    pub allow_override: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawShaderStage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawGpuBindingKind {
    UniformBuffer,
    StorageBufferRead,
    Texture2d,
    Texture2dArray,
    DepthTexture2d,
    FilteringSampler,
    ComparisonSampler,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawShaderBinding {
    pub name: String,
    pub binding: u32,
    pub kind: RawGpuBindingKind,
    pub visibility: Vec<RawShaderStage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawShaderBindGroup {
    pub name: String,
    pub group: u32,
    pub bindings: Vec<RawShaderBinding>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawShaderContract {
    pub label: String,
    #[serde(default)]
    pub bind_groups: Vec<RawShaderBindGroup>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawMaterialTextureKind {
    Albedo,
    Normal,
    Roughness,
    Mask,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawMaterialTextureSlot {
    pub name: String,
    pub kind: RawMaterialTextureKind,
    pub required: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawMaterialScalarParameter {
    pub name: String,
    pub default: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawMaterialColorParameter {
    pub name: String,
    pub default: Vec<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawMaterialFamily {
    pub label: String,
    pub surface_model: String,
    #[serde(default)]
    pub texture_slots: Vec<RawMaterialTextureSlot>,
    #[serde(default)]
    pub scalar_parameters: Vec<RawMaterialScalarParameter>,
    #[serde(default)]
    pub color_parameters: Vec<RawMaterialColorParameter>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawDepthCompare {
    Less,
    LessEqual,
    Always,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawRenderDepthState {
    pub write: bool,
    pub compare: RawDepthCompare,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawCullMode {
    None,
    Back,
    Front,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawBlendMode {
    Opaque,
    AlphaBlend,
    Additive,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawRenderTechniqueStages {
    pub vertex: ContentRef,
    #[serde(default)]
    pub fragment: Option<ContentRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawTechniqueProfileOverride {
    pub profile: ContentRef,
    #[serde(default)]
    pub enable_features: Vec<String>,
    #[serde(default)]
    pub disable_features: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawRenderTechnique {
    pub label: String,
    pub pass: String,
    pub stages: RawRenderTechniqueStages,
    pub vertex_layout: String,
    pub material_family: ContentRef,
    #[serde(default)]
    pub contracts: Vec<ContentRef>,
    pub depth: RawRenderDepthState,
    pub culling: RawCullMode,
    pub blend: RawBlendMode,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub profile_overrides: Vec<RawTechniqueProfileOverride>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawRenderProfile {
    pub label: String,
    pub quality_class: String,
    pub shadows: String,
    pub antialiasing: String,
    pub bloom: bool,
    pub fog: bool,
    pub water: String,
    pub clouds: bool,
    pub normal_maps: bool,
    pub roughness_maps: bool,
    pub texture_filtering: String,
    #[serde(default)]
    pub feature_overrides: HashMap<String, bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawRenderGraphPass {
    pub name: String,
    #[serde(default)]
    pub technique: Option<ContentRef>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawRenderGraph {
    pub label: String,
    pub passes: Vec<RawRenderGraphPass>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawAllowedShaderImports {
    #[serde(default)]
    pub allowed_prefixes: Vec<String>,
    #[serde(default)]
    pub denied_prefixes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawFeatureBudget {
    pub max_features_per_technique: usize,
    pub known_features: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawPerformanceClasses {
    pub quality_classes: Vec<String>,
    pub shader_feature_classes: Vec<String>,
    pub surface_models: Vec<String>,
    pub texture_slots: Vec<String>,
    pub render_passes: Vec<String>,
    pub graph_external_inputs: Vec<String>,
    pub graph_outputs: Vec<String>,
    pub vertex_layouts: Vec<String>,
    pub technique_outputs: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{RawMaterialFamily, RawRenderProfile, RawRenderTechnique, RawShaderModule};

    #[test]
    fn parses_shader_module_ron() {
        let module: RawShaderModule = ron::from_str(
            r#"(
                language: wgsl,
                imports: ["core:render/shader_modules/math/constants"],
                feature_class: "surface",
                contracts: ["core:render/shader_contracts/frame_globals"],
                allow_override: true,
            )"#,
        )
        .expect("shader module");
        assert_eq!(module.imports.len(), 1);
    }

    #[test]
    fn parses_render_technique_ron() {
        let technique: RawRenderTechnique = ron::from_str(
            r#"(
                label: "Terrain Opaque",
                pass: "terrain_opaque",
                stages: (
                    vertex: "core:render/shader_modules/voxel/terrain_vertex",
                    fragment: Some("core:render/shader_modules/surface/stylized_pbr_lite"),
                ),
                vertex_layout: "terrain_chunk_mesh",
                material_family: "core:render/material_families/voxel_surface",
                contracts: [],
                depth: (write: true, compare: less_equal),
                culling: back,
                blend: opaque,
                outputs: ["main_color"],
                features: ["texture_array"],
                profile_overrides: [],
            )"#,
        )
        .expect("render technique");
        assert_eq!(technique.pass, "terrain_opaque");
    }

    #[test]
    fn parses_material_family_ron() {
        let family: RawMaterialFamily = ron::from_str(
            r#"(
                label: "Voxel Surface",
                surface_model: "stylized_pbr_lite",
                texture_slots: [
                    (name: "albedo", kind: albedo, required: true),
                ],
                scalar_parameters: [],
                color_parameters: [],
            )"#,
        )
        .expect("material family");
        assert_eq!(family.texture_slots.len(), 1);
    }

    #[test]
    fn parses_render_profile_ron() {
        let profile: RawRenderProfile = ron::from_str(
            r#"(
                label: "Balanced",
                quality_class: "balanced",
                shadows: "pcf_low",
                antialiasing: "fxaa",
                bloom: false,
                fog: true,
                water: "cheap",
                clouds: false,
                normal_maps: true,
                roughness_maps: true,
                texture_filtering: "nearest",
                feature_overrides: {"texture_array": true},
            )"#,
        )
        .expect("render profile");
        assert_eq!(profile.quality_class, "balanced");
    }
}
