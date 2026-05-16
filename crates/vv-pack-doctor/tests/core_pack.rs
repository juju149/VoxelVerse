//! End-to-end V1 gate for the bundled `core` pack.
//!
//! Pack Doctor is allowed to keep scanning after broken files, but the core
//! V1 pack itself must not ship with diagnostics.

use std::path::Path;

use vv_pack_doctor::run;

fn core_pack() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core")
}

#[test]
fn core_pack_pipeline_runs_to_completion() {
    let report = run(&core_pack()).expect("pack doctor should always run");
    assert!(
        report.summary.world_files + report.summary.items > 0,
        "scan must have discovered some content"
    );
    assert!(report.health_score <= 100);
}

#[test]
fn core_pack_v1_has_no_diagnostics() {
    let report = run(&core_pack()).expect("pack doctor should run");
    assert!(
        report.errors.is_empty(),
        "core pack must have zero Pack Doctor errors:\n{:#?}",
        report.errors
    );
    assert!(
        report.warnings.is_empty(),
        "core pack must have zero Pack Doctor warnings:\n{:#?}",
        report.warnings
    );
    assert_eq!(report.health_score, 100);
}

#[test]
fn core_pack_v1_counts_recipes_and_media_contracts() {
    let report = run(&core_pack()).expect("pack doctor should run");
    assert!(report.summary.recipes > 0, "recipes must not be ignored");
    assert!(report.summary.items > 0, "items must be discovered");
    assert!(report.summary.voxels > 0, "voxel media must be discovered");
}

#[test]
fn core_pack_emits_only_structured_diagnostics() {
    let report = run(&core_pack()).expect("pack doctor should run");
    for err in &report.errors {
        assert!(
            err.path.is_some() || err.id.is_some(),
            "error has neither path nor id: {:?}",
            err
        );
    }
}
