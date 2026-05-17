use crate::app::runtime_state::{GameRuntime, InventoryInputContext};
use crate::ui::InventoryUiState;
use vv_gameplay::{quick_craft_recipe_indices, Controller, Hotbar, HotbarSlot, Inventory, Player};
use vv_pack_compiler::{CompiledIngredient, CompiledRecipe, CompiledRecipeKind, RecipeRegistry};
use vv_render::{
    RenderCamera, RenderConsoleSnapshot, RenderCraftIngredient, RenderCraftRecipe,
    RenderCraftSnapshot, RenderDebugFlags, RenderFrameSnapshot, RenderHeldStack,
    RenderHotbarSnapshot, RenderInventorySnapshot, RenderInventoryUiSnapshot, RenderItemStack,
    RenderUiSnapshot,
};
use vv_world::PlanetData;

pub(super) fn frame_from_runtime(
    runtime: &GameRuntime,
    width: f32,
    height: f32,
) -> RenderFrameSnapshot<'_> {
    let dev = runtime.dev_state();
    RenderFrameSnapshot {
        camera: camera_snapshot(
            runtime.controller(),
            runtime.player(),
            width,
            height,
            runtime.first_person(),
            runtime.cursor_id(),
        ),
        planet: runtime.planet(),
        hotbar: hotbar_snapshot(runtime.hotbar()),
        inventory: inventory_snapshot(runtime.inventory()),
        ui: RenderUiSnapshot {
            inventory: inventory_ui_snapshot(runtime.inventory_ui()),
        },
        craft: craft_snapshot(runtime.planet(), runtime.recipes(), runtime.inventory_ui()),
        console: RenderConsoleSnapshot {
            height_fraction: runtime.console().height_fraction,
            history: &runtime.console().history,
            input_buffer: &runtime.console().input_buffer,
        },
        debug: RenderDebugFlags {
            show_collisions: dev.show_collisions,
            freeze_culling: dev.freeze_culling,
            is_wireframe: dev.is_wireframe,
            debug_mode: runtime.dev_mode(),
        },
    }
}

pub(super) fn frame_from_inventory_context<'a>(
    ctx: &InventoryInputContext<'a>,
    width: f32,
    height: f32,
) -> RenderFrameSnapshot<'a> {
    RenderFrameSnapshot {
        camera: camera_snapshot(
            ctx.controller,
            ctx.player,
            width,
            height,
            ctx.controller.first_person,
            ctx.controller.cursor_id,
        ),
        planet: ctx.planet,
        hotbar: hotbar_snapshot(ctx.hotbar),
        inventory: inventory_snapshot(ctx.inventory),
        ui: RenderUiSnapshot {
            inventory: inventory_ui_snapshot(ctx.inventory_ui),
        },
        craft: craft_snapshot(ctx.planet, ctx.recipes, ctx.inventory_ui),
        console: RenderConsoleSnapshot {
            height_fraction: ctx.console.height_fraction,
            history: &ctx.console.history,
            input_buffer: &ctx.console.input_buffer,
        },
        debug: RenderDebugFlags {
            show_collisions: ctx.dev.show_collisions,
            freeze_culling: ctx.dev.freeze_culling,
            is_wireframe: ctx.dev.is_wireframe,
            debug_mode: false,
        },
    }
}

fn camera_snapshot(
    controller: &Controller,
    player: &Player,
    width: f32,
    height: f32,
    is_first_person: bool,
    cursor_id: Option<vv_voxel::VoxelCoord>,
) -> RenderCamera {
    RenderCamera {
        view_proj: controller.get_matrix(player, width, height),
        camera_pos: controller.get_camera_pos(player),
        player_pos: player.position,
        model_matrix: player.get_model_matrix(),
        is_first_person,
        cursor_id,
    }
}

fn hotbar_snapshot(hotbar: &Hotbar) -> RenderHotbarSnapshot {
    RenderHotbarSnapshot {
        slots: hotbar.slots().map(|slot| slot.map(render_stack)),
        selected_index: hotbar.selected_index(),
        revision: hotbar.revision(),
        notice_text: hotbar.notice_text(),
    }
}

fn inventory_snapshot(inventory: &Inventory) -> RenderInventorySnapshot {
    RenderInventorySnapshot {
        slots: inventory.slots().map(|slot| slot.map(render_stack)),
        total_count: inventory.total_count(),
    }
}

fn inventory_ui_snapshot(ui: &InventoryUiState) -> RenderInventoryUiSnapshot {
    RenderInventoryUiSnapshot {
        is_open: ui.is_open,
        search_query: ui.search_query.clone(),
        held: ui.held.map(RenderHeldStack::from),
        cursor: ui.cursor,
        hovered_slot: ui.hovered_slot,
        hovered_button: ui.hovered_button,
        hovered_search: ui.hovered_search,
        hovered_filter: ui.hovered_filter,
        hovered_recipe: ui.hovered_recipe,
        active_filter: ui.active_filter,
        selected_recipe: ui.selected_recipe,
        craft_quantity: ui.craft_quantity,
        search_focused: ui.search_focused,
        user_zoom: ui.user_zoom,
        capacity_kg: ui.capacity_kg,
    }
}

fn craft_snapshot(
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
        .items
        .get(recipe.output_item)
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
                .items
                .get(item_id)
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

fn render_stack(stack: HotbarSlot) -> RenderItemStack {
    RenderItemStack {
        item_id: stack.item_id,
        quantity: stack.quantity,
    }
}

#[cfg(test)]
mod tests {
    use super::{hotbar_snapshot, inventory_snapshot};
    use vv_gameplay::{Hotbar, Inventory};
    use vv_pack_compiler::ItemId;

    #[test]
    fn hotbar_snapshot_contains_render_owned_slot_data() {
        let item = ItemId::from_raw(7);
        let mut hotbar = Hotbar::new();
        assert!(hotbar.add(item, 3, 99));
        hotbar.select(0);

        let snapshot = hotbar_snapshot(&hotbar);

        assert_eq!(snapshot.slots[0].unwrap().item_id, item);
        assert_eq!(snapshot.slots[0].unwrap().quantity, 3);
        assert_eq!(snapshot.selected_index, 0);
        assert_eq!(snapshot.revision, hotbar.revision());
    }

    #[test]
    fn inventory_snapshot_copies_slots_and_total_count() {
        let first = ItemId::from_raw(2);
        let second = ItemId::from_raw(5);
        let mut inventory = Inventory::new();
        assert!(inventory.add(first, 4, 99));
        assert!(inventory.add(second, 6, 99));

        let snapshot = inventory_snapshot(&inventory);

        assert_eq!(snapshot.slots[0].unwrap().item_id, first);
        assert_eq!(snapshot.slots[0].unwrap().quantity, 4);
        assert_eq!(snapshot.slots[1].unwrap().item_id, second);
        assert_eq!(snapshot.total_count, 10);
    }
}
