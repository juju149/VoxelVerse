use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};

/// What the player wants to do with an input event.
///
/// `WindowEvent` is converted into an `InputIntent` by the input routers.
/// Gameplay systems then react to intents, never to raw hardware events.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum InputIntent {
    /// Begin holding the mining / destroy button.
    StartMining,
    /// Release the mining button.
    StopMining,
    /// Place the selected hotbar item onto the target face.
    PlaceBlock,
    /// Select a specific hotbar slot (0-indexed).
    SelectHotbarSlot(usize),
    /// Scroll the hotbar selection by a signed offset.
    ScrollHotbar(i32),
    /// Open or close the inventory screen.
    ToggleInventory,
}

/// Decode a mouse button event into the matching gameplay intent, if any.
pub(super) fn intent_for_mouse_button(
    button: MouseButton,
    state: ElementState,
) -> Option<InputIntent> {
    match (button, state) {
        (MouseButton::Left, ElementState::Pressed) => Some(InputIntent::StartMining),
        (MouseButton::Left, ElementState::Released) => Some(InputIntent::StopMining),
        (MouseButton::Right, ElementState::Pressed) => Some(InputIntent::PlaceBlock),
        _ => None,
    }
}

/// Decode a keyboard press into a hotbar-select intent, if the key is a digit key.
pub(super) fn intent_for_hotbar_key(key: PhysicalKey) -> Option<InputIntent> {
    let slot = match key {
        PhysicalKey::Code(KeyCode::Digit1) => 0,
        PhysicalKey::Code(KeyCode::Digit2) => 1,
        PhysicalKey::Code(KeyCode::Digit3) => 2,
        PhysicalKey::Code(KeyCode::Digit4) => 3,
        PhysicalKey::Code(KeyCode::Digit5) => 4,
        PhysicalKey::Code(KeyCode::Digit6) => 5,
        PhysicalKey::Code(KeyCode::Digit7) => 6,
        PhysicalKey::Code(KeyCode::Digit8) => 7,
        PhysicalKey::Code(KeyCode::Digit9) => 8,
        _ => return None,
    };
    Some(InputIntent::SelectHotbarSlot(slot))
}

/// Decode a scroll wheel delta into a hotbar-scroll intent, if the delta is significant.
pub(super) fn intent_for_scroll(delta: MouseScrollDelta) -> Option<InputIntent> {
    let y = match delta {
        MouseScrollDelta::LineDelta(_, y) => y,
        MouseScrollDelta::PixelDelta(p) => p.y as f32 * 0.01,
    };
    if y.abs() <= f32::EPSILON {
        return None;
    }
    Some(InputIntent::ScrollHotbar(if y > 0.0 { -1 } else { 1 }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_press_maps_to_start_mining() {
        assert_eq!(
            intent_for_mouse_button(MouseButton::Left, ElementState::Pressed),
            Some(InputIntent::StartMining)
        );
    }

    #[test]
    fn left_release_maps_to_stop_mining() {
        assert_eq!(
            intent_for_mouse_button(MouseButton::Left, ElementState::Released),
            Some(InputIntent::StopMining)
        );
    }

    #[test]
    fn right_press_maps_to_place_block() {
        assert_eq!(
            intent_for_mouse_button(MouseButton::Right, ElementState::Pressed),
            Some(InputIntent::PlaceBlock)
        );
    }

    #[test]
    fn digit1_maps_to_slot_0() {
        assert_eq!(
            intent_for_hotbar_key(PhysicalKey::Code(KeyCode::Digit1)),
            Some(InputIntent::SelectHotbarSlot(0))
        );
    }

    #[test]
    fn digit9_maps_to_slot_8() {
        assert_eq!(
            intent_for_hotbar_key(PhysicalKey::Code(KeyCode::Digit9)),
            Some(InputIntent::SelectHotbarSlot(8))
        );
    }

    #[test]
    fn scroll_up_maps_to_negative_offset() {
        let result = intent_for_scroll(MouseScrollDelta::LineDelta(0.0, 1.0));
        assert_eq!(result, Some(InputIntent::ScrollHotbar(-1)));
    }

    #[test]
    fn scroll_down_maps_to_positive_offset() {
        let result = intent_for_scroll(MouseScrollDelta::LineDelta(0.0, -1.0));
        assert_eq!(result, Some(InputIntent::ScrollHotbar(1)));
    }

    #[test]
    fn scroll_zero_produces_no_intent() {
        let result = intent_for_scroll(MouseScrollDelta::LineDelta(0.0, 0.0));
        assert_eq!(result, None);
    }
}
