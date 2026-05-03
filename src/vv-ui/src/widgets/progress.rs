use crate::{UiFrame, UiLayer, UiProgressStyle, UiRect};

#[derive(Debug, Clone, Copy)]
pub struct UiProgressBar {
    pub rect: UiRect,
    pub value: f32,
    pub style: UiProgressStyle,
    pub layer: UiLayer,
}

impl UiProgressBar {
    pub fn new(rect: UiRect, value: f32, style: UiProgressStyle) -> Self {
        Self {
            rect,
            value,
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
            crate::UiShadow::NONE,
        );

        let fill_width = self.rect.width * self.value.clamp(0.0, 1.0);
        if fill_width > 0.0 {
            frame.rounded_rect(
                self.layer,
                UiRect::new(self.rect.x, self.rect.y, fill_width, self.rect.height),
                self.style.fill,
                self.style.radius,
                crate::UiBorder::NONE,
                crate::UiShadow::NONE,
            );
        }
    }
}
