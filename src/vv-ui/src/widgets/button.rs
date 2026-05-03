use crate::{
    UiButtonStyle, UiFrame, UiIconId, UiInput, UiInteraction, UiLayer, UiMouseButton, UiRect,
    UiTextAlign, UiWidgetId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiButtonResponse {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
}

#[derive(Debug, Clone)]
pub struct UiButton {
    pub id: UiWidgetId,
    pub rect: UiRect,
    pub label: String,
    pub icon: Option<UiIconId>,
    pub style: UiButtonStyle,
    pub layer: UiLayer,
    pub disabled: bool,
}

impl UiButton {
    pub fn new(
        id: UiWidgetId,
        rect: UiRect,
        label: impl Into<String>,
        style: UiButtonStyle,
    ) -> Self {
        Self {
            id,
            rect,
            label: label.into(),
            icon: None,
            style,
            layer: UiLayer::Menu,
            disabled: false,
        }
    }

    pub fn icon(mut self, icon: UiIconId) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn layer(mut self, layer: UiLayer) -> Self {
        self.layer = layer;
        self
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
    ) -> UiButtonResponse {
        let interaction = UiInteraction::from_input(self.id, self.rect, input, active);

        let background = if self.disabled {
            self.style.background.darken(0.35)
        } else if interaction.pressed {
            self.style.background_pressed
        } else if interaction.hovered {
            self.style.background_hover
        } else {
            self.style.background
        };

        frame.rounded_rect(
            self.layer,
            self.rect,
            background,
            self.style.radius,
            self.style.border,
            self.style.shadow,
        );

        let mut text_rect = self.rect;
        if let Some(icon) = self.icon {
            let icon_size = (self.rect.height * 0.46).min(28.0);
            let icon_rect = UiRect::new(
                self.rect.x + 18.0,
                self.rect.y + (self.rect.height - icon_size) * 0.5,
                icon_size,
                icon_size,
            );
            frame.icon(self.layer, icon_rect, icon, self.style.text);
            text_rect.x += icon_size + 24.0;
            text_rect.width -= icon_size + 24.0;
        }

        let color = if self.disabled {
            self.style.text_disabled
        } else {
            self.style.text
        };

        frame.text_aligned(
            self.layer,
            text_rect,
            self.label,
            (self.rect.height * 0.32).clamp(13.0, 22.0),
            color,
            UiTextAlign::Center,
        );

        UiButtonResponse {
            hovered: interaction.hovered,
            pressed: interaction.pressed,
            clicked: !self.disabled
                && interaction.hovered
                && input.pointer_released(UiMouseButton::Primary),
        }
    }
}
