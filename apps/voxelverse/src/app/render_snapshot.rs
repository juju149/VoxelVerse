use crate::app::runtime_state::{GameRuntime, InventoryInputContext};
use crate::ui::{craft_panel_snapshot, inventory_panel_snapshot, InventoryUiState};
use vv_gameplay::{Controller, Hotbar, HotbarSlot, Player};
use vv_render::{
    RenderCamera, RenderConsoleSnapshot, RenderDebugFlags, RenderFrameSnapshot, RenderHeldStack,
    RenderHotbarSnapshot, RenderInventoryUiSnapshot, RenderItemStack, RenderUiSnapshot,
};

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
        inventory: inventory_panel_snapshot(
            runtime.inventory(),
            runtime.planet(),
            runtime.inventory_ui(),
        ),
        ui: RenderUiSnapshot {
            inventory: inventory_ui_snapshot(runtime.inventory_ui()),
        },
        craft: craft_panel_snapshot(runtime.planet(), runtime.recipes(), runtime.inventory_ui()),
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
        weather: None,
        celestial: None,
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
        inventory: inventory_panel_snapshot(ctx.inventory, ctx.planet, ctx.inventory_ui),
        ui: RenderUiSnapshot {
            inventory: inventory_ui_snapshot(ctx.inventory_ui),
        },
        craft: craft_panel_snapshot(ctx.planet, ctx.recipes, ctx.inventory_ui),
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
        weather: None,
        celestial: None,
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

fn render_stack(stack: HotbarSlot) -> RenderItemStack {
    RenderItemStack {
        item_id: stack.item_id,
        quantity: stack.quantity,
    }
}

#[cfg(test)]
mod tests {
    use super::hotbar_snapshot;
    use crate::ui::{inventory_panel_snapshot, InventoryUiState};
    use vv_gameplay::{Hotbar, Inventory};
    use vv_pack_compiler::ItemId;
    use vv_world::PlanetData;

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

        let content = crate::app::content_bootstrap::load_core_content()
            .expect("core pack must load in tests");
        let planet = PlanetData::new(
            content.planet,
            content.blocks,
            content.items,
            content.procedural,
            content.procedural_planet_index,
        );

        let snapshot = inventory_panel_snapshot(&inventory, &planet, &InventoryUiState::new());

        assert_eq!(snapshot.slots[0].unwrap().item_id, first);
        assert_eq!(snapshot.slots[0].unwrap().quantity, 4);
        assert_eq!(snapshot.slots[1].unwrap().item_id, second);
        assert!(snapshot.visible_slots.iter().all(|visible| *visible));
        assert_eq!(snapshot.total_count, 10);
    }
}
