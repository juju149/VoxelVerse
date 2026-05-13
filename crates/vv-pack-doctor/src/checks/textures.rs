//! Texture-level checks:
//!   - flag PNG files that no parsed object references
//!   - flag block-texture groups missing one of the PBR-lite channels
//!   - confirm every PNG is non-empty and decodes (cheap header probe)
//!
//! Reference resolution from objects to disk paths is performed in
//! `blocks.rs` and `items.rs`; this module operates on what's left over.

use std::collections::BTreeSet;
use std::fs::File;

use vv_content_schema::RawObjectTexture;

use crate::allowed::AllowedUnused;
use crate::index::PackIndex;
use crate::report::{Diagnostic, Report};

const CHECK: &str = "textures";
const CHANNELS: &[&str] = &["albedo", "normal", "roughness"];

pub fn run(index: &PackIndex<'_>, allowed: &AllowedUnused, report: &mut Report) {
    let used = collect_used_paths(index);

    for tex in &index.scan.texture_files {
        // Validate PNG decodes — empty/corrupt files break the runtime.
        if let Err(e) = probe_png(&tex.abs_path) {
            report.error(
                Diagnostic::new(CHECK, format!("invalid PNG '{}': {}", tex.rel_path, e))
                    .with_path(tex.rel_path.clone()),
            );
        }

        if !used.contains(&tex.rel_path)
            && !allowed.textures.contains(&tex.rel_path)
            && !allowed.textures.contains(&format!(
                "{}:{}",
                index.scan.namespace,
                tex.rel_path
                    .trim_start_matches("media/textures/")
                    .trim_end_matches(".png")
            ))
        {
            report.unused.textures.push(tex.rel_path.clone());
        }
    }

    check_pbr_groups(index, report);
}

fn collect_used_paths(index: &PackIndex<'_>) -> BTreeSet<String> {
    let mut used = BTreeSet::new();
    for obj in &index.scan.objects {
        if let Some(block) = &obj.def.block {
            collect_block_refs(&block.texture, &mut used);
        }
        if let Some(item) = &obj.def.item {
            if let Some(icon) = &item.icon {
                used.insert(format!("media/textures/{}.png", icon));
            }
        }
    }
    used
}

fn collect_block_refs(texture: &RawObjectTexture, out: &mut BTreeSet<String>) {
    let refs = match texture {
        RawObjectTexture::None => Vec::new(),
        RawObjectTexture::All(r) => vec![r.clone()],
        RawObjectTexture::Cube { top, side, bottom } => {
            vec![top.clone(), side.clone(), bottom.clone()]
        }
        RawObjectTexture::Column { top, side } => vec![top.clone(), side.clone()],
    };
    for r in refs {
        let parts: Vec<&str> = r.split('/').collect();
        if parts.len() < 2 {
            continue;
        }
        let group = parts[parts.len() - 2];
        let face = parts[parts.len() - 1];
        for ch in CHANNELS {
            out.insert(format!(
                "media/textures/{}/{}_{}_{}.png",
                parts[..parts.len() - 1].join("/"),
                group,
                face,
                ch
            ));
        }
    }
}

fn check_pbr_groups(index: &PackIndex<'_>, report: &mut Report) {
    // Group disk PNGs by `media/textures/.../<group>_<face>` prefix and
    // ensure each group has all three channels.
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, BTreeSet<&'static str>> = BTreeMap::new();
    for tex in &index.scan.texture_files {
        let stem = tex
            .rel_path
            .strip_suffix(".png")
            .unwrap_or(&tex.rel_path)
            .to_string();
        for ch in CHANNELS {
            let suffix = format!("_{ch}");
            if let Some(base) = stem.strip_suffix(&suffix) {
                groups.entry(base.to_string()).or_default().insert(ch);
            }
        }
    }
    for (base, present) in groups {
        for ch in CHANNELS {
            if !present.contains(ch) {
                report.warn(
                    Diagnostic::new(
                        CHECK,
                        format!(
                            "texture group '{}.png' is missing the '{}' channel",
                            base, ch
                        ),
                    )
                    .with_path(format!("{base}_{ch}.png"))
                    .with_suggestion(
                        "PBR-lite expects albedo + normal + roughness per texture group"
                            .to_string(),
                    ),
                );
            }
        }
    }
}

fn probe_png(path: &std::path::Path) -> Result<(), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let decoder = png::Decoder::new(file);
    decoder.read_info().map(|_| ()).map_err(|e| e.to_string())
}
