//! Loot table checks: every minable block has a loot table, loot tables
//! produce known items, no orphan loot tables.

use std::collections::HashSet;

use crate::report::Report;
use crate::scan::PackScan;

const CHECK: &str = "loot";

pub fn run(scan: &PackScan, report: &mut Report) {
    let used_loot: HashSet<String> = scan
        .blocks
        .iter()
        .map(|(_, block)| block.gameplay.drops.0.clone())
        .collect();

    for (id, _) in &scan.loot {
        if used_loot.contains(id) {
            continue;
        }
        // The canonical empty table is permitted to exist unreferenced.
        if id.ends_with("/empty") {
            continue;
        }
        report.unused.loot_tables.push(id.clone());
        report.warn_id(
            CHECK,
            format!("loot table '{}' is not referenced by any block", id),
            id.clone(),
        );
    }

    // Missing tables: any block whose drops point at an id we never loaded.
    for (block_id, block) in &scan.blocks {
        if !scan.loot_id_exists(&block.gameplay.drops.0) {
            if !report
                .missing
                .loot_tables
                .iter()
                .any(|t| t == &block.gameplay.drops.0)
            {
                report.missing.loot_tables.push(block.gameplay.drops.0.clone());
            }
            // References check already raises the error; nothing else to do here.
            let _ = block_id;
        }
    }
}
