//! VoxelVerse Pack Doctor.
//!
//! Walks a content pack, parses every `.ron` file individually, builds a
//! reference graph and runs a series of checks. Nothing is allowed to fail
//! silently — broken parses, dangling references, missing assets and
//! inconsistent paths all become structured diagnostics with a path, a field
//! and a suggested fix.
//!
//! The doctor never writes inside the pack itself; it only writes JSON / HTML
//! reports to a caller-provided destination.

pub mod allowed;
pub mod checks;
pub mod index;
pub mod output;
pub mod parse;
pub mod report;
pub mod scan;

use std::path::Path;

use crate::index::PackIndex;
use crate::report::Report;

/// Run every check against a pack root and return the assembled report.
pub fn run(pack_root: &Path) -> Result<Report, String> {
    let mut report = Report::new(pack_root);
    let scan = scan::PackScan::scan(pack_root)?;
    let allowed = allowed::AllowedUnused::load(pack_root)?;
    let index = PackIndex::build(&scan);

    checks::parse::run(&scan, &mut report);
    checks::filesystem::run(&scan, &mut report);
    checks::naming::run(&scan, &mut report);
    checks::source_contract::run(&scan, &mut report);
    checks::paths::run(&index, &mut report);

    checks::blocks::run(&index, &mut report);
    checks::items::run(&index, &mut report);
    checks::recipes::run(&index, &mut report);
    checks::loot::run(&index, &mut report);
    checks::references::run(&index, &mut report);
    checks::tags::run(&index, &mut report);
    checks::stations::run(&index, &mut report);

    checks::textures::run(&index, &allowed, &mut report);
    checks::voxel::run(&index, &mut report);
    checks::render::validate(&scan, &mut report);
    checks::render_features::run(&scan, &mut report);
    checks::worldgen::run(&index, &mut report);
    checks::weather_cosmos::run(&scan, &mut report);

    checks::progression::run(&scan, &mut report);

    report.finalize(&scan);
    Ok(report)
}
