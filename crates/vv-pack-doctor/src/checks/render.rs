//! Render shader-library validation.
//!
//! Render pipelines are Rust-owned. The pack only supplies WGSL source under
//! `render/shaders`, so Pack Doctor validates paths and conventions instead
//! of parsing render `.ron` manifests.
//!
//! Include expansion and resolution go through `vv_pack_compiler::shader`,
//! the same module the renderer uses at runtime. Each pass shader is then
//! naga-parsed so a broken pack fails Pack Doctor *before* it could ever
//! crash `device.create_shader_module`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use vv_pack_compiler::shader::{self, EnumeratedShader, PackShaderRoot, ShaderResolver};

use crate::index::PackIndex;
use crate::report::{Diagnostic, RenderProfileSummary, Report};
use crate::scan::PackScan;

const CHECK: &str = "render";
const MAX_SHADER_LINES: usize = 420;

/// Engine-required shader files (paths relative to `render/shaders/`).
/// Mirrors `vv_render::pipeline::graph::ShaderPath::REQUIRED`. Keep in sync.
const EXPECTED_SHADERS: &[&str] = &[
    "passes/terrain/terrain.vert.wgsl",
    "passes/terrain/terrain.frag.wgsl",
    "passes/terrain/terrain_depth.vert.wgsl",
    "passes/sky/sky.vert.wgsl",
    "passes/sky/sky.frag.wgsl",
    "passes/clouds/clouds.vert.wgsl",
    "passes/clouds/clouds.frag.wgsl",
    "passes/atmosphere/volumetric_fog.vert.wgsl",
    "passes/atmosphere/volumetric_fog.frag.wgsl",
    "passes/water/water.vert.wgsl",
    "passes/water/water.frag.wgsl",
    "passes/foliage/foliage.vert.wgsl",
    "passes/foliage/foliage.frag.wgsl",
    "passes/post/fullscreen.vert.wgsl",
    "passes/post/final_composite.frag.wgsl",
    "passes/post/fxaa.frag.wgsl",
    "passes/post/bloom_downsample.frag.wgsl",
    "passes/post/bloom_upsample.frag.wgsl",
    "passes/ui/ui.vert.wgsl",
    "passes/ui/ui.frag.wgsl",
    "passes/debug/normals.frag.wgsl",
    "passes/debug/depth.frag.wgsl",
    "passes/debug/lighting.frag.wgsl",
];

const REQUIRED_INCLUDE_DIRS: &[&str] = &[
    "render/shaders/include/common",
    "render/shaders/include/interface",
    "render/shaders/include/voxel",
];

pub fn run(_index: &PackIndex<'_>, _report: &mut Report) {}

pub fn validate(scan: &PackScan, report: &mut Report) {
    // 1. Hard structural rules that don't need the resolver.
    for ron in &scan.render.ron_files {
        report.error(
            Diagnostic::new(
                CHECK,
                "render directory must not contain .ron pipeline manifests",
            )
            .with_path(ron.clone())
            .with_suggestion("move pipeline/profile/render-graph ownership to vv-render Rust"),
        );
    }

    for dir in REQUIRED_INCLUDE_DIRS {
        if !scan.pack_root.join(dir).is_dir() {
            report.error(
                Diagnostic::new(CHECK, format!("shader include directory is missing: {dir}"))
                    .with_path((*dir).to_string()),
            );
        }
    }

    for file in &scan.render.wgsl_files {
        if !file.rel_path.starts_with("render/shaders/") {
            report.error(
                Diagnostic::new(CHECK, "WGSL render shader is outside render/shaders")
                    .with_path(file.rel_path.clone()),
            );
        }
    }

    check_duplicate_names(scan, report);
    check_oversized_files(scan, report);

    // 2. Build the resolver as a single-pack stack and validate every shader
    //    through the same include-expansion path the renderer uses.
    let pack_stack = vec![PackShaderRoot::new("pack", scan.pack_root.clone())];
    let mut resolver = match ShaderResolver::new(&pack_stack) {
        Ok(r) => r,
        Err(e) => {
            // No shader root at all: nothing more to do, but still finalize
            // the render-profile section so downstream report shape stays
            // stable.
            report.error(
                Diagnostic::new(CHECK, format!("cannot build shader resolver: {e}"))
                    .with_path("render/shaders".to_string()),
            );
            finalize_profiles(report);
            return;
        }
    };

    let enumerated = match resolver.enumerate_wgsl() {
        Ok(list) => list,
        Err(e) => {
            report.error(
                Diagnostic::new(CHECK, format!("cannot enumerate shaders: {e}"))
                    .with_path("render/shaders".to_string()),
            );
            finalize_profiles(report);
            return;
        }
    };

    let enumerated_paths: BTreeSet<String> = enumerated
        .iter()
        .map(|e| rel_as_str(&e.relative_path))
        .collect();

    for expected in EXPECTED_SHADERS {
        if !enumerated_paths.contains(*expected) {
            let full = format!("render/shaders/{expected}");
            report.error(
                Diagnostic::new(CHECK, format!("expected shader is missing: {full}"))
                    .with_path(full.clone())
                    .with_suggestion("create the WGSL file or update vv-render::pipeline::graph"),
            );
            report.missing.shaders.push(full);
        }
    }

    // 3. Expand + naga-parse every non-include shader (passes / features /
    //    debug). Includes are implicitly validated because expansion inlines
    //    them; if an include is broken, the expanded source naga-parse fails.
    for entry in &enumerated {
        let rel_str = rel_as_str(&entry.relative_path);
        if is_include(&rel_str) {
            continue;
        }
        validate_shader(&mut resolver, entry, report);
    }

    // 4. Includes that no pass ever pulled in are dead weight. After every
    //    pass has been expanded, the resolver's resolved-paths set is the
    //    closure of every file the engine would have touched at runtime.
    let reached: BTreeSet<String> = resolver
        .resolved_paths()
        .into_iter()
        .map(|p| rel_as_str(&p))
        .collect();
    for entry in &enumerated {
        let rel_str = rel_as_str(&entry.relative_path);
        if !is_include(&rel_str) {
            continue;
        }
        if !reached.contains(&rel_str) {
            report.warn(
                Diagnostic::new(CHECK, "shader include is not referenced by any pass")
                    .with_path(format!("render/shaders/{rel_str}"))
                    .with_suggestion("remove it or include it from a pass shader"),
            );
        }
    }

    // 5. Override report: surface every pack-shadowed file. With a single
    //    pack in the stack this is always empty, but we keep the hook so
    //    multi-pack runs naturally produce diagnostics.
    for ov in resolver.override_report().overrides {
        report.warn(
            Diagnostic::new(
                CHECK,
                format!(
                    "shader overridden by '{}' (shadowed: [{}])",
                    ov.winner,
                    ov.shadowed.join(", ")
                ),
            )
            .with_path(format!("render/shaders/{}", rel_as_str(&ov.relative_path))),
        );
    }

    // 6. Drop expected shaders from the unused list (they are intentionally
    //    referenced by the engine even if the pack doesn't see a code path).
    let expected_full: BTreeSet<String> = EXPECTED_SHADERS
        .iter()
        .map(|p| format!("render/shaders/{p}"))
        .collect();
    report
        .unused
        .shaders
        .retain(|path| !expected_full.contains(path));

    finalize_profiles(report);
}

fn finalize_profiles(report: &mut Report) {
    report.planet.render_profiles = vec![
        profile_summary("potato", "Potato", false, "cheap", 0),
        profile_summary("balanced", "Balanced", true, "clean", 3),
        profile_summary("high", "High", true, "cinematic", 6),
        profile_summary("ultra", "Ultra", true, "cinematic+", 8),
    ];
    report.planet.counts.render_profiles = report.planet.render_profiles.len();
}

fn validate_shader(resolver: &mut ShaderResolver, entry: &EnumeratedShader, report: &mut Report) {
    let rel_str = rel_as_str(&entry.relative_path);
    let full = format!("render/shaders/{rel_str}");

    let source = match resolver.expand(&entry.relative_path) {
        Ok(src) => src,
        Err(e) => {
            report.error(
                Diagnostic::new(CHECK, format!("shader expansion failed: {e}")).with_path(full),
            );
            return;
        }
    };

    if let Err(e) = shader::validate_wgsl(&source, &full) {
        report.error(Diagnostic::new(CHECK, e).with_path(full));
    }
}

fn check_duplicate_names(scan: &PackScan, report: &mut Report) {
    let mut by_name: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in &scan.render.wgsl_files {
        let Some(name) = Path::new(&file.rel_path)
            .file_name()
            .and_then(|name| name.to_str())
        else {
            continue;
        };
        by_name
            .entry(name.to_string())
            .or_default()
            .push(file.rel_path.clone());
    }
    for (name, paths) in by_name {
        if paths.len() > 1 {
            report.warn(
                Diagnostic::new(CHECK, format!("duplicate WGSL file name '{name}'"))
                    .with_path(paths.join(", "))
                    .with_suggestion("prefer unique pass/include names for clear diagnostics"),
            );
        }
    }
}

fn check_oversized_files(scan: &PackScan, report: &mut Report) {
    for file in &scan.render.wgsl_files {
        let Ok(metadata) = std::fs::metadata(&file.abs_path) else {
            continue;
        };
        if metadata.len() == 0 {
            report.error(
                Diagnostic::new(CHECK, "WGSL source is empty").with_path(file.rel_path.clone()),
            );
            continue;
        }
        let Ok(source) = std::fs::read_to_string(&file.abs_path) else {
            continue;
        };
        let lines = source.lines().count();
        if lines > MAX_SHADER_LINES {
            report.warn(
                Diagnostic::new(
                    CHECK,
                    format!("WGSL source has {lines} lines; split shared code into includes"),
                )
                .with_path(file.rel_path.clone()),
            );
        }
    }
}

fn profile_summary(
    id: &str,
    label: &str,
    fog: bool,
    water: &str,
    enabled_features: usize,
) -> RenderProfileSummary {
    RenderProfileSummary {
        id: format!("vv-render:{id}"),
        label: label.to_string(),
        quality_class: id.to_string(),
        fog,
        water: water.to_string(),
        enabled_features,
    }
}

fn rel_as_str(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn is_include(rel: &str) -> bool {
    rel.starts_with("include/")
}
