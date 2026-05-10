use std::collections::HashMap;
use vv_content_schema::{
    RawBlendMode, RawCullMode, RawDepthCompare, RawGpuBindingKind, RawShaderStage,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct ShaderModuleId(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct ShaderContractId(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct RenderTechniqueId(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct MaterialFamilyId(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct RenderProfileId(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct RenderGraphId(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct ShaderVariantId(u32);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct RenderFeatureMask(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ShaderVariantKey {
    pub technique_id: RenderTechniqueId,
    pub render_profile_id: RenderProfileId,
    pub material_family_id: MaterialFamilyId,
    pub feature_mask: RenderFeatureMask,
    pub vertex_layout_id: u32,
}

#[derive(Clone, Debug)]
pub struct CompiledShaderModule {
    pub id: ShaderModuleId,
    pub key: String,
    pub source_path: String,
    pub source: String,
    pub imports: Vec<ShaderModuleId>,
    pub feature_class: String,
    pub contracts: Vec<ShaderContractId>,
    pub allow_override: bool,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CompiledShaderBinding {
    pub name: String,
    pub binding: u32,
    pub kind: RawGpuBindingKind,
    pub visibility: Vec<RawShaderStage>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CompiledShaderBindGroup {
    pub name: String,
    pub group: u32,
    pub bindings: Vec<CompiledShaderBinding>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CompiledShaderContract {
    pub id: ShaderContractId,
    pub key: String,
    pub label: String,
    pub bind_groups: Vec<CompiledShaderBindGroup>,
}

#[derive(Clone, Debug)]
pub struct CompiledMaterialTextureSlot {
    pub name: String,
    pub required: bool,
}

#[derive(Clone, Debug)]
pub struct CompiledMaterialFamily {
    pub id: MaterialFamilyId,
    pub key: String,
    pub label: String,
    pub surface_model: String,
    pub texture_slots: Vec<CompiledMaterialTextureSlot>,
}

#[derive(Clone, Debug)]
pub struct CompiledRenderProfile {
    pub id: RenderProfileId,
    pub key: String,
    pub label: String,
    pub quality_class: String,
    pub feature_overrides: RenderFeatureMask,
}

#[derive(Clone, Debug)]
pub struct CompiledDepthState {
    pub write: bool,
    pub compare: RawDepthCompare,
}

#[derive(Clone, Debug)]
pub struct CompiledRenderTechnique {
    pub id: RenderTechniqueId,
    pub key: String,
    pub label: String,
    pub pass: String,
    pub vertex_stage: ShaderModuleId,
    pub fragment_stage: Option<ShaderModuleId>,
    pub vertex_layout: String,
    pub vertex_layout_id: u32,
    pub material_family: MaterialFamilyId,
    pub contracts: Vec<ShaderContractId>,
    pub depth: CompiledDepthState,
    pub culling: RawCullMode,
    pub blend: RawBlendMode,
    pub outputs: Vec<String>,
    pub feature_mask: RenderFeatureMask,
}

#[derive(Clone, Debug)]
pub struct CompiledRenderGraphPass {
    pub name: String,
    pub technique: Option<RenderTechniqueId>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct CompiledRenderGraph {
    pub id: RenderGraphId,
    pub key: String,
    pub label: String,
    pub passes: Vec<CompiledRenderGraphPass>,
}

#[derive(Debug)]
pub struct RenderRegistry {
    shader_modules: Vec<CompiledShaderModule>,
    shader_contracts: Vec<CompiledShaderContract>,
    material_families: Vec<CompiledMaterialFamily>,
    render_profiles: Vec<CompiledRenderProfile>,
    render_techniques: Vec<CompiledRenderTechnique>,
    render_graphs: Vec<CompiledRenderGraph>,
    feature_bits: HashMap<String, u8>,
    shader_module_by_key: HashMap<String, ShaderModuleId>,
    #[allow(dead_code)]
    shader_contract_by_key: HashMap<String, ShaderContractId>,
    material_family_by_key: HashMap<String, MaterialFamilyId>,
    render_profile_by_key: HashMap<String, RenderProfileId>,
    render_technique_by_key: HashMap<String, RenderTechniqueId>,
    #[allow(dead_code)]
    render_graph_by_key: HashMap<String, RenderGraphId>,
}

#[derive(Debug)]
pub struct CompiledRenderContent {
    pub registry: RenderRegistry,
}

impl RenderRegistry {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        shader_modules: Vec<CompiledShaderModule>,
        shader_contracts: Vec<CompiledShaderContract>,
        material_families: Vec<CompiledMaterialFamily>,
        render_profiles: Vec<CompiledRenderProfile>,
        render_techniques: Vec<CompiledRenderTechnique>,
        render_graphs: Vec<CompiledRenderGraph>,
        feature_bits: HashMap<String, u8>,
    ) -> Self {
        let shader_module_by_key = shader_modules
            .iter()
            .map(|item| (item.key.clone(), item.id))
            .collect();
        let shader_contract_by_key = shader_contracts
            .iter()
            .map(|item| (item.key.clone(), item.id))
            .collect();
        let material_family_by_key = material_families
            .iter()
            .map(|item| (item.key.clone(), item.id))
            .collect();
        let render_profile_by_key = render_profiles
            .iter()
            .map(|item| (item.key.clone(), item.id))
            .collect();
        let render_technique_by_key = render_techniques
            .iter()
            .map(|item| (item.key.clone(), item.id))
            .collect();
        let render_graph_by_key = render_graphs
            .iter()
            .map(|item| (item.key.clone(), item.id))
            .collect();

        Self {
            shader_modules,
            shader_contracts,
            material_families,
            render_profiles,
            render_techniques,
            render_graphs,
            feature_bits,
            shader_module_by_key,
            shader_contract_by_key,
            material_family_by_key,
            render_profile_by_key,
            render_technique_by_key,
            render_graph_by_key,
        }
    }

    pub fn shader_module(&self, id: ShaderModuleId) -> Option<&CompiledShaderModule> {
        self.shader_modules.get(id.0 as usize)
    }

    pub fn shader_module_by_key(&self, key: &str) -> Option<&CompiledShaderModule> {
        self.shader_module_by_key
            .get(key)
            .and_then(|id| self.shader_module(*id))
    }

    pub fn technique(&self, id: RenderTechniqueId) -> Option<&CompiledRenderTechnique> {
        self.render_techniques.get(id.0 as usize)
    }

    pub fn technique_by_key(&self, key: &str) -> Option<&CompiledRenderTechnique> {
        self.render_technique_by_key
            .get(key)
            .and_then(|id| self.technique(*id))
    }

    pub fn profile_by_key(&self, key: &str) -> Option<&CompiledRenderProfile> {
        self.render_profile_by_key
            .get(key)
            .and_then(|id| self.render_profiles.get(id.0 as usize))
    }

    pub fn material_family_by_key(&self, key: &str) -> Option<&CompiledMaterialFamily> {
        self.material_family_by_key
            .get(key)
            .and_then(|id| self.material_families.get(id.0 as usize))
    }

    pub fn shader_module_count(&self) -> usize {
        self.shader_modules.len()
    }

    pub fn technique_count(&self) -> usize {
        self.render_techniques.len()
    }

    pub fn graph_count(&self) -> usize {
        self.render_graphs.len()
    }

    pub fn shader_contract_count(&self) -> usize {
        self.shader_contracts.len()
    }

    pub fn material_family_count(&self) -> usize {
        self.material_families.len()
    }

    pub fn profile_count(&self) -> usize {
        self.render_profiles.len()
    }

    pub fn feature_count(&self) -> usize {
        self.feature_bits.len()
    }

    pub fn variant_key(
        &self,
        technique_id: RenderTechniqueId,
        profile_id: RenderProfileId,
    ) -> Option<ShaderVariantKey> {
        let technique = self.technique(technique_id)?;
        Some(ShaderVariantKey {
            technique_id,
            render_profile_id: profile_id,
            material_family_id: technique.material_family,
            feature_mask: technique.feature_mask,
            vertex_layout_id: technique.vertex_layout_id,
        })
    }
}

impl ShaderModuleId {
    pub(crate) fn from_index(index: usize) -> Self {
        Self(index as u32)
    }
}

impl ShaderContractId {
    pub(crate) fn from_index(index: usize) -> Self {
        Self(index as u32)
    }
}

impl RenderTechniqueId {
    pub(crate) fn from_index(index: usize) -> Self {
        Self(index as u32)
    }
}

impl MaterialFamilyId {
    pub(crate) fn from_index(index: usize) -> Self {
        Self(index as u32)
    }
}

impl RenderProfileId {
    pub(crate) fn from_index(index: usize) -> Self {
        Self(index as u32)
    }
}

impl RenderGraphId {
    pub(crate) fn from_index(index: usize) -> Self {
        Self(index as u32)
    }
}
