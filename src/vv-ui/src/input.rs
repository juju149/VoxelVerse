use crate::{UiPoint, UiRect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiWidgetId(pub u64);

impl UiWidgetId {
    pub const NONE: Self = Self(0);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMouseButton {
    Primary,
    Secondary,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiPointerPhase {
    Pressed,
    Released,
    Moved,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiPointerEvent {
    pub phase: UiPointerPhase,
    pub button: Option<UiMouseButton>,
    pub position: UiPoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiKeyboardEvent {
    pub key: String,
    pub pressed: bool,
}

#[derive(Debug, Clone, Default)]
pub struct UiInput {
    pub pointer_position: Option<UiPoint>,
    pub pointer_events: Vec<UiPointerEvent>,
    pub keyboard_events: Vec<UiKeyboardEvent>,
    pub scroll_delta_y: f32,
    pub dt: f32,
}

impl UiInput {
    pub fn pointer_pressed(&self, button: UiMouseButton) -> bool {
        self.pointer_events
            .iter()
            .any(|event| event.phase == UiPointerPhase::Pressed && event.button == Some(button))
    }

    pub fn pointer_released(&self, button: UiMouseButton) -> bool {
        self.pointer_events
            .iter()
            .any(|event| event.phase == UiPointerPhase::Released && event.button == Some(button))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiInteraction {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
    pub focused: bool,
    pub disabled: bool,
}

impl UiInteraction {
    pub fn from_input(
        id: UiWidgetId,
        rect: UiRect,
        input: &UiInput,
        active: Option<UiWidgetId>,
    ) -> Self {
        let hovered = input
            .pointer_position
            .map(|point| rect.contains(point))
            .unwrap_or(false);

        let pressed = active == Some(id);
        let clicked = hovered && input.pointer_released(UiMouseButton::Primary);

        Self {
            hovered,
            pressed,
            clicked,
            focused: false,
            disabled: false,
        }
    }
}
