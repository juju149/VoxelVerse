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

use super::Renderer;
use crate::ui::{
    ComponentState, InventoryButton, InventoryLayout, InventoryUiState, UiColor, UiRect, UiTheme,
    UiViewport,
};
use crate::Vertex;
use vv_gameplay::{Hotbar, Inventory, SlotRef};
use vv_pack_compiler::BlockMaterialLayers;
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
        self.draw_craft_panel(&mut verts, &mut inds, &theme, &layout);
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
        push_craft_placeholder(&mut specs, &theme, &layout);
        push_held_quantity(&mut specs, &theme, &layout, ui);
        push_footer_keys(&mut specs, &theme, &layout);

        let _ = planet;
        specs
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
        // Compact 2×2 crafting grid placeholder so the panel reads
        // as "craft" at a glance, without wasting vertical space.
        let grid = craft_grid_rect(layout.right_panel, layout.scale);
        let mini_slot = (layout.right_panel.w * 0.22).min(36.0 * layout.scale);
        let mini_gap = mini_slot * 0.20;
        let mini_r = 5.0 * layout.scale;
        for row in 0..2_u32 {
            for col in 0..2_u32 {
                let cell = UiRect {
                    x: grid.x + col as f32 * (mini_slot + mini_gap),
                    y: grid.y + row as f32 * (mini_slot + mini_gap),
                    w: mini_slot,
                    h: mini_slot,
                };
                self.fill_rounded_rect(verts, inds, cell, theme.slot.fill_empty, mini_r);
                self.stroke_rounded_rect(
                    verts,
                    inds,
                    cell,
                    theme.slot.border_empty,
                    (1.0 * layout.scale).max(1.0),
                    mini_r,
                );
            }
        }
        // Arrow → output slot below the grid.
        let arrow_cx = grid.x + grid.w * 0.5;
        let arrow_top = grid.y + grid.h + mini_gap * 1.5;
        let arrow_bot = arrow_top + mini_slot * 0.55;
        self.draw_line_thick(
            verts,
            inds,
            arrow_cx,
            arrow_top,
            arrow_cx,
            arrow_bot,
            (2.0 * layout.scale).max(1.5),
            theme.panel.border,
        );
        let out_size = mini_slot * 1.15;
        let out_slot = UiRect {
            x: arrow_cx - out_size * 0.5,
            y: arrow_bot + mini_gap,
            w: out_size,
            h: out_size,
        };
        self.fill_rounded_rect(verts, inds, out_slot, theme.slot.fill_empty, mini_r * 1.2);
        self.stroke_rounded_rect(
            verts,
            inds,
            out_slot,
            theme.slot.border_empty,
            (1.0 * layout.scale).max(1.0),
            mini_r * 1.2,
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
        let (base, layers) = match planet.resolve_item_voxel(held.stack.item_id) {
            Some(v) => (
                planet.content.color(v),
                Some(planet.content.visual(v).layers),
            ),
            None => ([0.6, 0.6, 0.6], None),
        };
        self.draw_iso_block(verts, inds, ghost, base, 1.0, layers);
    }
}

// =============================================================================
// Component primitives
// =============================================================================

impl<'a> Renderer<'a> {
    /// Stylized bag silhouette. No external asset.
    fn draw_bag_icon(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        theme: &UiTheme,
        scale: f32,
    ) {
        let gold = theme.panel.border_strong;
        let dark = theme.panel.fill;
        let cx = rect.x + rect.w * 0.5;
        let body_w = rect.w * 0.84;
        let body_h = rect.h * 0.62;
        let body = UiRect {
            x: cx - body_w * 0.5,
            y: rect.y + rect.h * 0.32,
            w: body_w,
            h: body_h,
        };
        self.fill_rounded_rect(verts, inds, body, gold, body.h * 0.22);
        let body_inner_inset = (3.0 * scale).max(2.0);
        let body_inner = UiRect {
            x: body.x + body_inner_inset,
            y: body.y + body_inner_inset,
            w: body.w - body_inner_inset * 2.0,
            h: body.h - body_inner_inset * 2.0,
        };
        self.fill_rounded_rect(verts, inds, body_inner, dark, body_inner.h * 0.22);

        // Strap arc above the bag body.
        let strap_w = rect.w * 0.46;
        let strap_h = rect.h * 0.22;
        let strap_outer = UiRect {
            x: cx - strap_w * 0.5,
            y: rect.y + rect.h * 0.10,
            w: strap_w,
            h: strap_h,
        };
        self.fill_rounded_rect(verts, inds, strap_outer, gold, strap_outer.h * 0.5);
        let strap_inner_inset = (3.0 * scale).max(2.0);
        let strap_inner = UiRect {
            x: strap_outer.x + strap_inner_inset,
            y: strap_outer.y + strap_inner_inset,
            w: strap_outer.w - strap_inner_inset * 2.0,
            h: strap_outer.h - strap_inner_inset * 2.0,
        };
        self.fill_rounded_rect(verts, inds, strap_inner, dark, strap_inner.h * 0.5);

        // Bag bottom: mask the lower half of the strap so it really sits
        // BEHIND the bag body.
        let mask = UiRect {
            x: strap_outer.x,
            y: body.y,
            w: strap_outer.w,
            h: strap_outer.h * 0.7,
        };
        self.fill_rect(verts, inds, mask, dark);

        // A small accent line on the flap.
        let flap = UiRect {
            x: body.x + body.w * 0.18,
            y: body.y + body.h * 0.22,
            w: body.w * 0.64,
            h: (2.0 * scale).max(1.5),
        };
        self.fill_rect(verts, inds, flap, gold);
    }

    fn draw_magnifier(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        color: UiColor,
        scale: f32,
    ) {
        let r = rect.w * 0.32;
        let cx = rect.x + rect.w * 0.4;
        let cy = rect.y + rect.h * 0.4;
        self.fill_circle(verts, inds, cx, cy, r, color);
        let inset = (2.0 * scale).max(1.5);
        // Drill out the centre so it reads as a ring.
        // (Use the search bar fill so it blends in regardless of focus.)
        // We approximate by drawing a smaller darker circle on top.
        let inner = UiColor::rgba(
            color.rgb[0] * 0.10,
            color.rgb[1] * 0.10,
            color.rgb[2] * 0.10,
            1.0,
        );
        let _ = inner;
        // Instead of carving a hole (which would need stencil), stroke the
        // ring with a small inner darker fill.
        self.stroke_circle(verts, inds, cx, cy, r, (2.0 * scale).max(1.5), color);
        // Handle: diagonal short line from the bottom-right of the circle.
        let hx = cx + r * 0.70;
        let hy = cy + r * 0.70;
        let h2x = hx + r * 0.65;
        let h2y = hy + r * 0.65;
        let _ = inset;
        self.draw_line_thick(verts, inds, hx, hy, h2x, h2y, (2.5 * scale).max(2.0), color);
    }

    fn draw_round_button(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        state: ComponentState,
        theme: &UiTheme,
        scale: f32,
    ) {
        let radius = 8.0 * scale;
        let fill = theme.button.fill_for(state);
        let border = match state {
            ComponentState::Hovered => theme.button.border_hovered,
            ComponentState::Disabled => theme.button.border_disabled,
            _ => theme.button.border,
        };
        self.fill_rounded_rect(verts, inds, rect, fill, radius);
        self.stroke_rounded_rect(verts, inds, rect, border, (1.5 * scale).max(1.0), radius);
    }

    fn draw_action_button(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        state: ComponentState,
        theme: &UiTheme,
        scale: f32,
    ) {
        let radius = 8.0 * scale;
        let fill = theme.button.fill_for(state);
        let border = match state {
            ComponentState::Hovered => theme.button.border_hovered,
            ComponentState::Disabled => theme.button.border_disabled,
            _ => theme.button.border,
        };
        self.fill_rounded_rect(verts, inds, rect, fill, radius);
        self.stroke_rounded_rect(verts, inds, rect, border, (1.5 * scale).max(1.0), radius);
    }

    pub(super) fn draw_inventory_slot(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        content: Option<vv_gameplay::HotbarSlot>,
        state: ComponentState,
        planet: &PlanetData,
        theme: &UiTheme,
        scale: f32,
    ) {
        let radius = 6.0 * scale;
        let border = theme.slot.border_for(state);
        let fill = theme.slot.fill_for(state);
        self.fill_rounded_rect(verts, inds, rect, border, radius);
        let inner_inset = (theme.slot.inner_inset * scale).max(2.0);
        let inner = UiRect {
            x: rect.x + inner_inset,
            y: rect.y + inner_inset,
            w: rect.w - inner_inset * 2.0,
            h: rect.h - inner_inset * 2.0,
        };
        self.fill_rounded_rect(verts, inds, inner, fill, (radius - inner_inset).max(2.0));
        if state == ComponentState::Selected {
            self.stroke_rounded_rect(
                verts,
                inds,
                rect,
                theme.slot.border_selected,
                theme.slot.selected_border_width * scale,
                radius,
            );
        }
        if let Some(slot) = content {
            let (base_color, layers) = match planet.resolve_item_voxel(slot.item_id) {
                Some(voxel) => (
                    planet.content.color(voxel),
                    Some(planet.content.visual(voxel).layers),
                ),
                None => ([0.6, 0.6, 0.6], None),
            };
            let dim = if state == ComponentState::Disabled {
                0.45
            } else {
                1.0
            };
            self.draw_iso_block(verts, inds, rect, base_color, dim, layers);
        }
    }

    pub(super) fn draw_quantity_badge(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        slot_rect: UiRect,
        quantity: u32,
        theme: &UiTheme,
        scale: f32,
    ) {
        let font = theme.quantity_badge.font_size * scale;
        let digits = quantity.to_string().chars().count() as f32;
        let badge_w = (digits * font * 0.6 + 10.0 * scale).max(font * 1.4);
        let badge_h = font + 6.0 * scale;
        let x = slot_rect.x + slot_rect.w - 4.0 * scale - badge_w;
        let y = slot_rect.y + slot_rect.h - 3.0 * scale - badge_h;
        let badge = UiRect {
            x,
            y,
            w: badge_w,
            h: badge_h,
        };
        self.fill_rounded_rect(
            verts,
            inds,
            badge,
            theme.quantity_badge.shadow,
            badge.h * 0.30,
        );
    }
}

// =============================================================================
// Text spec helpers
// =============================================================================

fn push_header_text(specs: &mut Vec<InventoryTextSpec>, theme: &UiTheme, layout: &InventoryLayout) {
    specs.push(InventoryTextSpec {
        text: "INVENTAIRE".to_string(),
        left: layout.title_origin.0,
        top: layout.title_origin.1,
        size: theme.text.title_size * layout.scale,
        color: theme.text.title.as_rgb8(),
    });
    specs.push(InventoryTextSpec {
        text: "Sac, equipement et ressources".to_string(),
        left: layout.subtitle_origin.0,
        top: layout.subtitle_origin.1,
        size: theme.text.muted_size * layout.scale,
        color: theme.text.muted.as_rgb8(),
    });
}

fn push_column_titles(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
) {
    let size = theme.text.section_size * layout.scale;
    specs.push(InventoryTextSpec {
        text: "EQUIPEMENT".to_string(),
        left: layout.left_title_origin.0,
        top: layout.left_title_origin.1,
        size,
        color: theme.text.section.as_rgb8(),
    });
    specs.push(InventoryTextSpec {
        text: "SAC A DOS".to_string(),
        left: layout.center_title_origin.0,
        top: layout.center_title_origin.1,
        size,
        color: theme.text.section.as_rgb8(),
    });
    specs.push(InventoryTextSpec {
        text: "ARTISANAT RAPIDE".to_string(),
        left: layout.right_title_origin.0,
        top: layout.right_title_origin.1,
        size,
        color: theme.text.section.as_rgb8(),
    });
}

fn push_search_text(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
    ui: &InventoryUiState,
) {
    let (text, color) = if ui.search_query.is_empty() {
        (
            "Rechercher un objet...".to_string(),
            theme.search_bar.placeholder.as_rgb8(),
        )
    } else {
        let suffix = if ui.search_focused { "_" } else { "" };
        (
            format!("{}{}", ui.search_query, suffix),
            theme.search_bar.text.as_rgb8(),
        )
    };
    specs.push(InventoryTextSpec {
        text,
        left: layout.search_text_origin.0,
        top: layout.search_text_origin.1,
        size: theme.text.body_size * layout.scale,
        color,
    });
}

fn push_filter_text(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
    ui: &InventoryUiState,
) {
    let size = theme.text.control_size * layout.scale;
    for (filter, rect) in &layout.filter_chips {
        let selected = *filter == ui.active_filter;
        let color = if selected {
            theme.filter_chip.text_selected
        } else {
            theme.filter_chip.text
        };
        // Center label horizontally inside the chip.
        let label = filter.label();
        let est_w = label.chars().count() as f32 * size * 0.62;
        let left = rect.x + (rect.w - est_w) * 0.5;
        let top = rect.y + (rect.h - size) * 0.5 - 2.0;
        specs.push(InventoryTextSpec {
            text: label.to_string(),
            left,
            top,
            size,
            color: color.as_rgb8(),
        });
    }
}

fn push_quantity_badges(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
    inventory: &Inventory,
) {
    let font = theme.quantity_badge.font_size * layout.scale;
    for (i, rect) in layout.inventory_slots.iter().enumerate() {
        let Some(slot) = inventory.slot(i) else {
            continue;
        };
        if slot.quantity <= 1 {
            continue;
        }
        let digits = slot.quantity.to_string().chars().count() as f32;
        let badge_w = (digits * font * 0.6 + 10.0 * layout.scale).max(font * 1.4);
        let badge_h = font + 6.0 * layout.scale;
        let badge_x = rect.x + rect.w - 4.0 * layout.scale - badge_w;
        let badge_y = rect.y + rect.h - 3.0 * layout.scale - badge_h;
        specs.push(InventoryTextSpec {
            text: slot.quantity.to_string(),
            left: badge_x + 5.0 * layout.scale,
            top: badge_y + 2.0 * layout.scale,
            size: font,
            color: theme.quantity_badge.text.as_rgb8(),
        });
    }
}

fn push_weight_text(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
    ui: &InventoryUiState,
    inventory: &Inventory,
    _hotbar: &Hotbar,
) {
    let total_kg = total_weight_kg(inventory, ui);
    let label = format!("{:.1} / {:.0} kg", total_kg, ui.capacity_kg);
    specs.push(InventoryTextSpec {
        text: label,
        left: layout.weight_bar_label_origin.0,
        top: layout.weight_bar_label_origin.1,
        size: theme.text.muted_size * layout.scale,
        color: theme.text.muted.as_rgb8(),
    });
}

fn push_sort_text(specs: &mut Vec<InventoryTextSpec>, theme: &UiTheme, layout: &InventoryLayout) {
    let size = theme.text.control_size * layout.scale;
    let label = "Trier";
    let est_w = label.chars().count() as f32 * size * 0.6;
    specs.push(InventoryTextSpec {
        text: label.to_string(),
        left: layout.sort_button.x + (layout.sort_button.w - est_w) * 0.5,
        top: layout.sort_text_origin.1,
        size,
        color: theme.button.text.as_rgb8(),
    });
}

fn push_equipment_placeholder(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
) {
    // Label each equipment slot to its right: Casque / Plastron / Bottes.
    let size = theme.text.muted_size * layout.scale;
    let slots = equip_slot_rects(layout.left_panel, layout.scale);
    for (rect, label) in &slots {
        let lx = rect.x + rect.w + 8.0 * layout.scale;
        let ly = rect.y + (rect.h - size) * 0.5;
        specs.push(InventoryTextSpec {
            text: label.to_string(),
            left: lx,
            top: ly,
            size,
            color: theme.text.muted.as_rgb8(),
        });
    }
    // Suppress unused-field warning.
    let _ = layout.equipment_placeholder_origin;
}

fn push_craft_placeholder(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
) {
    // Position text below the 2×2 craft grid that is drawn in the mesh path.
    let grid = craft_grid_rect(layout.right_panel, layout.scale);
    // The grid height plus arrow + output slot ≈ grid.h * 2.2
    let content_bottom = grid.y + grid.h * 2.5;

    let main = "Bientot disponible";
    let main_size = theme.text.body_size * layout.scale;
    let main_w = main.chars().count() as f32 * main_size * 0.6;
    let cx = layout.right_panel.x + layout.right_panel.w * 0.5;
    specs.push(InventoryTextSpec {
        text: main.to_string(),
        left: cx - main_w * 0.5,
        top: content_bottom + 8.0 * layout.scale,
        size: main_size,
        color: theme.text.body.as_rgb8(),
    });

    let sub = "Recettes a venir";
    let sub_size = theme.text.muted_size * layout.scale;
    let sub_w = sub.chars().count() as f32 * sub_size * 0.6;
    specs.push(InventoryTextSpec {
        text: sub.to_string(),
        left: cx - sub_w * 0.5,
        top: content_bottom + 8.0 * layout.scale + main_size + 4.0 * layout.scale,
        size: sub_size,
        color: theme.text.muted.as_rgb8(),
    });
    // Suppress unused-field warnings.
    let _ = layout.craft_placeholder_origin;
    let _ = layout.craft_subtitle_origin;
}

fn push_held_quantity(
    specs: &mut Vec<InventoryTextSpec>,
    theme: &UiTheme,
    layout: &InventoryLayout,
    ui: &InventoryUiState,
) {
    let Some(held) = ui.held else { return };
    if held.stack.quantity <= 1 {
        return;
    }
    let ghost = layout.inventory_slot_size * 0.85;
    let font = theme.quantity_badge.font_size * layout.scale;
    let digits = held.stack.quantity.to_string().chars().count() as f32;
    let badge_w = (digits * font * 0.6 + 10.0 * layout.scale).max(font * 1.4);
    let badge_h = font + 6.0 * layout.scale;
    let badge_x = ui.cursor.0 + ghost * 0.5 - 4.0 * layout.scale - badge_w;
    let badge_y = ui.cursor.1 + ghost * 0.5 - 3.0 * layout.scale - badge_h;
    specs.push(InventoryTextSpec {
        text: held.stack.quantity.to_string(),
        left: badge_x + 5.0 * layout.scale,
        top: badge_y + 2.0 * layout.scale,
        size: font,
        color: theme.quantity_badge.text.as_rgb8(),
    });
}

fn push_footer_keys(specs: &mut Vec<InventoryTextSpec>, theme: &UiTheme, layout: &InventoryLayout) {
    let size = theme.text.muted_size * layout.scale;
    specs.push(InventoryTextSpec {
        text: "E/ECHAP fermer  -  Clic: prendre/poser  -  Clic droit: 1/2  -  Maj+clic: deplacer  -  1-9: hotbar  -  Q: jeter".to_string(),
        left: layout.modal.x + 18.0 * layout.scale,
        top: layout.modal.y + layout.modal.h - size - 10.0 * layout.scale,
        size,
        color: theme.text.muted.as_rgb8(),
    });
}

fn total_weight_kg(inventory: &Inventory, ui: &InventoryUiState) -> f32 {
    let count = inventory.total_count() as f32;
    let held = ui.held.map(|h| h.stack.quantity as f32).unwrap_or(0.0);
    (count + held) * InventoryUiState::UNIT_WEIGHT_KG
}

// =============================================================================
// Pixel-level shape primitives (rounded rects, circles, lines, ...)
// =============================================================================

impl<'a> Renderer<'a> {
    /// Component-state mapping shared by all slot drawers.
    fn slot_state(
        &self,
        content: Option<vv_gameplay::HotbarSlot>,
        visible: bool,
        hovered: bool,
        selected: bool,
    ) -> ComponentState {
        if !visible {
            return ComponentState::Disabled;
        }
        if selected {
            return ComponentState::Selected;
        }
        if hovered {
            return ComponentState::Hovered;
        }
        if content.is_none() {
            return ComponentState::Empty;
        }
        ComponentState::Normal
    }

    fn inventory_button_state(
        &self,
        ui: &InventoryUiState,
        button: InventoryButton,
    ) -> ComponentState {
        if matches!(ui.hovered_button, Some(b) if b == button) {
            ComponentState::Hovered
        } else {
            ComponentState::Normal
        }
    }

    pub(super) fn fill_rect(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        color: UiColor,
    ) {
        self.add_ui_rect_rgba(
            verts,
            inds,
            rect.x,
            rect.y,
            rect.x + rect.w,
            rect.y + rect.h,
            color,
        );
    }

    pub(super) fn fill_rounded_rect(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        color: UiColor,
        radius: f32,
    ) {
        let r = radius.max(0.0).min(rect.w * 0.5).min(rect.h * 0.5);
        if r <= 0.5 {
            self.fill_rect(verts, inds, rect, color);
            return;
        }
        // Central horizontal band.
        self.fill_rect(
            verts,
            inds,
            UiRect {
                x: rect.x,
                y: rect.y + r,
                w: rect.w,
                h: rect.h - r * 2.0,
            },
            color,
        );
        // Top / bottom strips between the corners.
        self.fill_rect(
            verts,
            inds,
            UiRect {
                x: rect.x + r,
                y: rect.y,
                w: rect.w - r * 2.0,
                h: r,
            },
            color,
        );
        self.fill_rect(
            verts,
            inds,
            UiRect {
                x: rect.x + r,
                y: rect.y + rect.h - r,
                w: rect.w - r * 2.0,
                h: r,
            },
            color,
        );
        // 4 corner arcs (screen Y points down).
        let pi = std::f32::consts::PI;
        let segs = corner_segments(r);
        self.fan_arc(
            verts,
            inds,
            rect.x + r,
            rect.y + r,
            r,
            pi,
            1.5 * pi,
            segs,
            color,
        );
        self.fan_arc(
            verts,
            inds,
            rect.x + rect.w - r,
            rect.y + r,
            r,
            1.5 * pi,
            2.0 * pi,
            segs,
            color,
        );
        self.fan_arc(
            verts,
            inds,
            rect.x + rect.w - r,
            rect.y + rect.h - r,
            r,
            0.0,
            0.5 * pi,
            segs,
            color,
        );
        self.fan_arc(
            verts,
            inds,
            rect.x + r,
            rect.y + rect.h - r,
            r,
            0.5 * pi,
            pi,
            segs,
            color,
        );
    }

    /// Hollow rounded-rect outline. Four straight rim strips on the edges
    /// + four corner "ring arcs" (triangle strip between an outer arc and
    /// an inner arc). No body fill is drawn so the caller must paint the
    /// inside separately *before* stroking.
    pub(super) fn stroke_rounded_rect(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        color: UiColor,
        width: f32,
        radius: f32,
    ) {
        let w = width.max(1.0);
        let r = radius.max(0.0).min(rect.w * 0.5).min(rect.h * 0.5);
        let x0 = rect.x;
        let y0 = rect.y;
        let x1 = rect.x + rect.w;
        let y1 = rect.y + rect.h;

        // Edge strips (excluding corners).
        let middle_w = (rect.w - r * 2.0).max(0.0);
        let middle_h = (rect.h - r * 2.0).max(0.0);
        if middle_w > 0.0 {
            self.fill_rect(
                verts,
                inds,
                UiRect {
                    x: x0 + r,
                    y: y0,
                    w: middle_w,
                    h: w,
                },
                color,
            );
            self.fill_rect(
                verts,
                inds,
                UiRect {
                    x: x0 + r,
                    y: y1 - w,
                    w: middle_w,
                    h: w,
                },
                color,
            );
        }
        if middle_h > 0.0 {
            self.fill_rect(
                verts,
                inds,
                UiRect {
                    x: x0,
                    y: y0 + r,
                    w,
                    h: middle_h,
                },
                color,
            );
            self.fill_rect(
                verts,
                inds,
                UiRect {
                    x: x1 - w,
                    y: y0 + r,
                    w,
                    h: middle_h,
                },
                color,
            );
        }

        // Corner ring arcs.
        if r > 0.5 {
            let inner_r = (r - w).max(0.0);
            let pi = std::f32::consts::PI;
            self.ring_arc(verts, inds, x0 + r, y0 + r, r, inner_r, pi, 1.5 * pi, color);
            self.ring_arc(
                verts,
                inds,
                x1 - r,
                y0 + r,
                r,
                inner_r,
                1.5 * pi,
                2.0 * pi,
                color,
            );
            self.ring_arc(
                verts,
                inds,
                x1 - r,
                y1 - r,
                r,
                inner_r,
                0.0,
                0.5 * pi,
                color,
            );
            self.ring_arc(verts, inds, x0 + r, y1 - r, r, inner_r, 0.5 * pi, pi, color);
        }
    }

    fn ring_arc(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        cx: f32,
        cy: f32,
        r_outer: f32,
        r_inner: f32,
        a0: f32,
        a1: f32,
        color: UiColor,
    ) {
        let segments = corner_segments(r_outer).max(3);
        let mut prev_outer = self.push_vert(
            verts,
            cx + r_outer * a0.cos(),
            cy + r_outer * a0.sin(),
            color.rgb,
        );
        let mut prev_inner = self.push_vert(
            verts,
            cx + r_inner * a0.cos(),
            cy + r_inner * a0.sin(),
            color.rgb,
        );
        for i in 1..=segments {
            let angle = a0 + (a1 - a0) * (i as f32 / segments as f32);
            let cos = angle.cos();
            let sin = angle.sin();
            let next_outer =
                self.push_vert(verts, cx + r_outer * cos, cy + r_outer * sin, color.rgb);
            let next_inner =
                self.push_vert(verts, cx + r_inner * cos, cy + r_inner * sin, color.rgb);
            inds.extend([prev_outer, prev_inner, next_inner]);
            inds.extend([prev_outer, next_inner, next_outer]);
            prev_outer = next_outer;
            prev_inner = next_inner;
        }
    }

    pub(super) fn fan_arc(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        cx: f32,
        cy: f32,
        r: f32,
        a0: f32,
        a1: f32,
        segments: usize,
        color: UiColor,
    ) {
        let center = self.push_vert(verts, cx, cy, color.rgb);
        let mut prev = self.push_vert(verts, cx + r * a0.cos(), cy + r * a0.sin(), color.rgb);
        let segments = segments.max(2);
        for i in 1..=segments {
            let angle = a0 + (a1 - a0) * (i as f32 / segments as f32);
            let next = self.push_vert(verts, cx + r * angle.cos(), cy + r * angle.sin(), color.rgb);
            inds.extend([center, prev, next]);
            prev = next;
        }
    }

    fn fill_circle(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        cx: f32,
        cy: f32,
        r: f32,
        color: UiColor,
    ) {
        let segments = corner_segments(r).max(10);
        let center = self.push_vert(verts, cx, cy, color.rgb);
        let mut prev = self.push_vert(verts, cx + r, cy, color.rgb);
        let two_pi = std::f32::consts::TAU;
        for i in 1..=segments {
            let angle = two_pi * (i as f32 / segments as f32);
            let next = self.push_vert(verts, cx + r * angle.cos(), cy + r * angle.sin(), color.rgb);
            inds.extend([center, prev, next]);
            prev = next;
        }
    }

    fn stroke_circle(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        cx: f32,
        cy: f32,
        r: f32,
        width: f32,
        color: UiColor,
    ) {
        let segments = corner_segments(r).max(12);
        let two_pi = std::f32::consts::TAU;
        let r_outer = r;
        let r_inner = (r - width).max(0.0);
        let mut prev_outer = self.push_vert(verts, cx + r_outer, cy, color.rgb);
        let mut prev_inner = self.push_vert(verts, cx + r_inner, cy, color.rgb);
        for i in 1..=segments {
            let angle = two_pi * (i as f32 / segments as f32);
            let cos = angle.cos();
            let sin = angle.sin();
            let next_outer =
                self.push_vert(verts, cx + r_outer * cos, cy + r_outer * sin, color.rgb);
            let next_inner =
                self.push_vert(verts, cx + r_inner * cos, cy + r_inner * sin, color.rgb);
            inds.extend([prev_outer, prev_inner, next_inner]);
            inds.extend([prev_outer, next_inner, next_outer]);
            prev_outer = next_outer;
            prev_inner = next_inner;
        }
    }

    fn draw_line_thick(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        thickness: f32,
        color: UiColor,
    ) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt().max(1e-3);
        let nx = -dy / len;
        let ny = dx / len;
        let half = thickness * 0.5;
        let (ax, ay) = (x0 + nx * half, y0 + ny * half);
        let (bx, by) = (x1 + nx * half, y1 + ny * half);
        let (cx, cy) = (x1 - nx * half, y1 - ny * half);
        let (dx2, dy2) = (x0 - nx * half, y0 - ny * half);
        self.add_ui_quad(
            verts,
            inds,
            (ax, ay),
            (bx, by),
            (cx, cy),
            (dx2, dy2),
            color.rgb,
        );
    }

    pub(super) fn draw_diagonal(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        inset: f32,
        thickness: f32,
        color: UiColor,
        flip: bool,
    ) {
        let (x0, y0, x1, y1) = if flip {
            (
                rect.x + rect.w - inset,
                rect.y + inset,
                rect.x + inset,
                rect.y + rect.h - inset,
            )
        } else {
            (
                rect.x + inset,
                rect.y + inset,
                rect.x + rect.w - inset,
                rect.y + rect.h - inset,
            )
        };
        self.draw_line_thick(verts, inds, x0, y0, x1, y1, thickness, color);
    }

    /// Draw an isometric voxel block using 3 shaded faces.
    ///
    /// When `texture` is `Some`, each face samples its assigned atlas layer
    /// and the vertex color carries pure grayscale shading. The UI shader
    /// expects 1-based atlas indices (0 = "no material"), so each layer is
    /// shifted by +1 at this boundary — the registry stores them 0-based.
    ///
    /// When `texture` is `None`, the cube is flat-colored from `base_rgb`.
    pub(super) fn draw_iso_block(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        slot: UiRect,
        base_rgb: [f32; 3],
        dim: f32,
        texture: Option<BlockMaterialLayers>,
    ) {
        let cx = slot.x + slot.w * 0.5;
        let cy = slot.y + slot.h * 0.5;
        let span = slot.w.min(slot.h) * 0.70;
        let u = span / 4.0;

        let textured = texture.is_some();
        let face_color = |factor: f32| -> [f32; 3] {
            let m = (factor * dim).clamp(0.0, 1.6);
            if textured {
                [m.min(1.0); 3]
            } else {
                [
                    (base_rgb[0] * m * 1.10).min(1.0),
                    (base_rgb[1] * m * 1.10).min(1.0),
                    (base_rgb[2] * m * 1.10).min(1.0),
                ]
            }
        };

        // UI shader is 1-based; world atlas index 0 → tex_index 1.
        let face_tex = |layer: u32| -> u32 {
            if textured {
                layer + 1
            } else {
                0
            }
        };

        let layers = texture.unwrap_or_default();
        let top_color = face_color(1.18);
        let left_color = face_color(0.70);
        let right_color = face_color(0.95);

        let p_top = (cx, cy - 2.0 * u);
        let p_right = (cx + 2.0 * u, cy - u);
        let p_front = (cx, cy);
        let p_left = (cx - 2.0 * u, cy - u);

        // Top face: p_top(TL) → p_right(TR) → p_front(BR) → p_left(BL)
        self.add_ui_quad_tex(
            verts,
            inds,
            [p_top, p_right, p_front, p_left],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            top_color,
            face_tex(layers.top),
        );

        let p_bottom = (cx, cy + 2.0 * u);
        let p_left_bottom = (cx - 2.0 * u, cy + u);

        // Left face uses the block's "left" (nx) material — the iso cube's
        // visible left side. UV is mapped so the texture's top edge sits at
        // the cube's top edge.
        self.add_ui_quad_tex(
            verts,
            inds,
            [p_left, p_front, p_bottom, p_left_bottom],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            left_color,
            face_tex(layers.left),
        );

        let p_right_bottom = (cx + 2.0 * u, cy + u);

        // Right face uses the block's "right" (px) material.
        self.add_ui_quad_tex(
            verts,
            inds,
            [p_right, p_right_bottom, p_bottom, p_front],
            [[1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]],
            right_color,
            face_tex(layers.right),
        );

        // Top-front rim highlight (always flat, no texture).
        let glint_color = [
            (base_rgb[0] * 1.35).min(1.0),
            (base_rgb[1] * 1.35).min(1.0),
            (base_rgb[2] * 1.35).min(1.0),
        ];
        let highlight_w = (u * 0.18).max(1.5);
        self.add_ui_quad(
            verts,
            inds,
            p_top,
            p_right,
            (p_right.0 - u * 0.08, p_right.1 + highlight_w * 0.6),
            (p_top.0 + u * 0.08, p_top.1 + highlight_w * 0.6),
            glint_color,
        );
    }

    /// Like `add_ui_quad` but each corner carries its own UV and a shared
    /// `tex_index` for atlas sampling. Use `tex_index = 0` to fall back to
    /// the vertex color (same sentinel as flat geometry).
    fn add_ui_quad_tex(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        corners: [(f32, f32); 4],
        uvs: [[f32; 2]; 4],
        rgb: [f32; 3],
        tex_index: u32,
    ) {
        let i0 = self.push_vert_uv(verts, corners[0].0, corners[0].1, rgb, uvs[0], tex_index);
        let i1 = self.push_vert_uv(verts, corners[1].0, corners[1].1, rgb, uvs[1], tex_index);
        let i2 = self.push_vert_uv(verts, corners[2].0, corners[2].1, rgb, uvs[2], tex_index);
        let i3 = self.push_vert_uv(verts, corners[3].0, corners[3].1, rgb, uvs[3], tex_index);
        inds.extend([i0, i1, i2, i0, i2, i3]);
    }

    fn push_vert_uv(
        &self,
        verts: &mut Vec<Vertex>,
        x: f32,
        y: f32,
        rgb: [f32; 3],
        uv: [f32; 2],
        tex_index: u32,
    ) -> u32 {
        let width = self.config.width.max(1) as f32;
        let height = self.config.height.max(1) as f32;
        let pos = [(x / width) * 2.0 - 1.0, 1.0 - (y / height) * 2.0, 0.0];
        let idx = verts.len() as u32;
        verts.push(Vertex {
            pos,
            uv,
            color: rgb,
            normal: [0.0, 0.0, 1.0],
            tex_index,
        });
        idx
    }

    pub(super) fn add_ui_quad(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        a: (f32, f32),
        b: (f32, f32),
        c: (f32, f32),
        d: (f32, f32),
        rgb: [f32; 3],
    ) {
        let i0 = self.push_vert(verts, a.0, a.1, rgb);
        let i1 = self.push_vert(verts, b.0, b.1, rgb);
        let i2 = self.push_vert(verts, c.0, c.1, rgb);
        let i3 = self.push_vert(verts, d.0, d.1, rgb);
        inds.extend([i0, i1, i2, i0, i2, i3]);
    }

    pub(super) fn add_ui_rect_rgba(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: UiColor,
    ) {
        let rgb = color.rgb;
        let i0 = self.push_vert(verts, x0, y0, rgb);
        let i1 = self.push_vert(verts, x1, y0, rgb);
        let i2 = self.push_vert(verts, x0, y1, rgb);
        let i3 = self.push_vert(verts, x1, y1, rgb);
        inds.extend([i0, i2, i1, i1, i2, i3]);
    }

    fn push_vert(&self, verts: &mut Vec<Vertex>, x: f32, y: f32, rgb: [f32; 3]) -> u32 {
        let width = self.config.width.max(1) as f32;
        let height = self.config.height.max(1) as f32;
        let pos = [(x / width) * 2.0 - 1.0, 1.0 - (y / height) * 2.0, 0.0];
        let idx = verts.len() as u32;
        verts.push(Vertex {
            pos,
            uv: [0.0, 0.0],
            color: rgb,
            normal: [0.0, 0.0, 1.0],
            tex_index: 0,
        });
        idx
    }
}

/// Pick a triangle count for a rounded-corner arc. Bigger radii get more
/// segments so the silhouette stays smooth on 4K displays.
fn corner_segments(radius: f32) -> usize {
    if radius <= 4.0 {
        4
    } else if radius <= 10.0 {
        6
    } else if radius <= 20.0 {
        8
    } else {
        10
    }
}

/// Compute the three equipment-slot rects and their labels inside the given
/// left panel. Each slot is left-aligned with a right-side label. Used by
/// both the mesh drawer and the text-spec emitter so the positions always
/// agree.
fn equip_slot_rects(left_panel: UiRect, scale: f32) -> [(UiRect, &'static str); 3] {
    let slot_size = (left_panel.w * 0.36).min(52.0 * scale);
    let gap = slot_size * 0.40;
    let total_h = slot_size * 3.0 + gap * 2.0;
    let top = left_panel.y + (left_panel.h - total_h) * 0.5;
    let left = left_panel.x + left_panel.w * 0.12;
    [
        (
            UiRect {
                x: left,
                y: top,
                w: slot_size,
                h: slot_size,
            },
            "Casque",
        ),
        (
            UiRect {
                x: left,
                y: top + slot_size + gap,
                w: slot_size,
                h: slot_size,
            },
            "Plastron",
        ),
        (
            UiRect {
                x: left,
                y: top + (slot_size + gap) * 2.0,
                w: slot_size,
                h: slot_size,
            },
            "Bottes",
        ),
    ]
}

/// Bounding rect of the 2×2 crafting grid rendered inside the right panel.
/// Keeping this in a function ensures the mesh path and text-spec path
/// compute the same position without duplicating the formula.
fn craft_grid_rect(right_panel: UiRect, scale: f32) -> UiRect {
    let mini_slot = (right_panel.w * 0.22).min(36.0 * scale);
    let mini_gap = mini_slot * 0.20;
    let grid_size = mini_slot * 2.0 + mini_gap;
    let cx = right_panel.x + right_panel.w * 0.5;
    let cy = right_panel.y + right_panel.h * 0.38;
    UiRect {
        x: cx - grid_size * 0.5,
        y: cy - grid_size * 0.5,
        w: grid_size,
        h: grid_size,
    }
}
