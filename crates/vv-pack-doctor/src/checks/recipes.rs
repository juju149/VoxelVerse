//! Recipe-level structural validation.
//!
//! Ingredient resolution lives in `references.rs`. Here we check the shape of
//! the recipe itself: grid dimensions, symbol coverage, station kind, and
//! that exactly one of (shaped/shapeless/inputs) is provided.

use std::collections::HashSet;

use vv_content_schema::RawObjectRecipeSection;

use crate::index::{parse_tag_ref, PackIndex};
use crate::report::{Diagnostic, Report};
use crate::scan::ParsedObject;

const CHECK: &str = "recipes";
const MAX_GRID: usize = 3;

pub fn run(index: &PackIndex<'_>, report: &mut Report) {
    for obj in &index.scan.objects {
        let Some(recipe) = &obj.def.recipe else { continue };
        check_form(obj, recipe, report);
        check_station(obj, recipe, index, report);

        if let Some(shaped) = &recipe.shaped {
            check_shaped(obj, recipe, shaped, report);
        }
        if recipe.output.count == 0 {
            report.error(
                Diagnostic::new(CHECK, "recipe output.count must be >= 1")
                    .with_path(obj.rel_path.clone())
                    .with_id(obj.id.clone())
                    .with_field("recipe.output.count"),
            );
        }
    }
}

fn check_form(obj: &ParsedObject, recipe: &RawObjectRecipeSection, report: &mut Report) {
    let forms: Vec<&str> = [
        ("shaped", recipe.shaped.is_some()),
        ("shapeless", recipe.shapeless.is_some()),
        ("inputs", recipe.inputs.is_some()),
    ]
    .into_iter()
    .filter_map(|(name, present)| if present { Some(name) } else { None })
    .collect();

    if forms.is_empty() {
        report.error(
            Diagnostic::new(
                CHECK,
                "recipe has no ingredient list — provide one of shaped / shapeless / inputs",
            )
            .with_path(obj.rel_path.clone())
            .with_id(obj.id.clone())
            .with_field("recipe"),
        );
    } else if forms.len() > 1 {
        report.error(
            Diagnostic::new(
                CHECK,
                format!(
                    "recipe declares multiple ingredient forms ({}). Pick exactly one.",
                    forms.join(", ")
                ),
            )
            .with_path(obj.rel_path.clone())
            .with_id(obj.id.clone())
            .with_field("recipe"),
        );
    }
}

fn check_shaped(
    obj: &ParsedObject,
    recipe: &RawObjectRecipeSection,
    shaped: &[String],
    report: &mut Report,
) {
    if shaped.is_empty() || shaped.len() > MAX_GRID {
        report.error(
            Diagnostic::new(
                CHECK,
                format!(
                    "shaped recipe must have 1..={MAX_GRID} rows; got {}",
                    shaped.len()
                ),
            )
            .with_path(obj.rel_path.clone())
            .with_id(obj.id.clone())
            .with_field("recipe.shaped"),
        );
        return;
    }
    let width = shaped[0].chars().count();
    if width == 0 || width > MAX_GRID {
        report.error(
            Diagnostic::new(
                CHECK,
                format!(
                    "shaped recipe row width must be 1..={MAX_GRID}; got {width}"
                ),
            )
            .with_path(obj.rel_path.clone())
            .with_id(obj.id.clone())
            .with_field("recipe.shaped[0]"),
        );
        return;
    }
    let mut symbols: HashSet<char> = HashSet::new();
    for (i, row) in shaped.iter().enumerate() {
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
                .with_field(format!("recipe.shaped[{i}]"))
                .with_suggestion("pad shorter rows with spaces so all rows match".to_string()),
            );
        }
        for c in row.chars() {
            if c != ' ' {
                symbols.insert(c);
            }
        }
    }
    let legend = recipe.legend.as_ref();
    for c in &symbols {
        let key = c.to_string();
        let present = legend.map(|m| m.contains_key(&key)).unwrap_or(false);
        if !present {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!("shaped pattern uses '{c}' but legend has no mapping for it"),
                )
                .with_path(obj.rel_path.clone())
                .with_id(obj.id.clone())
                .with_field("recipe.legend")
                .with_suggestion(format!(
                    "add `{c}: <item>,` to the legend entry"
                )),
            );
        }
    }
    if let Some(legend) = legend {
        for sym in legend.keys() {
            if let Some(c) = sym.chars().next() {
                if !symbols.contains(&c) {
                    report.warn(
                        Diagnostic::new(
                            CHECK,
                            format!("legend defines '{sym}' but pattern never uses it"),
                        )
                        .with_path(obj.rel_path.clone())
                        .with_id(obj.id.clone())
                        .with_field("recipe.legend"),
                    );
                }
            }
        }
    }
}

fn check_station(
    obj: &ParsedObject,
    recipe: &RawObjectRecipeSection,
    index: &PackIndex<'_>,
    report: &mut Report,
) {
    let s = &recipe.station;
    if s.is_empty() {
        return; // inventory crafting grid
    }
    match parse_tag_ref(s) {
        Some(("station", name)) => {
            if !index.stations_declared.contains(name) {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!("recipe requires station '{}' but no object carries that station tag", s),
                    )
                    .with_path(obj.rel_path.clone())
                    .with_id(obj.id.clone())
                    .with_field("recipe.station")
                    .with_suggestion(format!(
                        "add `station.{name}` to the relevant station object's `tags:` list"
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
                .with_field("recipe.station")
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
                .with_field("recipe.station"),
            );
        }
    }
}
