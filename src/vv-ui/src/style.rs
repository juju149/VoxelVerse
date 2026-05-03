use crate::{UiBorder, UiColor, UiGradient, UiShadow, UiTheme};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiPanelStyle {
    pub background: UiColor,
    pub border: UiBorder,
    pub radius: f32,
    pub shadow: UiShadow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiButtonStyle {
    pub background: UiColor,
    pub background_hover: UiColor,
    pub background_pressed: UiColor,
    pub background_gradient: Option<UiGradient>,
    pub background_hover_gradient: Option<UiGradient>,
    pub background_pressed_gradient: Option<UiGradient>,
    pub text: UiColor,
    pub text_disabled: UiColor,
    pub border: UiBorder,
    pub radius: f32,
    pub shadow: UiShadow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiCardStyle {
    pub background: UiColor,
    pub background_hover: UiColor,
    pub border: UiBorder,
    pub radius: f32,
    pub shadow: UiShadow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiSlotStyle {
    pub background: UiColor,
    pub background_hover: UiColor,
    pub background_selected: UiColor,
    pub border: UiBorder,
    pub selected_border: UiBorder,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiToggleStyle {
    pub track_off: UiColor,
    pub track_on: UiColor,
    pub thumb: UiColor,
    pub border: UiBorder,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiSliderStyle {
    pub track: UiColor,
    pub fill: UiColor,
    pub thumb: UiColor,
    pub border: UiBorder,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiDropdownStyle {
    pub background: UiColor,
    pub background_hover: UiColor,
    pub border: UiBorder,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiTabsStyle {
    pub background: UiColor,
    pub active_background: UiColor,
    pub text: UiColor,
    pub active_text: UiColor,
    pub border: UiBorder,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiSearchStyle {
    pub background: UiColor,
    pub border: UiBorder,
    pub radius: f32,
    pub text: UiColor,
    pub placeholder: UiColor,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiProgressStyle {
    pub background: UiColor,
    pub fill: UiColor,
    pub border: UiBorder,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiStyle {
    pub panel: UiPanelStyle,
    pub glass_panel: UiPanelStyle,
    pub button: UiButtonStyle,
    pub primary_button: UiButtonStyle,
    pub card: UiCardStyle,
    pub slot: UiSlotStyle,
    pub toggle: UiToggleStyle,
    pub slider: UiSliderStyle,
    pub dropdown: UiDropdownStyle,
    pub tabs: UiTabsStyle,
    pub search: UiSearchStyle,
    pub progress: UiProgressStyle,
}

impl UiStyle {
    pub fn from_theme(theme: &UiTheme) -> Self {
        let panel_shadow = UiShadow::new(0.0, 18.0, 38.0, 0.0, UiColor::rgba(0.0, 0.0, 0.0, 0.34));

        Self {
            panel: UiPanelStyle {
                background: theme.panel,
                border: UiBorder::new(1.0, theme.border_soft),
                radius: theme.radius.lg,
                shadow: panel_shadow,
            },
            glass_panel: UiPanelStyle {
                background: theme.panel,
                border: UiBorder::new(1.15, theme.border.with_alpha(0.62)),
                radius: theme.radius.lg,
                shadow: panel_shadow,
            },
            button: UiButtonStyle {
                background: theme.panel_subtle,
                background_hover: theme.panel_hover,
                background_pressed: theme.panel_active.multiply_alpha(0.50),
                background_gradient: Some(UiGradient::vertical(
                    UiColor::rgba(0.022, 0.038, 0.044, 0.94),
                    UiColor::rgba(0.004, 0.014, 0.018, 0.98),
                )),
                background_hover_gradient: Some(UiGradient::vertical(
                    UiColor::rgba(0.036, 0.058, 0.068, 0.98),
                    UiColor::rgba(0.006, 0.020, 0.026, 1.0),
                )),
                background_pressed_gradient: Some(UiGradient::vertical(
                    UiColor::rgba(0.34, 0.22, 0.075, 0.98),
                    UiColor::rgba(0.12, 0.070, 0.020, 1.0),
                )),
                text: theme.text_muted,
                text_disabled: theme.text_disabled,
                border: UiBorder::new(1.15, theme.border_soft.with_alpha(0.85)),
                radius: theme.radius.md,
                shadow: UiShadow::NONE,
            },
            primary_button: UiButtonStyle {
                background: theme.panel_active,
                background_hover: theme.accent_hover.with_alpha(0.92),
                background_pressed: theme.accent.darken(0.18),
                background_gradient: Some(UiGradient::vertical(
                    UiColor::rgba(0.92, 0.64, 0.28, 0.98),
                    UiColor::rgba(0.45, 0.27, 0.070, 1.0),
                )),
                background_hover_gradient: Some(UiGradient::vertical(
                    UiColor::rgba(1.0, 0.74, 0.34, 1.0),
                    UiColor::rgba(0.52, 0.31, 0.08, 1.0),
                )),
                background_pressed_gradient: Some(UiGradient::vertical(
                    UiColor::rgba(0.70, 0.45, 0.16, 1.0),
                    UiColor::rgba(0.31, 0.17, 0.04, 1.0),
                )),
                text: theme.text_primary,
                text_disabled: theme.text_disabled,
                border: UiBorder::new(1.35, theme.border.with_alpha(0.90)),
                radius: theme.radius.md,
                shadow: UiShadow::new(0.0, 0.0, 14.0, 0.0, theme.accent.multiply_alpha(0.18)),
            },
            card: UiCardStyle {
                background: theme.panel_subtle,
                background_hover: theme.panel_hover,
                border: UiBorder::new(1.0, theme.border_soft),
                radius: theme.radius.md,
                shadow: UiShadow::NONE,
            },
            slot: UiSlotStyle {
                background: UiColor::rgba(0.002, 0.017, 0.020, 0.99),
                background_hover: UiColor::rgba(0.012, 0.036, 0.042, 1.0),
                background_selected: UiColor::rgba(0.020, 0.045, 0.052, 1.0),
                border: UiBorder::new(2.0, UiColor::rgba(0.62, 0.39, 0.16, 0.78)),
                selected_border: UiBorder::new(2.35, theme.accent.with_alpha(0.92)),
                radius: theme.radius.md,
            },
            toggle: UiToggleStyle {
                track_off: UiColor::rgba(0.16, 0.17, 0.18, 0.95),
                track_on: theme.success,
                thumb: theme.text_primary,
                border: UiBorder::new(1.0, theme.border_soft),
                radius: theme.radius.pill,
            },
            slider: UiSliderStyle {
                track: UiColor::rgba(0.004, 0.014, 0.018, 0.96),
                fill: theme.accent_hover,
                thumb: theme.accent_hover,
                border: UiBorder::NONE,
                radius: theme.radius.pill,
            },
            dropdown: UiDropdownStyle {
                background: theme.panel_subtle,
                background_hover: theme.panel_hover,
                border: UiBorder::new(1.0, theme.border_soft),
                radius: theme.radius.md,
            },
            tabs: UiTabsStyle {
                background: UiColor::rgba(0.006, 0.018, 0.024, 0.64),
                active_background: theme.panel_active,
                text: theme.text_muted,
                active_text: theme.text_primary,
                border: UiBorder::new(1.0, theme.border_soft.with_alpha(0.55)),
                radius: theme.radius.md,
            },
            search: UiSearchStyle {
                background: UiColor::rgba(0.002, 0.012, 0.016, 0.98),
                border: UiBorder::new(1.2, theme.border_soft.with_alpha(0.86)),
                radius: theme.radius.md,
                text: theme.text_primary,
                placeholder: theme.text_muted,
            },
            progress: UiProgressStyle {
                background: UiColor::rgba(0.002, 0.012, 0.016, 0.98),
                fill: theme.accent_hover,
                border: UiBorder::new(1.0, theme.border_soft.with_alpha(0.55)),
                radius: theme.radius.pill,
            },
        }
    }
}
