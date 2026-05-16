use crate::app::runtime_state::GameRuntime;

/// Tick world-level systems that advance every frame independently of player input.
pub(super) fn tick_world_frame(runtime: &mut GameRuntime, dt: f32) {
    runtime.planet_mut().world_time.tick(dt);
}
