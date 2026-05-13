//! Per-item validation:
//!   - item.icon paths point at a real PNG under `media/textures/`
//!   - item.model paths point at a real `.vox` under `media/voxel/`
//!   - stack sizes are positive

use crate::index::PackIndex;
use crate::report::{Diagnostic, Report};

const CHECK: &str = "items";

pub fn run(index: &PackIndex<'_>, report: &mut Report) {
    for obj in &index.scan.objects {
        let Some(item) = &obj.def.item else { continue };

        if item.stack == 0 {
            report.error(
                Diagnostic::new(CHECK, "item.stack must be >= 1")
                    .with_path(obj.rel_path.clone())
                    .with_id(obj.id.clone())
                    .with_field("item.stack"),
            );
        }

        if let Some(icon) = &item.icon {
            let candidate = format!("media/textures/{}.png", icon);
            if !index.texture_exists(&candidate) {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!("item icon '{}' is missing on disk", icon),
                    )
                    .with_path(obj.rel_path.clone())
                    .with_id(obj.id.clone())
                    .with_field("item.icon")
                    .with_suggestion(format!("create {} or fix the path", candidate)),
                );
            }
        }

        if let Some(model) = &item.model {
            let candidate = format!("media/{}.vox", model);
            if !index.voxel_exists(&candidate) {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!("item model '{}' is missing on disk", model),
                    )
                    .with_path(obj.rel_path.clone())
                    .with_id(obj.id.clone())
                    .with_field("item.model")
                    .with_suggestion(format!(
                        "place a .vox file at {} (note: the path starts with `voxel/`)",
                        candidate
                    )),
                );
            }
        }
    }
}
