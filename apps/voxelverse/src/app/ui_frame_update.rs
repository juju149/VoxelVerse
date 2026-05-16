use crate::app::runtime_state::GameRuntime;

/// Tick UI systems that animate every frame regardless of gameplay state.
pub(super) fn tick_ui_frame(runtime: &mut GameRuntime, dt: f32) {
    runtime.console_mut().update_animation(dt);
    runtime.hotbar_mut().update(dt);
}
