//! Render-pack validation.
//!
//! Checks shader metadata/source pairs, render graph wiring, validation
//! presets, profile classes and feature budgets. WGSL compilation is still
//! owned by the render compiler; Pack Doctor performs cheap structural checks
//! so broken or missing shader assets cannot slip through silently.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use vv_content_schema::{RawAllowedShaderImports, RawFeatureBudget, RawPerformanceClasses};

use crate::index::PackIndex;
use crate::parse::read_typed;
use crate::report::{Diagnostic, RenderProfileSummary, Report};
use crate::scan::PackScan;

const CHECK: &str = "render";

pub fn run(_index: &PackIndex<'_>, _report: &mut Report) {}

pub fn validate(scan: &PackScan, report: &mut Report) {
    let feature_budget = read_validation::<RawFeatureBudget>(scan, report, "feature_budget.ron");
    let performance =
        read_validation::<RawPerformanceClasses>(scan, report, "performance_classes.ron");
    let allowed_imports =
        read_validation::<RawAllowedShaderImports>(scan, report, "allowed_shader_imports.ron");

    let module_ids: BTreeSet<String> = scan
        .render
        .shader_modules
        .iter()
        .map(|m| m.id.clone())
        .collect();
    let contract_ids: BTreeSet<String> = scan
        .render
        .shader_contracts
        .iter()
        .map(|c| c.id.clone())
        .collect();
    let family_ids: BTreeSet<String> = scan
        .render
        .material_families
        .iter()
        .map(|m| m.id.clone())
        .collect();
    let technique_ids: BTreeSet<String> = scan
        .render
        .techniques
        .iter()
        .map(|t| t.id.clone())
        .collect();
    let profile_ids: BTreeSet<String> = scan.render.profiles.iter().map(|p| p.id.clone()).collect();

    let module_by_id: BTreeMap<&str, &_> = scan
        .render
        .shader_modules
        .iter()
        .map(|m| (m.id.as_str(), m))
        .collect();

    for module in &scan.render.shader_modules {
        let metadata_abs = scan.pack_root.join(&module.rel_path);
        let wgsl = metadata_abs.with_extension("wgsl");
        if !wgsl.exists() {
            report
                .missing
                .shaders
                .push(pack_rel(&scan.pack_root, &wgsl));
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!("shader module '{}' is missing its WGSL source", module.id),
                )
                .with_path(module.rel_path.clone())
                .with_id(module.id.clone())
                .with_suggestion(format!(
                    "create {} alongside the metadata file",
                    pack_rel(&scan.pack_root, &wgsl)
                )),
            );
        } else {
            check_wgsl_source(scan, report, &module.id, &wgsl);
        }

        if let Some(perf) = &performance {
            if !perf
                .shader_feature_classes
                .iter()
                .any(|class| class == &module.value.feature_class)
            {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!(
                            "shader module '{}' uses unknown feature_class '{}'",
                            module.id, module.value.feature_class
                        ),
                    )
                    .with_path(module.rel_path.clone())
                    .with_id(module.id.clone())
                    .with_field("feature_class"),
                );
            }
        }
    }

    for module in &scan.render.shader_modules {
        for imp in &module.value.imports {
            if !module_ids.contains(imp.as_str()) {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!("shader import '{}' does not match any module", imp),
                    )
                    .with_path(module.rel_path.clone())
                    .with_id(module.id.clone())
                    .with_field("imports"),
                );
            }
            if let Some(rules) = &allowed_imports {
                if rules
                    .denied_prefixes
                    .iter()
                    .any(|prefix| imp.starts_with(prefix))
                {
                    report.error(
                        Diagnostic::new(
                            CHECK,
                            format!("shader import '{}' is explicitly denied", imp),
                        )
                        .with_path(module.rel_path.clone())
                        .with_id(module.id.clone())
                        .with_field("imports"),
                    );
                }
                if !rules.allowed_prefixes.is_empty()
                    && !rules
                        .allowed_prefixes
                        .iter()
                        .any(|prefix| imp.starts_with(prefix))
                {
                    report.error(
                        Diagnostic::new(
                            CHECK,
                            format!("shader import '{}' is outside allowed prefixes", imp),
                        )
                        .with_path(module.rel_path.clone())
                        .with_id(module.id.clone())
                        .with_field("imports"),
                    );
                }
            }
        }
        for contract in &module.value.contracts {
            if !contract_ids.contains(contract.as_str()) {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!("shader contract '{}' does not exist", contract),
                    )
                    .with_path(module.rel_path.clone())
                    .with_id(module.id.clone())
                    .with_field("contracts"),
                );
            }
        }
    }

    for tech in &scan.render.techniques {
        check_technique_budget(report, &feature_budget, tech);
        check_technique_classes(report, &performance, tech);
        check_ref(
            report,
            &tech.value.stages.vertex,
            &module_ids,
            &tech.id,
            &tech.rel_path,
            "stages.vertex",
        );
        if let Some(fragment) = &tech.value.stages.fragment {
            check_ref(
                report,
                fragment,
                &module_ids,
                &tech.id,
                &tech.rel_path,
                "stages.fragment",
            );
        }
        if !family_ids.contains(&tech.value.material_family) {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!(
                        "technique '{}' references unknown material_family '{}'",
                        tech.id, tech.value.material_family
                    ),
                )
                .with_path(tech.rel_path.clone())
                .with_id(tech.id.clone())
                .with_field("material_family"),
            );
        }
        for contract in &tech.value.contracts {
            if !contract_ids.contains(contract.as_str()) {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!(
                            "technique '{}' references unknown contract '{}'",
                            tech.id, contract
                        ),
                    )
                    .with_path(tech.rel_path.clone())
                    .with_id(tech.id.clone())
                    .with_field("contracts"),
                );
            }
        }
        for (i, override_) in tech.value.profile_overrides.iter().enumerate() {
            if !profile_ids.contains(&override_.profile) {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!(
                            "technique '{}' profile_override #{} points at unknown profile '{}'",
                            tech.id,
                            i + 1,
                            override_.profile
                        ),
                    )
                    .with_path(tech.rel_path.clone())
                    .with_id(tech.id.clone())
                    .with_field(format!("profile_overrides[{i}].profile")),
                );
            }
            if let Some(budget) = &feature_budget {
                for (j, feature) in override_.enable_features.iter().enumerate() {
                    check_known_feature(
                        report,
                        budget,
                        &tech.id,
                        &tech.rel_path,
                        &format!("profile_overrides[{i}].enable_features"),
                        j,
                        feature,
                    );
                }
                for (j, feature) in override_.disable_features.iter().enumerate() {
                    check_known_feature(
                        report,
                        budget,
                        &tech.id,
                        &tech.rel_path,
                        &format!("profile_overrides[{i}].disable_features"),
                        j,
                        feature,
                    );
                }
            }
        }
    }

    for graph in &scan.render.render_graphs {
        let mut produced: BTreeSet<String> = BTreeSet::new();
        for (i, pass) in graph.value.passes.iter().enumerate() {
            if let Some(technique) = &pass.technique {
                if !technique_ids.contains(technique.as_str()) {
                    report.error(
                        Diagnostic::new(
                            CHECK,
                            format!(
                                "render graph '{}' pass '{}' references unknown technique '{}'",
                                graph.id, pass.name, technique
                            ),
                        )
                        .with_path(graph.rel_path.clone())
                        .with_id(graph.id.clone())
                        .with_field(format!("passes[{i}].technique")),
                    );
                }
            }
            if let Some(perf) = &performance {
                for input in &pass.inputs {
                    let external = perf
                        .graph_external_inputs
                        .iter()
                        .any(|known| known == input);
                    if !external && !produced.contains(input) {
                        report.error(
                            Diagnostic::new(
                                CHECK,
                                format!(
                                    "render graph '{}' pass '{}' reads unknown input '{}'",
                                    graph.id, pass.name, input
                                ),
                            )
                            .with_path(graph.rel_path.clone())
                            .with_id(graph.id.clone())
                            .with_field(format!("passes[{i}].inputs")),
                        );
                    }
                }
                for output in &pass.outputs {
                    if !perf.graph_outputs.iter().any(|known| known == output) {
                        report.error(
                            Diagnostic::new(
                                CHECK,
                                format!(
                                    "render graph '{}' pass '{}' writes unknown output '{}'",
                                    graph.id, pass.name, output
                                ),
                            )
                            .with_path(graph.rel_path.clone())
                            .with_id(graph.id.clone())
                            .with_field(format!("passes[{i}].outputs")),
                        );
                    }
                }
            }
            produced.extend(pass.outputs.iter().cloned());
        }
    }

    for profile in &scan.render.profiles {
        if let Some(perf) = &performance {
            if !perf
                .quality_classes
                .iter()
                .any(|class| class == &profile.value.quality_class)
            {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!(
                            "render profile '{}' uses unknown quality_class '{}'",
                            profile.id, profile.value.quality_class
                        ),
                    )
                    .with_path(profile.rel_path.clone())
                    .with_id(profile.id.clone())
                    .with_field("quality_class"),
                );
            }
        }
        if let Some(budget) = &feature_budget {
            for feature in profile.value.feature_overrides.keys() {
                if !budget.known_features.iter().any(|known| known == feature) {
                    report.error(
                        Diagnostic::new(
                            CHECK,
                            format!(
                                "render profile '{}' overrides unknown feature '{}'",
                                profile.id, feature
                            ),
                        )
                        .with_path(profile.rel_path.clone())
                        .with_id(profile.id.clone())
                        .with_field("feature_overrides"),
                    );
                }
            }
        }
        report.planet.render_profiles.push(RenderProfileSummary {
            id: profile.id.clone(),
            label: profile.value.label.clone(),
            quality_class: profile.value.quality_class.clone(),
            fog: profile.value.fog,
            water: profile.value.water.clone(),
            enabled_features: profile
                .value
                .feature_overrides
                .values()
                .filter(|enabled| **enabled)
                .count(),
        });
    }
    report.planet.counts.render_profiles = report.planet.render_profiles.len();

    let _ = module_by_id;
}

fn check_technique_budget(
    report: &mut Report,
    feature_budget: &Option<RawFeatureBudget>,
    tech: &crate::scan::RenderItem<vv_content_schema::RawRenderTechnique>,
) {
    let Some(budget) = feature_budget else { return };
    if tech.value.features.len() > budget.max_features_per_technique {
        report.error(
            Diagnostic::new(
                CHECK,
                format!(
                    "technique '{}' enables {} features but budget allows {}",
                    tech.id,
                    tech.value.features.len(),
                    budget.max_features_per_technique
                ),
            )
            .with_path(tech.rel_path.clone())
            .with_id(tech.id.clone())
            .with_field("features"),
        );
    }
    for (i, feature) in tech.value.features.iter().enumerate() {
        check_known_feature(
            report,
            budget,
            &tech.id,
            &tech.rel_path,
            "features",
            i,
            feature,
        );
    }
}

fn check_technique_classes(
    report: &mut Report,
    performance: &Option<RawPerformanceClasses>,
    tech: &crate::scan::RenderItem<vv_content_schema::RawRenderTechnique>,
) {
    let Some(perf) = performance else { return };
    if !perf
        .render_passes
        .iter()
        .any(|pass| pass == &tech.value.pass)
    {
        report.error(
            Diagnostic::new(
                CHECK,
                format!(
                    "technique '{}' uses unknown render pass '{}'",
                    tech.id, tech.value.pass
                ),
            )
            .with_path(tech.rel_path.clone())
            .with_id(tech.id.clone())
            .with_field("pass"),
        );
    }
    if !perf
        .vertex_layouts
        .iter()
        .any(|layout| layout == &tech.value.vertex_layout)
    {
        report.error(
            Diagnostic::new(
                CHECK,
                format!(
                    "technique '{}' uses unknown vertex_layout '{}'",
                    tech.id, tech.value.vertex_layout
                ),
            )
            .with_path(tech.rel_path.clone())
            .with_id(tech.id.clone())
            .with_field("vertex_layout"),
        );
    }
    for (i, output) in tech.value.outputs.iter().enumerate() {
        if !perf.technique_outputs.iter().any(|known| known == output) {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!("technique '{}' writes unknown output '{}'", tech.id, output),
                )
                .with_path(tech.rel_path.clone())
                .with_id(tech.id.clone())
                .with_field(format!("outputs[{i}]")),
            );
        }
    }
}

fn read_validation<T: serde::de::DeserializeOwned>(
    scan: &PackScan,
    report: &mut Report,
    file_name: &str,
) -> Option<T> {
    let path = scan
        .pack_root
        .join("render")
        .join("validation")
        .join(file_name);
    if !path.exists() {
        report.error(
            Diagnostic::new(
                CHECK,
                format!("render validation preset '{}' is missing", file_name),
            )
            .with_path(format!("render/validation/{file_name}"))
            .with_suggestion(
                "render packs must declare validation presets, not rely on compiler defaults",
            ),
        );
        return None;
    }
    match read_typed::<T>(&scan.pack_root, &path) {
        Ok(value) => Some(value),
        Err(error) => {
            report.add_parse_error(&error);
            None
        }
    }
}

fn check_ref(
    report: &mut Report,
    reference: &str,
    valid: &BTreeSet<String>,
    parent_id: &str,
    parent_path: &str,
    field: &str,
) {
    if !valid.contains(reference) {
        report.error(
            Diagnostic::new(
                CHECK,
                format!(
                    "technique '{}' {} references unknown module '{}'",
                    parent_id, field, reference
                ),
            )
            .with_path(parent_path.to_string())
            .with_id(parent_id.to_string())
            .with_field(field.to_string()),
        );
    }
}

fn check_known_feature(
    report: &mut Report,
    budget: &RawFeatureBudget,
    parent_id: &str,
    parent_path: &str,
    field: &str,
    index: usize,
    feature: &str,
) {
    if !budget.known_features.iter().any(|known| known == feature) {
        report.error(
            Diagnostic::new(
                CHECK,
                format!(
                    "technique '{}' references unknown feature '{}'",
                    parent_id, feature
                ),
            )
            .with_path(parent_path.to_string())
            .with_id(parent_id.to_string())
            .with_field(format!("{field}[{index}]")),
        );
    }
}

fn check_wgsl_source(scan: &PackScan, report: &mut Report, module_id: &str, path: &Path) {
    let rel = pack_rel(&scan.pack_root, path);
    let Ok(source) = std::fs::read_to_string(path) else {
        report.error(
            Diagnostic::new(
                CHECK,
                format!("cannot read WGSL source for '{}'", module_id),
            )
            .with_path(rel)
            .with_id(module_id.to_string()),
        );
        return;
    };
    if source.trim().is_empty() {
        report.error(
            Diagnostic::new(CHECK, format!("WGSL source for '{}' is empty", module_id))
                .with_path(rel.clone())
                .with_id(module_id.to_string()),
        );
    }
    for (open, close, name) in [
        ('{', '}', "braces"),
        ('(', ')', "parentheses"),
        ('[', ']', "brackets"),
    ] {
        if !balanced_delimiters(&source, open, close) {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!("WGSL source for '{}' has unbalanced {}", module_id, name),
                )
                .with_path(rel.clone())
                .with_id(module_id.to_string()),
            );
        }
    }
}

fn balanced_delimiters(source: &str, open: char, close: char) -> bool {
    let mut depth = 0i32;
    for c in source.chars() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth < 0 {
                return false;
            }
        }
    }
    depth == 0
}

fn pack_rel(pack_root: &Path, abs: &Path) -> String {
    abs.strip_prefix(pack_root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| abs.display().to_string())
}
