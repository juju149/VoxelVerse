use crate::render_registry::{
    CompiledDepthState, CompiledMaterialFamily, CompiledMaterialTextureSlot, CompiledRenderContent,
    CompiledRenderGraph, CompiledRenderGraphPass, CompiledRenderProfile, CompiledRenderTechnique,
    CompiledShaderBindGroup, CompiledShaderBinding, CompiledShaderContract, CompiledShaderModule,
    MaterialFamilyId, RenderFeatureMask, RenderGraphId, RenderProfileId, RenderRegistry,
    RenderTechniqueId, ShaderContractId, ShaderModuleId,
};
use crate::ContentCompiler;
use std::collections::{HashMap, HashSet};
use vv_content_schema::{
    RawBlendMode, RawFeatureBudget, RawMaterialFamily, RawPerformanceClasses, RawRenderGraph,
    RawRenderProfile, RawRenderTechnique,
};
use vv_pack_loader::{LoadedPack, LoadedShaderModule};

const DEFAULT_MAX_FEATURES_PER_TECHNIQUE: usize = 16;

impl ContentCompiler {
    pub fn compile_render_content(pack: &LoadedPack) -> Result<CompiledRenderContent, Vec<String>> {
        compile_render_pack(pack).map(|registry| CompiledRenderContent { registry })
    }
}

fn compile_render_pack(pack: &LoadedPack) -> Result<RenderRegistry, Vec<String>> {
    let mut errors = Vec::new();
    let render = &pack.render;

    let feature_budget = render
        .feature_budget
        .clone()
        .unwrap_or_else(default_feature_budget);
    let performance = render
        .performance_classes
        .clone()
        .unwrap_or_else(default_performance_classes);
    let feature_bits = feature_bits(&feature_budget, &mut errors);

    let default_allowed_imports = ["core:render/shader_modules/".to_string()];
    let allowed_imports = render
        .allowed_shader_imports
        .as_ref()
        .map(|rules| rules.allowed_prefixes.as_slice())
        .unwrap_or(&default_allowed_imports[..]);
    let denied_imports = render
        .allowed_shader_imports
        .as_ref()
        .map(|rules| rules.denied_prefixes.as_slice())
        .unwrap_or(&[][..]);

    validate_duplicate_keys(
        render.shader_modules.iter().map(|m| m.key.as_str()),
        "shader module",
        &mut errors,
    );
    validate_duplicate_keys(
        render.shader_contracts.iter().map(|(k, _)| k.as_str()),
        "shader contract",
        &mut errors,
    );
    validate_duplicate_keys(
        render.material_families.iter().map(|(k, _)| k.as_str()),
        "material family",
        &mut errors,
    );
    validate_duplicate_keys(
        render.profiles.iter().map(|(k, _)| k.as_str()),
        "render profile",
        &mut errors,
    );
    validate_duplicate_keys(
        render.techniques.iter().map(|(k, _)| k.as_str()),
        "render technique",
        &mut errors,
    );
    validate_duplicate_keys(
        render.render_graphs.iter().map(|(k, _)| k.as_str()),
        "render graph",
        &mut errors,
    );

    let shader_key_to_id = id_map_modules(&render.shader_modules);
    let contract_key_to_id = id_map_pairs(&render.shader_contracts, ShaderContractId::from_index);
    let material_key_to_id = id_map_pairs(&render.material_families, MaterialFamilyId::from_index);
    let profile_key_to_id = id_map_pairs(&render.profiles, RenderProfileId::from_index);
    let technique_key_to_id = id_map_pairs(&render.techniques, RenderTechniqueId::from_index);

    validate_shader_imports(
        &render.shader_modules,
        &shader_key_to_id,
        allowed_imports,
        denied_imports,
        &performance,
        &mut errors,
    );
    validate_import_cycles(&render.shader_modules, &shader_key_to_id, &mut errors);

    let shader_contracts = compile_shader_contracts(&render.shader_contracts);
    let material_families =
        compile_material_families(&render.material_families, &performance, &mut errors);
    let profiles = compile_profiles(&render.profiles, &performance, &feature_bits, &mut errors);
    let shader_modules = compile_shader_modules(
        &render.shader_modules,
        &shader_key_to_id,
        &contract_key_to_id,
        &performance,
        &mut errors,
    );
    let techniques = compile_techniques(
        &render.techniques,
        &shader_key_to_id,
        &contract_key_to_id,
        &material_key_to_id,
        &profile_key_to_id,
        &feature_budget,
        &feature_bits,
        &performance,
        &mut errors,
    );
    let graphs = compile_graphs(
        &render.render_graphs,
        &technique_key_to_id,
        &performance,
        &mut errors,
    );

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(RenderRegistry::new(
        shader_modules,
        shader_contracts,
        material_families,
        profiles,
        techniques,
        graphs,
        feature_bits,
    ))
}

fn compile_shader_contracts(
    raw: &[(String, vv_content_schema::RawShaderContract)],
) -> Vec<CompiledShaderContract> {
    raw.iter()
        .enumerate()
        .map(|(idx, (key, def))| CompiledShaderContract {
            id: ShaderContractId::from_index(idx),
            key: key.clone(),
            label: def.label.clone(),
            bind_groups: def
                .bind_groups
                .iter()
                .map(|group| CompiledShaderBindGroup {
                    name: group.name.clone(),
                    group: group.group,
                    bindings: group
                        .bindings
                        .iter()
                        .map(|binding| CompiledShaderBinding {
                            name: binding.name.clone(),
                            binding: binding.binding,
                            kind: binding.kind,
                            visibility: binding.visibility.clone(),
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect()
}

fn compile_material_families(
    raw: &[(String, RawMaterialFamily)],
    performance: &RawPerformanceClasses,
    errors: &mut Vec<String>,
) -> Vec<CompiledMaterialFamily> {
    raw.iter()
        .enumerate()
        .map(|(idx, (key, def))| {
            if def.surface_model.trim().is_empty() {
                errors.push(render_error(
                    key,
                    "surface_model",
                    "material family must declare a surface model",
                    "set surface_model to a known model such as 'stylized_pbr_lite'",
                ));
            } else if !performance.surface_models.contains(&def.surface_model) {
                errors.push(render_error(
                    key,
                    "surface_model",
                    &format!("unknown surface model '{}'", def.surface_model),
                    "add it to render/validation/performance_classes.ron or fix the material family",
                ));
            }
            for slot in &def.texture_slots {
                if !performance.texture_slots.contains(&slot.name) {
                    errors.push(render_error(
                        key,
                        "texture_slots",
                        &format!("unknown texture slot '{}'", slot.name),
                        "use an allowed texture slot or extend render validation data",
                    ));
                }
            }
            CompiledMaterialFamily {
                id: MaterialFamilyId::from_index(idx),
                key: key.clone(),
                label: def.label.clone(),
                surface_model: def.surface_model.clone(),
                texture_slots: def
                    .texture_slots
                    .iter()
                    .map(|slot| CompiledMaterialTextureSlot {
                        name: slot.name.clone(),
                        required: slot.required,
                    })
                    .collect(),
            }
        })
        .collect()
}

fn compile_profiles(
    raw: &[(String, RawRenderProfile)],
    performance: &RawPerformanceClasses,
    feature_bits: &HashMap<String, u8>,
    errors: &mut Vec<String>,
) -> Vec<CompiledRenderProfile> {
    raw.iter()
        .enumerate()
        .map(|(idx, (key, def))| {
            if !performance.quality_classes.contains(&def.quality_class) {
                errors.push(render_error(
                    key,
                    "quality_class",
                    &format!("unknown render quality class '{}'", def.quality_class),
                    "use a class declared in render/validation/performance_classes.ron",
                ));
            }
            let mut mask = 0_u64;
            for (feature, enabled) in &def.feature_overrides {
                match feature_bits.get(feature) {
                    Some(bit) if *enabled => mask |= 1_u64 << bit,
                    Some(_) => {}
                    None => errors.push(render_error(
                        key,
                        "feature_overrides",
                        &format!("unknown feature '{}'", feature),
                        "add the feature to render/validation/feature_budget.ron or remove it",
                    )),
                }
            }
            CompiledRenderProfile {
                id: RenderProfileId::from_index(idx),
                key: key.clone(),
                label: def.label.clone(),
                quality_class: def.quality_class.clone(),
                feature_overrides: RenderFeatureMask(mask),
            }
        })
        .collect()
}

fn compile_shader_modules(
    raw: &[LoadedShaderModule],
    shader_key_to_id: &HashMap<String, ShaderModuleId>,
    contract_key_to_id: &HashMap<String, ShaderContractId>,
    performance: &RawPerformanceClasses,
    errors: &mut Vec<String>,
) -> Vec<CompiledShaderModule> {
    raw.iter()
        .enumerate()
        .map(|(idx, module)| {
            if !performance
                .shader_feature_classes
                .contains(&module.metadata.feature_class)
            {
                errors.push(render_error(
                    &module.key,
                    "feature_class",
                    &format!(
                        "unknown shader feature class '{}'",
                        module.metadata.feature_class
                    ),
                    "use a class declared in render/validation/performance_classes.ron",
                ));
            }

            let imports = module
                .metadata
                .imports
                .iter()
                .filter_map(|r| shader_key_to_id.get(r.as_str()).copied())
                .collect();
            let contracts = module
                .metadata
                .contracts
                .iter()
                .filter_map(|r| {
                    let id = contract_key_to_id.get(r.as_str()).copied();
                    if id.is_none() {
                        errors.push(render_error(
                            &module.key,
                            "contracts",
                            &format!("unknown shader contract '{}'", r),
                            "reference an existing render/shader_contracts file",
                        ));
                    }
                    id
                })
                .collect();

            CompiledShaderModule {
                id: ShaderModuleId::from_index(idx),
                key: module.key.clone(),
                source_path: module.source_path.clone(),
                source: module.source.clone(),
                imports,
                feature_class: module.metadata.feature_class.clone(),
                contracts,
                allow_override: module.metadata.allow_override,
            }
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn compile_techniques(
    raw: &[(String, RawRenderTechnique)],
    shader_key_to_id: &HashMap<String, ShaderModuleId>,
    contract_key_to_id: &HashMap<String, ShaderContractId>,
    material_key_to_id: &HashMap<String, MaterialFamilyId>,
    profile_key_to_id: &HashMap<String, RenderProfileId>,
    feature_budget: &RawFeatureBudget,
    feature_bits: &HashMap<String, u8>,
    performance: &RawPerformanceClasses,
    errors: &mut Vec<String>,
) -> Vec<CompiledRenderTechnique> {
    raw.iter()
        .enumerate()
        .map(|(idx, (key, def))| {
            if !performance.render_passes.contains(&def.pass) {
                errors.push(render_error(
                    key,
                    "pass",
                    &format!("unknown render pass '{}'", def.pass),
                    "declare the pass in render/validation/performance_classes.ron or fix the technique",
                ));
            }
            if !performance.vertex_layouts.contains(&def.vertex_layout) {
                errors.push(render_error(
                    key,
                    "vertex_layout",
                    &format!("unknown vertex layout '{}'", def.vertex_layout),
                    "use a renderer-supported vertex layout",
                ));
            }
            if def.pass.contains("opaque") && def.blend != RawBlendMode::Opaque {
                errors.push(render_error(
                    key,
                    "blend",
                    "opaque passes must use blend = opaque",
                    "move the technique to an alpha pass or set blend to opaque",
                ));
            }
            if def.features.len() > feature_budget.max_features_per_technique {
                errors.push(render_error(
                    key,
                    "features",
                    &format!(
                        "declares {} features, above the budget of {}",
                        def.features.len(),
                        feature_budget.max_features_per_technique
                    ),
                    "split the technique or raise the budget explicitly",
                ));
            }

            let vertex_stage = match shader_key_to_id.get(&def.stages.vertex).copied() {
                Some(id) => id,
                None => {
                    errors.push(render_error(
                        key,
                        "stages.vertex",
                        &format!("unknown shader module '{}'", def.stages.vertex),
                        "reference an existing render/shader_modules WGSL module",
                    ));
                    ShaderModuleId::from_index(0)
                }
            };
            let fragment_stage = def.stages.fragment.as_ref().and_then(|stage| {
                let id = shader_key_to_id.get(stage.as_str()).copied();
                if id.is_none() {
                    errors.push(render_error(
                        key,
                        "stages.fragment",
                        &format!("unknown shader module '{}'", stage),
                        "reference an existing render/shader_modules WGSL module",
                    ));
                }
                id
            });
            let material_family = match material_key_to_id.get(def.material_family.as_str()).copied() {
                Some(id) => id,
                None => {
                    errors.push(render_error(
                        key,
                        "material_family",
                        &format!("unknown material family '{}'", def.material_family),
                        "reference an existing render/material_families file",
                    ));
                    MaterialFamilyId::from_index(0)
                }
            };
            let contracts = def
                .contracts
                .iter()
                .filter_map(|contract| {
                    let id = contract_key_to_id.get(contract.as_str()).copied();
                    if id.is_none() {
                        errors.push(render_error(
                            key,
                            "contracts",
                            &format!("unknown shader contract '{}'", contract),
                            "reference an existing render/shader_contracts file",
                        ));
                    }
                    id
                })
                .collect();
            for output in &def.outputs {
                if !performance.technique_outputs.contains(output) {
                    errors.push(render_error(
                        key,
                        "outputs",
                        &format!("unknown technique output '{}'", output),
                        "use a declared render target output",
                    ));
                }
            }
            for override_def in &def.profile_overrides {
                if !profile_key_to_id.contains_key(override_def.profile.as_str()) {
                    errors.push(render_error(
                        key,
                        "profile_overrides.profile",
                        &format!("unknown render profile '{}'", override_def.profile),
                        "reference an existing render/profiles file",
                    ));
                }
            }

            let feature_mask = mask_features(key, &def.features, feature_bits, errors);
            CompiledRenderTechnique {
                id: RenderTechniqueId::from_index(idx),
                key: key.clone(),
                label: def.label.clone(),
                pass: def.pass.clone(),
                vertex_stage,
                fragment_stage,
                vertex_layout: def.vertex_layout.clone(),
                vertex_layout_id: stable_vertex_layout_id(&def.vertex_layout),
                material_family,
                contracts,
                depth: CompiledDepthState {
                    write: def.depth.write,
                    compare: def.depth.compare,
                },
                culling: def.culling,
                blend: def.blend,
                outputs: def.outputs.clone(),
                feature_mask,
            }
        })
        .collect()
}

fn compile_graphs(
    raw: &[(String, RawRenderGraph)],
    technique_key_to_id: &HashMap<String, RenderTechniqueId>,
    performance: &RawPerformanceClasses,
    errors: &mut Vec<String>,
) -> Vec<CompiledRenderGraph> {
    raw.iter()
        .enumerate()
        .map(|(idx, (key, def))| {
            let mut produced: HashSet<String> =
                performance.graph_external_inputs.iter().cloned().collect();
            let mut passes = Vec::with_capacity(def.passes.len());
            for pass in &def.passes {
                for input in &pass.inputs {
                    if !produced.contains(input) {
                        errors.push(render_error(
                            key,
                            "passes.inputs",
                            &format!("pass '{}' reads '{}' before it is produced", pass.name, input),
                            "add a previous producing pass or declare the input as an external graph input",
                        ));
                    }
                }
                for output in &pass.outputs {
                    if !performance.graph_outputs.contains(output) {
                        errors.push(render_error(
                            key,
                            "passes.outputs",
                            &format!("unknown graph output '{}'", output),
                            "declare the resource in render/validation/performance_classes.ron",
                        ));
                    }
                    produced.insert(output.clone());
                }
                let technique = pass.technique.as_ref().and_then(|technique| {
                    let id = technique_key_to_id.get(technique.as_str()).copied();
                    if id.is_none() {
                        errors.push(render_error(
                            key,
                            "passes.technique",
                            &format!("unknown render technique '{}'", technique),
                            "reference an existing render/techniques file",
                        ));
                    }
                    id
                });
                passes.push(CompiledRenderGraphPass {
                    name: pass.name.clone(),
                    technique,
                    inputs: pass.inputs.clone(),
                    outputs: pass.outputs.clone(),
                });
            }

            CompiledRenderGraph {
                id: RenderGraphId::from_index(idx),
                key: key.clone(),
                label: def.label.clone(),
                passes,
            }
        })
        .collect()
}

fn validate_shader_imports(
    modules: &[LoadedShaderModule],
    shader_key_to_id: &HashMap<String, ShaderModuleId>,
    allowed_prefixes: &[String],
    denied_prefixes: &[String],
    performance: &RawPerformanceClasses,
    errors: &mut Vec<String>,
) {
    for module in modules {
        for import in &module.metadata.imports {
            if !shader_key_to_id.contains_key(import.as_str()) {
                errors.push(render_error(
                    &module.key,
                    "imports",
                    &format!("unknown shader module '{}'", import),
                    "reference an existing render/shader_modules WGSL module",
                ));
            }
            if !allowed_prefixes
                .iter()
                .any(|prefix| import.starts_with(prefix))
            {
                errors.push(render_error(
                    &module.key,
                    "imports",
                    &format!("shader import '{}' is not in an allowed prefix", import),
                    "use allowed_shader_imports.ron to explicitly permit the import root",
                ));
            }
            if denied_prefixes
                .iter()
                .any(|prefix| import.starts_with(prefix))
            {
                errors.push(render_error(
                    &module.key,
                    "imports",
                    &format!("shader import '{}' is denied", import),
                    "remove the import or change the validation policy",
                ));
            }
        }
        if !performance
            .shader_feature_classes
            .contains(&module.metadata.feature_class)
        {
            errors.push(render_error(
                &module.key,
                "feature_class",
                &format!(
                    "unknown shader feature class '{}'",
                    module.metadata.feature_class
                ),
                "declare the class in render/validation/performance_classes.ron",
            ));
        }
    }
}

fn validate_import_cycles(
    modules: &[LoadedShaderModule],
    shader_key_to_id: &HashMap<String, ShaderModuleId>,
    errors: &mut Vec<String>,
) {
    let imports_by_key: HashMap<&str, Vec<&str>> = modules
        .iter()
        .map(|module| {
            (
                module.key.as_str(),
                module
                    .metadata
                    .imports
                    .iter()
                    .filter(|r| shader_key_to_id.contains_key(r.as_str()))
                    .map(|r| r.as_str())
                    .collect(),
            )
        })
        .collect();
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    let mut stack = Vec::new();
    for module in modules {
        dfs_imports(
            module.key.as_str(),
            &imports_by_key,
            &mut visiting,
            &mut visited,
            &mut stack,
            errors,
        );
    }
}

fn dfs_imports<'a>(
    key: &'a str,
    imports_by_key: &HashMap<&'a str, Vec<&'a str>>,
    visiting: &mut HashSet<&'a str>,
    visited: &mut HashSet<&'a str>,
    stack: &mut Vec<&'a str>,
    errors: &mut Vec<String>,
) {
    if visited.contains(key) {
        return;
    }
    if visiting.contains(key) {
        let cycle_start = stack.iter().position(|item| *item == key).unwrap_or(0);
        let mut cycle = stack[cycle_start..].to_vec();
        cycle.push(key);
        errors.push(render_error(
            key,
            "imports",
            &format!("shader import cycle detected: {}", cycle.join(" -> ")),
            "remove one import edge so shader modules form a DAG",
        ));
        return;
    }
    visiting.insert(key);
    stack.push(key);
    if let Some(imports) = imports_by_key.get(key) {
        for import in imports {
            dfs_imports(import, imports_by_key, visiting, visited, stack, errors);
        }
    }
    stack.pop();
    visiting.remove(key);
    visited.insert(key);
}

fn validate_duplicate_keys<'a>(
    keys: impl Iterator<Item = &'a str>,
    label: &str,
    errors: &mut Vec<String>,
) {
    let mut seen = HashSet::new();
    for key in keys {
        if !seen.insert(key) {
            errors.push(format!(
                "render content: duplicate {} key '{}'; path-as-identity must resolve to one owner",
                label, key
            ));
        }
    }
}

fn id_map_modules(modules: &[LoadedShaderModule]) -> HashMap<String, ShaderModuleId> {
    modules
        .iter()
        .enumerate()
        .map(|(idx, module)| (module.key.clone(), ShaderModuleId::from_index(idx)))
        .collect()
}

fn id_map_pairs<T, Id: Copy>(
    raw: &[(String, T)],
    from_index: fn(usize) -> Id,
) -> HashMap<String, Id> {
    raw.iter()
        .enumerate()
        .map(|(idx, (key, _))| (key.clone(), from_index(idx)))
        .collect()
}

fn feature_bits(budget: &RawFeatureBudget, errors: &mut Vec<String>) -> HashMap<String, u8> {
    let mut bits = HashMap::new();
    for (idx, feature) in budget.known_features.iter().enumerate() {
        if idx >= 64 {
            errors.push(format!(
                "render/validation/feature_budget.ron: feature '{}' exceeds the 64-bit runtime feature mask; split or remove features",
                feature
            ));
            continue;
        }
        if bits.insert(feature.clone(), idx as u8).is_some() {
            errors.push(format!(
                "render/validation/feature_budget.ron: duplicate feature '{}'",
                feature
            ));
        }
    }
    bits
}

fn mask_features(
    key: &str,
    features: &[String],
    feature_bits: &HashMap<String, u8>,
    errors: &mut Vec<String>,
) -> RenderFeatureMask {
    let mut mask = 0_u64;
    for feature in features {
        match feature_bits.get(feature) {
            Some(bit) => mask |= 1_u64 << bit,
            None => errors.push(render_error(
                key,
                "features",
                &format!("unknown feature '{}'", feature),
                "add the feature to render/validation/feature_budget.ron or remove it",
            )),
        }
    }
    RenderFeatureMask(mask)
}

fn stable_vertex_layout_id(layout: &str) -> u32 {
    match layout {
        "terrain_chunk_mesh" => 1,
        "fullscreen_triangle" => 2,
        "ui_flat" => 3,
        _ => 0,
    }
}

fn default_feature_budget() -> RawFeatureBudget {
    RawFeatureBudget {
        max_features_per_technique: DEFAULT_MAX_FEATURES_PER_TECHNIQUE,
        known_features: Vec::new(),
    }
}

fn default_performance_classes() -> RawPerformanceClasses {
    RawPerformanceClasses {
        quality_classes: Vec::new(),
        shader_feature_classes: Vec::new(),
        surface_models: Vec::new(),
        texture_slots: Vec::new(),
        render_passes: Vec::new(),
        graph_external_inputs: Vec::new(),
        graph_outputs: Vec::new(),
        vertex_layouts: Vec::new(),
        technique_outputs: Vec::new(),
    }
}

fn render_error(resource: &str, field: &str, reason: &str, correction: &str) -> String {
    format!(
        "render '{}', field '{}': {}. Fix: {}.",
        resource, field, reason, correction
    )
}

#[cfg(test)]
#[path = "render_compiler_tests.rs"]
mod tests;

