//! Cross-reference checks.
//!
//! Validates that every `core:*` reference that maps to a content kind we
//! understand actually points at a known piece of content. References to
//! kinds we don't track (tags, icons, sounds, atlases, ...) are passed
//! through silently, mirroring the legacy `validate_content.ps1`.

use vv_content_schema::ContentRef;

use crate::report::Report;
use crate::scan::PackScan;

const CHECK: &str = "references";

pub fn run(scan: &PackScan, report: &mut Report) {
    // Block references.
    for (id, block) in &scan.blocks {
        check_block_model_ref(scan, report, id, &block.model);
        check_loot_ref(scan, report, id, &block.gameplay.drops);
        for (slot, mat) in &block.visual.materials {
            check_material_ref(scan, report, id, mat, slot);
        }
    }

    // Item references.
    for (id, item) in &scan.items {
        if let vv_content_schema::RawItemGameplayDef::PlaceBlock(target) = &item.gameplay {
            if !scan.block_id_exists(&target.0) {
                report.error_id(
                    CHECK,
                    format!(
                        "item '{}' places block '{}' which does not exist",
                        id, target.0
                    ),
                    id.clone(),
                );
            }
        }
        if let vv_content_schema::RawItemWorldModel::BlockItem(target) = &item.visual.world_model {
            if !scan.block_id_exists(&target.0) {
                report.error_id(
                    CHECK,
                    format!(
                        "item '{}' world_model references missing block '{}'",
                        id, target.0
                    ),
                    id.clone(),
                );
            }
        }
    }

    // Material references.
    for (id, mat) in &scan.materials {
        check_texture_ref(scan, report, id, &mat.albedo, "albedo");
        if let Some(n) = &mat.normal {
            check_texture_ref(scan, report, id, n, "normal");
        }
        if let Some(r) = &mat.roughness {
            check_texture_ref(scan, report, id, r, "roughness");
        }
    }

    // Loot references.
    for (id, table) in &scan.loot {
        for entry in &table.entries {
            if !scan.item_id_exists(&entry.item.0) {
                report.error_id(
                    CHECK,
                    format!(
                        "loot table '{}' drops missing item '{}'",
                        id, entry.item.0
                    ),
                    id.clone(),
                );
            }
        }
    }
}

fn check_block_model_ref(scan: &PackScan, report: &mut Report, block_id: &str, mref: &ContentRef) {
    if !scan.block_model_id_exists(&mref.0) {
        report.error_id(
            CHECK,
            format!(
                "block '{}' references missing block model '{}'",
                block_id, mref.0
            ),
            block_id.to_string(),
        );
    }
}

fn check_loot_ref(scan: &PackScan, report: &mut Report, block_id: &str, lref: &ContentRef) {
    if !scan.loot_id_exists(&lref.0) {
        report.error_id(
            CHECK,
            format!(
                "block '{}' drops missing loot table '{}'",
                block_id, lref.0
            ),
            block_id.to_string(),
        );
    }
}

fn check_material_ref(
    scan: &PackScan,
    report: &mut Report,
    block_id: &str,
    mref: &ContentRef,
    slot: &str,
) {
    if !scan.material_id_exists(&mref.0) {
        report.error_id(
            CHECK,
            format!(
                "block '{}' slot '{}' references missing material '{}'",
                block_id, slot, mref.0
            ),
            block_id.to_string(),
        );
    }
}

fn check_texture_ref(
    scan: &PackScan,
    report: &mut Report,
    material_id: &str,
    tref: &ContentRef,
    role: &str,
) {
    if !scan.texture_id_exists(&tref.0) {
        report.error_id(
            CHECK,
            format!(
                "material '{}' {} references missing texture '{}'",
                material_id, role, tref.0
            ),
            material_id.to_string(),
        );
    }
}
