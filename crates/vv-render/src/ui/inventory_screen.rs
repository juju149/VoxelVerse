use super::theme::{UiAnchor, UiTheme, UiViewport, UserZoom};
use vv_gameplay::{
    HotbarSlot, SlotRef, HOTBAR_SLOT_COUNT, INVENTORY_COLS, INVENTORY_ROWS, INVENTORY_SIZE,
};

/// A stack the player has picked up. Follows the cursor and is deposited on
/// the next click — Minecraft-style. The source is remembered so we can put
/// the stack back if the inventory is closed while still holding it.
#[derive(Clone, Copy, Debug)]
pub struct HeldStack {
    pub stack: HotbarSlot,
    pub source: SlotRef,
}

/// Functional buttons in the modal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InventoryButton {
    Close,
    Sort,
    ClearSearch,
}

/// Filter chips above the inventory grid. Only `All` actually returns
/// every item today — the other categories will hook into a content-level
/// tag system later (see TODO in `InventoryUiState::matches_filter`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InventoryFilter {
    All,
    Resources,
    Tools,
    Food,
    Misc,
}

impl InventoryFilter {
    pub const ALL: [InventoryFilter; 5] = [
        InventoryFilter::All,
        InventoryFilter::Resources,
        InventoryFilter::Tools,
        InventoryFilter::Food,
        InventoryFilter::Misc,
    ];

    pub fn label(self) -> &'static str {
        match self {
            InventoryFilter::All => "Tout",
            InventoryFilter::Resources => "Ressources",
            InventoryFilter::Tools => "Outils",
            InventoryFilter::Food => "Nourriture",
            InventoryFilter::Misc => "Divers",
        }
    }
}

#[derive(Clone, Debug)]
pub struct InventoryUiState {
    pub is_open: bool,
    pub search_query: String,
    pub held: Option<HeldStack>,
    pub cursor: (f32, f32),
    pub hovered_slot: Option<SlotRef>,
    pub hovered_button: Option<InventoryButton>,
    pub hovered_search: bool,
    pub hovered_filter: Option<InventoryFilter>,
    pub active_filter: InventoryFilter,
    pub search_focused: bool,
    pub user_zoom: UserZoom,
    /// Carrying capacity in arbitrary "kg" units. Each stack contributes
    /// `quantity * UNIT_WEIGHT_KG` until the content layer ships per-block
    /// weights.
    pub capacity_kg: f32,
}

impl InventoryUiState {
    pub const UNIT_WEIGHT_KG: f32 = 0.7;
    pub const DEFAULT_CAPACITY_KG: f32 = 120.0;

    pub fn new() -> Self {
        Self {
            is_open: false,
            search_query: String::new(),
            held: None,
            cursor: (0.0, 0.0),
            hovered_slot: None,
            hovered_button: None,
            hovered_search: false,
            hovered_filter: None,
            active_filter: InventoryFilter::All,
            search_focused: false,
            user_zoom: UserZoom::Normal,
            capacity_kg: Self::DEFAULT_CAPACITY_KG,
        }
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        if !self.is_open {
            self.search_focused = false;
            self.hovered_slot = None;
            self.hovered_button = None;
            self.hovered_filter = None;
        }
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.search_focused = false;
        self.hovered_slot = None;
        self.hovered_button = None;
        self.hovered_filter = None;
    }

    pub fn matches_search(&self, name: &str) -> bool {
        let q = self.search_query.to_lowercase();
        if q.is_empty() {
            return true;
        }
        name.to_lowercase().contains(&q)
    }

    /// Returns true if `category` matches the active filter.
    /// Block categories come from the `.ron` `category` field
    /// (e.g. `"terrain"`, `"ore"`, `"tool"`, `"food"`, `"consumable"`).
    pub fn matches_filter(&self, category: &str) -> bool {
        match self.active_filter {
            InventoryFilter::All => true,
            InventoryFilter::Resources => {
                matches!(
                    category,
                    "resource" | "ore" | "terrain" | "natural/log" | "natural/leaves" | "flora"
                )
            }
            InventoryFilter::Tools => matches!(category, "tool" | "weapon"),
            InventoryFilter::Food => matches!(category, "food" | "consumable"),
            InventoryFilter::Misc => !matches!(
                category,
                "resource"
                    | "ore"
                    | "terrain"
                    | "natural/log"
                    | "natural/leaves"
                    | "flora"
                    | "tool"
                    | "weapon"
                    | "food"
                    | "consumable"
            ),
        }
    }
}

impl Default for InventoryUiState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl UiRect {
    pub fn contains(self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }
}

/// Pixel-space layout of the inventory modal, recomputed each frame so the
/// UI stays sharp under resize and DPI changes. All rects are absolute
/// screen pixels — never UI units.
#[derive(Clone, Debug)]
pub struct InventoryLayout {
    pub viewport: UiViewport,
    pub scale: f32,

    // Window chrome
    pub modal: UiRect,
    pub header_strip: UiRect,
    pub bag_icon: UiRect,
    pub close_button: UiRect,

    // Column panels
    pub left_panel: UiRect,
    pub center_panel: UiRect,
    pub right_panel: UiRect,

    // Center column — toolbar
    pub search_bar: UiRect,
    pub search_icon: UiRect,
    pub clear_search_button: UiRect,
    pub filter_chips: Vec<(InventoryFilter, UiRect)>,

    // Center column — grid
    pub inventory_slots: Vec<UiRect>,

    // Center column — footer
    pub center_separator: UiRect,
    pub weight_bar_track: UiRect,
    pub weight_bar_label_origin: (f32, f32),
    pub sort_button: UiRect,

    // Hotbar mirror (positioned identically to the real in-game hotbar)
    pub hotbar_slots: Vec<UiRect>,

    // Text anchors
    pub title_origin: (f32, f32),
    pub subtitle_origin: (f32, f32),
    pub left_title_origin: (f32, f32),
    pub right_title_origin: (f32, f32),
    pub center_title_origin: (f32, f32),
    pub search_text_origin: (f32, f32),
    pub sort_text_origin: (f32, f32),
    pub equipment_placeholder_origin: (f32, f32),
    pub craft_placeholder_origin: (f32, f32),
    pub craft_subtitle_origin: (f32, f32),

    // Useful derived sizes
    pub inventory_slot_size: f32,
    pub hotbar_slot_size: f32,
}

impl InventoryLayout {
    pub fn compute(theme: &UiTheme, viewport: UiViewport, user_zoom: UserZoom) -> Self {
        const MODAL_WIDTH_FRACTION: f32 = 0.92;
        const LEFT_FRACTION: f32 = 0.22;
        const CENTER_FRACTION: f32 = 0.56;
        const RIGHT_FRACTION: f32 = 0.22;

        let scale = theme.effective_scale(viewport, user_zoom);
        let s = |units: f32| units * scale;

        // Real hotbar position — kept in sync with `Renderer::hotbar_layout`
        // in [rendering/renderer/ui.rs] so the modal's mirror sits exactly
        // on top of the in-game hotbar.
        let real_hotbar_slot = (viewport.height * theme.hotbar.slot_height_ratio)
            .clamp(theme.hotbar.slot_size_min, theme.hotbar.slot_size_max);
        let real_hotbar_gap = theme.hotbar.slot_gap * (real_hotbar_slot / 56.0).max(1.0);
        let real_hotbar_total_w = HOTBAR_SLOT_COUNT as f32 * real_hotbar_slot
            + (HOTBAR_SLOT_COUNT - 1) as f32 * real_hotbar_gap;
        let real_hotbar_bottom_margin = (viewport.height * 0.035).clamp(
            theme.spacing.hotbar_bottom_margin_min,
            theme.spacing.hotbar_bottom_margin_max,
        );
        let real_hotbar_top = viewport.height - real_hotbar_bottom_margin - real_hotbar_slot;

        // Modal sits in the upper portion. Its bottom leaves room for the
        // hotbar plus a comfortable air gap.
        let modal_w = viewport.width * MODAL_WIDTH_FRACTION;
        let top_margin = viewport.height * 0.035;
        let bottom_reserved =
            real_hotbar_slot + real_hotbar_bottom_margin + s(theme.spacing.xlarge);
        let modal_h = (viewport.height - top_margin - bottom_reserved).max(s(420.0));
        let modal_x = (viewport.width - modal_w) * 0.5;
        let modal = UiRect {
            x: modal_x,
            y: top_margin,
            w: modal_w,
            h: modal_h,
        };
        let _ = UiAnchor::Center;

        // Header strip — bag icon, title, subtitle, close button.
        let header_h = s(theme.text.title_size + theme.text.muted_size + 32.0);
        let header_strip = UiRect {
            x: modal.x,
            y: modal.y,
            w: modal.w,
            h: header_h,
        };

        let bag_size = s(48.0);
        let bag_icon = UiRect {
            x: modal.x + s(theme.spacing.large),
            y: modal.y + (header_h - bag_size) * 0.5,
            w: bag_size,
            h: bag_size,
        };
        let title_origin = (
            bag_icon.x + bag_icon.w + s(theme.spacing.medium),
            modal.y + s(theme.spacing.medium) - s(2.0),
        );
        let subtitle_origin = (
            title_origin.0,
            title_origin.1 + s(theme.text.title_size) + s(2.0),
        );

        let close_size = s(44.0);
        let close_button = UiRect {
            x: modal.x + modal.w - close_size - s(theme.spacing.medium),
            y: modal.y + (header_h - close_size) * 0.5,
            w: close_size,
            h: close_size,
        };

        // Columns sit below the header strip.
        let panel_pad = s(theme.panel.padding);
        let panel_gap = s(theme.spacing.large);
        let column_y = modal.y + header_h;
        let column_h = modal.h - header_h - panel_pad;
        let usable_w = modal.w - panel_pad * 2.0 - panel_gap * 2.0;
        let total_frac = LEFT_FRACTION + CENTER_FRACTION + RIGHT_FRACTION;
        let left_w = usable_w * LEFT_FRACTION / total_frac;
        let center_w = usable_w * CENTER_FRACTION / total_frac;
        let right_w = usable_w * RIGHT_FRACTION / total_frac;

        let left_x = modal.x + panel_pad;
        let center_x = left_x + left_w + panel_gap;
        let right_x = center_x + center_w + panel_gap;

        let left_panel = UiRect {
            x: left_x,
            y: column_y,
            w: left_w,
            h: column_h,
        };
        let center_panel = UiRect {
            x: center_x,
            y: column_y,
            w: center_w,
            h: column_h,
        };
        let right_panel = UiRect {
            x: right_x,
            y: column_y,
            w: right_w,
            h: column_h,
        };

        // ---------------- Center column internals ----------------
        let section_pad = s(theme.spacing.large);
        let center_inner_x = center_panel.x + section_pad;
        let center_inner_w = center_panel.w - section_pad * 2.0;

        let center_title_origin = (center_inner_x, center_panel.y + section_pad);
        let section_title_height = s(theme.text.section_size) + s(theme.spacing.medium);

        // Search bar with magnifier icon on the right.
        let search_h = s(theme.search_bar.height + 4.0);
        let search_top = center_panel.y + section_pad + section_title_height;
        let search_bar = UiRect {
            x: center_inner_x,
            y: search_top,
            w: center_inner_w,
            h: search_h,
        };
        let icon_size = s(20.0);
        let search_icon = UiRect {
            x: search_bar.x + search_bar.w - icon_size - s(theme.spacing.medium),
            y: search_bar.y + (search_h - icon_size) * 0.5,
            w: icon_size,
            h: icon_size,
        };
        let clear_button_size = s(28.0);
        let clear_search_button = UiRect {
            x: search_icon.x - clear_button_size - s(theme.spacing.small),
            y: search_bar.y + (search_h - clear_button_size) * 0.5,
            w: clear_button_size,
            h: clear_button_size,
        };
        let search_text_origin = (
            search_bar.x + s(theme.search_bar.padding_x),
            search_bar.y + (search_h - s(theme.text.body_size)) * 0.5 - s(2.0),
        );

        // Filter chips row.
        let chip_h = s(theme.filter_chip.height);
        let chip_gap = s(theme.spacing.small);
        let chip_top = search_bar.y + search_bar.h + s(theme.spacing.medium);
        // Estimate each chip width from its label length (monospace heuristic).
        let mut chip_x = center_inner_x;
        let mut filter_chips: Vec<(InventoryFilter, UiRect)> = Vec::new();
        for filter in InventoryFilter::ALL {
            let label = filter.label();
            let chip_w = (label.chars().count() as f32 * s(theme.text.control_size) * 0.62
                + s(theme.filter_chip.padding_x * 2.0))
            .max(s(72.0));
            filter_chips.push((
                filter,
                UiRect {
                    x: chip_x,
                    y: chip_top,
                    w: chip_w,
                    h: chip_h,
                },
            ));
            chip_x += chip_w + chip_gap;
        }

        // 9x4 grid that fills the panel width.
        let cols = INVENTORY_COLS as f32;
        let gap_ratio = 0.10_f32;
        let inv_slot = (center_inner_w / (cols + (cols - 1.0) * gap_ratio)).floor();
        let inv_gap = (inv_slot * gap_ratio).max(s(4.0));
        let grid_w = inv_slot * cols + inv_gap * (cols - 1.0);
        let grid_left = center_inner_x + (center_inner_w - grid_w) * 0.5;
        let grid_top = chip_top + chip_h + s(theme.spacing.large);
        let mut inventory_slots = Vec::with_capacity(INVENTORY_SIZE);
        for row in 0..INVENTORY_ROWS {
            for col in 0..INVENTORY_COLS {
                inventory_slots.push(UiRect {
                    x: grid_left + col as f32 * (inv_slot + inv_gap),
                    y: grid_top + row as f32 * (inv_slot + inv_gap),
                    w: inv_slot,
                    h: inv_slot,
                });
            }
        }

        // Footer area at the bottom of the center panel.
        let sort_button_w = s(140.0);
        let sort_button_h = s(theme.button.height);
        let footer_bottom = center_panel.y + center_panel.h - s(theme.spacing.large);
        let sort_button = UiRect {
            x: center_inner_x + center_inner_w - sort_button_w,
            y: footer_bottom - sort_button_h,
            w: sort_button_w,
            h: sort_button_h,
        };
        let sort_text_origin = (
            sort_button.x + s(28.0),
            sort_button.y + (sort_button_h - s(theme.text.control_size)) * 0.5 - s(2.0),
        );

        let weight_bar_h = s(8.0);
        let weight_bar_track = UiRect {
            x: center_inner_x,
            y: sort_button.y + (sort_button_h - weight_bar_h) * 0.5 - s(2.0),
            w: center_inner_w - sort_button_w - s(theme.spacing.large),
            h: weight_bar_h,
        };
        let weight_bar_label_origin = (
            center_inner_x,
            weight_bar_track.y - s(theme.text.muted_size) - s(4.0),
        );

        let center_separator = UiRect {
            x: center_inner_x,
            y: weight_bar_label_origin.1 - s(theme.spacing.small) - s(2.0),
            w: center_inner_w,
            h: s(1.0).max(1.0),
        };

        // ---------------- Side columns ----------------
        let left_title_origin = (left_panel.x + section_pad, left_panel.y + section_pad);
        let right_title_origin = (right_panel.x + section_pad, right_panel.y + section_pad);

        let equipment_placeholder_origin = (
            left_panel.x + left_panel.w * 0.5,
            left_panel.y + left_panel.h * 0.5,
        );
        let craft_placeholder_origin = (
            right_panel.x + right_panel.w * 0.5,
            right_panel.y + right_panel.h * 0.5,
        );
        let craft_subtitle_origin = (
            craft_placeholder_origin.0,
            craft_placeholder_origin.1 + s(theme.text.section_size) + s(6.0),
        );

        // ---------------- Hotbar mirror ----------------
        let hotbar_left = (viewport.width - real_hotbar_total_w) * 0.5;
        let mut hotbar_slots = Vec::with_capacity(HOTBAR_SLOT_COUNT);
        for i in 0..HOTBAR_SLOT_COUNT {
            hotbar_slots.push(UiRect {
                x: hotbar_left + i as f32 * (real_hotbar_slot + real_hotbar_gap),
                y: real_hotbar_top,
                w: real_hotbar_slot,
                h: real_hotbar_slot,
            });
        }

        Self {
            viewport,
            scale,
            modal,
            header_strip,
            bag_icon,
            close_button,
            left_panel,
            center_panel,
            right_panel,
            search_bar,
            search_icon,
            clear_search_button,
            filter_chips,
            inventory_slots,
            center_separator,
            weight_bar_track,
            weight_bar_label_origin,
            sort_button,
            hotbar_slots,
            title_origin,
            subtitle_origin,
            left_title_origin,
            right_title_origin,
            center_title_origin,
            search_text_origin,
            sort_text_origin,
            equipment_placeholder_origin,
            craft_placeholder_origin,
            craft_subtitle_origin,
            inventory_slot_size: inv_slot,
            hotbar_slot_size: real_hotbar_slot,
        }
    }

    pub fn slot_under_cursor(&self, px: f32, py: f32) -> Option<SlotRef> {
        for (i, rect) in self.inventory_slots.iter().enumerate() {
            if rect.contains(px, py) {
                return Some(SlotRef::Inventory(i));
            }
        }
        for (i, rect) in self.hotbar_slots.iter().enumerate() {
            if rect.contains(px, py) {
                return Some(SlotRef::Hotbar(i));
            }
        }
        None
    }

    pub fn button_under_cursor(&self, px: f32, py: f32) -> Option<InventoryButton> {
        if self.close_button.contains(px, py) {
            return Some(InventoryButton::Close);
        }
        if self.sort_button.contains(px, py) {
            return Some(InventoryButton::Sort);
        }
        if self.clear_search_button.contains(px, py) {
            return Some(InventoryButton::ClearSearch);
        }
        None
    }

    pub fn filter_under_cursor(&self, px: f32, py: f32) -> Option<InventoryFilter> {
        for (filter, rect) in &self.filter_chips {
            if rect.contains(px, py) {
                return Some(*filter);
            }
        }
        None
    }
}
