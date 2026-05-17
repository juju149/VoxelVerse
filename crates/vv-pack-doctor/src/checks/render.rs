//! Render shader-library validation.
//!
//! Render pipelines are Rust-owned. The pack only supplies WGSL source under
//! `render/shaders`, so Pack Doctor validates paths, includes and conventions
//! instead of parsing render `.ron` manifests.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use crate::index::PackIndex;
use crate::report::{Diagnostic, RenderProfileSummary, Report};
use crate::scan::PackScan;

const CHECK: &str = "render";
const MAX_SHADER_LINES: usize = 420;

const EXPECTED_SHADERS: &[&str] = &[
    "render/shaders/passes/terrain/terrain.vert.wgsl",
    "render/shaders/passes/terrain/terrain.frag.wgsl",
    "render/shaders/passes/terrain/terrain_depth.vert.wgsl",
    "render/shaders/passes/sky/sky.vert.wgsl",
    "render/shaders/passes/sky/sky.frag.wgsl",
    "render/shaders/passes/clouds/clouds.vert.wgsl",
    "render/shaders/passes/clouds/clouds.frag.wgsl",
    "render/shaders/passes/atmosphere/volumetric_fog.vert.wgsl",
    "render/shaders/passes/atmosphere/volumetric_fog.frag.wgsl",
    "render/shaders/passes/water/water.vert.wgsl",
    "render/shaders/passes/water/water.frag.wgsl",
    "render/shaders/passes/foliage/foliage.vert.wgsl",
    "render/shaders/passes/foliage/foliage.frag.wgsl",
    "render/shaders/passes/post/fullscreen.vert.wgsl",
    "render/shaders/passes/post/final_composite.frag.wgsl",
    "render/shaders/passes/post/fxaa.frag.wgsl",
    "render/shaders/passes/post/bloom_downsample.frag.wgsl",
    "render/shaders/passes/post/bloom_upsample.frag.wgsl",
    "render/shaders/passes/ui/ui.vert.wgsl",
    "render/shaders/passes/ui/ui.frag.wgsl",
    "render/shaders/passes/debug/normals.frag.wgsl",
    "render/shaders/passes/debug/depth.frag.wgsl",
    "render/shaders/passes/debug/lighting.frag.wgsl",
];

const REQUIRED_INCLUDE_DIRS: &[&str] = &[
    "render/shaders/include/common",
    "render/shaders/include/interface",
    "render/shaders/include/voxel",
];

pub fn run(_index: &PackIndex<'_>, _report: &mut Report) {}

pub fn validate(scan: &PackScan, report: &mut Report) {
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

    let shader_paths: BTreeSet<String> = scan
        .render
        .wgsl_files
        .iter()
        .map(|file| file.rel_path.clone())
        .collect();

    for expected in EXPECTED_SHADERS {
        if !shader_paths.contains(*expected) {
            report.error(
                Diagnostic::new(CHECK, format!("expected shader is missing: {expected}"))
                    .with_path((*expected).to_string())
                    .with_suggestion("create the WGSL file or update vv-render::render_graph"),
            );
            report.missing.shaders.push((*expected).to_string());
        }
    }

    for dir in REQUIRED_INCLUDE_DIRS {
        if !scan.pack_root.join(dir).is_dir() {
            report.error(
                Diagnostic::new(CHECK, format!("shader include directory is missing: {dir}"))
                    .with_path((*dir).to_string()),
            );
        }
    }

    check_duplicate_names(scan, report);

    let mut include_references = BTreeSet::new();
    for file in &scan.render.wgsl_files {
        if !file.rel_path.starts_with("render/shaders/") {
            report.error(
                Diagnostic::new(CHECK, "WGSL render shader is outside render/shaders")
                    .with_path(file.rel_path.clone()),
            );
        }
        check_wgsl_source(
            scan,
            report,
            &file.rel_path,
            &file.abs_path,
            &mut include_references,
        );
    }

    for file in &scan.render.wgsl_files {
        let is_include = file.rel_path.starts_with("render/shaders/include/");
        if is_include && !include_references.contains(&file.rel_path) {
            report.warn(
                Diagnostic::new(CHECK, "shader include is not referenced by any pass")
                    .with_path(file.rel_path.clone())
                    .with_suggestion("remove it or include it from a pass shader"),
            );
        }
    }

    for expected in EXPECTED_SHADERS {
        if !shader_paths.contains(*expected) {
            continue;
        }
        report.unused.shaders.retain(|path| path != expected);
    }

    report.planet.render_profiles = vec![
        profile_summary("potato", "Potato", false, "cheap", 0),
        profile_summary("balanced", "Balanced", true, "clean", 3),
        profile_summary("high", "High", true, "cinematic", 6),
        profile_summary("ultra", "Ultra", true, "cinematic+", 8),
    ];
    report.planet.counts.render_profiles = report.planet.render_profiles.len();
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

fn check_wgsl_source(
    scan: &PackScan,
    report: &mut Report,
    rel_path: &str,
    abs_path: &Path,
    include_references: &mut BTreeSet<String>,
) {
    let Ok(source) = std::fs::read_to_string(abs_path) else {
        report.error(
            Diagnostic::new(CHECK, "cannot read WGSL source").with_path(rel_path.to_string()),
        );
        return;
    };
    if source.trim().is_empty() {
        report
            .error(Diagnostic::new(CHECK, "WGSL source is empty").with_path(rel_path.to_string()));
    }
    let line_count = source.lines().count();
    if line_count > MAX_SHADER_LINES {
        report.warn(
            Diagnostic::new(
                CHECK,
                format!("WGSL source has {line_count} lines; split shared code into includes"),
            )
            .with_path(rel_path.to_string()),
        );
    }
    for (open, close, name) in [
        ('{', '}', "braces"),
        ('(', ')', "parentheses"),
        ('[', ']', "brackets"),
    ] {
        if !balanced_delimiters(&source, open, close) {
            report.error(
                Diagnostic::new(CHECK, format!("WGSL source has unbalanced {name}"))
                    .with_path(rel_path.to_string()),
            );
        }
    }

    let base_dir = Path::new(rel_path)
        .parent()
        .unwrap_or_else(|| Path::new("render/shaders"));
    for (line_index, line) in source.lines().enumerate() {
        let Some(include) = parse_include(line) else {
            continue;
        };
        let include_rel = match resolve_include(base_dir, include) {
            Ok(path) => path,
            Err(error) => {
                report.error(
                    Diagnostic::new(CHECK, error)
                        .with_path(rel_path.to_string())
                        .with_field(format!("line {}", line_index + 1)),
                );
                continue;
            }
        };
        let include_rel = include_rel.to_string_lossy().replace('\\', "/");
        if !include_rel.starts_with("render/shaders/include/")
            && !include_rel.starts_with("render/shaders/passes/")
        {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!("include points outside allowed include tree: {include}"),
                )
                .with_path(rel_path.to_string())
                .with_field(format!("line {}", line_index + 1)),
            );
        }
        if !scan.pack_root.join(&include_rel).is_file() {
            report.error(
                Diagnostic::new(CHECK, format!("include target is missing: {include_rel}"))
                    .with_path(rel_path.to_string())
                    .with_field(format!("line {}", line_index + 1)),
            );
        }
        include_references.insert(include_rel);
    }
}

fn parse_include(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix("#include")?.trim();
    rest.strip_prefix('"')?
        .split_once('"')
        .map(|(path, _)| path)
}

fn resolve_include(base_dir: &Path, include: &str) -> Result<PathBuf, String> {
    let include_path = Path::new(include);
    if include_path.is_absolute() {
        return Err(format!("absolute include path is forbidden: {include}"));
    }
    let mut joined = if include.starts_with("include/") || include.starts_with("passes/") {
        PathBuf::from("render/shaders").join(include)
    } else if include.starts_with("render/") {
        PathBuf::from(include)
    } else {
        base_dir.join(include)
    };
    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(format!("include escapes shader root: {include}"));
                }
            }
            other => return Err(format!("unsupported include path component {other:?}")),
        }
    }
    joined = normalized;
    if !joined.starts_with("render/shaders/") {
        return Err(format!("include escapes render/shaders: {include}"));
    }
    Ok(joined)
}

fn balanced_delimiters(source: &str, open: char, close: char) -> bool {
    let mut depth = 0i32;
    for c in source.chars() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth < 0 {
                return false;
            }
        }
    }
    depth == 0
}
