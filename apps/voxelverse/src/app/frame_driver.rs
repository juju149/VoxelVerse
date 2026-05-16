use crate::app::cursor::release_cursor;
use crate::app::feedback_router::{route_feedback, sound_kind, AppFeedback};
use crate::app::game_app::GameApp;
use vv_gameplay::{
    BlockAction, BlockInteraction, BlockSelection, BlockSelectionMode, Hotbar, HotbarNotice,
    Inventory, ItemId, MiningFeedback, MiningStrikeInput, PlayerController,
};
use vv_pack_compiler::LootRegistry;
use vv_render::StreamingView;
use vv_world::{BlockDamageResult, PlanetData};

pub(super) fn tick_game_frame(app: &mut GameApp<'_>, dt: f32) {
    app.console.update_animation(dt);
    app.hotbar.update(dt);
    app.planet.world_time.tick(dt);

    if !app.console.is_open && !app.inventory_ui.is_open {
        let player_input = app.controller.sample_player_input();
        PlayerController::update(&mut app.player, &app.planet, player_input, dt);

        let width = app.renderer.config.width as f32;
        let height = app.renderer.config.height as f32;
        let ray = app.controller.view_ray(&app.player, width, height);
        let ray_result = BlockSelection::trace(
            ray,
            app.controller.interaction_reach(),
            &app.planet,
            BlockSelectionMode::HitSolid,
        );
        app.controller.cursor_id = ray_result.map(|(id, _)| id);
        if app.mining_button_held {
            tick_mining(app, dt);
        }
    } else {
        app.controller.clear_transient_input();
        app.controller.cursor_id = None;
        release_cursor(app.renderer.window);
    }

    app.renderer
        .update_cursor(&app.planet, app.controller.cursor_id);
    app.renderer
        .update_block_damage_overlay(&app.planet, app.controller.cursor_id);
    let selected_item = app
        .hotbar
        .selected_item_id()
        .and_then(|item_id| app.planet.items.get(item_id));
    app.renderer.update_first_person_item(dt, selected_item);
    let width = app.renderer.config.width as f32;
    let height = app.renderer.config.height as f32;
    let view_ray = app.controller.view_ray(&app.player, width, height);
    app.renderer.update_view(
        StreamingView {
            player_pos: app.player.position,
            camera_pos: app.controller.get_camera_pos(&app.player),
            view_dir: view_ray.direction,
            cursor_id: app.controller.cursor_id,
        },
        &app.planet,
    );
    if !app.first_scene_snapshot_logged && app.renderer.has_active_scene_chunks() {
        app.renderer.log_engine_snapshot("first-scene", &app.planet);
        app.first_scene_snapshot_logged = true;
    }
    app.renderer.window.request_redraw();
}

fn tick_mining(app: &mut GameApp<'_>, dt: f32) {
    let coord = app.controller.cursor_id;
    let voxel = coord.map(|coord| app.planet.get_voxel(coord));
    let block = voxel.and_then(|voxel| app.planet.content.block(voxel));
    let feedback = app.mining.tick(MiningStrikeInput {
        coord,
        voxel,
        block,
        selected_item: app.hotbar.selected_item_id(),
        items: &app.planet.items,
        dt,
        wants_mining: true,
    });

    match feedback {
        MiningFeedback::None => {}
        MiningFeedback::Blocked => {
            app.hotbar.show_notice(HotbarNotice::ProtectedBlock);
            app.renderer.window.request_redraw();
        }
        MiningFeedback::Hit {
            coord,
            voxel,
            damage,
            break_threshold,
            impact_strength,
            drops_enabled,
            ..
        } => {
            let sound = app
                .planet
                .content
                .block(voxel)
                .map(|block| sound_kind(block.sound_kind))
                .unwrap_or_default();
            route_feedback(
                &mut app.renderer,
                &mut app.audio,
                AppFeedback::ToolSwing {
                    strength: impact_strength,
                },
            );
            match app
                .planet
                .apply_block_damage(coord, damage, break_threshold)
            {
                BlockDamageResult::Unchanged => {}
                BlockDamageResult::Damaged { .. } => {
                    route_feedback(
                        &mut app.renderer,
                        &mut app.audio,
                        AppFeedback::BlockHit {
                            sound_kind: sound,
                            strength: impact_strength,
                        },
                    );
                    app.renderer.window.request_redraw();
                }
                BlockDamageResult::Broken => {
                    handle_broken_block(
                        &mut app.renderer,
                        &mut app.planet,
                        &mut app.hotbar,
                        &mut app.inventory,
                        &app.loot,
                        coord,
                        voxel,
                        drops_enabled,
                    );
                    route_feedback(
                        &mut app.renderer,
                        &mut app.audio,
                        AppFeedback::BlockBreak {
                            sound_kind: sound,
                            strength: impact_strength,
                        },
                    );
                }
            }
        }
        MiningFeedback::Broken { .. } => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_broken_block(
    renderer: &mut vv_render::Renderer<'_>,
    planet: &mut PlanetData,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    loot: &LootRegistry,
    coord: vv_voxel::VoxelCoord,
    voxel: vv_voxel::VoxelId,
    drops_enabled: bool,
) {
    let edit = BlockInteraction::apply(BlockAction::Mine(coord), planet);
    if edit.dirty_chunks.is_empty() {
        hotbar.show_notice(HotbarNotice::ProtectedBlock);
        renderer.window.request_redraw();
        return;
    }

    if drops_enabled {
        if let Some(block) = planet.content.block(voxel) {
            for (item_id, count) in roll_block_drops(&block.drops_key, loot) {
                add_drop_to_player(item_id, count, planet, hotbar, inventory);
            }
        }
    }
    renderer.refresh_dirty_chunks(edit.dirty_chunks);
    renderer.window.request_redraw();
}

fn add_drop_to_player(
    item_id: ItemId,
    count: u32,
    planet: &PlanetData,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
) {
    let item = planet.items.get(item_id);
    let max_stack = item.map(|i| i.stack_size.0).unwrap_or(99);
    if !hotbar.add(item_id, count, max_stack) && !inventory.add(item_id, count, max_stack) {
        hotbar.show_notice(HotbarNotice::Full);
    }
}

fn roll_block_drops(drops_key: &str, loot: &LootRegistry) -> Vec<(ItemId, u32)> {
    match loot.get_by_key(drops_key) {
        Some(table) => table.roll(|| 0.0),
        None => Vec::new(),
    }
}
