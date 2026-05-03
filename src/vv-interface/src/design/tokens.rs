use vv_ui::{UiColor, UiSurface};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InventoryUiTokens {
    pub colors: InventoryColorTokens,
    pub radius: InventoryRadiusTokens,
    pub stroke: InventoryStrokeTokens,
    pub layout: InventoryLayoutTokens,
    pub grid: InventoryGridTokens,
    pub text: InventoryTextTokens,
}

impl InventoryUiTokens {
    pub fn current() -> Self {
        Self::default()
    }

    pub fn panel_surface(self) -> UiSurface {
        UiSurface::new(self.colors.panel_fill)
            .border(self.colors.panel_border, self.stroke.panel)
            .radius(self.radius.panel)
    }

    pub fn input_surface(self) -> UiSurface {
        UiSurface::new(self.colors.input_fill)
            .border(self.colors.input_border, self.stroke.control)
            .radius(self.radius.input)
    }

    pub fn button_surface(self) -> UiSurface {
        UiSurface::new(self.colors.control_fill)
            .border(self.colors.control_border, self.stroke.control)
            .radius(self.radius.control)
    }

    pub fn tab_surface(self, active: bool) -> UiSurface {
        if active {
            UiSurface::new(self.colors.tab_active_fill)
                .border(self.colors.tab_active_border, self.stroke.control_active)
                .radius(self.radius.tab)
        } else {
            UiSurface::new(self.colors.tab_fill)
                .border(self.colors.tab_border, self.stroke.control)
                .radius(self.radius.tab)
        }
    }

    pub fn slot_surface(self) -> UiSurface {
        UiSurface::new(self.colors.slot_fill)
            .border(self.colors.slot_border, self.stroke.slot)
            .radius(self.radius.slot)
    }
}

impl Default for InventoryUiTokens {
    fn default() -> Self {
        Self {
            colors: InventoryColorTokens::default(),
            radius: InventoryRadiusTokens::default(),
            stroke: InventoryStrokeTokens::default(),
            layout: InventoryLayoutTokens::default(),
            grid: InventoryGridTokens::default(),
            text: InventoryTextTokens::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InventoryColorTokens {
    pub screen_dim: UiColor,

    pub panel_fill: UiColor,
    pub panel_border: UiColor,

    pub input_fill: UiColor,
    pub input_border: UiColor,

    pub control_fill: UiColor,
    pub control_border: UiColor,

    pub tab_fill: UiColor,
    pub tab_border: UiColor,
    pub tab_active_fill: UiColor,
    pub tab_active_border: UiColor,

    pub slot_fill: UiColor,
    pub slot_border: UiColor,

    pub title: UiColor,
    pub text_primary: UiColor,
    pub text_secondary: UiColor,
    pub text_active: UiColor,
}

impl Default for InventoryColorTokens {
    fn default() -> Self {
        Self {
            screen_dim: UiColor::rgba(0.0, 0.0, 0.0, 0.36),

            // Exact #061622.
            panel_fill: UiColor::rgba(0.023529, 0.086275, 0.133333, 1.0),
            panel_border: UiColor::rgba(0.74, 0.49, 0.18, 0.92),

            input_fill: UiColor::rgba(0.010, 0.035, 0.052, 0.96),
            input_border: UiColor::rgba(0.72, 0.47, 0.17, 0.84),

            control_fill: UiColor::rgba(0.018, 0.056, 0.078, 0.94),
            control_border: UiColor::rgba(0.62, 0.39, 0.14, 0.72),

            tab_fill: UiColor::rgba(0.018, 0.056, 0.078, 0.72),
            tab_border: UiColor::rgba(0.48, 0.31, 0.12, 0.42),
            tab_active_fill: UiColor::rgba(0.62, 0.39, 0.11, 0.96),
            tab_active_border: UiColor::rgba(0.93, 0.65, 0.25, 0.96),

            slot_fill: UiColor::rgba(0.010, 0.036, 0.052, 0.98),
            slot_border: UiColor::rgba(0.54, 0.34, 0.12, 0.82),

            title: UiColor::rgba(0.95, 0.62, 0.16, 1.0),
            text_primary: UiColor::rgba(0.94, 0.88, 0.76, 1.0),
            text_secondary: UiColor::rgba(0.70, 0.68, 0.62, 1.0),
            text_active: UiColor::rgba(1.0, 0.92, 0.78, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InventoryRadiusTokens {
    pub panel: f32,
    pub input: f32,
    pub control: f32,
    pub tab: f32,
    pub slot: f32,
}

impl Default for InventoryRadiusTokens {
    fn default() -> Self {
        Self {
            panel: 9.0,
            input: 7.0,
            control: 7.0,
            tab: 7.0,
            slot: 7.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InventoryStrokeTokens {
    pub panel: f32,
    pub control: f32,
    pub control_active: f32,
    pub slot: f32,
}

impl Default for InventoryStrokeTokens {
    fn default() -> Self {
        Self {
            panel: 1.5,
            control: 1.25,
            control_active: 1.5,
            slot: 1.25,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InventoryLayoutTokens {
    pub design_width: f32,
    pub design_height: f32,
    pub scale_min: f32,
    pub scale_max: f32,

    pub outer_margin: f32,
    pub panel_gap: f32,
    pub panel_height_ratio: f32,

    pub equipment_width_ratio: f32,
    pub backpack_width_ratio: f32,
    pub crafting_width_ratio: f32,

    pub panel_padding: f32,
    pub title_top: f32,

    pub search_top: f32,
    pub search_height: f32,
    pub sort_button_width: f32,
    pub control_gap: f32,

    pub tabs_top: f32,
    pub tab_height: f32,
    pub tab_gap: f32,
}

impl Default for InventoryLayoutTokens {
    fn default() -> Self {
        Self {
            design_width: 2048.0,
            design_height: 1152.0,
            scale_min: 0.62,
            scale_max: 1.42,

            outer_margin: 28.0,
            panel_gap: 22.0,
            panel_height_ratio: 0.70,

            equipment_width_ratio: 0.30,
            backpack_width_ratio: 0.40,
            crafting_width_ratio: 0.30,

            panel_padding: 26.0,
            title_top: 24.0,

            search_top: 78.0,
            search_height: 54.0,
            sort_button_width: 150.0,
            control_gap: 22.0,

            tabs_top: 154.0,
            tab_height: 46.0,
            tab_gap: 10.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InventoryGridTokens {
    pub columns: usize,
    pub rows: usize,
    pub top: f32,
    pub gap: f32,
    pub bottom_reserved: f32,
    pub slot_min: f32,
    pub slot_max: f32,
}

impl Default for InventoryGridTokens {
    fn default() -> Self {
        Self {
            columns: 8,
            rows: 6,
            top: 226.0,
            gap: 11.0,
            bottom_reserved: 124.0,
            slot_min: 42.0,
            slot_max: 78.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InventoryTextTokens {
    pub panel_title: f32,
    pub body: f32,
    pub button: f32,
    pub tab: f32,
}

impl Default for InventoryTextTokens {
    fn default() -> Self {
        Self {
            panel_title: 24.0,
            body: 16.0,
            button: 17.0,
            tab: 15.0,
        }
    }
}
