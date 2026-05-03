use crate::{UiFrame, UiLayer, UiPanelStyle, UiRect};

#[derive(Debug, Clone, Copy)]
pub struct UiPanel {
    pub rect: UiRect,
    pub style: UiPanelStyle,
    pub layer: UiLayer,
}

impl UiPanel {
    pub fn new(rect: UiRect, style: UiPanelStyle) -> Self {
        Self {
            rect,
            style,
            layer: UiLayer::Menu,
        }
    }

    pub fn layer(mut self, layer: UiLayer) -> Self {
        self.layer = layer;
        self
    }

    pub fn draw(self, frame: &mut UiFrame) {
        frame.rounded_rect(
            self.layer,
            self.rect,
            self.style.background,
            self.style.radius,
            self.style.border,
            self.style.shadow,
        );
    }
}
