//! Texture-level checks: existence on disk, dimensions, usage tracking,
//! albedo/normal/roughness coherence.

use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

use crate::allowed::AllowedUnused;
use crate::report::Report;
use crate::scan::PackScan;

const CHECK: &str = "textures";
const BLOCK_TEXTURE_DIM: u32 = 256;

pub fn run(scan: &PackScan, allowed: &AllowedUnused, report: &mut Report) {
    let mut used_textures: HashSet<String> = HashSet::new();

    // Track texture references and warn on block-surface dimensions.
    for (mat_id, mat) in &scan.materials {
        let is_block_surface = matches!(
            mat.category,
            vv_content_schema::RawMaterialCategory::BlockSurface
        );

        record_texture_use(scan, report, mat_id, &mat.albedo.0, "albedo", &mut used_textures);
        if let Some(n) = &mat.normal {
            record_texture_use(scan, report, mat_id, &n.0, "normal", &mut used_textures);
        }
        if let Some(r) = &mat.roughness {
            record_texture_use(scan, report, mat_id, &r.0, "roughness", &mut used_textures);
        }

        if is_block_surface {
            // Dimensions.
            if let Some(file) = scan
                .texture_files
                .iter()
                .find(|t| t.texture_ref == mat.albedo.0)
            {
                check_png_dimensions(report, &file.abs_path, &file.rel_path, BLOCK_TEXTURE_DIM);
            }
            if let Some(n) = &mat.normal {
                if let Some(file) = scan
                    .texture_files
                    .iter()
                    .find(|t| t.texture_ref == n.0)
                {
                    check_png_dimensions(report, &file.abs_path, &file.rel_path, BLOCK_TEXTURE_DIM);
                }
            }
            if let Some(r) = &mat.roughness {
                if let Some(file) = scan
                    .texture_files
                    .iter()
                    .find(|t| t.texture_ref == r.0)
                {
                    check_png_dimensions(report, &file.abs_path, &file.rel_path, BLOCK_TEXTURE_DIM);
                }
            }
        }
    }

    // Unused textures.
    for tex in &scan.texture_files {
        if used_textures.contains(&tex.texture_ref) {
            continue;
        }
        if allowed.textures.contains(&tex.texture_ref)
            || allowed.textures.contains(&tex.rel_path)
        {
            continue;
        }
        report.unused.textures.push(tex.texture_ref.clone());
        report.warn_id(
            CHECK,
            format!("texture '{}' is not referenced by any material", tex.rel_path),
            tex.texture_ref.clone(),
        );
    }
}

fn record_texture_use(
    scan: &PackScan,
    report: &mut Report,
    material_id: &str,
    tref: &str,
    role: &str,
    used: &mut HashSet<String>,
) {
    used.insert(tref.to_string());
    if !scan.texture_id_exists(tref) {
        // References check already complains; we mirror it as "missing" entry.
        if !report.missing.textures.iter().any(|t| t == tref) {
            report.missing.textures.push(tref.to_string());
        }
        let _ = (material_id, role);
    }
}

fn check_png_dimensions(report: &mut Report, path: &Path, rel: &str, expected: u32) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            report.error_path(
                CHECK,
                format!("cannot open texture '{}': {}", rel, e),
                rel.to_string(),
            );
            return;
        }
    };
    let decoder = png::Decoder::new(file);
    match decoder.read_info() {
        Ok(reader) => {
            let info = reader.info();
            if info.width != expected || info.height != expected {
                report.warn_path(
                    CHECK,
                    format!(
                        "block-surface texture '{}' is {}x{}, expected {}x{}",
                        rel, info.width, info.height, expected, expected
                    ),
                    rel.to_string(),
                );
            }
        }
        Err(e) => {
            report.error_path(
                CHECK,
                format!("invalid PNG '{}': {}", rel, e),
                rel.to_string(),
            );
        }
    }
}
