use crate::app::runtime_state::GameRuntime;
use vv_render::{Renderer, StreamingView};

/// Update all renderer state that depends on the current frame's gameplay data.
///
/// Returns `true` when the window must be repainted this frame.
/// The game loop runs at display rate — this currently always returns `true`.
/// When the renderer exposes per-system change signals, this can become selective.
pub(super) fn tick_render_frame(
    runtime: &mut GameRuntime,
    renderer: &mut Renderer<'_>,
    dt: f32,
) -> bool {
    let cursor_id = runtime.cursor_id();
    renderer.update_cursor(runtime.planet(), cursor_id);
    renderer.update_block_damage_overlay(runtime.planet(), cursor_id);

    let selected_item = runtime
        .hotbar()
        .selected_item_id()
        .and_then(|item_id| runtime.planet().items.get(item_id));
    renderer.update_first_person_item(dt, selected_item);

    let width = renderer.config.width as f32;
    let height = renderer.config.height as f32;
    let view_ray = runtime
        .controller()
        .view_ray(runtime.player(), width, height);
    renderer.update_view(
        StreamingView {
            player_pos: runtime.player().position,
            camera_pos: runtime.controller().get_camera_pos(runtime.player()),
            view_dir: view_ray.direction,
            cursor_id,
        },
        runtime.planet(),
    );

    if !runtime.scene_snapshot_logged() && renderer.has_active_scene_chunks() {
        renderer.log_engine_snapshot("first-scene", runtime.planet());
        runtime.mark_scene_snapshot_logged();
    }

    true
}
