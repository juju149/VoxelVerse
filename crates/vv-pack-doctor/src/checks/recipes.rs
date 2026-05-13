//! Recipe-level structural validation.
//!
//! Ingredient resolution lives in `references.rs`. Here we check the shape of
//! the recipe itself: grid dimensions, symbol coverage, station kind, and
//! that the recipe kind enum is well-formed.

use std::collections::HashSet;

use vv_content_schema::{RawObjectRecipeKind, RawObjectRecipeSection, RawShapedRecipe};

use crate::index::{parse_tag_ref, PackIndex};
use crate::report::{Diagnostic, Report};
use crate::scan::ParsedObject;

const CHECK: &str = "recipes";
const MAX_GRID: usize = 3;

pub fn run(index: &PackIndex<'_>, report: &mut Report) {
    for obj in &index.scan.objects {
        for (i, recipe) in obj.def.recipes.iter().enumerate() {
            let prefix = format!("recipes[{i}]");
            check_station(obj, &prefix, recipe, index, report);
            if let RawObjectRecipeKind::Shaped(shaped) = &recipe.kind {
                check_shaped(obj, &prefix, shaped, report);
            }
            if recipe.output.count == 0 {
                report.error(
                    Diagnostic::new(CHECK, "recipe output.count must be >= 1")
                        .with_path(obj.rel_path.clone())
                        .with_id(obj.id.clone())
                        .with_field(format!("{prefix}.output.count")),
                );
            }
        }
    }
}

fn check_shaped(obj: &ParsedObject, prefix: &str, shaped: &RawShapedRecipe, report: &mut Report) {
    if shaped.pattern.is_empty() || shaped.pattern.len() > MAX_GRID {
        report.error(
            Diagnostic::new(
                CHECK,
                format!(
                    "shaped recipe must have 1..={MAX_GRID} rows; got {}",
                    shaped.pattern.len()
                ),
            )
            .with_path(obj.rel_path.clone())
            .with_id(obj.id.clone())
            .with_field(format!("{prefix}.kind.shaped.pattern")),
        );
        return;
    }
    let width = shaped.pattern[0].chars().count();
    if width == 0 || width > MAX_GRID {
        report.error(
            Diagnostic::new(
                CHECK,
                format!("shaped recipe row width must be 1..={MAX_GRID}; got {width}"),
            )
            .with_path(obj.rel_path.clone())
            .with_id(obj.id.clone())
            .with_field(format!("{prefix}.kind.shaped.pattern[0]")),
        );
        return;
    }
    let mut symbols: HashSet<char> = HashSet::new();
    for (i, row) in shaped.pattern.iter().enumerate() {
        if row.chars().count() != width {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!(
                        "shaped recipe row {} has width {} but row 0 has width {width}",
                        i,
                        row.chars().count()
                    ),
                )
                .with_path(obj.rel_path.clone())
                .with_id(obj.id.clone())
                .with_field(format!("{prefix}.kind.shaped.pattern[{i}]"))
                .with_suggestion("pad shorter rows with spaces so all rows match".to_string()),
            );
        }
        for c in row.chars() {
            if c != ' ' {
                symbols.insert(c);
            }
        }
    }
    for c in &symbols {
        let key = c.to_string();
        if !shaped.legend.contains_key(&key) {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!("shaped pattern uses '{c}' but legend has no mapping for it"),
                )
                .with_path(obj.rel_path.clone())
                .with_id(obj.id.clone())
                .with_field(format!("{prefix}.kind.shaped.legend"))
                .with_suggestion(format!("add `\"{c}\": <item>,` to the legend")),
            );
        }
    }
    for sym in shaped.legend.keys() {
        if let Some(c) = sym.chars().next() {
            if !symbols.contains(&c) {
                report.warn(
                    Diagnostic::new(
                        CHECK,
                        format!("legend defines '{sym}' but pattern never uses it"),
                    )
                    .with_path(obj.rel_path.clone())
                    .with_id(obj.id.clone())
                    .with_field(format!("{prefix}.kind.shaped.legend")),
                );
            }
        }
    }
}

fn check_station(
    obj: &ParsedObject,
    prefix: &str,
    recipe: &RawObjectRecipeSection,
    index: &PackIndex<'_>,
    report: &mut Report,
) {
    let Some(s) = recipe.station.as_deref() else {
        return; // None = personal 2x2 grid
    };
    if s.is_empty() {
        return;
    }
    match parse_tag_ref(s) {
        Some(("station", name)) => {
            if !index.stations_declared.contains(name) {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!(
                            "recipe requires station '{}' but no object carries that station tag",
                            s
                        ),
                    )
                    .with_path(obj.rel_path.clone())
                    .with_id(obj.id.clone())
                    .with_field(format!("{prefix}.station"))
                    .with_suggestion(format!(
                        "add `station.{name}` to the station's `station_tags`"
                    )),
                );
            }
        }
        Some((other, _)) => {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!("recipe.station uses unknown tag namespace '#{other}.'"),
                )
                .with_path(obj.rel_path.clone())
                .with_id(obj.id.clone())
                .with_field(format!("{prefix}.station"))
                .with_suggestion("station references must use `#station.<name>`".to_string()),
            );
        }
        None => {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!(
                        "recipe.station '{}' is not a tag — expected '#station.<name>'",
                        s
                    ),
                )
                .with_path(obj.rel_path.clone())
                .with_id(obj.id.clone())
                .with_field(format!("{prefix}.station")),
            );
        }
    }
}
