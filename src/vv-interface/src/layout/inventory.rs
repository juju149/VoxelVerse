use vv_gameplay::Inventory;
use vv_registry::RecipeId;
use vv_ui::{UiPoint, UiRect};

const DESIGN_W: f32 = 2048.0;
const DESIGN_H: f32 = 1152.0;

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

    pub screen: UiRect,
    pub title_bar: UiRect,

    pub equipment_panel: UiRect,
    pub backpack_panel: UiRect,
    pub crafting_panel: UiRect,

    pub hotbar_panel: UiRect,
    pub hotbar_slots: Vec<InventorySlotRect>,

    pub inventory_slots: Vec<InventorySlotRect>,
    pub recipe_slots: Vec<RecipeSlotRect>,

    pub backpack_grid: UiRect,
    pub backpack_cells: Vec<UiRect>,
    pub backpack_columns: usize,
    pub backpack_rows: usize,

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

    pub fn inventory(screen_w: f32, screen_h: f32, inventory: &Inventory) -> Self {
        let scale = responsive_scale(screen_w, screen_h);
        let screen = UiRect::new(0.0, 0.0, screen_w.max(0.0), screen_h.max(0.0));

        let outer_margin = (30.0 * scale).round();
        let panel_gap = (24.0 * scale).round();

        let content_x = outer_margin;
        let content_w = (screen_w - outer_margin * 2.0).max(320.0);

        let panel_total_w = (content_w - panel_gap * 2.0).max(320.0);

        let equipment_w = (panel_total_w * 0.30).round();
        let backpack_w = (panel_total_w * 0.40).round();
        let crafting_w = (panel_total_w - equipment_w - backpack_w).round();

        let panel_h = (screen_h * 0.70).round();
        let panel_y = ((screen_h - panel_h) * 0.5).round();

        let equipment_panel = UiRect::new(content_x.round(), panel_y, equipment_w, panel_h);

        let backpack_panel = UiRect::new(
            (equipment_panel.right() + panel_gap).round(),
            panel_y,
            backpack_w,
            panel_h,
        );

        let crafting_panel = UiRect::new(
            (backpack_panel.right() + panel_gap).round(),
            panel_y,
            crafting_w,
            panel_h,
        );

        let title_bar = UiRect::new(
            outer_margin,
            outer_margin,
            (screen_w - outer_margin * 2.0).max(0.0),
            (78.0 * scale).round(),
        );

        let (hotbar_panel, hotbar_slots, hotbar_gap) =
            build_hotbar_layout(screen_w, screen_h, inventory, scale);

        Self {
            scale,
            slot: 0.0,
            gap: hotbar_gap,

            screen,
            title_bar,

            equipment_panel,
            backpack_panel,
            crafting_panel,

            hotbar_panel,
            hotbar_slots,

            inventory_slots: Vec::new(),
            recipe_slots: Vec::new(),

            backpack_grid: UiRect::ZERO,
            backpack_cells: Vec::new(),
            backpack_columns: 0,
            backpack_rows: 0,

            recipe_list: UiRect::ZERO,
            recipe_detail: UiRect::ZERO,
        }
    }

    pub fn hotbar_only(screen_w: f32, screen_h: f32, inventory: &Inventory) -> Self {
        let scale = responsive_scale(screen_w, screen_h);
        let screen = UiRect::new(0.0, 0.0, screen_w.max(0.0), screen_h.max(0.0));

        let (hotbar_panel, hotbar_slots, hotbar_gap) =
            build_hotbar_layout(screen_w, screen_h, inventory, scale);

        Self {
            scale,
            slot: hotbar_panel.height,
            gap: hotbar_gap,

            screen,
            title_bar: UiRect::ZERO,

            equipment_panel: UiRect::ZERO,
            backpack_panel: UiRect::ZERO,
            crafting_panel: UiRect::ZERO,

            hotbar_panel,
            hotbar_slots,

            inventory_slots: Vec::new(),
            recipe_slots: Vec::new(),

            backpack_grid: UiRect::ZERO,
            backpack_cells: Vec::new(),
            backpack_columns: 0,
            backpack_rows: 0,

            recipe_list: UiRect::ZERO,
            recipe_detail: UiRect::ZERO,
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

    pub fn add_hand_recipes(&mut self, _recipes: impl Iterator<Item = RecipeId>) {
        self.recipe_slots.clear();
    }
}

fn build_hotbar_layout(
    screen_w: f32,
    screen_h: f32,
    inventory: &Inventory,
    scale: f32,
) -> (UiRect, Vec<InventorySlotRect>, f32) {
    let safe = 24.0 * scale;
    let slot = (70.0 * scale).clamp(48.0, 76.0).round();
    let gap = (10.0 * scale).clamp(6.0, 13.0).round();
    let len = inventory.hotbar_len();

    let width = len as f32 * slot + len.saturating_sub(1) as f32 * gap;
    let x = ((screen_w - width) * 0.5).round();
    let y = (screen_h - safe - slot).round();

    let slots = (0..len)
        .map(|index| InventorySlotRect {
            index,
            rect: UiRect::new((x + index as f32 * (slot + gap)).round(), y, slot, slot),
        })
        .collect();

    (UiRect::new(x, y, width.round(), slot), slots, gap)
}

fn responsive_scale(screen_w: f32, screen_h: f32) -> f32 {
    let sx = screen_w.max(1.0) / DESIGN_W;
    let sy = screen_h.max(1.0) / DESIGN_H;
    sx.min(sy).clamp(0.72, 1.35)
}
