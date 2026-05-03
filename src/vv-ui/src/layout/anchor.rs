use crate::{UiEdgeInsets, UiRect, UiSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiAnchorLayout {
    pub bounds: UiRect,
    pub margin: UiEdgeInsets,
}

impl UiAnchorLayout {
    pub fn new(bounds: UiRect) -> Self {
        Self {
            bounds,
            margin: UiEdgeInsets::ZERO,
        }
    }

    pub fn with_margin(mut self, margin: UiEdgeInsets) -> Self {
        self.margin = margin;
        self
    }

    pub fn place(self, anchor: UiAnchor, size: UiSize) -> UiRect {
        let bounds = self.bounds.inset(self.margin);

        let x = match anchor {
            UiAnchor::TopLeft | UiAnchor::CenterLeft | UiAnchor::BottomLeft => bounds.left(),
            UiAnchor::TopCenter | UiAnchor::Center | UiAnchor::BottomCenter => {
                bounds.left() + (bounds.width - size.width) * 0.5
            }
            UiAnchor::TopRight | UiAnchor::CenterRight | UiAnchor::BottomRight => {
                bounds.right() - size.width
            }
        };

        let y = match anchor {
            UiAnchor::TopLeft | UiAnchor::TopCenter | UiAnchor::TopRight => bounds.top(),
            UiAnchor::CenterLeft | UiAnchor::Center | UiAnchor::CenterRight => {
                bounds.top() + (bounds.height - size.height) * 0.5
            }
            UiAnchor::BottomLeft | UiAnchor::BottomCenter | UiAnchor::BottomRight => {
                bounds.bottom() - size.height
            }
        };

        UiRect::new(x, y, size.width, size.height)
    }
}
