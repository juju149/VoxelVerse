//! Voxel-model checks.
//!
//! Confirms every stable `.vox` used by gameplay is declared through a
//! `defs/voxel_models/**/*.voxel_model.ron` manifest. Objects reference
//! manifests; manifests reference raw media.

use std::collections::BTreeSet;

use crate::index::PackIndex;
use crate::report::{Diagnostic, Report};

const CHECK: &str = "voxel";

pub fn run(index: &PackIndex<'_>, report: &mut Report) {
    let mut manifest_sources = BTreeSet::new();
    for manifest in &index.scan.voxel_models {
        if !manifest.rel_path.ends_with(".voxel_model.ron") {
            report.error(
                Diagnostic::new(
                    CHECK,
                    "voxel model manifest must use `.voxel_model.ron` suffix",
                )
                .with_path(manifest.rel_path.clone())
                .with_id(manifest.id.clone()),
            );
        }
        let candidate = voxel_source_candidate(&manifest.def.source);
        manifest_sources.insert(candidate.clone());
        if !index.voxel_exists(&candidate) {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!(
                        "voxel model manifest source '{}' is missing on disk",
                        manifest.def.source
                    ),
                )
                .with_path(manifest.rel_path.clone())
                .with_id(manifest.id.clone())
                .with_field("source")
                .with_suggestion(format!("create {candidate} or fix the manifest source")),
            );
        }
        if manifest.def.usage.is_empty() {
            report.error(
                Diagnostic::new(CHECK, "voxel model manifest usage list must not be empty")
                    .with_path(manifest.rel_path.clone())
                    .with_id(manifest.id.clone())
                    .with_field("usage"),
            );
        }
    }

    for obj in &index.scan.objects {
        if let Some(entity) = &obj.def.entity {
            if let Some(model) = &entity.model {
                if !index.voxel_model_exists(model) {
                    report.error(
                        Diagnostic::new(
                            CHECK,
                            format!("entity model '{}' has no voxel model manifest", model),
                        )
                        .with_path(obj.rel_path.clone())
                        .with_id(obj.id.clone())
                        .with_field("entity.model")
                        .with_suggestion(format!(
                            "create defs/voxel_models/**/{}.voxel_model.ron",
                            model.rsplit('/').next().unwrap_or(model)
                        )),
                    );
                }
            }
        }
    }
    for world in &index.scan.world_files {
        collect_world_models(&world.value, &mut Vec::new(), &mut manifest_sources);
    }

    for file in &index.scan.voxel_files {
        if !manifest_sources.contains(&file.rel_path) {
            report.unused.voxels.push(file.rel_path.clone());
        }
    }
}

fn collect_world_models(
    value: &ron::Value,
    path: &mut Vec<String>,
    referenced_sources: &mut BTreeSet<String>,
) {
    if path.last().map(String::as_str) == Some("model") {
        if let ron::Value::String(model) = value {
            referenced_sources.insert(voxel_source_candidate(model));
        }
    }
    match value {
        ron::Value::Map(map) => {
            for (key, child) in map.iter() {
                let key = match key {
                    ron::Value::String(s) => s.clone(),
                    ron::Value::Char(c) => c.to_string(),
                    other => format!("{other:?}"),
                };
                path.push(key);
                collect_world_models(child, path, referenced_sources);
                path.pop();
            }
        }
        ron::Value::Seq(seq) => {
            for (i, child) in seq.iter().enumerate() {
                path.push(format!("[{i}]"));
                collect_world_models(child, path, referenced_sources);
                path.pop();
            }
        }
        ron::Value::Option(Some(child)) => collect_world_models(child, path, referenced_sources),
        _ => {}
    }
}

fn voxel_source_candidate(model: &str) -> String {
    let stripped = model.strip_prefix("core:").unwrap_or(model);
    if stripped.starts_with("voxel/") {
        format!("media/{stripped}.vox")
    } else {
        format!("media/voxel/{stripped}.vox")
    }
}
