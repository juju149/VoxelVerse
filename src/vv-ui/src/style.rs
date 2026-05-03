use crate::{UiBorder, UiColor, UiShadow, UiTheme};

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
        let soft_shadow = UiShadow::new(0.0, 12.0, 24.0, 0.0, UiColor::rgba(0.0, 0.0, 0.0, 0.35));

        Self {
            panel: UiPanelStyle {
                background: theme.panel,
                border: UiBorder::new(1.0, theme.border_soft),
                radius: theme.radius.lg,
                shadow: soft_shadow,
            },
            glass_panel: UiPanelStyle {
                background: theme.panel.multiply_alpha(0.82),
                border: UiBorder::new(1.0, theme.border_soft),
                radius: theme.radius.lg,
                shadow: soft_shadow,
            },
            button: UiButtonStyle {
                background: theme.panel,
                background_hover: theme.panel_hover,
                background_pressed: theme.panel_active.darken(0.12),
                text: theme.text_primary,
                text_disabled: theme.text_disabled,
                border: UiBorder::new(1.0, theme.border_soft),
                radius: theme.radius.md,
                shadow: UiShadow::NONE,
            },
            primary_button: UiButtonStyle {
                background: theme.panel_active,
                background_hover: theme.accent_hover.with_alpha(0.92),
                background_pressed: theme.accent.darken(0.18),
                text: theme.text_primary,
                text_disabled: theme.text_disabled,
                border: UiBorder::new(1.0, theme.border),
                radius: theme.radius.md,
                shadow: UiShadow::new(0.0, 0.0, 18.0, 0.0, theme.accent.multiply_alpha(0.32)),
            },
            card: UiCardStyle {
                background: theme.panel_subtle,
                background_hover: theme.panel_hover,
                border: UiBorder::new(1.0, theme.border_soft),
                radius: theme.radius.md,
                shadow: UiShadow::NONE,
            },
            slot: UiSlotStyle {
                background: theme.panel_subtle,
                background_hover: theme.panel_hover,
                background_selected: theme.panel_active.multiply_alpha(0.45),
                border: UiBorder::new(1.0, theme.border_soft),
                selected_border: UiBorder::new(2.0, theme.accent),
                radius: theme.radius.sm,
            },
            toggle: UiToggleStyle {
                track_off: UiColor::rgba(0.16, 0.17, 0.18, 0.95),
                track_on: theme.success,
                thumb: theme.text_primary,
                border: UiBorder::new(1.0, theme.border_soft),
                radius: theme.radius.pill,
            },
            slider: UiSliderStyle {
                track: UiColor::rgba(0.18, 0.18, 0.16, 0.8),
                fill: theme.accent,
                thumb: theme.accent_hover,
                border: UiBorder::NONE,
                radius: theme.radius.pill,
            },
            dropdown: UiDropdownStyle {
                background: theme.panel_subtle,
                background_hover: theme.panel_hover,
                border: UiBorder::new(1.0, theme.border_soft),
                radius: theme.radius.sm,
            },
            tabs: UiTabsStyle {
                background: theme.panel_subtle,
                active_background: theme.panel_active,
                text: theme.text_muted,
                active_text: theme.text_primary,
                border: UiBorder::new(1.0, theme.border_soft),
                radius: theme.radius.md,
            },
            search: UiSearchStyle {
                background: theme.panel_subtle,
                border: UiBorder::new(1.0, theme.border_soft),
                radius: theme.radius.sm,
                text: theme.text_primary,
                placeholder: theme.text_muted,
            },
            progress: UiProgressStyle {
                background: theme.panel_subtle,
                fill: theme.accent,
                border: UiBorder::new(1.0, theme.border_soft),
                radius: theme.radius.pill,
            },
        }
    }
}
