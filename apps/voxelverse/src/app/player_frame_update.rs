use crate::app::action_result::ActionResult;
use crate::app::cursor::release_cursor;
use crate::app::frame_commands::apply_action_result;
use crate::app::runtime_state::GameRuntime;
use vv_audio::AudioEngine;
use vv_gameplay::mine_block;
use vv_gameplay::BlockSelection;
use vv_gameplay::BlockSelectionMode;
use vv_render::Renderer;

/// Advance player-controlled systems: movement, cursor ray, and mining tick.
pub(super) fn tick_player_frame(
    runtime: &mut GameRuntime,
    renderer: &mut Renderer<'_>,
    audio: &mut AudioEngine,
    dt: f32,
) {
    if !runtime.ui_captures_input() {
        let player_input = runtime.sample_player_input();
        runtime.update_player_movement(player_input, dt);

        let width = renderer.config.width as f32;
        let height = renderer.config.height as f32;
        let ray = runtime
            .controller()
            .view_ray(runtime.player(), width, height);
        let ray_result = BlockSelection::trace(
            ray,
            runtime.controller().interaction_reach(),
            runtime.planet(),
            BlockSelectionMode::HitSolid,
        );
        runtime.set_cursor_id(ray_result.map(|(id, _)| id));

        if runtime.mining_button_held() {
            let result = ActionResult::from_gameplay(mine_block(runtime.as_mine_context(), dt));
            apply_action_result(result, renderer, audio, runtime);
        }
    } else {
        runtime.controller_mut().clear_transient_input();
        runtime.set_cursor_id(None);
        release_cursor(renderer.window);
    }
}
