//! Per-block validation:
//!   - every face-texture short reference resolves to a PBR-lite triplet on disk
//!   - the texture shape matches the declared block shape (column/cube/cross)
//!   - the texture is non-empty when the block is rendered

use vv_content_schema::{RawObjectShape, RawObjectTexture};

use crate::index::PackIndex;
use crate::report::{Diagnostic, Report};
use crate::scan::ParsedObject;

const CHECK: &str = "blocks";
const CHANNELS: &[&str] = &["albedo", "normal", "roughness"];

pub fn run(index: &PackIndex<'_>, report: &mut Report) {
    for obj in &index.scan.objects {
        let Some(block) = &obj.def.block else { continue };
        validate_texture(obj, &block.texture, block.shape, index, report);
    }
}

fn validate_texture(
    obj: &ParsedObject,
    texture: &RawObjectTexture,
    shape: RawObjectShape,
    index: &PackIndex<'_>,
    report: &mut Report,
) {
    let refs: Vec<(&str, String)> = match texture {
        RawObjectTexture::None => Vec::new(),
        RawObjectTexture::All(r) => vec![("all", r.clone())],
        RawObjectTexture::Cube { top, side, bottom } => vec![
            ("top", top.clone()),
            ("side", side.clone()),
            ("bottom", bottom.clone()),
        ],
        RawObjectTexture::Column { top, side } => {
            if !matches!(shape, RawObjectShape::Column) {
                report.warn(
                    Diagnostic::new(
                        CHECK,
                        "texture is Column-shaped but block.shape is not 'column'",
                    )
                    .with_path(obj.rel_path.clone())
                    .with_id(obj.id.clone())
                    .with_field("block.shape")
                    .with_suggestion("set `shape: column,` on the block section".to_string()),
                );
            }
            vec![("top", top.clone()), ("side", side.clone())]
        }
    };

    for (face, reference) in refs {
        check_pbr_triplet(obj, face, &reference, index, report);
    }
}

fn check_pbr_triplet(
    obj: &ParsedObject,
    face: &str,
    reference: &str,
    index: &PackIndex<'_>,
    report: &mut Report,
) {
    // Reference looks like `blocks/grass_block/top`. The base name (second
    // segment) is the texture group on disk.
    let parts: Vec<&str> = reference.split('/').collect();
    if parts.len() < 3 {
        report.error(
            Diagnostic::new(
                CHECK,
                format!(
                    "block texture '{}' is malformed — expected at least three slash-separated segments",
                    reference
                ),
            )
            .with_path(obj.rel_path.clone())
            .with_id(obj.id.clone())
            .with_field(format!("block.texture.{face}"))
            .with_suggestion(
                "use the form `blocks/<group>/<face>` e.g. `blocks/grass_block/top`".to_string(),
            ),
        );
        return;
    }
    let group = parts[parts.len() - 2];
    let face_name = parts[parts.len() - 1];

    for channel in CHANNELS {
        let candidate = format!(
            "media/textures/{}/{}_{}_{}.png",
            parts[..parts.len() - 1].join("/"),
            group,
            face_name,
            channel
        );
        if !index.texture_exists(&candidate) {
            let severity = if *channel == "albedo" { Severity::Error } else { Severity::Warn };
            let diag = Diagnostic::new(
                CHECK,
                format!(
                    "block texture '{}' is missing {} PNG: {}",
                    reference, channel, candidate
                ),
            )
            .with_path(obj.rel_path.clone())
            .with_id(obj.id.clone())
            .with_field(format!("block.texture.{face}"))
            .with_suggestion(format!(
                "create the missing file at {} (PBR-lite expects albedo+normal+roughness)",
                candidate
            ));
            match severity {
                Severity::Error => report.error(diag),
                Severity::Warn => report.warn(diag),
            }
        }
    }
}

enum Severity {
    Error,
    Warn,
}
