//! Filesystem-level checks: required pack layout, legacy paths, empty dirs.

use std::path::Path;

use crate::report::Report;
use crate::scan::PackScan;

const CHECK: &str = "filesystem";

const REQUIRED_DIRS: &[&str] = &[
    "defs",
    "media",
    "media/textures",
    "media/voxel",
    "generated",
    "generated/registries",
];

const REQUIRED_FILES: &[&str] = &["pack.ron", "README.md"];

const FORBIDDEN_LEGACY: &[&str] = &[
    "legacy_imports",
    "blocks",
    "worldgen",
    "textures",
    "items",
    "voxel",
    "pack.toml",
];

pub fn run(scan: &PackScan, report: &mut Report) {
    for f in REQUIRED_FILES {
        let path = scan.pack_root.join(f);
        if !path.exists() {
            report.error_path(CHECK, format!("missing required file: {}", f), f.to_string());
        }
    }
    for d in REQUIRED_DIRS {
        let path = scan.pack_root.join(d);
        if !path.is_dir() {
            report.error_path(CHECK, format!("missing required directory: {}", d), d.to_string());
        }
    }
    for legacy in FORBIDDEN_LEGACY {
        let path = scan.pack_root.join(legacy);
        if path.exists() {
            report.error_path(
                CHECK,
                format!("legacy path still exists: {}", legacy),
                legacy.to_string(),
            );
        }
    }
    check_empty_dirs(&scan.pack_root, &scan.pack_root, report);
}

fn check_empty_dirs(pack_root: &Path, dir: &Path, report: &mut Report) {
    let rel = dir
        .strip_prefix(pack_root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| dir.to_string_lossy().to_string());

    // Skip subtrees Pack Doctor owns or that intentionally hold human notes.
    if rel.starts_with("source") || rel == "generated/reports" {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut had_child = false;
    let mut children = Vec::new();
    for entry in entries.flatten() {
        had_child = true;
        let path = entry.path();
        if path.is_dir() {
            children.push(path);
        }
    }
    if !had_child && dir != pack_root {
        report.error_path(CHECK, format!("empty directory: {}", rel), rel);
    }
    for child in children {
        check_empty_dirs(pack_root, &child, report);
    }
}
