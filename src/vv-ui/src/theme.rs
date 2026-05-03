use crate::UiColor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiSpacingScale {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

impl Default for UiSpacingScale {
    fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 18.0,
            xl: 28.0,
            xxl: 40.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiRadiusScale {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub pill: f32,
}

impl Default for UiRadiusScale {
    fn default() -> Self {
        Self {
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            xl: 18.0,
            pill: 999.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiTypographyScale {
    pub tiny: f32,
    pub small: f32,
    pub body: f32,
    pub heading: f32,
    pub title: f32,
    pub hero: f32,
}

impl Default for UiTypographyScale {
    fn default() -> Self {
        Self {
            tiny: 11.0,
            small: 13.0,
            body: 16.0,
            heading: 22.0,
            title: 34.0,
            hero: 52.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiTheme {
    pub background_dim: UiColor,
    pub panel: UiColor,
    pub panel_hover: UiColor,
    pub panel_active: UiColor,
    pub panel_subtle: UiColor,
    pub border: UiColor,
    pub border_soft: UiColor,
    pub text_primary: UiColor,
    pub text_muted: UiColor,
    pub text_disabled: UiColor,
    pub accent: UiColor,
    pub accent_hover: UiColor,
    pub success: UiColor,
    pub danger: UiColor,
    pub warning: UiColor,
    pub spacing: UiSpacingScale,
    pub radius: UiRadiusScale,
    pub typography: UiTypographyScale,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            background_dim: UiColor::rgba(0.015, 0.025, 0.035, 0.76),
            panel: UiColor::rgba(0.035, 0.055, 0.070, 0.82),
            panel_hover: UiColor::rgba(0.065, 0.090, 0.110, 0.88),
            panel_active: UiColor::rgba(0.76, 0.48, 0.14, 0.92),
            panel_subtle: UiColor::rgba(0.020, 0.030, 0.040, 0.55),
            border: UiColor::rgba(0.82, 0.55, 0.20, 0.76),
            border_soft: UiColor::rgba(0.82, 0.55, 0.20, 0.28),
            text_primary: UiColor::rgba(0.94, 0.91, 0.84, 1.0),
            text_muted: UiColor::rgba(0.62, 0.66, 0.70, 1.0),
            text_disabled: UiColor::rgba(0.38, 0.40, 0.42, 1.0),
            accent: UiColor::rgba(0.96, 0.62, 0.18, 1.0),
            accent_hover: UiColor::rgba(1.0, 0.72, 0.28, 1.0),
            success: UiColor::rgba(0.42, 0.72, 0.30, 1.0),
            danger: UiColor::rgba(0.86, 0.22, 0.16, 1.0),
            warning: UiColor::rgba(0.94, 0.68, 0.18, 1.0),
            spacing: UiSpacingScale::default(),
            radius: UiRadiusScale::default(),
            typography: UiTypographyScale::default(),
        }
    }
}
