use vv_gameplay::Inventory;
use vv_registry::RecipeId;
use vv_ui::{UiPoint, UiRect};

const DESIGN_W: f32 = 1920.0;
const DESIGN_H: f32 = 1080.0;
const DESIGN_CONTENT_W: f32 = 1868.0;
const DESIGN_PANEL_H: f32 = 780.0;

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
        let scale = responsive_scale(screen_w, screen_h);
        let safe = 24.0 * scale;
        let (hotbar_panel, hotbar_slots, hotbar_gap) =
            build_hotbar_layout(screen_w, screen_h, inventory, scale, safe);

        Self {
            scale,
            slot: hotbar_panel.height,
            gap: hotbar_gap,
            title_bar: UiRect::ZERO,
            equipment_panel: UiRect::ZERO,
            backpack_panel: UiRect::ZERO,
            crafting_panel: UiRect::ZERO,
            hotbar_panel,
            hotbar_slots,
            inventory_slots: Vec::new(),
            recipe_slots: Vec::new(),
            backpack_grid: UiRect::ZERO,
            recipe_list: UiRect::ZERO,
            recipe_detail: UiRect::ZERO,
        }
    }

    pub fn inventory(screen_w: f32, screen_h: f32, inventory: &Inventory) -> Self {
        let scale = responsive_scale(screen_w, screen_h);
        let safe = 24.0 * scale;
        let panel_gap = 18.0 * scale;

        let (hotbar_panel, hotbar_slots, _hotbar_gap) =
            build_hotbar_layout(screen_w, screen_h, inventory, scale, safe);

        let content_w = (DESIGN_CONTENT_W * scale).min((screen_w - safe * 2.0).max(320.0));
        let content_x = ((screen_w - content_w) * 0.5).round();

        let title_h = 76.0 * scale;
        let title_y = safe.round();
        let title_bar = UiRect::new(content_x, title_y, content_w.round(), title_h.round());

        let panel_y_min = title_bar.bottom() + 18.0 * scale;
        let available_panel_h = hotbar_panel.y - panel_y_min - 22.0 * scale;
        let panel_h = available_panel_h
            .min(DESIGN_PANEL_H * scale)
            .max(430.0 * scale)
            .round();

        let panel_y = if available_panel_h > panel_h {
            panel_y_min + (available_panel_h - panel_h) * 0.44
        } else {
            panel_y_min
        }
        .round();

        let equipment_w = (500.0 * scale).round();
        let crafting_w = (620.0 * scale).round();
        let backpack_w = (content_w - equipment_w - crafting_w - panel_gap * 2.0)
            .max(500.0 * scale)
            .round();

        let equipment_panel = UiRect::new(content_x, panel_y, equipment_w, panel_h);

        let backpack_panel = UiRect::new(
            (equipment_panel.right() + panel_gap).round(),
            panel_y,
            backpack_w,
            panel_h,
        );

        let crafting_panel = UiRect::new(
            (backpack_panel.right() + panel_gap).round(),
            panel_y,
            (content_x + content_w - backpack_panel.right() - panel_gap).round(),
            panel_h,
        );

        let pad = 24.0 * scale;
        let columns = Inventory::DEFAULT_MAIN_COLUMNS;
        let main_count = inventory
            .slot_count()
            .saturating_sub(inventory.hotbar_len());
        let rows = ceil_div(main_count, columns).max(Inventory::DEFAULT_MAIN_ROWS);

        let grid_gap = (10.0 * scale).clamp(6.0, 14.0);
        let available_grid_w = backpack_panel.width - pad * 2.0;
        let slot_from_width =
            (available_grid_w - grid_gap * columns.saturating_sub(1) as f32) / columns as f32;

        let available_grid_h = backpack_panel.height - (200.0 * scale);
        let slot_from_height =
            (available_grid_h - grid_gap * rows.saturating_sub(1) as f32) / rows as f32;

        let slot = slot_from_width
            .min(slot_from_height)
            .min(64.0 * scale)
            .clamp(34.0, 84.0)
            .round();

        let backpack_grid_w = columns as f32 * slot + columns.saturating_sub(1) as f32 * grid_gap;
        let backpack_grid_h = rows as f32 * slot + rows.saturating_sub(1) as f32 * grid_gap;

        let backpack_grid = UiRect::new(
            (backpack_panel.x + (backpack_panel.width - backpack_grid_w) * 0.5).round(),
            (backpack_panel.y + 174.0 * scale).round(),
            backpack_grid_w.round(),
            backpack_grid_h.round(),
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
                    (backpack_grid.x + col as f32 * (slot + grid_gap)).round(),
                    (backpack_grid.y + row as f32 * (slot + grid_gap)).round(),
                    slot,
                    slot,
                ),
            });
        }

        inventory_slots.extend(hotbar_slots.iter().copied());

        let recipe_list_w = (crafting_panel.width * 0.38).clamp(150.0 * scale, 250.0 * scale);

        let recipe_list = UiRect::new(
            (crafting_panel.x + pad).round(),
            (crafting_panel.y + 102.0 * scale).round(),
            recipe_list_w.round(),
            (crafting_panel.height - 180.0 * scale).max(260.0).round(),
        );

        let recipe_detail_x = recipe_list.right() + 26.0 * scale;

        let recipe_detail = UiRect::new(
            recipe_detail_x.round(),
            recipe_list.y,
            (crafting_panel.right() - recipe_detail_x - pad)
                .max(160.0)
                .round(),
            recipe_list.height,
        );

        Self {
            scale,
            slot,
            gap: grid_gap,
            title_bar,
            equipment_panel,
            backpack_panel,
            crafting_panel,
            hotbar_panel,
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

        if self.recipe_list.width <= 0.0 || self.recipe_list.height <= 0.0 {
            return;
        }

        let row_h = (64.0 * self.scale).clamp(44.0, 78.0).round();
        let row_gap = (10.0 * self.scale).clamp(6.0, 14.0).round();

        for (index, recipe) in recipes.take(8).enumerate() {
            let y = self.recipe_list.y + index as f32 * (row_h + row_gap);

            if y + row_h > self.recipe_list.bottom() {
                break;
            }

            self.recipe_slots.push(RecipeSlotRect {
                recipe,
                rect: UiRect::new(self.recipe_list.x, y.round(), self.recipe_list.width, row_h),
            });
        }
    }
}

fn build_hotbar_layout(
    screen_w: f32,
    screen_h: f32,
    inventory: &Inventory,
    scale: f32,
    safe: f32,
) -> (UiRect, Vec<InventorySlotRect>, f32) {
    let slot = hotbar_slot_size(scale);
    let gap = hotbar_gap_size(scale);
    let len = inventory.hotbar_len();

    let width = len as f32 * slot + len.saturating_sub(1) as f32 * gap;
    let x = ((screen_w - width) * 0.5).round();
    let y = (screen_h - safe - slot).round();

    let slots = (0..len)
        .map(|index| InventorySlotRect {
            index,
            rect: UiRect::new(
                (x + index as f32 * (slot + gap)).round(),
                y,
                slot.round(),
                slot.round(),
            ),
        })
        .collect();

    (UiRect::new(x, y, width.round(), slot.round()), slots, gap)
}

fn hotbar_slot_size(scale: f32) -> f32 {
    (70.0 * scale).clamp(48.0, 76.0)
}

fn hotbar_gap_size(scale: f32) -> f32 {
    (10.0 * scale).clamp(6.0, 13.0)
}

fn responsive_scale(screen_w: f32, screen_h: f32) -> f32 {
    let sx = screen_w.max(1.0) / DESIGN_W;
    let sy = screen_h.max(1.0) / DESIGN_H;
    sx.min(sy).clamp(0.62, 1.42)
}

fn ceil_div(value: usize, divisor: usize) -> usize {
    if divisor == 0 {
        return 0;
    }

    (value + divisor - 1) / divisor
}
