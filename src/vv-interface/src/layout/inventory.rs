use vv_gameplay::Inventory;
use vv_registry::RecipeId;
use vv_ui::{UiPoint, UiRect};

use crate::design::InventoryUiTokens;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InventorySlotRect {
    pub index: usize,
    pub rect: UiRect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecipeSlotRect {
    pub recipe: RecipeId,
    pub rect: UiRect,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InventoryUiLayout {
    pub scale: f32,

    pub equipment_panel: UiRect,
    pub backpack_panel: UiRect,
    pub crafting_panel: UiRect,

    pub backpack_title: UiRect,
    pub backpack_search: UiRect,
    pub backpack_sort: UiRect,
    pub backpack_tabs: UiRect,
    pub backpack_grid: UiRect,
    pub backpack_cells: Vec<UiRect>,

    pub hotbar_panel: UiRect,
    pub hotbar_slots: Vec<InventorySlotRect>,

    pub inventory_slots: Vec<InventorySlotRect>,
    pub recipe_slots: Vec<RecipeSlotRect>,

    pub recipe_list: UiRect,
    pub recipe_detail: UiRect,

    pub slot: f32,
    pub gap: f32,
    pub title_bar: UiRect,
}

impl InventoryUiLayout {
    pub fn new(screen_w: f32, screen_h: f32, inventory: &Inventory, inventory_open: bool) -> Self {
        if inventory_open {
            Self::inventory(screen_w, screen_h, inventory)
        } else {
            Self::hotbar_only(screen_w, screen_h, inventory)
        }
    }

    pub fn hotbar_only(screen_w: f32, screen_h: f32, inventory: &Inventory) -> Self {
        let tokens = InventoryUiTokens::current();
        let scale = responsive_scale(screen_w, screen_h, &tokens);

        let slot = (78.0 * scale).clamp(54.0, 92.0).round();
        let gap = (10.0 * scale).clamp(7.0, 14.0).round();

        let hotbar_w = inventory.hotbar_len() as f32 * slot
            + inventory.hotbar_len().saturating_sub(1) as f32 * gap;

        let hotbar_x = ((screen_w - hotbar_w) * 0.5).round();
        let hotbar_y = (screen_h - 28.0 * scale - slot).round();

        let hotbar_slots = (0..inventory.hotbar_len())
            .map(|index| InventorySlotRect {
                index,
                rect: UiRect::new(hotbar_x + index as f32 * (slot + gap), hotbar_y, slot, slot),
            })
            .collect();

        Self {
            scale,
            equipment_panel: UiRect::ZERO,
            backpack_panel: UiRect::ZERO,
            crafting_panel: UiRect::ZERO,
            backpack_title: UiRect::ZERO,
            backpack_search: UiRect::ZERO,
            backpack_sort: UiRect::ZERO,
            backpack_tabs: UiRect::ZERO,
            backpack_grid: UiRect::ZERO,
            backpack_cells: Vec::new(),
            hotbar_panel: UiRect::new(hotbar_x, hotbar_y, hotbar_w, slot),
            hotbar_slots,
            inventory_slots: Vec::new(),
            recipe_slots: Vec::new(),
            recipe_list: UiRect::ZERO,
            recipe_detail: UiRect::ZERO,
            slot,
            gap,
            title_bar: UiRect::ZERO,
        }
    }

    pub fn inventory(screen_w: f32, screen_h: f32, inventory: &Inventory) -> Self {
        let tokens = InventoryUiTokens::current();
        let scale = responsive_scale(screen_w, screen_h, &tokens);

        let outer = tokens.layout.outer_margin * scale;
        let panel_gap = tokens.layout.panel_gap * scale;
        let content_w = (screen_w - outer * 2.0).max(320.0);
        let usable_w = content_w - panel_gap * 2.0;

        let equipment_w = (usable_w * tokens.layout.equipment_width_ratio).round();
        let backpack_w = (usable_w * tokens.layout.backpack_width_ratio).round();
        let crafting_w = (content_w - equipment_w - backpack_w - panel_gap * 2.0).round();

        let panel_h = (screen_h * tokens.layout.panel_height_ratio).round();
        let panel_y = ((screen_h - panel_h) * 0.42).max(outer).round();
        let panel_x = outer.round();

        let equipment_panel = UiRect::new(panel_x, panel_y, equipment_w, panel_h);
        let backpack_panel = UiRect::new(
            equipment_panel.right() + panel_gap,
            panel_y,
            backpack_w,
            panel_h,
        );
        let crafting_panel = UiRect::new(
            backpack_panel.right() + panel_gap,
            panel_y,
            crafting_w,
            panel_h,
        );

        let pad = tokens.layout.panel_padding * scale;

        let backpack_title = UiRect::new(
            backpack_panel.x + pad,
            backpack_panel.y + tokens.layout.title_top * scale,
            backpack_panel.width - pad * 2.0,
            34.0 * scale,
        );

        let search_y = backpack_panel.y + tokens.layout.search_top * scale;
        let search_h = tokens.layout.search_height * scale;
        let sort_w = tokens.layout.sort_button_width * scale;
        let control_gap = tokens.layout.control_gap * scale;

        let backpack_sort = UiRect::new(
            backpack_panel.right() - pad - sort_w,
            search_y,
            sort_w,
            search_h,
        );

        let backpack_search = UiRect::new(
            backpack_panel.x + pad,
            search_y,
            backpack_sort.x - control_gap - backpack_panel.x - pad,
            search_h,
        );

        let backpack_tabs = UiRect::new(
            backpack_panel.x + pad,
            backpack_panel.y + tokens.layout.tabs_top * scale,
            backpack_panel.width - pad * 2.0,
            tokens.layout.tab_height * scale,
        );

        let grid_top = backpack_panel.y + tokens.grid.top * scale;
        let grid_gap = (tokens.grid.gap * scale).round();
        let columns = tokens.grid.columns;
        let rows = tokens.grid.rows;

        let available_grid_w = backpack_panel.width - pad * 2.0;
        let available_grid_h =
            backpack_panel.bottom() - grid_top - tokens.grid.bottom_reserved * scale;

        let slot_from_w =
            (available_grid_w - grid_gap * columns.saturating_sub(1) as f32) / columns as f32;
        let slot_from_h =
            (available_grid_h - grid_gap * rows.saturating_sub(1) as f32) / rows as f32;

        let slot = slot_from_w
            .min(slot_from_h)
            .clamp(tokens.grid.slot_min * scale, tokens.grid.slot_max * scale)
            .round();

        let grid_w = columns as f32 * slot + columns.saturating_sub(1) as f32 * grid_gap;
        let grid_h = rows as f32 * slot + rows.saturating_sub(1) as f32 * grid_gap;

        let backpack_grid = UiRect::new(
            (backpack_panel.x + (backpack_panel.width - grid_w) * 0.5).round(),
            grid_top.round(),
            grid_w.round(),
            grid_h.round(),
        );

        let mut backpack_cells = Vec::with_capacity(columns * rows);
        let mut inventory_slots = Vec::new();
        let main_start = inventory.main_start();
        let main_count = inventory
            .slot_count()
            .saturating_sub(inventory.hotbar_len());

        for row in 0..rows {
            for col in 0..columns {
                let cell_index = row * columns + col;
                let rect = UiRect::new(
                    backpack_grid.x + col as f32 * (slot + grid_gap),
                    backpack_grid.y + row as f32 * (slot + grid_gap),
                    slot,
                    slot,
                );

                backpack_cells.push(rect);

                if cell_index < main_count {
                    inventory_slots.push(InventorySlotRect {
                        index: main_start + cell_index,
                        rect,
                    });
                }
            }
        }

        let hotbar = Self::hotbar_only(screen_w, screen_h, inventory);

        Self {
            scale,
            equipment_panel,
            backpack_panel,
            crafting_panel,
            backpack_title,
            backpack_search,
            backpack_sort,
            backpack_tabs,
            backpack_grid,
            backpack_cells,
            hotbar_panel: hotbar.hotbar_panel,
            hotbar_slots: hotbar.hotbar_slots,
            inventory_slots,
            recipe_slots: Vec::new(),
            recipe_list: UiRect::ZERO,
            recipe_detail: UiRect::ZERO,
            slot,
            gap: grid_gap,
            title_bar: UiRect::new(panel_x, panel_y, content_w, panel_h),
        }
    }

    pub fn inventory_slot_at(&self, point: UiPoint) -> Option<usize> {
        self.inventory_slots
            .iter()
            .find(|slot| slot.rect.contains(point))
            .map(|slot| slot.index)
    }

    pub fn recipe_at(&self, point: UiPoint) -> Option<RecipeId> {
        self.recipe_slots
            .iter()
            .find(|slot| slot.rect.contains(point))
            .map(|slot| slot.recipe)
    }

    pub fn add_hand_recipes(&mut self, recipes: impl Iterator<Item = RecipeId>) {
        self.recipe_slots.clear();

        for recipe in recipes.take(8) {
            self.recipe_slots.push(RecipeSlotRect {
                recipe,
                rect: UiRect::ZERO,
            });
        }
    }
}

fn responsive_scale(screen_w: f32, screen_h: f32, tokens: &InventoryUiTokens) -> f32 {
    let sx = screen_w.max(1.0) / tokens.layout.design_width;
    let sy = screen_h.max(1.0) / tokens.layout.design_height;
    sx.min(sy)
        .clamp(tokens.layout.scale_min, tokens.layout.scale_max)
}
