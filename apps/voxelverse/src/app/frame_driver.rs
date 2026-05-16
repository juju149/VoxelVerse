use crate::app::game_app::GameApp;
use crate::app::player_frame_update::tick_player_frame;
use crate::app::render_frame_update::tick_render_frame;
use crate::app::ui_frame_update::tick_ui_frame;
use crate::app::world_frame_update::tick_world_frame;

/// Drive one logical game frame: world time → UI animation → player → render.
pub(super) fn tick_game_frame(app: &mut GameApp<'_>, dt: f32) {
    tick_world_frame(&mut app.runtime, dt);
    tick_ui_frame(&mut app.runtime, dt);
    tick_player_frame(&mut app.runtime, &mut app.renderer, &mut app.audio, dt);
    tick_render_frame(&mut app.runtime, &mut app.renderer, dt);
}
