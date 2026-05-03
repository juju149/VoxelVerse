use crate::{UiRect, UiTextAlign, UiTextOverflow, UiTextStyleId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTextVAlign {
    Top,
    Center,
    Bottom,
}

impl Default for UiTextVAlign {
    fn default() -> Self {
        Self::Top
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiTextLayout {
    pub align: UiTextAlign,
    pub vertical_align: UiTextVAlign,
    pub overflow: UiTextOverflow,
    pub style_id: Option<UiTextStyleId>,
}

impl UiTextLayout {
    pub const fn new(align: UiTextAlign, vertical_align: UiTextVAlign) -> Self {
        Self {
            align,
            vertical_align,
            overflow: UiTextOverflow::Clip,
            style_id: None,
        }
    }

    pub const fn centered() -> Self {
        Self::new(UiTextAlign::Center, UiTextVAlign::Center)
    }

    pub const fn left_centered() -> Self {
        Self::new(UiTextAlign::Left, UiTextVAlign::Center)
    }

    pub const fn right_centered() -> Self {
        Self::new(UiTextAlign::Right, UiTextVAlign::Center)
    }
}

impl Default for UiTextLayout {
    fn default() -> Self {
        Self {
            align: UiTextAlign::Left,
            vertical_align: UiTextVAlign::Top,
            overflow: UiTextOverflow::Clip,
            style_id: None,
        }
    }
}

pub fn vertical_text_rect(rect: UiRect, size: f32, vertical_align: UiTextVAlign) -> UiRect {
    let line_height = (size + 4.0).max(1.0);

    let y = match vertical_align {
        UiTextVAlign::Top => rect.y,
        UiTextVAlign::Center => rect.y + (rect.height - line_height) * 0.5 - size * 0.03,
        UiTextVAlign::Bottom => rect.bottom() - line_height,
    };

    UiRect::new(rect.x, y, rect.width, line_height)
}
