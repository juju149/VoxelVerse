//! Progression sanity check.
//!
//! V1 confirms that the canonical first-hour gameplay loop is reachable.
//! Each canonical block id (suffix-match) must exist and have a `block`
//! section. The list is data-derived (objects in the pack) — we just check
//! suffixes so different category prefixes still work.

use crate::report::{Diagnostic, Report};
use crate::scan::PackScan;

const CHECK: &str = "progression";

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
        let found = scan
            .objects
            .iter()
            .any(|o| o.id.ends_with(suffix) && o.def.block.is_some());
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
            let name = s.trim_start_matches('/');
            report.progression.notes.push(format!(
                "MVP block '{}' is missing - first-hour loop is broken.",
                name
            ));
            report.warn(Diagnostic::new(CHECK, format!("MVP block '{}' missing", name)));
        }
    }
}
