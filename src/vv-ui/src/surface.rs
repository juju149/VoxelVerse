use crate::{UiBorder, UiColor, UiEdgeInsets, UiFrame, UiLayer, UiRect, UiShadow};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiSurface {
    pub fill: UiColor,
    pub border: UiColor,
    pub border_width: f32,
    pub radius: f32,
    pub shadow: UiShadow,
}

impl UiSurface {
    pub const fn new(fill: UiColor) -> Self {
        Self {
            fill,
            border: UiColor::TRANSPARENT,
            border_width: 0.0,
            radius: 0.0,
            shadow: UiShadow::NONE,
        }
    }

    pub const fn border(mut self, color: UiColor, width: f32) -> Self {
        self.border = color;
        self.border_width = width;
        self
    }

    pub const fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub const fn shadow(mut self, shadow: UiShadow) -> Self {
        self.shadow = shadow;
        self
    }

    pub fn draw(self, frame: &mut UiFrame, layer: UiLayer, rect: UiRect) {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }

        let has_border = self.border_width > 0.0 && self.border.a > 0.001;

        if has_border {
            frame.rounded_rect(
                layer,
                rect,
                self.border,
                self.radius,
                UiBorder::NONE,
                self.shadow,
            );

            let inner = rect.inset(UiEdgeInsets::all(self.border_width));

            if inner.width > 0.0 && inner.height > 0.0 {
                frame.rounded_rect(
                    layer,
                    inner,
                    self.fill,
                    (self.radius - self.border_width).max(0.0),
                    UiBorder::NONE,
                    UiShadow::NONE,
                );
            }
        } else {
            frame.rounded_rect(
                layer,
                rect,
                self.fill,
                self.radius,
                UiBorder::NONE,
                self.shadow,
            );
        }
    }
}
