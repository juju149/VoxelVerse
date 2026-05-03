use crate::{UiCardStyle, UiFrame, UiInput, UiInteraction, UiLayer, UiRect, UiWidgetId};

#[derive(Debug, Clone, Copy)]
pub struct UiCard {
    pub id: UiWidgetId,
    pub rect: UiRect,
    pub style: UiCardStyle,
    pub layer: UiLayer,
}

impl UiCard {
    pub fn new(id: UiWidgetId, rect: UiRect, style: UiCardStyle) -> Self {
        Self {
            id,
            rect,
            style,
            layer: UiLayer::Menu,
        }
    }

    pub fn layer(mut self, layer: UiLayer) -> Self {
        self.layer = layer;
        self
    }

    pub fn draw(self, frame: &mut UiFrame, input: &UiInput) -> bool {
        let hovered = input
            .pointer_position
            .map(|point| self.rect.contains(point))
            .unwrap_or(false);

        let background = if hovered {
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

        hovered
    }

    pub fn interaction(self, input: &UiInput, active: Option<UiWidgetId>) -> UiInteraction {
        UiInteraction::from_input(self.id, self.rect, input, active)
    }
}
