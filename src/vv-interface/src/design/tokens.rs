use vv_ui::UiColor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VvInventoryUiTokens {
    pub design_width: f32,
    pub design_height: f32,
    pub scale_min: f32,
    pub scale_max: f32,
    pub layout: VvInventoryLayoutTokens,
    pub panel: VvInventoryPanelTokens,
    pub controls: VvInventoryControlTokens,
    pub colors: VvInventoryColorTokens,
    pub text: VvInventoryTextTokens,
}

impl VvInventoryUiTokens {
    pub fn current() -> Self {
        Self::default()
    }
}

impl Default for VvInventoryUiTokens {
    fn default() -> Self {
        Self {
            design_width: 2048.0,
            design_height: 1152.0,
            scale_min: 0.72,
            scale_max: 1.35,
            layout: VvInventoryLayoutTokens::default(),
            panel: VvInventoryPanelTokens::default(),
            controls: VvInventoryControlTokens::default(),
            colors: VvInventoryColorTokens::default(),
            text: VvInventoryTextTokens::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VvInventoryLayoutTokens {
    pub outer_margin: f32,
    pub panel_gap: f32,
    pub equipment_width_ratio: f32,
    pub backpack_width_ratio: f32,
    pub crafting_width_ratio: f32,
    pub panel_height_ratio: f32,
}

impl Default for VvInventoryLayoutTokens {
    fn default() -> Self {
        Self {
            outer_margin: 30.0,
            panel_gap: 24.0,
            equipment_width_ratio: 0.30,
            backpack_width_ratio: 0.40,
            crafting_width_ratio: 0.30,
            panel_height_ratio: 0.70,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VvInventoryPanelTokens {
    pub radius: f32,
    pub border_width: f32,
    pub padding_x: f32,
    pub title_top: f32,
}

impl Default for VvInventoryPanelTokens {
    fn default() -> Self {
        Self {
            radius: 10.0,
            border_width: 1.5,
            padding_x: 24.0,
            title_top: 24.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VvInventoryControlTokens {
    pub search_top: f32,
    pub control_height: f32,
    pub control_radius: f32,
    pub control_border_width: f32,
    pub search_padding_x: f32,
    pub search_sort_gap: f32,
    pub sort_button_width: f32,
}

impl Default for VvInventoryControlTokens {
    fn default() -> Self {
        Self {
            search_top: 76.0,
            control_height: 46.0,
            control_radius: 8.0,
            control_border_width: 1.25,
            search_padding_x: 16.0,
            search_sort_gap: 16.0,
            sort_button_width: 138.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VvInventoryColorTokens {
    pub screen_dim: UiColor,
    pub panel_fill: UiColor,
    pub panel_border: UiColor,
    pub panel_title: UiColor,
    pub text_primary: UiColor,
    pub text_muted: UiColor,

    pub control_fill: UiColor,
    pub control_fill_hoverless: UiColor,
    pub control_border: UiColor,
    pub control_text: UiColor,
    pub control_placeholder: UiColor,
}

impl Default for VvInventoryColorTokens {
    fn default() -> Self {
        Self {
            screen_dim: UiColor::rgba(0.001, 0.006, 0.010, 0.52),

            // #061622
            panel_fill: UiColor::rgb(0.02, 0.09, 0.13),

            // #A66A18
            panel_border: UiColor::rgba(0.65, 0.42, 0.09, 0.82),

            // #F2A51F
            panel_title: UiColor::rgb(0.95, 0.65, 0.12),

            // #F4E6CF
            text_primary: UiColor::rgb(0.96, 0.90, 0.81),

            // #BFAE93
            text_muted: UiColor::rgb(0.75, 0.68, 0.58),

            // Très sombre, proche de la référence.
            control_fill: UiColor::rgba(0.01, 0.04, 0.05, 0.92),
            control_fill_hoverless: UiColor::rgba(0.02, 0.05, 0.06, 0.86),
            control_border: UiColor::rgba(0.6509804, 0.41568628, 0.09411765, 0.62),
            control_text: UiColor::rgb(0.95686275, 0.9019608, 0.8117647),
            control_placeholder: UiColor::rgb(0.7490196, 0.68235296, 0.5764706),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VvInventoryTextTokens {
    pub panel_title_size: f32,
    pub control_text_size: f32,
}

impl Default for VvInventoryTextTokens {
    fn default() -> Self {
        Self {
            panel_title_size: 20.0,
            control_text_size: 15.0,
        }
    }
}
