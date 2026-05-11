//! Progression sanity check.
//!
//! V1 is intentionally narrow: it confirms the canonical first-hour loop
//! pieces exist. It is not a full reachability solver.

use crate::report::Report;
use crate::scan::PackScan;

const CHECK: &str = "progression";

/// Blocks the basic loop relies on (core MVP). Each entry is the block id
/// suffix - we match by `ends_with` so different category prefixes still work.
const BASIC_BLOCK_SUFFIXES: &[&str] = &[
    "/grass",
    "/dirt",
    "/stone",
    "/coal_ore",
    "/iron_ore",
    "/oak_log",
];

pub fn run(scan: &PackScan, report: &mut Report) {
    let mut missing = Vec::new();
    for suffix in BASIC_BLOCK_SUFFIXES {
        let found = scan.blocks.iter().any(|(id, _)| id.ends_with(suffix));
        if !found {
            missing.push((*suffix).to_string());
        }
    }

    if missing.is_empty() {
        report.progression.basic_loop_reachable = true;
        report.progression.notes.push(
            "Basic loop blocks all present (grass, dirt, stone, oak_log, coal_ore, iron_ore)."
                .to_string(),
        );
    } else {
        report.progression.basic_loop_reachable = false;
        for s in &missing {
            report.progression.notes.push(format!(
                "MVP block '{}' is missing - first-hour loop is broken.",
                s.trim_start_matches('/')
            ));
            report.warn(
                CHECK,
                format!("MVP block '{}' missing", s.trim_start_matches('/')),
            );
        }
    }
}
