use vv_registry::{CompiledContent, CompiledIngredient, CompiledRecipe};

use crate::Inventory;

/// Returns true when the inventory can satisfy a recipe ingredient.
///
/// Current state:
/// - Item ingredients are supported.
/// - Tag ingredients require a registry-level item tag index.
///
/// This module exists to keep crafting.rs small and to make tag support the canonical path.
pub fn has_recipe_ingredients(
    inventory: &Inventory,
    recipe: &CompiledRecipe,
    content: &CompiledContent,
) -> bool {
    recipe
        .ingredients
        .iter()
        .all(|ingredient| has_ingredient(inventory, ingredient, content))
}

pub fn has_ingredient(
    inventory: &Inventory,
    ingredient: &CompiledIngredient,
    _content: &CompiledContent,
) -> bool {
    match ingredient {
        CompiledIngredient::Item { item, count } => inventory.item_count(*item) >= *count,
        CompiledIngredient::Tag { .. } => {
            // TODO:
            // Implement once vv-registry exposes an item tag index:
            // tag -> item ids.
            false
        }
    }
}
