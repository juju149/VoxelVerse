pub(crate) mod content_bootstrap;
mod cursor;
mod event_router;
mod feedback_router;
mod frame_driver;
mod game_app;
mod gameplay_actions;
mod golden_scene;
mod inventory_events;
mod runtime_loop;
mod runtime_state;

pub fn run() {
    runtime_loop::run();
}
