#![allow(clippy::too_many_arguments)]

use super::Renderer;
use crate::ui::{ComponentState, UiColor, UiRect, UiTheme};
use crate::Vertex;
use vv_pack_compiler::CompiledItemGameplay;
use vv_world::PlanetData;

// =============================================================================
// Component primitives
// =============================================================================

impl<'a> Renderer<'a> {
    /// Stylized bag silhouette. No external asset.
    pub(super) fn draw_bag_icon(
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

    pub(super) fn draw_magnifier(
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

    pub(super) fn draw_round_button(
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

    pub(super) fn draw_action_button(
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

    pub(super) fn draw_recipe_row(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        state: ComponentState,
        theme: &UiTheme,
        scale: f32,
    ) {
        let radius = 7.0 * scale;
        let fill = match state {
            ComponentState::Selected => theme.button.fill_selected,
            ComponentState::Hovered => theme.filter_chip.fill_hovered,
            _ => theme.filter_chip.fill,
        };
        let border = match state {
            ComponentState::Selected => theme.filter_chip.border_selected,
            ComponentState::Hovered => theme.filter_chip.border_hovered,
            _ => theme.filter_chip.border,
        };
        self.fill_rounded_rect(verts, inds, rect, fill, radius);
        self.stroke_rounded_rect(verts, inds, rect, border, (1.0 * scale).max(1.0), radius);
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
            if let Some(layers) = layers {
                self.draw_iso_block(verts, inds, rect, base_color, dim, Some(layers));
            } else if let Some(item) = planet.items.get(slot.item_id) {
                self.draw_item_glyph(
                    verts,
                    inds,
                    rect,
                    item.category.as_str(),
                    &item.gameplay,
                    dim,
                );
            } else {
                self.draw_iso_block(verts, inds, rect, base_color, dim, None);
            }
        }
    }

    pub(super) fn draw_item_glyph(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        category: &str,
        gameplay: &CompiledItemGameplay,
        dim: f32,
    ) {
        match gameplay {
            CompiledItemGameplay::Tool(_) | CompiledItemGameplay::Weapon(_) => {
                let metal = UiColor::rgba(0.78 * dim, 0.72 * dim, 0.62 * dim, 1.0);
                let wood = UiColor::rgba(0.48 * dim, 0.27 * dim, 0.12 * dim, 1.0);
                self.draw_line_thick(
                    verts,
                    inds,
                    rect.x + rect.w * 0.30,
                    rect.y + rect.h * 0.74,
                    rect.x + rect.w * 0.70,
                    rect.y + rect.h * 0.28,
                    rect.w * 0.08,
                    wood,
                );
                let head = UiRect {
                    x: rect.x + rect.w * 0.50,
                    y: rect.y + rect.h * 0.20,
                    w: rect.w * 0.24,
                    h: rect.h * 0.20,
                };
                self.fill_rounded_rect(verts, inds, head, metal, rect.w * 0.04);
            }
            CompiledItemGameplay::Food(_) | CompiledItemGameplay::Consumable(_) => {
                let color = if category == "food" {
                    UiColor::rgba(0.78 * dim, 0.12 * dim, 0.08 * dim, 1.0)
                } else {
                    UiColor::rgba(0.20 * dim, 0.55 * dim, 0.78 * dim, 1.0)
                };
                self.fill_circle(
                    verts,
                    inds,
                    rect.x + rect.w * 0.50,
                    rect.y + rect.h * 0.52,
                    rect.w * 0.24,
                    color,
                );
                self.fill_circle(
                    verts,
                    inds,
                    rect.x + rect.w * 0.60,
                    rect.y + rect.h * 0.34,
                    rect.w * 0.08,
                    UiColor::rgba(0.26 * dim, 0.62 * dim, 0.16 * dim, 1.0),
                );
            }
            _ => {
                let fill = if category == "resource" {
                    UiColor::rgba(0.76 * dim, 0.56 * dim, 0.28 * dim, 1.0)
                } else {
                    UiColor::rgba(0.48 * dim, 0.58 * dim, 0.64 * dim, 1.0)
                };
                let a = (rect.x + rect.w * 0.50, rect.y + rect.h * 0.22);
                let b = (rect.x + rect.w * 0.74, rect.y + rect.h * 0.50);
                let c = (rect.x + rect.w * 0.50, rect.y + rect.h * 0.78);
                let d = (rect.x + rect.w * 0.26, rect.y + rect.h * 0.50);
                self.add_ui_quad(verts, inds, a, b, c, d, fill.rgb);
            }
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
