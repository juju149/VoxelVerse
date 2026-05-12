use std::path::Path;

use vv_pack_doctor::run;

fn core_pack() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core")
}

#[test]
fn core_pack_runs_and_summarizes() {
    let report = run(&core_pack()).expect("pack doctor should run on core pack");
    assert!(report.summary.blocks >= 20, "expected >= 20 blocks");
    assert!(report.summary.items >= 20, "expected >= 20 items");
    assert!(report.summary.textures >= 10, "expected >= 10 textures");
    assert!(report.health_score <= 100);
}

#[test]
fn core_pack_basic_progression_present() {
    let report = run(&core_pack()).expect("pack doctor should run");
    assert!(
        report.progression.basic_loop_reachable,
        "core pack must keep the first-hour loop reachable; notes = {:?}",
        report.progression.notes
    );
}

#[test]
fn core_pack_has_no_critical_reference_errors() {
    let report = run(&core_pack()).expect("pack doctor should run");
    let reference_errors: Vec<_> = report
        .errors
        .iter()
        .filter(|e| e.check == "references" || e.check == "worldgen")
        .collect();
    assert!(
        reference_errors.is_empty(),
        "core pack has broken references or worldgen entries: {:?}",
        reference_errors
    );
}
