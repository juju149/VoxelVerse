use vv_ui::UiColor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VvInventoryUiTokens {
    pub design_width: f32,
    pub design_height: f32,
    pub scale_min: f32,
    pub scale_max: f32,
    pub layout: VvInventoryLayoutTokens,
    pub panel: VvInventoryPanelTokens,
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
pub struct VvInventoryColorTokens {
    pub screen_dim: UiColor,
    pub panel_fill: UiColor,
    pub panel_border: UiColor,
    pub panel_title: UiColor,
    pub text_primary: UiColor,
    pub text_muted: UiColor,
}

impl Default for VvInventoryColorTokens {
    fn default() -> Self {
        Self {
            screen_dim: UiColor::rgba(0.001, 0.006, 0.010, 0.52),

            // #061622
            panel_fill: UiColor::rgb(0.023529412, 0.08627451, 0.13333334),

            // #A66A18, avec alpha pour éviter le jaune fluo.
            panel_border: UiColor::rgba(0.6509804, 0.41568628, 0.09411765, 0.82),

            // #F2A51F
            panel_title: UiColor::rgb(0.9490196, 0.64705884, 0.12156863),

            // #F4E6CF
            text_primary: UiColor::rgb(0.95686275, 0.9019608, 0.8117647),

            // #BFAE93
            text_muted: UiColor::rgb(0.7490196, 0.68235296, 0.5764706),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VvInventoryTextTokens {
    pub panel_title_size: f32,
}

impl Default for VvInventoryTextTokens {
    fn default() -> Self {
        Self {
            panel_title_size: 20.0,
        }
    }
}
