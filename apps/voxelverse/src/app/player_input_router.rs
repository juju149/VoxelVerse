use crate::app::action_result::ActionResult;
use crate::app::input_intent::InputIntent;
use crate::app::runtime_state::GameRuntime;
use vv_gameplay::place_block;

/// Dispatch a gameplay `InputIntent` produced from normal (non-UI, non-debug) player input.
///
/// Returns an `ActionResult` that the caller must apply via `frame_commands::apply_action_result`.
/// No feedback, commands, or UI events are applied inside this function.
pub(super) fn dispatch_intent(
    intent: InputIntent,
    runtime: &mut GameRuntime,
    view_width: f32,
    view_height: f32,
) -> ActionResult {
    let mut result = ActionResult::none();
    match intent {
        InputIntent::StartMining => {
            runtime.set_mining_button_held(true);
        }
        InputIntent::StopMining => {
            runtime.set_mining_button_held(false);
        }
        InputIntent::PlaceBlock => {
            result = ActionResult::from_gameplay(place_block(
                runtime.as_place_context(view_width, view_height),
            ));
        }
        InputIntent::SelectHotbarSlot(index) => {
            runtime.hotbar_mut().select(index);
            result.push_redraw();
        }
        InputIntent::ScrollHotbar(offset) => {
            runtime.hotbar_mut().select_offset(offset);
            result.push_redraw();
        }
        InputIntent::ToggleInventory => {
            runtime.inventory_ui_mut().toggle();
            result.push_redraw();
            if !runtime.inventory_ui().is_open && runtime.first_person() {
                result.push_grab_cursor();
            } else if runtime.inventory_ui().is_open {
                result.push_release_cursor();
            }
        }
    }
    result
}
