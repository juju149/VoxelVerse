//! Path-as-identity coherence.
//!
//! VoxelVerse derives the runtime id of any content from the file path. This
//! check enforces:
//!   - object files end in `.object.ron`
//!   - world files end in `.<category>.ron` (biome, ore, vegetation, …)
//!   - the namespace declared in `pack.ron` matches the pack directory
//!   - no two parsed objects collapse to the same short id (ambiguity)

use std::collections::BTreeMap;

use crate::index::PackIndex;
use crate::report::{Diagnostic, Report};

const CHECK: &str = "paths";

pub fn run(index: &PackIndex<'_>, report: &mut Report) {
    let scan = index.scan;

    // Manifest namespace ⇄ directory name.
    if let Some(m) = &scan.manifest {
        let dir = scan
            .pack_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if m.namespace != dir {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!(
                        "pack.ron namespace '{}' does not match directory '{}'",
                        m.namespace, dir
                    ),
                )
                .with_path("pack.ron".to_string())
                .with_field("namespace")
                .with_suggestion(
                    "rename the pack directory or the namespace so they match".to_string(),
                ),
            );
        }
    }

    // Object filenames must end in `.object.ron`.
    for obj in &scan.objects {
        if !obj.rel_path.ends_with(".object.ron") {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!(
                        "object file '{}' does not use the `.object.ron` suffix",
                        obj.rel_path
                    ),
                )
                .with_path(obj.rel_path.clone())
                .with_id(obj.id.clone())
                .with_suggestion(
                    "rename the file so it ends in `.object.ron`".to_string(),
                ),
            );
        }
    }

    // World filenames must use the expected suffix per category.
    for file in &scan.world_files {
        let expected = expected_suffix_for(file.category);
        if let Some(expected) = expected {
            if !file.rel_path.ends_with(expected) {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!(
                            "world file '{}' does not use the expected `{}` suffix",
                            file.rel_path, expected
                        ),
                    )
                    .with_path(file.rel_path.clone())
                    .with_id(file.id.clone()),
                );
            }
        }
    }

    // Ambiguous short names — two objects whose final path segment collapses
    // to the same short id break short-reference resolution.
    let mut by_short: BTreeMap<String, Vec<&crate::scan::ParsedObject>> = BTreeMap::new();
    for obj in &scan.objects {
        let short = obj
            .id
            .rsplit('/')
            .next()
            .unwrap_or(obj.id.as_str())
            .to_string();
        by_short.entry(short).or_default().push(obj);
    }
    for (short, objs) in by_short {
        if objs.len() > 1 {
            let ids: Vec<String> = objs.iter().map(|o| o.id.clone()).collect();
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!(
                        "short id '{}' is owned by multiple objects: {}",
                        short,
                        ids.join(", ")
                    ),
                )
                .with_suggestion(
                    "rename one of the files so each short id is unique across the pack"
                        .to_string(),
                ),
            );
        }
    }
}

fn expected_suffix_for(c: crate::scan::WorldCategory) -> Option<&'static str> {
    use crate::scan::WorldCategory::*;
    match c {
        Biome => Some(".biome.ron"),
        BiomeSet => Some(".biome_set.ron"),
        Caves => Some(".cave.ron"),
        Climate => Some(".climate.ron"),
        Noise => Some(".field.ron"),
        Ores => Some(".ore.ron"),
        Planets => Some(".profile.ron"),
        Structures => Some(".structure.ron"),
        Terrain => Some(".terrain_layers.ron"),
        Vegetation => Some(".vegetation.ron"),
        PropScatter => Some(".prop_scatter.ron"),
        Props | Other => None,
    }
}
