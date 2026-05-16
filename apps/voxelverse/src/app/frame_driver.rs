use crate::app::cursor::release_cursor;
use crate::app::game_app::GameApp;
use crate::app::gameplay_actions::{mine_block, MineBlockContext};
use vv_gameplay::{BlockSelection, BlockSelectionMode, PlayerController};
use vv_render::StreamingView;

pub(super) fn tick_game_frame(app: &mut GameApp<'_>, dt: f32) {
    app.runtime.ui.console.update_animation(dt);
    app.runtime.gameplay.hotbar.update(dt);
    app.runtime.planet.world_time.tick(dt);

    if !app.runtime.ui.console.is_open && !app.runtime.ui.inventory.is_open {
        let player_input = app.runtime.gameplay.controller.sample_player_input();
        PlayerController::update(
            &mut app.runtime.gameplay.player,
            &app.runtime.planet,
            player_input,
            dt,
        );

        let width = app.renderer.config.width as f32;
        let height = app.renderer.config.height as f32;
        let ray =
            app.runtime
                .gameplay
                .controller
                .view_ray(&app.runtime.gameplay.player, width, height);
        let ray_result = BlockSelection::trace(
            ray,
            app.runtime.gameplay.controller.interaction_reach(),
            &app.runtime.planet,
            BlockSelectionMode::HitSolid,
        );
        app.runtime.gameplay.controller.cursor_id = ray_result.map(|(id, _)| id);
        if app.runtime.gameplay.mining_button_held {
            mine_block(
                MineBlockContext {
                    renderer: &mut app.renderer,
                    audio: &mut app.audio,
                    controller: &app.runtime.gameplay.controller,
                    planet: &mut app.runtime.planet,
                    hotbar: &mut app.runtime.gameplay.hotbar,
                    inventory: &mut app.runtime.gameplay.inventory,
                    mining: &mut app.runtime.gameplay.mining,
                    loot: &app.runtime.content.loot,
                },
                dt,
            );
        }
    } else {
        app.runtime.gameplay.controller.clear_transient_input();
        app.runtime.gameplay.controller.cursor_id = None;
        release_cursor(app.renderer.window);
    }

    app.renderer.update_cursor(
        &app.runtime.planet,
        app.runtime.gameplay.controller.cursor_id,
    );
    app.renderer.update_block_damage_overlay(
        &app.runtime.planet,
        app.runtime.gameplay.controller.cursor_id,
    );
    let selected_item = app
        .runtime
        .gameplay
        .hotbar
        .selected_item_id()
        .and_then(|item_id| app.runtime.planet.items.get(item_id));
    app.renderer.update_first_person_item(dt, selected_item);
    let width = app.renderer.config.width as f32;
    let height = app.renderer.config.height as f32;
    let view_ray =
        app.runtime
            .gameplay
            .controller
            .view_ray(&app.runtime.gameplay.player, width, height);
    app.renderer.update_view(
        StreamingView {
            player_pos: app.runtime.gameplay.player.position,
            camera_pos: app
                .runtime
                .gameplay
                .controller
                .get_camera_pos(&app.runtime.gameplay.player),
            view_dir: view_ray.direction,
            cursor_id: app.runtime.gameplay.controller.cursor_id,
        },
        &app.runtime.planet,
    );
    if !app.runtime.first_scene_snapshot_logged && app.renderer.has_active_scene_chunks() {
        app.renderer
            .log_engine_snapshot("first-scene", &app.runtime.planet);
        app.runtime.first_scene_snapshot_logged = true;
    }
    app.renderer.window.request_redraw();
}
