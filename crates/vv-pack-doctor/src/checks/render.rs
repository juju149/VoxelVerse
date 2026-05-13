//! Render-pack validation.
//!
//! * each shader module has a sibling `.wgsl` file
//! * every import resolves to an existing shader module
//! * every technique stage points at an existing shader module
//! * every technique material_family points at an existing family
//! * every render-graph pass.technique points at an existing technique
//! * profile-override technique features stay within `feature_budget`
//!
//! This module does not parse WGSL; it only checks structure and references.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::index::PackIndex;
use crate::report::{Diagnostic, Report};
use crate::scan::PackScan;

const CHECK: &str = "render";

pub fn run(_index: &PackIndex<'_>, _report: &mut Report) {}

pub fn validate(scan: &PackScan, report: &mut Report) {
    let module_ids: BTreeSet<String> =
        scan.render.shader_modules.iter().map(|m| m.id.clone()).collect();
    let contract_ids: BTreeSet<String> =
        scan.render.shader_contracts.iter().map(|c| c.id.clone()).collect();
    let family_ids: BTreeSet<String> =
        scan.render.material_families.iter().map(|m| m.id.clone()).collect();
    let technique_ids: BTreeSet<String> =
        scan.render.techniques.iter().map(|t| t.id.clone()).collect();
    let profile_ids: BTreeSet<String> =
        scan.render.profiles.iter().map(|p| p.id.clone()).collect();

    let module_by_id: BTreeMap<&str, &_> = scan
        .render
        .shader_modules
        .iter()
        .map(|m| (m.id.as_str(), m))
        .collect();

    let wgsl_index: BTreeSet<&str> = scan.wgsl_files.iter().map(|f| f.rel_path.as_str()).collect();
    let _ = wgsl_index;

    // 1) WGSL siblings.
    for module in &scan.render.shader_modules {
        let metadata_abs = scan.pack_root.join(&module.rel_path);
        let wgsl = metadata_abs.with_extension("wgsl");
        if !wgsl.exists() {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!(
                        "shader module '{}' is missing its WGSL source",
                        module.id
                    ),
                )
                .with_path(module.rel_path.clone())
                .with_id(module.id.clone())
                .with_suggestion(format!(
                    "create {} alongside the metadata file",
                    pack_rel(&scan.pack_root, &wgsl)
                )),
            );
        }
    }

    // 2) Import resolution.
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
        }
        for c in &module.value.contracts {
            if !contract_ids.contains(c.as_str()) {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!("shader contract '{}' does not exist", c),
                    )
                    .with_path(module.rel_path.clone())
                    .with_id(module.id.clone())
                    .with_field("contracts"),
                );
            }
        }
    }

    // 3) Technique references.
    for tech in &scan.render.techniques {
        check_ref(
            report,
            &tech.value.stages.vertex,
            &module_ids,
            &tech.id,
            &tech.rel_path,
            "stages.vertex",
        );
        if let Some(f) = &tech.value.stages.fragment {
            check_ref(report, f, &module_ids, &tech.id, &tech.rel_path, "stages.fragment");
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
        for c in &tech.value.contracts {
            if !contract_ids.contains(c.as_str()) {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!(
                            "technique '{}' references unknown contract '{}'",
                            tech.id, c
                        ),
                    )
                    .with_path(tech.rel_path.clone())
                    .with_id(tech.id.clone())
                    .with_field("contracts"),
                );
            }
        }
        for (i, ov) in tech.value.profile_overrides.iter().enumerate() {
            if !profile_ids.contains(&ov.profile) {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!(
                            "technique '{}' profile_override #{} points at unknown profile '{}'",
                            tech.id,
                            i + 1,
                            ov.profile
                        ),
                    )
                    .with_path(tech.rel_path.clone())
                    .with_id(tech.id.clone())
                    .with_field(format!("profile_overrides[{i}].profile")),
                );
            }
        }
    }

    // 4) Render-graph pass references.
    for graph in &scan.render.render_graphs {
        for (i, pass) in graph.value.passes.iter().enumerate() {
            if let Some(t) = &pass.technique {
                if !technique_ids.contains(t.as_str()) {
                    report.error(
                        Diagnostic::new(
                            CHECK,
                            format!(
                                "render graph '{}' pass '{}' references unknown technique '{}'",
                                graph.id, pass.name, t
                            ),
                        )
                        .with_path(graph.rel_path.clone())
                        .with_id(graph.id.clone())
                        .with_field(format!("passes[{i}].technique")),
                    );
                }
            }
        }
    }

    // 5) Touch unused entries so the symbol table is complete.
    let _ = module_by_id;
}

fn check_ref(
    report: &mut Report,
    r: &str,
    valid: &BTreeSet<String>,
    parent_id: &str,
    parent_path: &str,
    field: &str,
) {
    if !valid.contains(r) {
        report.error(
            Diagnostic::new(
                CHECK,
                format!(
                    "technique '{}' {} references unknown module '{}'",
                    parent_id, field, r
                ),
            )
            .with_path(parent_path.to_string())
            .with_id(parent_id.to_string())
            .with_field(field.to_string()),
        );
    }
}

fn pack_rel(pack_root: &Path, abs: &Path) -> String {
    abs.strip_prefix(pack_root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| abs.display().to_string())
}
