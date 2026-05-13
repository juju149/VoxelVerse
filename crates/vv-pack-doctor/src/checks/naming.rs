//! Naming checks for content files.
//!
//! Filenames must be lowercase, hyphen-free, non-numeric, and free of banned
//! placeholder stems (`test`, `tmp`, etc.). Applies to every .ron / .png /
//! .vox we discovered during the scan.

use crate::report::{Diagnostic, Report};
use crate::scan::PackScan;

const CHECK: &str = "naming";

const BANNED_STEMS: &[&str] = &[
    "test",
    "new",
    "final",
    "temp",
    "tmp",
    "stuff",
    "placeholder",
];

pub fn run(scan: &PackScan, report: &mut Report) {
    for path in &scan.all_ron_files {
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let rel = scan.relative(path);
        check_name(name, &rel, report);
    }
    for tex in &scan.texture_files {
        let Some(name) = tex.abs_path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        check_name(name, &tex.rel_path, report);
    }
    for v in &scan.voxel_files {
        let Some(name) = v.abs_path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        check_name(name, &v.rel_path, report);
    }
}

fn check_name(name: &str, rel: &str, report: &mut Report) {
    if !is_safe_name(name) {
        report.error(
            Diagnostic::new(CHECK, format!("invalid filename: {}", name))
                .with_path(rel.to_string())
                .with_suggestion(
                    "use lowercase letters, digits and underscores only; no spaces, hyphens, or numeric-only stems"
                        .to_string(),
                ),
        );
    }
    let stem = stem_of(name);
    if BANNED_STEMS.iter().any(|b| stem == *b) {
        report.error(
            Diagnostic::new(CHECK, format!("banned stem '{}' in filename", stem))
                .with_path(rel.to_string())
                .with_suggestion(
                    "rename the file to something descriptive — placeholder names ship to runtime"
                        .to_string(),
                ),
        );
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
