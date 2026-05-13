//! End-to-end smoke test: run Pack Doctor against the bundled `core` pack
//! and confirm the pipeline survives whatever state the pack is in.
//!
//! The pipeline is tolerant by design — parse errors become diagnostics
//! rather than panics — so the test always succeeds in *executing* the
//! checks. Concrete content assertions live in dedicated rules tests below.

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
fn core_pack_emits_only_structured_diagnostics() {
    // Every error must have at least one of (path, id) so the report is
    // actionable. Anonymous errors are a regression.
    let report = run(&core_pack()).expect("pack doctor should run");
    for err in &report.errors {
        assert!(
            err.path.is_some() || err.id.is_some(),
            "error has neither path nor id: {:?}",
            err
        );
    }
}
