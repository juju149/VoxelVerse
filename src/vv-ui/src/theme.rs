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
            sm: 5.0,
            md: 9.0,
            lg: 14.0,
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
            background_dim: UiColor::rgba(0.0, 0.0, 0.0, 0.36),

            panel: UiColor::rgba(0.023529, 0.086275, 0.133333, 1.0),
            panel_hover: UiColor::rgba(0.034, 0.116, 0.172, 1.0),
            panel_active: UiColor::rgba(0.62, 0.39, 0.11, 0.96),
            panel_subtle: UiColor::rgba(0.018, 0.064, 0.096, 0.96),

            border: UiColor::rgba(0.74, 0.49, 0.18, 0.92),
            border_soft: UiColor::rgba(0.60, 0.38, 0.14, 0.80),

            text_primary: UiColor::rgba(0.94, 0.90, 0.82, 1.0),
            text_muted: UiColor::rgba(0.68, 0.70, 0.72, 1.0),
            text_disabled: UiColor::rgba(0.40, 0.42, 0.44, 1.0),

            accent: UiColor::rgba(1.0, 0.62, 0.17, 1.0),
            accent_hover: UiColor::rgba(1.0, 0.76, 0.36, 1.0),
            success: UiColor::rgba(0.42, 0.72, 0.30, 1.0),
            danger: UiColor::rgba(0.86, 0.22, 0.16, 1.0),
            warning: UiColor::rgba(0.94, 0.68, 0.18, 1.0),

            spacing: UiSpacingScale::default(),
            radius: UiRadiusScale::default(),
            typography: UiTypographyScale::default(),
        }
    }
}
