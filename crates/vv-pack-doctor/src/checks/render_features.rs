//! Render-feature manifest validation.
//!
//! Walks `defs/render/{features,profiles}/` via
//! [`vv_pack_compiler::compile_pack_render_features`], which parses every
//! manifest, naga-validates each referenced WGSL shader, confirms declared
//! entry points exist with the right shader stage, and rejects unauthorised
//! `(kind, target, blend)` combinations.
//!
//! Anything that compiles cleanly becomes a `CompiledRenderFeature` in the
//! report's planet summary so downstream tooling can see what the pack
//! actually contributes to the render graph.

use vv_pack_compiler::{compile_pack_render_features, shader::PackShaderRoot};

use crate::report::{Diagnostic, Report};
use crate::scan::PackScan;

const CHECK: &str = "render-features";

pub fn run(scan: &PackScan, report: &mut Report) {
    let pack_stack = vec![PackShaderRoot::new("pack", scan.pack_root.clone())];
    let (registry, errors) = compile_pack_render_features(&pack_stack, &scan.pack_root);

    for err in errors {
        let mut diag =
            Diagnostic::new(CHECK, err.message.clone()).with_path(err.source_rel_path.clone());
        if let Some(feature) = err.feature_name.clone() {
            diag = diag.with_field(format!("feature '{feature}'"));
        }
        report.error(diag);
    }

    if !registry.features.is_empty() || !registry.profiles.is_empty() {
        report.planet.counts.render_features = registry.features.len();
        report.planet.counts.render_profiles =
            report.planet.counts.render_profiles.max(registry.profiles.len());
    }
}
