//! Recipe checks: inputs / outputs must resolve to real items (or tags).

use vv_content_schema::{RawIngredient, RawRecipeKind};

use crate::report::Report;
use crate::scan::PackScan;

const CHECK: &str = "recipes";

pub fn run(scan: &PackScan, report: &mut Report) {
    for (id, recipe) in &scan.recipes {
        // Output item must exist.
        if !scan.item_id_exists(&recipe.result.item.0) {
            report.error_id(
                CHECK,
                format!(
                    "recipe '{}' produces missing item '{}'",
                    id, recipe.result.item.0
                ),
                id.clone(),
            );
        }
        if recipe.result.count == 0 {
            report.error_id(
                CHECK,
                format!("recipe '{}' has output count 0", id),
                id.clone(),
            );
        }

        // Ingredients must exist (items only - tags are passed through).
        match &recipe.recipe {
            RawRecipeKind::Shaped(shaped) => {
                for ing in shaped.keys.values() {
                    check_ingredient(scan, report, id, ing);
                }
            }
            RawRecipeKind::Shapeless(shapeless) => {
                for ing in &shapeless.ingredients {
                    check_ingredient(scan, report, id, ing);
                }
            }
            RawRecipeKind::Smelting(smelting) => {
                check_ingredient(scan, report, id, &smelting.ingredient);
                if smelting.fuel == 0 {
                    report.warn_id(
                        CHECK,
                        format!("recipe '{}' has fuel 0", id),
                        id.clone(),
                    );
                }
            }
        }
    }
}

fn check_ingredient(scan: &PackScan, report: &mut Report, recipe_id: &str, ing: &RawIngredient) {
    match ing {
        RawIngredient::Item(item) => {
            if !scan.item_id_exists(&item.0) {
                report.error_id(
                    CHECK,
                    format!(
                        "recipe '{}' uses missing item '{}'",
                        recipe_id, item.0
                    ),
                    recipe_id.to_string(),
                );
            }
        }
        RawIngredient::Tag(_) => {
            // Tag-based ingredients are resolved by the compiler; we let it
            // signal mismatches.
        }
    }
}
