//! Source-level pack constitution checks that are easier to catch before
//! schema parsing.

use crate::report::{Diagnostic, Report};
use crate::scan::PackScan;

const CHECK: &str = "source_contract";

pub fn run(scan: &PackScan, report: &mut Report) {
    for path in &scan.all_ron_files {
        let rel = scan.relative(path);
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        for (line_idx, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("recipe:") {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        "legacy `recipe:` field is forbidden; use `recipes: [(kind: ...)]`",
                    )
                    .with_path(rel.clone())
                    .with_field(format!("line {}", line_idx + 1))
                    .with_suggestion(
                        "wrap the recipe in `recipes: [...]` and use the canonical `kind` enum"
                            .to_string(),
                    ),
                );
            }
            if trimmed.contains("#station.") {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        "legacy station tag syntax is forbidden in V1",
                    )
                    .with_path(rel.clone())
                    .with_field(format!("line {}", line_idx + 1))
                    .with_suggestion(
                        "use `#core:tag/station/<name>` instead of `#station.<name>`"
                            .to_string(),
                    ),
                );
            }
            if trimmed.starts_with("icon:") {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        "legacy `item.icon` field is forbidden in V1",
                    )
                    .with_path(rel.clone())
                    .with_field(format!("line {}", line_idx + 1))
                    .with_suggestion(
                        "use `inventory_icon: texture(\"core:texture/items/...\")`".to_string(),
                    ),
                );
            }
        }
    }
}
