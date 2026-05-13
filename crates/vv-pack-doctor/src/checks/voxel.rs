//! Voxel-model checks.
//!
//! Confirms every `.vox` file referenced from a parsed object resolves to a
//! real file on disk, and flags `.vox` files that no parsed object references.
//! Cross-checks against `generated/registries/voxel_assets.ron` are intentionally
//! out of scope: that file is produced by the pipeline, not authored by hand.

use std::collections::BTreeSet;

use crate::index::PackIndex;
use crate::report::{Diagnostic, Report};

const CHECK: &str = "voxel";

pub fn run(index: &PackIndex<'_>, report: &mut Report) {
    let mut referenced = BTreeSet::new();
    for obj in &index.scan.objects {
        if let Some(entity) = &obj.def.entity {
            if let Some(model) = &entity.model {
                let candidate = format!("media/{}.vox", model);
                referenced.insert(candidate.clone());
                if !index.voxel_exists(&candidate) {
                    report.error(
                        Diagnostic::new(
                            CHECK,
                            format!("entity model '{}' is missing on disk", model),
                        )
                        .with_path(obj.rel_path.clone())
                        .with_id(obj.id.clone())
                        .with_field("entity.model")
                        .with_suggestion(format!(
                            "create {} (note: paths under `entity.model` start with `voxel/`)",
                            candidate
                        )),
                    );
                }
            }
        }
        if let Some(item) = &obj.def.item {
            if let Some(model) = &item.model {
                referenced.insert(format!("media/{}.vox", model));
            }
        }
    }
    for world in &index.scan.world_files {
        collect_world_models(&world.value, &mut Vec::new(), &mut referenced);
    }

    for file in &index.scan.voxel_files {
        if !referenced.contains(&file.rel_path) {
            report.unused.voxels.push(file.rel_path.clone());
        }
    }
}

fn collect_world_models(
    value: &ron::Value,
    path: &mut Vec<String>,
    referenced: &mut BTreeSet<String>,
) {
    if path.last().map(String::as_str) == Some("model") {
        if let ron::Value::String(model) = value {
            referenced.insert(voxel_candidate(model));
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
                collect_world_models(child, path, referenced);
                path.pop();
            }
        }
        ron::Value::Seq(seq) => {
            for (i, child) in seq.iter().enumerate() {
                path.push(format!("[{i}]"));
                collect_world_models(child, path, referenced);
                path.pop();
            }
        }
        ron::Value::Option(Some(child)) => collect_world_models(child, path, referenced),
        _ => {}
    }
}

fn voxel_candidate(model: &str) -> String {
    let stripped = model.strip_prefix("core:").unwrap_or(model);
    if stripped.starts_with("voxel/") {
        format!("media/{stripped}.vox")
    } else {
        format!("media/voxel/{stripped}.vox")
    }
}
