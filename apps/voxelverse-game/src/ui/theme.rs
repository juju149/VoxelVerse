#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiColor {
    pub rgb: [f32; 3],
    pub alpha: f32,
}

impl UiColor {
    pub const fn rgba(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        Self {
            rgb: [r, g, b],
            alpha,
        }
    }

    pub const fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self {
            rgb: [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0],
            alpha: 1.0,
        }
    }

    pub fn scale_rgb(self, factor: f32) -> Self {
        Self {
            rgb: [
                (self.rgb[0] * factor).min(1.0),
                (self.rgb[1] * factor).min(1.0),
                (self.rgb[2] * factor).min(1.0),
            ],
            alpha: self.alpha,
        }
    }

    pub fn as_rgb(self) -> [f32; 3] {
        self.rgb
    }

    pub fn as_rgb8(self) -> [u8; 3] {
        [
            (self.rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ComponentState {
    Normal,
    Hovered,
    Selected,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TextRole {
    Title,
    Section,
    Body,
    Muted,
    Badge,
    Notice,
    Control,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiSpacing {
    pub tiny: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    pub panel: f32,
    pub slot_gap: f32,
    pub slot_size: f32,
    pub hotbar_bottom_margin_min: f32,
    pub hotbar_bottom_margin_max: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelStyle {
    pub fill: UiColor,
    pub border: UiColor,
    pub border_strong: UiColor,
    pub shadow: UiColor,
    pub radius: f32,
    pub border_width: f32,
    pub padding: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SlotStyle {
    pub fill: UiColor,
    pub fill_hovered: UiColor,
    pub fill_selected: UiColor,
    pub inner_fill: UiColor,
    pub border: UiColor,
    pub border_hovered: UiColor,
    pub border_selected: UiColor,
    pub disabled_fill: UiColor,
    pub disabled_border: UiColor,
    pub highlight: UiColor,
    pub content_glint: UiColor,
    pub border_width: f32,
    pub selected_border_width: f32,
    pub inner_inset: f32,
    pub icon_inset: f32,
}

impl SlotStyle {
    pub fn fill_for(self, state: ComponentState) -> UiColor {
        match state {
            ComponentState::Normal => self.fill,
            ComponentState::Hovered => self.fill_hovered,
            ComponentState::Selected => self.fill_selected,
            ComponentState::Disabled => self.disabled_fill,
        }
    }

    pub fn border_for(self, state: ComponentState) -> UiColor {
        match state {
            ComponentState::Normal => self.border,
            ComponentState::Hovered => self.border_hovered,
            ComponentState::Selected => self.border_selected,
            ComponentState::Disabled => self.disabled_border,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ButtonStyle {
    pub fill: UiColor,
    pub fill_hovered: UiColor,
    pub fill_selected: UiColor,
    pub fill_disabled: UiColor,
    pub border: UiColor,
    pub text: UiColor,
    pub text_disabled: UiColor,
    pub height: f32,
    pub padding_x: f32,
    pub border_width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SearchBarStyle {
    pub fill: UiColor,
    pub border: UiColor,
    pub placeholder: UiColor,
    pub text: UiColor,
    pub icon: UiColor,
    pub height: f32,
    pub padding_x: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FilterChipStyle {
    pub fill: UiColor,
    pub fill_selected: UiColor,
    pub border: UiColor,
    pub border_selected: UiColor,
    pub text: UiColor,
    pub text_selected: UiColor,
    pub height: f32,
    pub padding_x: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InventoryGridStyle {
    pub columns: usize,
    pub slot_size: f32,
    pub gap: f32,
    pub row_gap: f32,
    pub empty_slot_fill: UiColor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HotbarStyle {
    pub slots: usize,
    pub slot_size_min: f32,
    pub slot_size_max: f32,
    pub slot_height_ratio: f32,
    pub panel_padding: f32,
    pub slot_gap: f32,
    pub notice_offset_y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct QuantityBadgeStyle {
    pub text: UiColor,
    pub notice: UiColor,
    pub shadow: UiColor,
    pub font_size: f32,
    pub line_height: f32,
    pub right_inset: f32,
    pub bottom_inset: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TextStyle {
    pub title: UiColor,
    pub section: UiColor,
    pub body: UiColor,
    pub muted: UiColor,
    pub badge: UiColor,
    pub notice: UiColor,
    pub control: UiColor,
    pub title_size: f32,
    pub section_size: f32,
    pub body_size: f32,
    pub badge_size: f32,
}

impl TextStyle {
    pub fn color_for(self, role: TextRole) -> UiColor {
        match role {
            TextRole::Title => self.title,
            TextRole::Section => self.section,
            TextRole::Body => self.body,
            TextRole::Muted => self.muted,
            TextRole::Badge => self.badge,
            TextRole::Notice => self.notice,
            TextRole::Control => self.control,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiTheme {
    pub name: &'static str,
    pub spacing: UiSpacing,
    pub panel: PanelStyle,
    pub slot: SlotStyle,
    pub button: ButtonStyle,
    pub search_bar: SearchBarStyle,
    pub filter_chip: FilterChipStyle,
    pub inventory_grid: InventoryGridStyle,
    pub hotbar: HotbarStyle,
    pub quantity_badge: QuantityBadgeStyle,
    pub text: TextStyle,
}

impl UiTheme {
    pub const VOXELVERSE: Self = Self {
        name: "voxelverse_adventure",
        spacing: UiSpacing {
            tiny: 4.0,
            small: 8.0,
            medium: 12.0,
            large: 18.0,
            panel: 24.0,
            slot_gap: 7.0,
            slot_size: 56.0,
            hotbar_bottom_margin_min: 24.0,
            hotbar_bottom_margin_max: 42.0,
        },
        panel: PanelStyle {
            fill: UiColor::rgba(0.030, 0.045, 0.048, 0.86),
            border: UiColor::rgba(0.72, 0.45, 0.17, 0.82),
            border_strong: UiColor::rgba(1.00, 0.66, 0.22, 0.95),
            shadow: UiColor::rgba(0.0, 0.0, 0.0, 0.58),
            radius: 8.0,
            border_width: 1.0,
            padding: 18.0,
        },
        slot: SlotStyle {
            fill: UiColor::rgba(0.105, 0.085, 0.055, 0.86),
            fill_hovered: UiColor::rgba(0.18, 0.13, 0.070, 0.91),
            fill_selected: UiColor::rgba(0.24, 0.15, 0.045, 0.94),
            inner_fill: UiColor::rgba(0.040, 0.057, 0.055, 0.84),
            border: UiColor::rgba(0.55, 0.34, 0.13, 0.86),
            border_hovered: UiColor::rgba(0.86, 0.55, 0.19, 0.95),
            border_selected: UiColor::rgba(1.00, 0.68, 0.26, 1.0),
            disabled_fill: UiColor::rgba(0.035, 0.040, 0.040, 0.60),
            disabled_border: UiColor::rgba(0.18, 0.16, 0.13, 0.55),
            highlight: UiColor::rgba(1.00, 0.80, 0.43, 0.88),
            content_glint: UiColor::rgba(1.00, 0.92, 0.72, 0.70),
            border_width: 1.0,
            selected_border_width: 4.0,
            inner_inset: 3.0,
            icon_inset: 12.0,
        },
        button: ButtonStyle {
            fill: UiColor::rgba(0.46, 0.29, 0.08, 0.88),
            fill_hovered: UiColor::rgba(0.58, 0.37, 0.11, 0.94),
            fill_selected: UiColor::rgba(0.72, 0.46, 0.13, 0.96),
            fill_disabled: UiColor::rgba(0.08, 0.08, 0.07, 0.55),
            border: UiColor::rgba(0.94, 0.61, 0.20, 0.86),
            text: UiColor::rgb8(255, 238, 208),
            text_disabled: UiColor::rgb8(118, 111, 98),
            height: 44.0,
            padding_x: 18.0,
            border_width: 1.0,
        },
        search_bar: SearchBarStyle {
            fill: UiColor::rgba(0.025, 0.036, 0.038, 0.82),
            border: UiColor::rgba(0.50, 0.31, 0.12, 0.82),
            placeholder: UiColor::rgb8(178, 166, 148),
            text: UiColor::rgb8(255, 238, 208),
            icon: UiColor::rgb8(255, 197, 112),
            height: 40.0,
            padding_x: 14.0,
        },
        filter_chip: FilterChipStyle {
            fill: UiColor::rgba(0.040, 0.052, 0.052, 0.78),
            fill_selected: UiColor::rgba(0.50, 0.32, 0.09, 0.92),
            border: UiColor::rgba(0.28, 0.22, 0.15, 0.70),
            border_selected: UiColor::rgba(0.95, 0.62, 0.22, 0.95),
            text: UiColor::rgb8(232, 222, 205),
            text_selected: UiColor::rgb8(255, 229, 178),
            height: 38.0,
            padding_x: 16.0,
        },
        inventory_grid: InventoryGridStyle {
            columns: 8,
            slot_size: 58.0,
            gap: 8.0,
            row_gap: 10.0,
            empty_slot_fill: UiColor::rgba(0.025, 0.035, 0.035, 0.64),
        },
        hotbar: HotbarStyle {
            slots: 9,
            slot_size_min: 46.0,
            slot_size_max: 58.0,
            slot_height_ratio: 0.060,
            panel_padding: 10.0,
            slot_gap: 7.0,
            notice_offset_y: 30.0,
        },
        quantity_badge: QuantityBadgeStyle {
            text: UiColor::rgb8(255, 235, 190),
            notice: UiColor::rgb8(255, 205, 140),
            shadow: UiColor::rgba(0.0, 0.0, 0.0, 0.72),
            font_size: 16.0,
            line_height: 20.0,
            right_inset: 20.0,
            bottom_inset: 21.0,
        },
        text: TextStyle {
            title: UiColor::rgb8(255, 237, 210),
            section: UiColor::rgb8(255, 176, 47),
            body: UiColor::rgb8(240, 231, 214),
            muted: UiColor::rgb8(176, 164, 144),
            badge: UiColor::rgb8(255, 235, 190),
            notice: UiColor::rgb8(255, 205, 140),
            control: UiColor::rgb8(255, 238, 208),
            title_size: 30.0,
            section_size: 18.0,
            body_size: 16.0,
            badge_size: 16.0,
        },
    };
}

#[cfg(test)]
mod tests {
    use super::{ComponentState, UiTheme};

    #[test]
    fn voxelverse_theme_declares_shared_hotbar_shape() {
        let theme = UiTheme::VOXELVERSE;

        assert_eq!(theme.hotbar.slots, 9);
        assert_eq!(theme.inventory_grid.gap, theme.spacing.slot_gap + 1.0);
        assert!(theme.hotbar.slot_size_min <= theme.spacing.slot_size);
        assert!(theme.spacing.slot_size <= theme.hotbar.slot_size_max);
    }

    #[test]
    fn selected_slot_uses_stronger_border_than_normal_slot() {
        let slot = UiTheme::VOXELVERSE.slot;

        assert_ne!(
            slot.border_for(ComponentState::Normal),
            slot.border_for(ComponentState::Selected)
        );
        assert!(slot.selected_border_width > slot.border_width);
    }
}
