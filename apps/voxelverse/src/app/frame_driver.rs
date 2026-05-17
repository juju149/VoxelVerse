use crate::app::game_app::GameApp;
use crate::app::player_frame_update::tick_player_frame;
use crate::app::render_frame_update::tick_render_frame;
use crate::app::ui_frame_update::tick_ui_frame;
use crate::app::world_frame_update::tick_world_frame;

/// Drive one logical game frame: flush input → world time → UI animation → player → render.
pub(super) fn tick_game_frame(app: &mut GameApp<'_>, dt: f32) {
    // Apply accumulated winit input to the controller before any gameplay tick.
    let input = app.input_accum.flush();
    app.runtime.apply_controller_input(input);

    tick_world_frame(&mut app.runtime, dt);
    tick_ui_frame(&mut app.runtime, dt);
    tick_player_frame(&mut app.runtime, &mut app.renderer, &mut app.audio, dt);
    if tick_render_frame(&mut app.runtime, &mut app.renderer, dt) {
        app.renderer.window.request_redraw();
    }
}
