//! Inventory modal rendering.
//!
//! All sub-screens of the modal (header, equipment panel, search bar, filter
//! chips, slot grid, weight bar, sort button, craft panel, held stack) are
//! drawn by their own `draw_*` method below. Geometry is rounded-rect with
//! a small triangle fan per corner — single colour-only pipeline, no extra
//! shader needed.
//!
//! The text path stays in [render_passes.rs](rendering/renderer/render_passes.rs):
//! `inventory_text_specs` emits every label with its target font size and
//! glyphon draws them after the mesh pass.

#![allow(clippy::too_many_arguments)]

use super::inventory_geometry::equip_slot_rects;
use super::inventory_text::{
    push_column_titles, push_craft_placeholder, push_equipment_placeholder, push_filter_text,
    push_footer_keys, push_header_text, push_held_quantity, push_quantity_badges, push_search_text,
    push_sort_text, push_weight_text, total_weight_kg,
};
use super::Renderer;
use crate::ui::{
    ComponentState, InventoryButton, InventoryLayout, InventoryUiState, UiColor, UiRect, UiTheme,
    UiViewport,
};
use crate::Vertex;
use vv_gameplay::{quick_craft_recipe_indices, Hotbar, HotbarSlot, Inventory, SlotRef};
use vv_pack_compiler::{CompiledIngredient, CompiledRecipe, CompiledRecipeKind, RecipeRegistry};
use vv_world::PlanetData;

pub(super) struct InventoryTextSpec {
    pub text: String,
    pub left: f32,
    pub top: f32,
    pub size: f32,
    pub color: [u8; 3],
}

// =============================================================================
// Public entry points
// =============================================================================

impl<'a> Renderer<'a> {
    pub fn update_inventory_mesh(
        &mut self,
        inventory: &Inventory,
        hotbar: &Hotbar,
        ui: &InventoryUiState,
        planet: &PlanetData,
        recipes: &RecipeRegistry,
    ) {
        if !ui.is_open {
            self.inventory_inds = 0;
            return;
        }

        let theme = UiTheme::VOXELVERSE;
        let viewport = UiViewport::new(self.config.width as f32, self.config.height as f32);
        let layout = InventoryLayout::compute(&theme, viewport, ui.user_zoom);

        let mut verts: Vec<Vertex> = Vec::with_capacity(2048);
        let mut inds: Vec<u32> = Vec::with_capacity(3072);

        self.draw_scrim(&mut verts, &mut inds, &theme, viewport, &layout);
        self.draw_modal_window(&mut verts, &mut inds, &theme, &layout);
        self.draw_header(&mut verts, &mut inds, &theme, &layout, ui);
        self.draw_equipment_panel(&mut verts, &mut inds, &theme, &layout);
        self.draw_craft_panel(&mut verts, &mut inds, &theme, &layout, ui, planet, recipes);
        self.draw_search_bar(&mut verts, &mut inds, &theme, &layout, ui);
        self.draw_filter_chips(&mut verts, &mut inds, &theme, &layout, ui);
        self.draw_inventory_grid(
            &mut verts, &mut inds, &theme, &layout, ui, inventory, planet,
        );
        self.draw_hotbar_mirror(&mut verts, &mut inds, &theme, &layout, hotbar, planet);
        self.draw_hotbar_hover_ring(&mut verts, &mut inds, &theme, &layout, ui);
        self.draw_footer(&mut verts, &mut inds, &theme, &layout, ui, inventory);
        self.draw_held_stack(&mut verts, &mut inds, &theme, &layout, ui, planet);

        self.queue
            .write_buffer(&self.inventory_v_buf, 0, bytemuck::cast_slice(&verts));
        self.queue
            .write_buffer(&self.inventory_i_buf, 0, bytemuck::cast_slice(&inds));
        self.inventory_inds = inds.len() as u32;
    }

    pub(super) fn inventory_text_specs(
        &self,
        inventory: &Inventory,
        hotbar: &Hotbar,
        ui: &InventoryUiState,
        planet: &PlanetData,
        recipes: &RecipeRegistry,
    ) -> Vec<InventoryTextSpec> {
        if !ui.is_open {
            return Vec::new();
        }
        let theme = UiTheme::VOXELVERSE;
        let viewport = UiViewport::new(self.config.width as f32, self.config.height as f32);
        let layout = InventoryLayout::compute(&theme, viewport, ui.user_zoom);
        let mut specs = Vec::new();

        push_header_text(&mut specs, &theme, &layout);
        push_column_titles(&mut specs, &theme, &layout);
        push_search_text(&mut specs, &theme, &layout, ui);
        push_filter_text(&mut specs, &theme, &layout, ui);
        push_quantity_badges(&mut specs, &theme, &layout, inventory);
        push_weight_text(&mut specs, &theme, &layout, ui, inventory, hotbar);
        push_sort_text(&mut specs, &theme, &layout);
        push_equipment_placeholder(&mut specs, &theme, &layout);
        push_craft_placeholder(&mut specs, &theme, &layout, ui, planet, recipes);
        push_held_quantity(&mut specs, &theme, &layout, ui);
        push_footer_keys(&mut specs, &theme, &layout);

        specs
    }
}

pub(super) fn selected_recipe_index(
    ui: &InventoryUiState,
    recipes: &RecipeRegistry,
) -> Option<usize> {
    let indices = quick_craft_recipe_indices(recipes);
    ui.selected_recipe
        .filter(|selected| indices.contains(selected))
        .or_else(|| indices.first().copied())
}

pub(super) fn recipe_ingredient_counts(recipe: &CompiledRecipe) -> Vec<(CompiledIngredient, u32)> {
    let ingredients: Vec<CompiledIngredient> = match &recipe.kind {
        CompiledRecipeKind::Shaped(shaped) => shaped.grid.iter().filter_map(Clone::clone).collect(),
        CompiledRecipeKind::Shapeless(shapeless) => shapeless.ingredients.clone(),
        CompiledRecipeKind::Smelting(_) => Vec::new(),
    };

    let mut counted: Vec<(CompiledIngredient, u32)> = Vec::new();
    for ingredient in ingredients {
        if let Some((_, count)) = counted
            .iter_mut()
            .find(|(existing, _)| same_ingredient(existing, &ingredient))
        {
            *count += 1;
        } else {
            counted.push((ingredient, 1));
        }
    }
    counted
}

fn same_ingredient(a: &CompiledIngredient, b: &CompiledIngredient) -> bool {
    match (a, b) {
        (CompiledIngredient::Item(a), CompiledIngredient::Item(b)) => a == b,
        (CompiledIngredient::Tag(a), CompiledIngredient::Tag(b)) => a == b,
        _ => false,
    }
}

// =============================================================================
// Sub-drawers
// =============================================================================

impl<'a> Renderer<'a> {
    /// Dim everything between the top of the screen and just above the
    /// in-game hotbar. Skips the hotbar zone so it stays fully visible.
    fn draw_scrim(
        &self,
        _verts: &mut Vec<Vertex>,
        _inds: &mut Vec<u32>,
        _theme: &UiTheme,
        _viewport: UiViewport,
        _layout: &InventoryLayout,
    ) {
        // Fully transparent — no scrim drawn.
    }

    /// Modal window — dark background at 0.3 opacity so the world shows
    /// through while text/slots stay readable.
    fn draw_modal_window(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        _theme: &UiTheme,
        layout: &InventoryLayout,
    ) {
        let radius = 14.0 * layout.scale;
        let fill = UiColor::rgba(0.03, 0.04, 0.05, 0.30);
        self.fill_rounded_rect(verts, inds, layout.modal, fill, radius);
    }

    /// Header strip — bag icon, title row separator, close button. Titles
    /// themselves are emitted as text specs.
    fn draw_header(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        theme: &UiTheme,
        layout: &InventoryLayout,
        ui: &InventoryUiState,
    ) {
        // A thin warm divider under the header.
        let divider = UiRect {
            x: layout.header_strip.x + 10.0 * layout.scale,
            y: layout.header_strip.y + layout.header_strip.h - 1.0,
            w: layout.header_strip.w - 20.0 * layout.scale,
            h: (1.0 * layout.scale).max(1.0),
        };
        self.fill_rect(verts, inds, divider, theme.panel.border);

        // Bag icon (drawn from primitives, no external assets).
        self.draw_bag_icon(verts, inds, layout.bag_icon, theme, layout.scale);

        // Close button.
        let close_state = self.inventory_button_state(ui, InventoryButton::Close);
        self.draw_round_button(
            verts,
            inds,
            layout.close_button,
            close_state,
            theme,
            layout.scale,
        );
        let stroke = (3.0 * layout.scale).max(2.0);
        let inset = layout.close_button.w * 0.30;
        let color = theme.button.text_for(close_state);
        self.draw_diagonal(
            verts,
            inds,
            layout.close_button,
            inset,
            stroke,
            color,
            false,
        );
        self.draw_diagonal(verts, inds, layout.close_button, inset, stroke, color, true);
    }

    fn draw_equipment_panel(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        theme: &UiTheme,
        layout: &InventoryLayout,
    ) {
        let radius = 10.0 * layout.scale;
        self.fill_rounded_rect(
            verts,
            inds,
            layout.left_panel,
            theme.panel.fill.scale_rgb(1.35),
            radius,
        );
        self.stroke_rounded_rect(
            verts,
            inds,
            layout.left_panel,
            theme.panel.border,
            (1.0 * layout.scale).max(1.0),
            radius,
        );
        // Three equipment slots (Casque / Plastron / Bottes) left-aligned
        // inside the panel — each slot drawn with the standard inventory-slot
        // style so the visual language is consistent.
        let slots = equip_slot_rects(layout.left_panel, layout.scale);
        for (rect, _) in &slots {
            let slot_r = 7.0 * layout.scale;
            let inset = (theme.slot.inner_inset * layout.scale).max(2.0);
            self.fill_rounded_rect(verts, inds, *rect, theme.slot.border_empty, slot_r);
            let inner = UiRect {
                x: rect.x + inset,
                y: rect.y + inset,
                w: rect.w - inset * 2.0,
                h: rect.h - inset * 2.0,
            };
            self.fill_rounded_rect(
                verts,
                inds,
                inner,
                theme.slot.fill_empty,
                (slot_r - inset).max(2.0),
            );
        }
    }

    fn draw_craft_panel(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        theme: &UiTheme,
        layout: &InventoryLayout,
        ui: &InventoryUiState,
        planet: &PlanetData,
        recipes: &RecipeRegistry,
    ) {
        let radius = 10.0 * layout.scale;
        self.fill_rounded_rect(
            verts,
            inds,
            layout.right_panel,
            theme.panel.fill.scale_rgb(1.35),
            radius,
        );
        self.stroke_rounded_rect(
            verts,
            inds,
            layout.right_panel,
            theme.panel.border,
            (1.0 * layout.scale).max(1.0),
            radius,
        );

        let indices = quick_craft_recipe_indices(recipes);
        for (row, recipe_index) in layout.craft_recipe_rows.iter().zip(indices.iter().copied()) {
            let selected = Some(recipe_index) == selected_recipe_index(ui, recipes);
            let hovered = ui.hovered_recipe == Some(recipe_index);
            let state = if selected {
                ComponentState::Selected
            } else if hovered {
                ComponentState::Hovered
            } else {
                ComponentState::Normal
            };
            self.draw_recipe_row(verts, inds, *row, state, theme, layout.scale);
            if let Some(recipe) = recipes.recipes().get(recipe_index) {
                let icon = UiRect {
                    x: row.x + 8.0 * layout.scale,
                    y: row.y + 6.0 * layout.scale,
                    w: row.h - 12.0 * layout.scale,
                    h: row.h - 12.0 * layout.scale,
                };
                self.draw_inventory_slot(
                    verts,
                    inds,
                    icon,
                    Some(HotbarSlot {
                        item_id: recipe.output_item,
                        quantity: recipe.output_count,
                    }),
                    ComponentState::Normal,
                    planet,
                    theme,
                    layout.scale * 0.78,
                );
            }
        }

        self.fill_rounded_rect(
            verts,
            inds,
            layout.craft_detail_panel,
            theme.search_bar.fill,
            8.0 * layout.scale,
        );
        self.stroke_rounded_rect(
            verts,
            inds,
            layout.craft_detail_panel,
            theme.slot.border_empty,
            (1.0 * layout.scale).max(1.0),
            8.0 * layout.scale,
        );

        let Some(recipe_index) = selected_recipe_index(ui, recipes) else {
            return;
        };
        let Some(recipe) = recipes.recipes().get(recipe_index) else {
            return;
        };

        self.draw_inventory_slot(
            verts,
            inds,
            layout.craft_output_slot,
            Some(HotbarSlot {
                item_id: recipe.output_item,
                quantity: recipe.output_count.saturating_mul(ui.craft_quantity),
            }),
            ComponentState::Selected,
            planet,
            theme,
            layout.scale,
        );

        for (row, ingredient) in layout
            .craft_ingredient_rows
            .iter()
            .zip(recipe_ingredient_counts(recipe))
        {
            self.fill_rounded_rect(verts, inds, *row, theme.slot.fill_empty, 5.0 * layout.scale);
            if let CompiledIngredient::Item(item_id) = ingredient.0 {
                let icon = UiRect {
                    x: row.x + 4.0 * layout.scale,
                    y: row.y + 3.0 * layout.scale,
                    w: row.h - 6.0 * layout.scale,
                    h: row.h - 6.0 * layout.scale,
                };
                self.draw_inventory_slot(
                    verts,
                    inds,
                    icon,
                    Some(HotbarSlot {
                        item_id,
                        quantity: ingredient.1.saturating_mul(ui.craft_quantity),
                    }),
                    ComponentState::Normal,
                    planet,
                    theme,
                    layout.scale * 0.55,
                );
            }
        }

        self.draw_action_button(
            verts,
            inds,
            layout.craft_quantity_down,
            self.inventory_button_state(ui, InventoryButton::CraftQuantityDown),
            theme,
            layout.scale,
        );
        self.draw_action_button(
            verts,
            inds,
            layout.craft_quantity_up,
            self.inventory_button_state(ui, InventoryButton::CraftQuantityUp),
            theme,
            layout.scale,
        );
        self.draw_action_button(
            verts,
            inds,
            layout.craft_max_button,
            self.inventory_button_state(ui, InventoryButton::CraftMax),
            theme,
            layout.scale,
        );
        self.fill_rounded_rect(
            verts,
            inds,
            layout.craft_quantity_value,
            theme.search_bar.fill,
            8.0 * layout.scale,
        );
        self.stroke_rounded_rect(
            verts,
            inds,
            layout.craft_quantity_value,
            theme.search_bar.border,
            (1.0 * layout.scale).max(1.0),
            8.0 * layout.scale,
        );
        self.draw_action_button(
            verts,
            inds,
            layout.craft_button,
            self.inventory_button_state(ui, InventoryButton::Craft),
            theme,
            layout.scale,
        );
    }

    fn draw_search_bar(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        theme: &UiTheme,
        layout: &InventoryLayout,
        ui: &InventoryUiState,
    ) {
        let radius = 8.0 * layout.scale;
        let fill = if ui.search_focused {
            theme.search_bar.fill_focused
        } else {
            theme.search_bar.fill
        };
        let border = if ui.search_focused {
            theme.search_bar.border_focused
        } else {
            theme.search_bar.border
        };
        self.fill_rounded_rect(verts, inds, layout.search_bar, fill, radius);
        self.stroke_rounded_rect(
            verts,
            inds,
            layout.search_bar,
            border,
            (1.5 * layout.scale).max(1.0),
            radius,
        );

        // Magnifier icon: circle + handle stroke.
        self.draw_magnifier(
            verts,
            inds,
            layout.search_icon,
            theme.search_bar.icon,
            layout.scale,
        );

        // Clear button when there is text.
        if !ui.search_query.is_empty() {
            let clear_state = self.inventory_button_state(ui, InventoryButton::ClearSearch);
            let cb = layout.clear_search_button;
            let cr = cb.h * 0.5;
            self.fill_rounded_rect(verts, inds, cb, theme.button.fill_for(clear_state), cr);
            self.stroke_rounded_rect(
                verts,
                inds,
                cb,
                theme.button.border,
                (1.0 * layout.scale).max(1.0),
                cr,
            );
            let s = (2.0 * layout.scale).max(1.5);
            let inset = cb.w * 0.30;
            let color = theme.button.text_for(clear_state);
            self.draw_diagonal(verts, inds, cb, inset, s, color, false);
            self.draw_diagonal(verts, inds, cb, inset, s, color, true);
        }
    }

    fn draw_filter_chips(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        theme: &UiTheme,
        layout: &InventoryLayout,
        ui: &InventoryUiState,
    ) {
        for (filter, rect) in &layout.filter_chips {
            let selected = *filter == ui.active_filter;
            let hovered = ui.hovered_filter == Some(*filter);
            let (fill, border) = match (selected, hovered) {
                (true, _) => (
                    theme.filter_chip.fill_selected,
                    theme.filter_chip.border_selected,
                ),
                (false, true) => (
                    theme.filter_chip.fill_hovered,
                    theme.filter_chip.border_hovered,
                ),
                (false, false) => (theme.filter_chip.fill, theme.filter_chip.border),
            };
            let radius = rect.h * 0.5;
            self.fill_rounded_rect(verts, inds, *rect, fill, radius);
            self.stroke_rounded_rect(
                verts,
                inds,
                *rect,
                border,
                if selected {
                    1.5 * layout.scale
                } else {
                    1.0 * layout.scale
                }
                .max(1.0),
                radius,
            );
        }
    }

    fn draw_inventory_grid(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        theme: &UiTheme,
        layout: &InventoryLayout,
        ui: &InventoryUiState,
        inventory: &Inventory,
        planet: &PlanetData,
    ) {
        let slots = inventory.slots();
        for (i, rect) in layout.inventory_slots.iter().enumerate() {
            let slot = slots[i];
            let visible = slot
                .map(|s| {
                    let item = planet.items.get(s.item_id);
                    let name = item.map(|i| i.display_name.as_str()).unwrap_or("");
                    let category = item.map(|i| i.category.as_str()).unwrap_or("");
                    ui.matches_search(name) && ui.matches_filter(category)
                })
                .unwrap_or(true);
            let is_hovered = matches!(ui.hovered_slot, Some(SlotRef::Inventory(idx)) if idx == i);
            let state = self.slot_state(slot, visible, is_hovered, false);
            self.draw_inventory_slot(verts, inds, *rect, slot, state, planet, theme, layout.scale);

            if let Some(slot) = slot {
                if slot.quantity > 1 {
                    self.draw_quantity_badge(
                        verts,
                        inds,
                        *rect,
                        slot.quantity,
                        theme,
                        layout.scale,
                    );
                }
            }
        }
    }

    fn draw_hotbar_mirror(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        theme: &UiTheme,
        layout: &InventoryLayout,
        hotbar: &vv_gameplay::Hotbar,
        planet: &PlanetData,
    ) {
        let slots = hotbar.slots();
        for (i, rect) in layout.hotbar_slots.iter().enumerate() {
            let content = slots[i];
            let selected = i == hotbar.selected_index();
            let state = if selected {
                crate::ui::ComponentState::Selected
            } else if content.is_none() {
                crate::ui::ComponentState::Empty
            } else {
                crate::ui::ComponentState::Normal
            };
            self.draw_inventory_slot(
                verts,
                inds,
                *rect,
                content,
                state,
                planet,
                theme,
                layout.hotbar_slot_size / layout.inventory_slot_size * layout.scale,
            );
        }
    }

    fn draw_hotbar_hover_ring(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        theme: &UiTheme,
        layout: &InventoryLayout,
        ui: &InventoryUiState,
    ) {
        if let Some(SlotRef::Hotbar(hover_idx)) = ui.hovered_slot {
            if let Some(rect) = layout.hotbar_slots.get(hover_idx) {
                self.stroke_rounded_rect(
                    verts,
                    inds,
                    *rect,
                    theme.slot.border_hovered,
                    (2.5 * layout.scale).max(2.0),
                    6.0 * layout.scale,
                );
            }
        }
    }

    /// Footer = separator line, weight bar (track + filled portion + label),
    /// sort button.
    fn draw_footer(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        theme: &UiTheme,
        layout: &InventoryLayout,
        ui: &InventoryUiState,
        inventory: &Inventory,
    ) {
        // Subtle separator above the weight area.
        self.fill_rect(verts, inds, layout.center_separator, theme.panel.border);

        // Weight bar — track + filled portion.
        let radius = layout.weight_bar_track.h * 0.5;
        self.fill_rounded_rect(
            verts,
            inds,
            layout.weight_bar_track,
            theme.slot.fill_empty,
            radius,
        );
        let total_kg = total_weight_kg(inventory, ui);
        let cap = ui.capacity_kg.max(1.0);
        let fill_ratio = (total_kg / cap).clamp(0.0, 1.0);
        let alert = fill_ratio > 0.85;
        if fill_ratio > 0.005 {
            let filled = UiRect {
                x: layout.weight_bar_track.x,
                y: layout.weight_bar_track.y,
                w: layout.weight_bar_track.w * fill_ratio,
                h: layout.weight_bar_track.h,
            };
            let color = if alert {
                theme.slot.border_alert
            } else {
                theme.panel.border_strong
            };
            self.fill_rounded_rect(verts, inds, filled, color, radius);
        }

        // Sort button.
        let sort_state = self.inventory_button_state(ui, InventoryButton::Sort);
        self.draw_action_button(
            verts,
            inds,
            layout.sort_button,
            sort_state,
            theme,
            layout.scale,
        );
    }

    fn draw_held_stack(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        theme: &UiTheme,
        layout: &InventoryLayout,
        ui: &InventoryUiState,
        planet: &PlanetData,
    ) {
        let Some(held) = ui.held else {
            return;
        };
        let size = layout.inventory_slot_size * 0.85;
        let ghost = UiRect {
            x: ui.cursor.0 - size * 0.5,
            y: ui.cursor.1 - size * 0.5,
            w: size,
            h: size,
        };
        let radius = 6.0 * layout.scale;
        // Shadow.
        let mut shadow = ghost;
        shadow.x += 4.0;
        shadow.y += 4.0;
        self.fill_rounded_rect(verts, inds, shadow, theme.panel.shadow, radius);
        // Body.
        self.fill_rounded_rect(verts, inds, ghost, theme.slot.fill_selected, radius);
        self.stroke_rounded_rect(
            verts,
            inds,
            ghost,
            theme.slot.border_selected,
            theme.slot.selected_border_width.max(2.0) * layout.scale,
            radius,
        );
        match planet.resolve_item_voxel(held.stack.item_id) {
            Some(v) => self.draw_iso_block(
                verts,
                inds,
                ghost,
                planet.content.color(v),
                1.0,
                Some(planet.content.visual(v).layers),
            ),
            None => {
                if let Some(item) = planet.items.get(held.stack.item_id) {
                    self.draw_item_glyph(
                        verts,
                        inds,
                        ghost,
                        item.category.as_str(),
                        &item.gameplay,
                        1.0,
                    );
                }
            }
        }
    }
}
