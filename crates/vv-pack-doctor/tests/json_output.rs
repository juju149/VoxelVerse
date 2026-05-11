//! JSON output smoke test: render against the core pack and confirm the
//! shape matches the documented contract.

use std::path::Path;

use vv_pack_doctor::{output, run};

#[test]
fn json_output_contains_expected_top_level_keys() {
    let core_pack = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core");
    let report = run(&core_pack).expect("pack doctor should run");
    let json = output::json::render(&report);

    for key in [
        "\"pack\":",
        "\"health_score\":",
        "\"summary\":",
        "\"errors\":",
        "\"warnings\":",
        "\"unused\":",
        "\"missing\":",
        "\"progression\":",
        "\"basic_loop_reachable\":",
    ] {
        assert!(
            json.contains(key),
            "expected JSON to contain {} but it did not.\nJSON:\n{}",
            key,
            json
        );
    }
}
