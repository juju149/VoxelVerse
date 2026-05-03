use vv_gameplay::Inventory;
use vv_registry::RecipeId;
use vv_ui::{UiPoint, UiRect};

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
    pub slot: f32,
    pub gap: f32,
    pub title_bar: UiRect,
    pub equipment_panel: UiRect,
    pub backpack_panel: UiRect,
    pub crafting_panel: UiRect,
    pub hotbar_panel: UiRect,
    pub hotbar_slots: Vec<InventorySlotRect>,
    pub inventory_slots: Vec<InventorySlotRect>,
    pub recipe_slots: Vec<RecipeSlotRect>,
    pub backpack_grid: UiRect,
    pub recipe_list: UiRect,
    pub recipe_detail: UiRect,
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
        let min_dim = screen_w.min(screen_h).max(1.0);
        let base_slot = 58.0;
        let base_gap = 8.0;
        let base_hotbar_w = inventory.hotbar_len() as f32 * base_slot
            + inventory.hotbar_len().saturating_sub(1) as f32 * base_gap;

        let fit_scale = ((screen_w - 32.0).max(1.0) / base_hotbar_w.max(1.0)).min(1.25);
        let scale = (min_dim / 1080.0)
            .clamp(0.72, 1.12)
            .min(fit_scale)
            .max(0.58);
        let slot = base_slot * scale;
        let gap = base_gap * scale;

        let hotbar_w = inventory.hotbar_len() as f32 * slot
            + inventory.hotbar_len().saturating_sub(1) as f32 * gap;
        let hotbar_x = (screen_w - hotbar_w) * 0.5;
        let hotbar_y = screen_h - slot - (24.0 * scale).max(12.0);

        let hotbar_slots = (0..inventory.hotbar_len())
            .map(|index| InventorySlotRect {
                index,
                rect: UiRect::new(hotbar_x + index as f32 * (slot + gap), hotbar_y, slot, slot),
            })
            .collect();

        Self {
            scale,
            slot,
            gap,
            title_bar: UiRect::ZERO,
            equipment_panel: UiRect::ZERO,
            backpack_panel: UiRect::ZERO,
            crafting_panel: UiRect::ZERO,
            hotbar_panel: UiRect::new(hotbar_x, hotbar_y, hotbar_w, slot),
            hotbar_slots,
            inventory_slots: Vec::new(),
            recipe_slots: Vec::new(),
            backpack_grid: UiRect::ZERO,
            recipe_list: UiRect::ZERO,
            recipe_detail: UiRect::ZERO,
        }
    }

    pub fn inventory(screen_w: f32, screen_h: f32, inventory: &Inventory) -> Self {
        let min_dim = screen_w.min(screen_h).max(1.0);
        let scale = (min_dim / 1080.0).clamp(0.70, 1.10);
        let outer = 28.0 * scale;
        let gap = 18.0 * scale;
        let title_h = 72.0 * scale;
        let bottom_hotbar_h = 82.0 * scale;

        let content_y = outer + title_h;
        let content_h = (screen_h - content_y - bottom_hotbar_h - outer).max(360.0 * scale);
        let total_w = screen_w - outer * 2.0;

        let equipment_w = (500.0 * scale).clamp(310.0, total_w * 0.30);
        let crafting_w = (640.0 * scale).clamp(340.0, total_w * 0.34);
        let backpack_w = (total_w - equipment_w - crafting_w - gap * 2.0).max(390.0);

        let equipment_panel = UiRect::new(outer, content_y, equipment_w, content_h);
        let backpack_panel = UiRect::new(
            equipment_panel.right() + gap,
            content_y,
            backpack_w,
            content_h,
        );
        let crafting_panel = UiRect::new(
            backpack_panel.right() + gap,
            content_y,
            screen_w - outer - (backpack_panel.right() + gap),
            content_h,
        );

        let slot = (58.0 * scale).clamp(38.0, 64.0);
        let slot_gap = (10.0 * scale).clamp(5.0, 12.0);
        let columns = Inventory::DEFAULT_MAIN_COLUMNS;
        let main_count = inventory
            .slot_count()
            .saturating_sub(inventory.hotbar_len());
        let rows = main_count
            .div_ceil(columns)
            .max(Inventory::DEFAULT_MAIN_ROWS);

        let search_h = 44.0 * scale;
        let tabs_h = 42.0 * scale;
        let pad = 24.0 * scale;

        let backpack_grid_w = columns as f32 * slot + columns.saturating_sub(1) as f32 * slot_gap;
        let backpack_grid_h = rows as f32 * slot + rows.saturating_sub(1) as f32 * slot_gap;
        let backpack_grid_x = backpack_panel.x + (backpack_panel.width - backpack_grid_w) * 0.5;
        let backpack_grid_y =
            backpack_panel.y + pad + 34.0 * scale + search_h + tabs_h + 16.0 * scale;
        let backpack_grid = UiRect::new(
            backpack_grid_x,
            backpack_grid_y,
            backpack_grid_w,
            backpack_grid_h,
        );

        let mut inventory_slots = Vec::new();

        let main_start = inventory.main_start();
        for main_index in 0..main_count {
            let index = main_start + main_index;
            let row = main_index / columns;
            let col = main_index % columns;

            inventory_slots.push(InventorySlotRect {
                index,
                rect: UiRect::new(
                    backpack_grid.x + col as f32 * (slot + slot_gap),
                    backpack_grid.y + row as f32 * (slot + slot_gap),
                    slot,
                    slot,
                ),
            });
        }

        let hotbar_slot = (60.0 * scale).clamp(42.0, 66.0);
        let hotbar_gap = (9.0 * scale).clamp(5.0, 11.0);
        let hotbar_w = inventory.hotbar_len() as f32 * hotbar_slot
            + inventory.hotbar_len().saturating_sub(1) as f32 * hotbar_gap;
        let hotbar_x = (screen_w - hotbar_w) * 0.5;
        let hotbar_y = screen_h - outer - hotbar_slot;

        let hotbar_slots = (0..inventory.hotbar_len())
            .map(|index| InventorySlotRect {
                index,
                rect: UiRect::new(
                    hotbar_x + index as f32 * (hotbar_slot + hotbar_gap),
                    hotbar_y,
                    hotbar_slot,
                    hotbar_slot,
                ),
            })
            .collect::<Vec<_>>();

        for slot_rect in &hotbar_slots {
            inventory_slots.push(*slot_rect);
        }

        let recipe_list = UiRect::new(
            crafting_panel.x + pad,
            crafting_panel.y + 92.0 * scale,
            crafting_panel.width * 0.38,
            crafting_panel.height - 168.0 * scale,
        );

        let recipe_detail = UiRect::new(
            recipe_list.right() + 26.0 * scale,
            recipe_list.y,
            crafting_panel.right() - recipe_list.right() - 50.0 * scale,
            recipe_list.height,
        );

        Self {
            scale,
            slot,
            gap: slot_gap,
            title_bar: UiRect::new(outer, outer, screen_w - outer * 2.0, title_h),
            equipment_panel,
            backpack_panel,
            crafting_panel,
            hotbar_panel: UiRect::new(hotbar_x, hotbar_y, hotbar_w, hotbar_slot),
            hotbar_slots,
            inventory_slots,
            recipe_slots: Vec::new(),
            backpack_grid,
            recipe_list,
            recipe_detail,
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

        if self.crafting_panel.width <= 0.0 {
            return;
        }

        let row_h = (64.0 * self.scale).clamp(44.0, 70.0);
        let row_gap = (10.0 * self.scale).clamp(6.0, 12.0);

        for (index, recipe) in recipes.take(8).enumerate() {
            let y = self.recipe_list.y + index as f32 * (row_h + row_gap);
            if y + row_h > self.recipe_list.bottom() {
                break;
            }

            self.recipe_slots.push(RecipeSlotRect {
                recipe,
                rect: UiRect::new(self.recipe_list.x, y, self.recipe_list.width, row_h),
            });
        }
    }
}
