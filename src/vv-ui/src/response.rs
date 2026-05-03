use crate::{UiInput, UiMouseButton, UiRect, UiWidgetId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiResponse {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
    pub focused: bool,
    pub disabled: bool,
}

impl UiResponse {
    pub fn from_input(
        id: UiWidgetId,
        rect: UiRect,
        input: &UiInput,
        active: Option<UiWidgetId>,
        disabled: bool,
    ) -> Self {
        let hovered = !disabled
            && input
                .pointer_position
                .map(|point| rect.contains(point))
                .unwrap_or(false);

        let pressed = !disabled && active == Some(id);
        let clicked = hovered && input.pointer_released(UiMouseButton::Primary);

        Self {
            hovered,
            pressed,
            clicked,
            focused: false,
            disabled,
        }
    }

    pub fn none() -> Self {
        Self::default()
    }
}
