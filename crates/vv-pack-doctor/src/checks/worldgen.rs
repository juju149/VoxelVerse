//! Worldgen reference checks: every block referenced by terrain, ores,
//! vegetation, and biomes must exist.

use crate::report::Report;
use crate::scan::PackScan;

const CHECK: &str = "worldgen";

pub fn run(scan: &PackScan, report: &mut Report) {
    for (id, set) in &scan.procedural.terrain_layers {
        for (i, layer) in set.layers.iter().enumerate() {
            if !scan.block_id_exists(&layer.block.0) {
                report.error_id(
                    CHECK,
                    format!(
                        "terrain layers '{}' layer {} references missing block '{}'",
                        id, i, layer.block.0
                    ),
                    id.clone(),
                );
            }
        }
    }

    for (id, ore) in &scan.procedural.ores {
        if !scan.block_id_exists(&ore.block.0) {
            report.error_id(
                CHECK,
                format!("ore '{}' references missing block '{}'", id, ore.block.0),
                id.clone(),
            );
        }
        for r in &ore.replace {
            if !scan.block_id_exists(&r.0) {
                report.error_id(
                    CHECK,
                    format!("ore '{}' replace target '{}' does not exist", id, r.0),
                    id.clone(),
                );
            }
        }
    }

    for (id, veg) in &scan.procedural.vegetation {
        if !scan.block_id_exists(&veg.stamp.trunk.0) {
            report.error_id(
                CHECK,
                format!(
                    "vegetation '{}' trunk '{}' does not exist",
                    id, veg.stamp.trunk.0
                ),
                id.clone(),
            );
        }
        if !scan.block_id_exists(&veg.stamp.leaves.0) {
            report.error_id(
                CHECK,
                format!(
                    "vegetation '{}' leaves '{}' does not exist",
                    id, veg.stamp.leaves.0
                ),
                id.clone(),
            );
        }
    }

    for (id, biome) in &scan.procedural.biomes {
        if !scan.block_id_exists(&biome.surface.top.0) {
            report.error_id(
                CHECK,
                format!(
                    "biome '{}' surface top '{}' does not exist",
                    id, biome.surface.top.0
                ),
                id.clone(),
            );
        }
        if !scan.block_id_exists(&biome.surface.under.0) {
            report.error_id(
                CHECK,
                format!(
                    "biome '{}' surface under '{}' does not exist",
                    id, biome.surface.under.0
                ),
                id.clone(),
            );
        }
        if let Some(slope) = &biome.surface.slope_override {
            if !scan.block_id_exists(&slope.top.0) {
                report.error_id(
                    CHECK,
                    format!(
                        "biome '{}' slope override top '{}' does not exist",
                        id, slope.top.0
                    ),
                    id.clone(),
                );
            }
        }
    }
}
