use crate::{
    UiBorder, UiFrame, UiInput, UiLayer, UiMouseButton, UiRect, UiSurface, UiToggleStyle,
    UiWidgetId,
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
        _active: Option<UiWidgetId>,
    ) -> (bool, bool) {
        let hovered = !self.disabled
            && input
                .pointer_position
                .map(|point| self.rect.contains(point))
                .unwrap_or(false);

        let clicked = hovered && input.pointer_released(UiMouseButton::Primary);
        let value = if clicked { !self.value } else { self.value };

        let track = if value {
            self.style.track_on
        } else {
            self.style.track_off
        };

        frame.surface(
            self.layer,
            self.rect,
            UiSurface::new(track)
                .border(self.style.border.color, self.style.border.width)
                .radius(self.style.radius),
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
            UiBorder::NONE,
            crate::UiShadow::NONE,
        );

        (value, clicked)
    }
}
