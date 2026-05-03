use crate::{
    UiButtonStyle, UiFrame, UiIconId, UiInput, UiLayer, UiRect, UiResponse, UiSurface, UiWidgetId,
};

pub type UiButtonResponse = UiResponse;

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
        let response = UiResponse::from_input(self.id, self.rect, input, active, self.disabled);

        let background = if self.disabled {
            self.style.background.darken(0.35)
        } else if response.pressed {
            self.style.background_pressed
        } else if response.hovered {
            self.style.background_hover
        } else {
            self.style.background
        };

        frame.surface(
            self.layer,
            self.rect,
            UiSurface::new(background)
                .border(self.style.border.color, self.style.border.width)
                .radius(self.style.radius)
                .shadow(self.style.shadow),
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

        frame.text_centered(
            self.layer,
            text_rect,
            self.label,
            (self.rect.height * 0.32).clamp(13.0, 22.0),
            color,
        );

        response
    }
}
