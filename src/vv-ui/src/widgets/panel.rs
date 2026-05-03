use crate::{UiFrame, UiLayer, UiPanelStyle, UiRect, UiSurface};

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
        frame.surface(
            self.layer,
            self.rect,
            UiSurface::new(self.style.background)
                .border(self.style.border.color, self.style.border.width)
                .radius(self.style.radius)
                .shadow(self.style.shadow),
        );
    }
}
