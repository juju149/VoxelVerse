//! Naming checks for content files.
//!
//! Mirrors the rules in `docs/content_rules.md` section 2.

use crate::report::Report;
use crate::scan::PackScan;

const CHECK: &str = "naming";

const BANNED_STEMS: &[&str] = &[
    "test", "new", "final", "temp", "tmp", "stuff", "placeholder",
];

pub fn run(scan: &PackScan, report: &mut Report) {
    for path in &scan.all_ron_files {
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let rel = scan.relative(path);
        if !is_safe_name(name) {
            report.error_path(CHECK, format!("invalid filename: {}", name), rel.clone());
        }
        let stem = stem_of(name);
        if BANNED_STEMS.iter().any(|b| stem == *b) {
            report.error_path(
                CHECK,
                format!("banned stem '{}' in filename", stem),
                rel.clone(),
            );
        }
    }

    for tex in &scan.texture_files {
        let Some(name) = tex
            .abs_path
            .file_name()
            .and_then(|s| s.to_str())
        else {
            continue;
        };
        if !is_safe_name(name) {
            report.error_path(
                CHECK,
                format!("invalid texture filename: {}", name),
                tex.rel_path.clone(),
            );
        }
        let stem = stem_of(name);
        if BANNED_STEMS.iter().any(|b| stem == *b) {
            report.error_path(
                CHECK,
                format!("banned stem '{}' in texture filename", stem),
                tex.rel_path.clone(),
            );
        }
    }

    for path in &scan.voxel_files {
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !is_safe_name(name) {
            report.error_path(
                CHECK,
                format!("invalid voxel filename: {}", name),
                scan.relative(path),
            );
        }
    }
}

fn is_safe_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    if name.chars().any(|c| c.is_ascii_uppercase()) {
        return false;
    }
    if name.chars().any(char::is_whitespace) {
        return false;
    }
    if name.contains('-') {
        return false;
    }
    let stem = stem_of(name);
    if !stem.is_empty() && stem.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    true
}

fn stem_of(name: &str) -> &str {
    name.split('.').next().unwrap_or(name)
}
