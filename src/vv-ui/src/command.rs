use crate::{UiColor, UiGradient, UiRect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UiLayer {
    Background,
    SceneOverlay,
    Hud,
    Menu,
    Popup,
    Tooltip,
    Cursor,
}

impl Default for UiLayer {
    fn default() -> Self {
        Self::Hud
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiImageId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiIconId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiTextStyleId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiBorder {
    pub width: f32,
    pub color: UiColor,
}

impl UiBorder {
    pub const NONE: Self = Self {
        width: 0.0,
        color: UiColor::TRANSPARENT,
    };

    pub const fn new(width: f32, color: UiColor) -> Self {
        Self { width, color }
    }
}

impl Default for UiBorder {
    fn default() -> Self {
        Self::NONE
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: UiColor,
}

impl UiShadow {
    pub const NONE: Self = Self {
        offset_x: 0.0,
        offset_y: 0.0,
        blur: 0.0,
        spread: 0.0,
        color: UiColor::TRANSPARENT,
    };

    pub const fn new(offset_x: f32, offset_y: f32, blur: f32, spread: f32, color: UiColor) -> Self {
        Self {
            offset_x,
            offset_y,
            blur,
            spread,
            color,
        }
    }
}

impl Default for UiShadow {
    fn default() -> Self {
        Self::NONE
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTextAlign {
    Left,
    Center,
    Right,
}

impl Default for UiTextAlign {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTextOverflow {
    Clip,
    Ellipsis,
}

impl Default for UiTextOverflow {
    fn default() -> Self {
        Self::Clip
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiTextCommand {
    pub rect: UiRect,
    pub text: String,
    pub size: f32,
    pub color: UiColor,
    pub align: UiTextAlign,
    pub overflow: UiTextOverflow,
    pub style_id: Option<UiTextStyleId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiCommand {
    Rect {
        layer: UiLayer,
        rect: UiRect,
        color: UiColor,
        radius: f32,
        border: UiBorder,
        shadow: UiShadow,
    },
    GradientRect {
        layer: UiLayer,
        rect: UiRect,
        gradient: UiGradient,
        radius: f32,
        border: UiBorder,
        shadow: UiShadow,
    },
    Image {
        layer: UiLayer,
        rect: UiRect,
        image: UiImageId,
        tint: UiColor,
        radius: f32,
    },
    Icon {
        layer: UiLayer,
        rect: UiRect,
        icon: UiIconId,
        color: UiColor,
    },
    Text {
        layer: UiLayer,
        command: UiTextCommand,
    },
    ClipStart {
        layer: UiLayer,
        rect: UiRect,
    },
    ClipEnd {
        layer: UiLayer,
    },
}
