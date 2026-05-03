use vv_ui::UiColor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VvDesignTokens {
    pub colors: VvColorTokens,
    pub text: VvTextTokens,
    pub button: VvButtonTokens,
    pub inventory_tabs: VvInventoryTabTokens,
}

impl VvDesignTokens {
    pub fn current() -> Self {
        Self::default()
    }
}

impl Default for VvDesignTokens {
    fn default() -> Self {
        Self {
            colors: VvColorTokens::default(),
            text: VvTextTokens::default(),
            button: VvButtonTokens::default(),
            inventory_tabs: VvInventoryTabTokens::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VvColorTokens {
    pub button_border: UiColor,
    pub button_border_active: UiColor,
    pub button_top: UiColor,
    pub button_bottom: UiColor,
    pub button_active_top: UiColor,
    pub button_active_bottom: UiColor,
    pub button_disabled_top: UiColor,
    pub button_disabled_bottom: UiColor,
    pub tab_separator: UiColor,
    pub tab_shadow: UiColor,
}

impl Default for VvColorTokens {
    fn default() -> Self {
        Self {
            button_border: UiColor::rgba(0.52, 0.34, 0.16, 0.58),
            button_border_active: UiColor::rgba(0.94, 0.66, 0.25, 0.94),
            button_top: UiColor::rgba(0.030, 0.052, 0.058, 0.76),
            button_bottom: UiColor::rgba(0.006, 0.020, 0.024, 0.82),
            button_active_top: UiColor::rgba(0.78, 0.55, 0.22, 0.95),
            button_active_bottom: UiColor::rgba(0.39, 0.23, 0.07, 0.96),
            button_disabled_top: UiColor::rgba(0.020, 0.034, 0.038, 0.62),
            button_disabled_bottom: UiColor::rgba(0.006, 0.014, 0.018, 0.70),
            tab_separator: UiColor::rgba(0.70, 0.45, 0.18, 0.16),
            tab_shadow: UiColor::rgba(0.0, 0.0, 0.0, 0.24),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VvTextTokens {
    pub button_size: f32,
    pub button_size_large: f32,
    pub tab_size: f32,
}

impl Default for VvTextTokens {
    fn default() -> Self {
        Self {
            button_size: 13.0,
            button_size_large: 17.0,
            tab_size: 14.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VvButtonTokens {
    pub radius_factor: f32,
    pub radius_min: f32,
    pub radius_max: f32,
    pub border_width: f32,
    pub active_border_width: f32,
}

impl Default for VvButtonTokens {
    fn default() -> Self {
        Self {
            radius_factor: 0.22,
            radius_min: 6.0,
            radius_max: 11.0,
            border_width: 1.25,
            active_border_width: 1.65,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VvInventoryTabTokens {
    pub height: f32,
    pub gap: f32,
    pub padding_x: f32,
    pub radius_factor: f32,
    pub radius_min: f32,
    pub radius_max: f32,
    pub min_width: f32,
    pub max_width: f32,
}

impl Default for VvInventoryTabTokens {
    fn default() -> Self {
        Self {
            height: 42.0,
            gap: 8.0,
            padding_x: 20.0,
            radius_factor: 0.20,
            radius_min: 7.0,
            radius_max: 12.0,
            min_width: 82.0,
            max_width: 142.0,
        }
    }
}
