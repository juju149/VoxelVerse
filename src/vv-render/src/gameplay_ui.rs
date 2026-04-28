use glam::Vec2;
use vv_gameplay::Inventory;
use vv_registry::RecipeId;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RectPx {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl RectPx {
    pub fn contains(self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.w
            && point.y >= self.y
            && point.y <= self.y + self.h
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SlotRect {
    pub index: usize,
    pub rect: RectPx,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RecipeRect {
    pub recipe: RecipeId,
    pub rect: RectPx,
}

#[derive(Debug, Clone)]
pub(crate) struct GameplayUiLayout {
    pub scale: f32,
    pub slot: f32,
    pub gap: f32,
    pub hotbar_slots: Vec<SlotRect>,
    pub inventory_slots: Vec<SlotRect>,
    pub recipe_slots: Vec<RecipeRect>,
    pub inventory_panel: Option<RectPx>,
}

impl GameplayUiLayout {
    pub fn new(screen_w: f32, screen_h: f32, inventory: &Inventory, inventory_open: bool) -> Self {
        let min_dim = screen_w.min(screen_h).max(1.0);
        let base_slot = 44.0;
        let base_gap = 4.0;
        let base_hotbar_w = inventory.hotbar_len() as f32 * base_slot
            + inventory.hotbar_len().saturating_sub(1) as f32 * base_gap;
        let fit_scale = ((screen_w - 24.0).max(1.0) / base_hotbar_w.max(1.0)).min(1.45);
        let scale = (min_dim / 720.0).clamp(0.72, 1.28).min(fit_scale).max(0.58);
        let slot = base_slot * scale;
        let gap = base_gap * scale;

        let hotbar_w = inventory.hotbar_len() as f32 * slot
            + inventory.hotbar_len().saturating_sub(1) as f32 * gap;
        let hotbar_x = (screen_w - hotbar_w) * 0.5;
        let hotbar_y = screen_h - slot - (18.0 * scale).max(10.0);
        let hotbar_slots = (0..inventory.hotbar_len())
            .map(|index| SlotRect {
                index,
                rect: RectPx {
                    x: hotbar_x + index as f32 * (slot + gap),
                    y: hotbar_y,
                    w: slot,
                    h: slot,
                },
            })
            .collect();

        let mut layout = Self {
            scale,
            slot,
            gap,
            hotbar_slots,
            inventory_slots: Vec::new(),
            recipe_slots: Vec::new(),
            inventory_panel: None,
        };

        if inventory_open {
            layout.build_inventory_panel(screen_w, screen_h, inventory);
        }

        layout
    }

    pub fn inventory_slot_at(&self, point: Vec2) -> Option<usize> {
        self.inventory_slots
            .iter()
            .find(|slot| slot.rect.contains(point))
            .map(|slot| slot.index)
    }

    pub fn recipe_at(&self, point: Vec2) -> Option<RecipeId> {
        self.recipe_slots
            .iter()
            .find(|slot| slot.rect.contains(point))
            .map(|slot| slot.recipe)
    }

    pub fn add_hand_recipes(&mut self, recipes: impl Iterator<Item = RecipeId>) {
        let Some(panel) = self.inventory_panel else {
            return;
        };
        let x = panel.x + panel.w + 12.0 * self.scale;
        let y = panel.y + 36.0 * self.scale;
        let w = self.slot * 1.35;
        let h = self.slot * 0.72;
        for (index, recipe) in recipes.take(12).enumerate() {
            self.recipe_slots.push(RecipeRect {
                recipe,
                rect: RectPx {
                    x,
                    y: y + index as f32 * (h + self.gap),
                    w,
                    h,
                },
            });
        }
    }

    fn build_inventory_panel(&mut self, screen_w: f32, screen_h: f32, inventory: &Inventory) {
        let columns = Inventory::DEFAULT_MAIN_COLUMNS;
        let rows = Inventory::DEFAULT_MAIN_ROWS;
        let pad = 16.0 * self.scale;
        let title_h = 26.0 * self.scale;
        let section_gap = 16.0 * self.scale;
        let grid_w = columns as f32 * self.slot + (columns - 1) as f32 * self.gap;
        let main_h = rows as f32 * self.slot + (rows - 1) as f32 * self.gap;
        let panel_w = grid_w + pad * 2.0;
        let panel_h = pad * 2.0 + title_h + main_h + section_gap + self.slot;
        let panel_x = (screen_w - panel_w) * 0.5;
        let panel_y = ((screen_h - panel_h) * 0.5).max(8.0 * self.scale);
        self.inventory_panel = Some(RectPx {
            x: panel_x,
            y: panel_y,
            w: panel_w,
            h: panel_h,
        });

        let main_start = inventory.main_start();
        let main_x = panel_x + pad;
        let main_y = panel_y + pad + title_h;
        for row in 0..rows {
            for col in 0..columns {
                let main_index = row * columns + col;
                let index = main_start + main_index;
                if index >= inventory.slot_count() {
                    continue;
                }
                self.inventory_slots.push(SlotRect {
                    index,
                    rect: RectPx {
                        x: main_x + col as f32 * (self.slot + self.gap),
                        y: main_y + row as f32 * (self.slot + self.gap),
                        w: self.slot,
                        h: self.slot,
                    },
                });
            }
        }

        let hotbar_y = main_y + main_h + section_gap;
        for col in 0..inventory.hotbar_len() {
            self.inventory_slots.push(SlotRect {
                index: col,
                rect: RectPx {
                    x: main_x + col as f32 * (self.slot + self.gap),
                    y: hotbar_y,
                    w: self.slot,
                    h: self.slot,
                },
            });
        }
    }
}
