use crate::{
    UiFrame, UiInput, UiInteraction, UiLayer, UiMouseButton, UiRect, UiToggleStyle, UiWidgetId,
};

#[derive(Debug, Clone, Copy)]
pub struct UiToggle {
    pub id: UiWidgetId,
    pub rect: UiRect,
    pub value: bool,
    pub style: UiToggleStyle,
    pub layer: UiLayer,
    pub disabled: bool,
}

impl UiToggle {
    pub fn new(id: UiWidgetId, rect: UiRect, value: bool, style: UiToggleStyle) -> Self {
        Self {
            id,
            rect,
            value,
            style,
            layer: UiLayer::Menu,
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn draw(
        self,
        frame: &mut UiFrame,
        input: &UiInput,
        active: Option<UiWidgetId>,
    ) -> (bool, bool) {
        let interaction = UiInteraction::from_input(self.id, self.rect, input, active);
        let clicked =
            !self.disabled && interaction.hovered && input.pointer_released(UiMouseButton::Primary);

        let value = if clicked { !self.value } else { self.value };
        let track = if value {
            self.style.track_on
        } else {
            self.style.track_off
        };

        frame.rounded_rect(
            self.layer,
            self.rect,
            track,
            self.style.radius,
            self.style.border,
            crate::UiShadow::NONE,
        );

        let pad = 3.0;
        let thumb_size = (self.rect.height - pad * 2.0).max(0.0);
        let thumb_x = if value {
            self.rect.right() - pad - thumb_size
        } else {
            self.rect.x + pad
        };

        frame.rounded_rect(
            self.layer,
            UiRect::new(thumb_x, self.rect.y + pad, thumb_size, thumb_size),
            self.style.thumb,
            self.style.radius,
            crate::UiBorder::NONE,
            crate::UiShadow::NONE,
        );

        (value, clicked)
    }
}
