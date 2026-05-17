mod action_result;
pub(crate) mod content_bootstrap;
mod cursor;
mod debug_input;
mod dev_state;
mod event_router;
mod feedback_router;
mod frame_commands;
mod frame_driver;
mod game_app;
mod gameplay_actions;
mod golden_scene;
mod input_accumulator;
mod input_intent;
mod inventory_events;
mod player_frame_update;
mod player_input_router;
mod render_frame_update;
mod runtime_loop;
mod runtime_state;
mod ui_frame_update;
mod ui_input_router;
mod world_frame_update;

pub fn run() {
    runtime_loop::run();
}
