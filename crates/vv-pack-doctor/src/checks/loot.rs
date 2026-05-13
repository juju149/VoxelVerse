//! Loot-section sanity check.
//!
//! Item resolution lives in `references.rs`; here we look at probability and
//! count ranges and warn when an entity is killed but drops nothing useful.

use crate::index::PackIndex;
use crate::report::{Diagnostic, Report};

const CHECK: &str = "loot";

pub fn run(index: &PackIndex<'_>, report: &mut Report) {
    for obj in &index.scan.objects {
        let Some(loot) = &obj.def.loot else { continue };
        if loot.when_killed.is_empty() {
            // Entities should declare `loot: ()` only if they intentionally
            // drop nothing — flag as a warning to surface accidental empty
            // tables.
            if obj.def.entity.is_some() {
                report.warn(
                    Diagnostic::new(CHECK, "entity declares an empty loot table on death")
                        .with_path(obj.rel_path.clone())
                        .with_id(obj.id.clone())
                        .with_field("loot.when_killed")
                        .with_suggestion(
                            "remove the `loot` section if drops are intentionally empty"
                                .to_string(),
                        ),
                );
            }
        }
        for (i, drop) in loot.when_killed.iter().enumerate() {
            if !(0.0..=1.0).contains(&drop.chance) {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!("loot drop chance {} is outside [0.0, 1.0]", drop.chance),
                    )
                    .with_path(obj.rel_path.clone())
                    .with_id(obj.id.clone())
                    .with_field(format!("loot.when_killed[{i}].chance")),
                );
            }
        }
    }
    // Touch the index so the parameter is used (so far purely structural).
    let _ = index;
}
