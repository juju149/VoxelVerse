//! Filesystem-level checks: required pack layout, legacy paths, empty dirs.

use std::path::Path;

use crate::report::{Diagnostic, Report};
use crate::scan::PackScan;

const CHECK: &str = "filesystem";

const REQUIRED_DIRS: &[&str] = &["defs", "media", "media/textures", "media/voxel", "render"];

const REQUIRED_FILES: &[&str] = &["pack.ron", "README.md"];

const FORBIDDEN_LEGACY: &[&str] = &[
    "legacy_imports",
    "blocks",
    "worldgen",
    "textures",
    "items",
    "voxel",
    "pack.toml",
    "defs/world/terrain",
    "media/_review",
];

pub fn run(scan: &PackScan, report: &mut Report) {
    for f in REQUIRED_FILES {
        let path = scan.pack_root.join(f);
        if !path.exists() {
            report.error(
                Diagnostic::new(CHECK, format!("missing required file: {}", f))
                    .with_path((*f).to_string())
                    .with_suggestion(format!("create {} at the pack root", f)),
            );
        }
    }
    for d in REQUIRED_DIRS {
        let path = scan.pack_root.join(d);
        if !path.is_dir() {
            report.error(
                Diagnostic::new(CHECK, format!("missing required directory: {}", d))
                    .with_path((*d).to_string())
                    .with_suggestion(format!("create {}/ at the pack root", d)),
            );
        }
    }
    for legacy in FORBIDDEN_LEGACY {
        let path = scan.pack_root.join(legacy);
        if path.exists() {
            report.error(
                Diagnostic::new(CHECK, format!("legacy path still exists: {}", legacy))
                    .with_path((*legacy).to_string())
                    .with_suggestion(format!(
                        "remove {} — the new layout is defs/ + media/ + render/",
                        legacy
                    )),
            );
        }
    }
    check_private_dirs(&scan.pack_root, &scan.pack_root, report);
    check_empty_dirs(&scan.pack_root, &scan.pack_root, report);
}

fn check_private_dirs(pack_root: &Path, dir: &Path, report: &mut Report) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let rel = path
            .strip_prefix(pack_root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.to_string_lossy().to_string());
        if name.starts_with('_') {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!(
                        "private review/import directory is inside the stable pack: {}",
                        rel
                    ),
                )
                .with_path(rel.clone())
                .with_suggestion(
                    "move review/import assets outside assets/packs/<namespace>".to_string(),
                ),
            );
            continue;
        }
        check_private_dirs(pack_root, &path, report);
    }
}

fn check_empty_dirs(pack_root: &Path, dir: &Path, report: &mut Report) {
    let rel = dir
        .strip_prefix(pack_root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| dir.to_string_lossy().to_string());

    if rel.starts_with("source") || rel == "generated/reports" || rel.starts_with("target") {
        return;
    }
    if dir
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| name.starts_with('_'))
    {
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
        report.warn(
            Diagnostic::new(CHECK, format!("empty directory: {}", rel))
                .with_path(rel.clone())
                .with_suggestion("remove the empty directory or fill it with content".to_string()),
        );
    }
    for child in children {
        check_empty_dirs(pack_root, &child, report);
    }
}
