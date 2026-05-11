//! VoxelVerse Pack Doctor
//!
//! Validates a content pack against the rules described in
//! `docs/content_rules.md` and the per-asset checklists under
//! `assets/packs/<pack>/source/production/`.
//!
//! The doctor never writes inside the pack itself; it only writes reports to
//! the path the caller supplies (usually `<pack>/generated/reports/`).

pub mod allowed;
pub mod checks;
pub mod output;
pub mod report;
pub mod scan;

use std::path::Path;

use crate::report::Report;

/// Run every V1 check against a pack root and return the assembled report.
pub fn run(pack_root: &Path) -> Result<Report, String> {
    let mut report = Report::new(pack_root);
    let scan = scan::PackScan::scan(pack_root)?;
    let allowed = allowed::AllowedUnused::load(pack_root)?;

    checks::filesystem::run(&scan, &mut report);
    checks::naming::run(&scan, &mut report);
    checks::references::run(&scan, &mut report);
    checks::textures::run(&scan, &allowed, &mut report);
    checks::blocks::run(&scan, &allowed, &mut report);
    checks::items::run(&scan, &allowed, &mut report);
    checks::recipes::run(&scan, &mut report);
    checks::loot::run(&scan, &mut report);
    checks::worldgen::run(&scan, &mut report);
    checks::progression::run(&scan, &mut report);

    report.finalize(&scan);
    Ok(report)
}
