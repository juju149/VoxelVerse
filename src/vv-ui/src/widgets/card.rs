use crate::{UiCardStyle, UiFrame, UiInput, UiLayer, UiRect, UiResponse, UiSurface, UiWidgetId};

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

    pub fn draw(self, frame: &mut UiFrame, input: &UiInput) -> UiResponse {
        let response = UiResponse::from_input(self.id, self.rect, input, None, false);

        let background = if response.hovered {
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

        response
    }
}
