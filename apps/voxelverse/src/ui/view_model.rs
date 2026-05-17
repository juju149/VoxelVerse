use crate::ui::{InventoryFilter, InventoryUiState};
use vv_gameplay::{quick_craft_recipe_indices, HotbarSlot, Inventory};
use vv_pack_compiler::{CompiledIngredient, CompiledRecipe, CompiledRecipeKind, RecipeRegistry};
use vv_render::{
    RenderCraftIngredient, RenderCraftRecipe, RenderCraftSnapshot, RenderInventorySnapshot,
    RenderItemStack,
};
use vv_world::PlanetData;

pub(crate) fn inventory_panel_snapshot(
    inventory: &Inventory,
    planet: &PlanetData,
    ui: &InventoryUiState,
) -> RenderInventorySnapshot {
    let slots = inventory.slots().map(|slot| slot.map(render_stack));
    let visible_slots =
        slots.map(|slot| slot.is_none_or(|stack| item_matches_view(stack, planet, ui)));

    RenderInventorySnapshot {
        slots,
        visible_slots,
        total_count: inventory.total_count(),
    }
}

fn item_matches_view(stack: RenderItemStack, planet: &PlanetData, ui: &InventoryUiState) -> bool {
    let Some(item) = planet.item(stack.item_id) else {
        return false;
    };

    matches_search(&ui.search_query, &item.display_name)
        && matches_filter(ui.active_filter, &item.category)
}

fn matches_search(query: &str, name: &str) -> bool {
    let query = query.to_lowercase();
    query.is_empty() || name.to_lowercase().contains(&query)
}

fn matches_filter(filter: InventoryFilter, category: &str) -> bool {
    match filter {
        InventoryFilter::All => true,
        InventoryFilter::Resources => matches!(
            category,
            "resource" | "ore" | "terrain" | "natural/log" | "natural/leaves" | "flora"
        ),
        InventoryFilter::Tools => matches!(category, "tool" | "weapon"),
        InventoryFilter::Food => matches!(category, "food" | "consumable"),
        InventoryFilter::Misc => !matches!(
            category,
            "resource"
                | "ore"
                | "terrain"
                | "natural/log"
                | "natural/leaves"
                | "flora"
                | "tool"
                | "weapon"
                | "food"
                | "consumable"
        ),
    }
}

fn render_stack(stack: HotbarSlot) -> RenderItemStack {
    RenderItemStack {
        item_id: stack.item_id,
        quantity: stack.quantity,
    }
}

pub(crate) fn craft_panel_snapshot(
    planet: &PlanetData,
    recipes: &RecipeRegistry,
    ui: &InventoryUiState,
) -> RenderCraftSnapshot {
    let recipe_indices = quick_craft_recipe_indices(recipes);
    let rows = recipe_indices
        .iter()
        .filter_map(|index| recipes.recipes().get(*index).map(|recipe| (*index, recipe)))
        .map(|(index, recipe)| craft_recipe_snapshot(index, recipe, planet, 1))
        .collect::<Vec<_>>();

    let selected_index = ui
        .selected_recipe
        .filter(|selected| recipe_indices.contains(selected))
        .or_else(|| recipe_indices.first().copied());
    let selected_recipe = selected_index
        .and_then(|index| recipes.recipes().get(index).map(|recipe| (index, recipe)))
        .map(|(index, recipe)| craft_recipe_snapshot(index, recipe, planet, ui.craft_quantity));

    RenderCraftSnapshot {
        recipes: rows,
        selected_recipe,
    }
}

fn craft_recipe_snapshot(
    index: usize,
    recipe: &CompiledRecipe,
    planet: &PlanetData,
    quantity: u32,
) -> RenderCraftRecipe {
    let output_count = recipe.output_count.saturating_mul(quantity.max(1));
    let output_name = planet
        .item(recipe.output_item)
        .expect("compiled recipe output item missing from runtime item registry")
        .display_name
        .clone();
    let station_label = recipe
        .station_tag
        .as_deref()
        .map(format_station_label)
        .unwrap_or_else(|| "Fabrication main".to_string());
    let ingredients = recipe_ingredient_counts(recipe)
        .into_iter()
        .map(|(ingredient, count)| craft_ingredient_snapshot(ingredient, count, planet, quantity))
        .collect();

    RenderCraftRecipe {
        index,
        output: RenderItemStack {
            item_id: recipe.output_item,
            quantity: output_count,
        },
        output_name,
        station_label,
        ingredients,
    }
}

fn craft_ingredient_snapshot(
    ingredient: CompiledIngredient,
    count: u32,
    planet: &PlanetData,
    quantity: u32,
) -> RenderCraftIngredient {
    match ingredient {
        CompiledIngredient::Item(item_id) => {
            let label = planet
                .item(item_id)
                .expect("compiled recipe ingredient item missing from runtime item registry")
                .display_name
                .clone();
            RenderCraftIngredient {
                icon: Some(RenderItemStack {
                    item_id,
                    quantity: count.saturating_mul(quantity.max(1)),
                }),
                label,
                count,
            }
        }
        CompiledIngredient::Tag(tag) => RenderCraftIngredient {
            icon: None,
            label: format!("Tag {}", tag),
            count,
        },
    }
}

fn recipe_ingredient_counts(recipe: &CompiledRecipe) -> Vec<(CompiledIngredient, u32)> {
    let ingredients: Vec<CompiledIngredient> = match &recipe.kind {
        CompiledRecipeKind::Shaped(shaped) => shaped.grid.iter().filter_map(Clone::clone).collect(),
        CompiledRecipeKind::Shapeless(shapeless) => shapeless.ingredients.clone(),
        CompiledRecipeKind::Smelting(_) => Vec::new(),
    };

    count_ingredients(ingredients)
}

fn count_ingredients(
    ingredients: impl IntoIterator<Item = CompiledIngredient>,
) -> Vec<(CompiledIngredient, u32)> {
    let mut counted: Vec<(CompiledIngredient, u32)> = Vec::new();
    for ingredient in ingredients {
        if let Some((_, count)) = counted
            .iter_mut()
            .find(|(existing, _)| same_ingredient(existing, &ingredient))
        {
            *count += 1;
        } else {
            counted.push((ingredient, 1));
        }
    }
    counted
}

fn same_ingredient(a: &CompiledIngredient, b: &CompiledIngredient) -> bool {
    match (a, b) {
        (CompiledIngredient::Item(a), CompiledIngredient::Item(b)) => a == b,
        (CompiledIngredient::Tag(a), CompiledIngredient::Tag(b)) => a == b,
        _ => false,
    }
}

fn format_station_label(station: &str) -> String {
    station
        .trim_start_matches("#station.")
        .trim_start_matches("core:tag/station/")
        .replace('_', " ")
}

#[cfg(test)]
mod tests {
    use super::{count_ingredients, format_station_label, matches_filter, matches_search};
    use crate::ui::InventoryFilter;
    use vv_pack_compiler::{CompiledIngredient, ItemId};

    #[test]
    fn station_labels_are_player_facing() {
        assert_eq!(
            format_station_label("core:tag/station/construction_table"),
            "construction table"
        );
    }

    #[test]
    fn ingredient_counts_merge_same_item_entries() {
        let counts = count_ingredients(vec![
            CompiledIngredient::Item(ItemId::from_raw(3)),
            CompiledIngredient::Item(ItemId::from_raw(3)),
            CompiledIngredient::Tag("core:tag/material/wood".into()),
        ]);

        assert!(counts.iter().any(|(ingredient, count)| {
            matches!(ingredient, CompiledIngredient::Item(id) if *id == ItemId::from_raw(3))
                && *count == 2
        }));
        assert!(counts.iter().any(|(ingredient, count)| {
            matches!(ingredient, CompiledIngredient::Tag(tag) if tag == "core:tag/material/wood")
                && *count == 1
        }));
    }

    #[test]
    fn inventory_search_is_case_insensitive() {
        assert!(matches_search("stone", "Polished Stone"));
        assert!(matches_search("", "Anything"));
        assert!(!matches_search("wood", "Stone"));
    }

    #[test]
    fn inventory_filters_match_v1_categories() {
        assert!(matches_filter(InventoryFilter::Resources, "ore"));
        assert!(matches_filter(InventoryFilter::Tools, "weapon"));
        assert!(matches_filter(InventoryFilter::Food, "consumable"));
        assert!(matches_filter(InventoryFilter::Misc, "decoration"));
        assert!(!matches_filter(InventoryFilter::Misc, "tool"));
    }
}
