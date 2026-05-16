use crate::app::action_result::{ActionResult, FrameCommand};
use crate::app::feedback_router::route_feedback_events;
use crate::app::gameplay_actions::place_block;
use crate::app::input_intent::InputIntent;
use crate::app::runtime_state::GameRuntime;
use vv_audio::AudioEngine;
use vv_render::Renderer;

/// Dispatch a gameplay `InputIntent` produced from normal (non-UI, non-debug) player input.
///
/// Returns any `FrameCommand`s that the frame driver should apply after this call.
/// Feedback events are forwarded immediately to renderer + audio.
pub(super) fn dispatch_intent(
    intent: InputIntent,
    runtime: &mut GameRuntime,
    renderer: &mut Renderer<'_>,
    audio: &mut AudioEngine,
) {
    match intent {
        InputIntent::StartMining => {
            runtime.set_mining_button_held(true);
        }
        InputIntent::StopMining => {
            runtime.set_mining_button_held(false);
        }
        InputIntent::PlaceBlock => {
            let w = renderer.config.width as f32;
            let h = renderer.config.height as f32;
            let result = place_block(runtime.as_place_context(w, h));
            apply_action_result(result, renderer, audio);
        }
        InputIntent::SelectHotbarSlot(index) => {
            runtime.hotbar_mut().select(index);
            renderer.window.request_redraw();
        }
        InputIntent::ScrollHotbar(offset) => {
            runtime.hotbar_mut().select_offset(offset);
            renderer.window.request_redraw();
        }
        InputIntent::ToggleInventory => {
            runtime.inventory_ui_mut().toggle();
            if runtime.inventory_ui().is_open {
                renderer.window.request_redraw();
                // Cursor release is handled by game_app sync_cursor_mode.
            } else {
                renderer.window.request_redraw();
            }
        }
    }
}

/// Apply the commands and feedback produced by a gameplay action.
pub(super) fn apply_action_result(
    result: ActionResult,
    renderer: &mut Renderer<'_>,
    audio: &mut AudioEngine,
) {
    route_feedback_events(renderer, audio, &result.feedback);
    for cmd in result.commands {
        apply_frame_command(cmd, renderer);
    }
}

fn apply_frame_command(cmd: FrameCommand, renderer: &mut Renderer<'_>) {
    match cmd {
        FrameCommand::Redraw => {
            renderer.window.request_redraw();
        }
        FrameCommand::GrabCursor => {
            crate::app::cursor::grab_cursor(renderer.window);
        }
        FrameCommand::RefreshDirtyChunks(keys) => {
            renderer.refresh_dirty_chunks(keys);
        }
    }
}
