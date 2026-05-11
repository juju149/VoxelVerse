//! Block-level checks: every placeable block needs an item, materials cover
//! the model slots, etc.

use std::collections::HashSet;

use crate::allowed::AllowedUnused;
use crate::report::Report;
use crate::scan::PackScan;

const CHECK: &str = "blocks";

pub fn run(scan: &PackScan, allowed: &AllowedUnused, report: &mut Report) {
    let used_blocks = collect_used_blocks(scan);

    for (id, block) in &scan.blocks {
        if !is_placeable(block) {
            continue;
        }
        let expected_item = expected_block_item_id(id);
        let has_item = scan.items.iter().any(|(item_id, item)| {
            item_id == &expected_item
                || matches!(
                    &item.gameplay,
                    vv_content_schema::RawItemGameplayDef::PlaceBlock(target)
                        if target.0 == *id
                )
        });
        if !has_item {
            report.missing.block_items.push(id.clone());
            report.warn_id(
                CHECK,
                format!("placeable block '{}' has no item", id),
                id.clone(),
            );
        }
    }

    // Unused blocks: not referenced in worldgen, loot, recipes, items.
    for (id, _) in &scan.blocks {
        if id.ends_with("/air") {
            continue;
        }
        if used_blocks.contains(id) {
            continue;
        }
        if allowed.blocks.contains(id) {
            continue;
        }
        report.unused.blocks.push(id.clone());
        report.warn_id(
            CHECK,
            format!(
                "block '{}' is not referenced by worldgen, loot, items, or recipes",
                id
            ),
            id.clone(),
        );
    }

    // Unused materials.
    let used_materials: HashSet<String> = scan
        .blocks
        .iter()
        .flat_map(|(_, block)| block.visual.materials.values().map(|m| m.0.clone()))
        .collect();
    for (mat_id, _) in &scan.materials {
        if used_materials.contains(mat_id) {
            continue;
        }
        if allowed.materials.contains(mat_id) {
            continue;
        }
        report.unused.materials.push(mat_id.clone());
        report.warn_id(
            CHECK,
            format!("material '{}' is not referenced by any block", mat_id),
            mat_id.clone(),
        );
    }
}

fn is_placeable(block: &vv_content_schema::RawBlockDef) -> bool {
    !matches!(block.visual.render, vv_content_schema::RawRenderMode::Invisible)
        && block.physical.solid
}

/// Convention: `core:block/<dirs>/<stem>` -> `core:item/block/<stem>`.
fn expected_block_item_id(block_id: &str) -> String {
    let stem = block_id.rsplit('/').next().unwrap_or(block_id);
    format!("core:item/block/{}", stem)
}

fn collect_used_blocks(scan: &PackScan) -> HashSet<String> {
    let mut out = HashSet::new();

    // Items that place blocks.
    for (_, item) in &scan.items {
        if let vv_content_schema::RawItemGameplayDef::PlaceBlock(b) = &item.gameplay {
            out.insert(b.0.clone());
        }
        if let vv_content_schema::RawItemWorldModel::BlockItem(b) = &item.visual.world_model {
            out.insert(b.0.clone());
        }
    }

    // Worldgen: terrain layers, ores, vegetation, biomes.
    for (_, layer_set) in &scan.procedural.terrain_layers {
        for l in &layer_set.layers {
            out.insert(l.block.0.clone());
        }
    }
    for (_, ore) in &scan.procedural.ores {
        out.insert(ore.block.0.clone());
        for r in &ore.replace {
            out.insert(r.0.clone());
        }
    }
    for (_, veg) in &scan.procedural.vegetation {
        out.insert(veg.stamp.trunk.0.clone());
        out.insert(veg.stamp.leaves.0.clone());
    }
    for (_, biome) in &scan.procedural.biomes {
        out.insert(biome.surface.top.0.clone());
        out.insert(biome.surface.under.0.clone());
        if let Some(slope) = &biome.surface.slope_override {
            out.insert(slope.top.0.clone());
        }
    }

    out
}
