use glam::Vec3;
use glyphon::{Attrs, Buffer, Family, Metrics, Shaping};

use vv_gameplay::{can_craft_hand_recipe, PlayerGameplayState};
use vv_input::Controller;
use vv_mesh::Vertex;
use vv_registry::{BlockRenderSource, CompiledContent, CompiledItemKind, ItemId};
use vv_world_runtime::PlanetData;

use crate::{
    block_feedback::{block_break_mesh, BlockBreakStyle},
    gameplay_ui::{GameplayUiLayout, RectPx},
};

use super::Renderer;

impl<'a> Renderer<'a> {
    pub(super) fn update_gameplay_ui_mesh(
        &mut self,
        controller: &Controller,
        gameplay: &PlayerGameplayState,
        content: &CompiledContent,
    ) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let mut idx = 0u32;

        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let mut layout = GameplayUiLayout::new(w, h, &gameplay.inventory, gameplay.inventory_open);
        if gameplay.inventory_open {
            layout.add_hand_recipes(content.recipes.recipes_for_station(None));
        }
        self.push_crosshair(controller, gameplay, &mut verts, &mut inds, &mut idx);

        if !gameplay.inventory_open {
            for slot in &layout.hotbar_slots {
                let selected = slot.index == gameplay.selected_hotbar_slot;
                self.push_slot(
                    &mut verts,
                    &mut inds,
                    &mut idx,
                    slot.rect,
                    selected,
                    gameplay.inventory.slots()[slot.index].stack.map(|stack| {
                        (
                            self.item_color(stack.item, content),
                            gameplay.inventory_drag.source_slot == Some(slot.index),
                        )
                    }),
                );
            }
        }

        if gameplay.inventory_open {
            if let Some(panel) = layout.inventory_panel {
                self.push_panel(&mut verts, &mut inds, &mut idx, panel);
            }

            for slot in &layout.inventory_slots {
                self.push_slot(
                    &mut verts,
                    &mut inds,
                    &mut idx,
                    slot.rect,
                    slot.index == gameplay.selected_hotbar_slot,
                    gameplay.inventory.slots()[slot.index].stack.map(|stack| {
                        (
                            self.item_color(stack.item, content),
                            gameplay.inventory_drag.source_slot == Some(slot.index),
                        )
                    }),
                );
            }

            for slot in &layout.recipe_slots {
                let enabled = can_craft_hand_recipe(&gameplay.inventory, slot.recipe, content);
                let color = content
                    .recipes
                    .get(slot.recipe)
                    .map(|recipe| self.item_color(recipe.result_item, content))
                    .unwrap_or([0.45, 0.45, 0.45]);
                Self::push_rect_px(
                    &mut verts,
                    &mut inds,
                    &mut idx,
                    w,
                    h,
                    slot.rect.x - 2.0,
                    slot.rect.y - 2.0,
                    slot.rect.w + 4.0,
                    slot.rect.h + 4.0,
                    if enabled {
                        [0.36, 0.48, 0.28]
                    } else {
                        [0.16, 0.17, 0.18]
                    },
                );
                Self::push_rect_px(
                    &mut verts,
                    &mut inds,
                    &mut idx,
                    w,
                    h,
                    slot.rect.x,
                    slot.rect.y,
                    slot.rect.w,
                    slot.rect.h,
                    [0.075, 0.08, 0.085],
                );
                Self::push_rect_px(
                    &mut verts,
                    &mut inds,
                    &mut idx,
                    w,
                    h,
                    slot.rect.x + slot.rect.w * 0.28,
                    slot.rect.y + slot.rect.h * 0.22,
                    slot.rect.w * 0.44,
                    slot.rect.h * 0.56,
                    if enabled {
                        color
                    } else {
                        [color[0] * 0.35, color[1] * 0.35, color[2] * 0.35]
                    },
                );
            }
        }

        if let Some(stack) = gameplay.inventory_drag.stack {
            let color = self.item_color(stack.item, content);
            let size = layout.slot * 0.78;
            let rect = RectPx {
                x: controller.mouse_pos.x - size * 0.5,
                y: controller.mouse_pos.y - size * 0.5,
                w: size,
                h: size,
            };
            Self::push_rect_px(
                &mut verts,
                &mut inds,
                &mut idx,
                w,
                h,
                rect.x - 3.0 * layout.scale,
                rect.y - 3.0 * layout.scale,
                rect.w + 6.0 * layout.scale,
                rect.h + 6.0 * layout.scale,
                [0.02, 0.02, 0.02],
            );
            Self::push_rect_px(
                &mut verts, &mut inds, &mut idx, w, h, rect.x, rect.y, rect.w, rect.h, color,
            );
        }

        if !verts.is_empty() {
            self.queue
                .write_buffer(&self.ui_v_buf, 0, bytemuck::cast_slice(&verts));
            self.queue
                .write_buffer(&self.ui_i_buf, 0, bytemuck::cast_slice(&inds));
        }
        self.ui_inds = inds.len() as u32;
    }

    fn push_crosshair(
        &self,
        controller: &Controller,
        gameplay: &PlayerGameplayState,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
    ) {
        if !controller.first_person || gameplay.inventory_open {
            return;
        }
        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let scale = (w.min(h) / 720.0).clamp(0.75, 1.35);
        let cx = w * 0.5;
        let cy = h * 0.5;
        let thickness = (2.0 * scale).max(1.5);
        let gap = 6.0 * scale;
        let arm = 10.0 * scale;
        let active = gameplay.target.is_some();
        let mining = gameplay.mining.progress > 0.0;
        let color = if mining {
            [0.95, 0.78, 0.35]
        } else if active {
            [0.92, 0.9, 0.78]
        } else {
            [0.82, 0.86, 0.88]
        };
        let shadow = [0.015, 0.018, 0.02];
        for (dx, dy, ww, hh) in [
            (-gap - arm, -thickness * 0.5, arm, thickness),
            (gap, -thickness * 0.5, arm, thickness),
            (-thickness * 0.5, -gap - arm, thickness, arm),
            (-thickness * 0.5, gap, thickness, arm),
        ] {
            Self::push_rect_px(
                verts,
                inds,
                idx,
                w,
                h,
                cx + dx + scale,
                cy + dy + scale,
                ww,
                hh,
                shadow,
            );
            Self::push_rect_px(verts, inds, idx, w, h, cx + dx, cy + dy, ww, hh, color);
        }
    }

    fn push_panel(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        rect: RectPx,
    ) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;
        Self::push_rect_px(
            verts,
            inds,
            idx,
            w,
            h,
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            [0.055, 0.06, 0.065],
        );
        Self::push_rect_px(
            verts,
            inds,
            idx,
            w,
            h,
            rect.x + 4.0,
            rect.y + 4.0,
            rect.w - 8.0,
            rect.h - 8.0,
            [0.115, 0.12, 0.125],
        );
    }

    fn push_slot(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        rect: RectPx,
        selected: bool,
        item: Option<([f32; 3], bool)>,
    ) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let border = if selected {
            [0.95, 0.88, 0.52]
        } else {
            [0.19, 0.2, 0.21]
        };
        let inset = (rect.w * 0.1).max(3.0);
        Self::push_rect_px(
            verts,
            inds,
            idx,
            w,
            h,
            rect.x - 2.0,
            rect.y - 2.0,
            rect.w + 4.0,
            rect.h + 4.0,
            border,
        );
        Self::push_rect_px(
            verts,
            inds,
            idx,
            w,
            h,
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            [0.07, 0.075, 0.08],
        );
        Self::push_rect_px(
            verts,
            inds,
            idx,
            w,
            h,
            rect.x + inset,
            rect.y + inset,
            rect.w - inset * 2.0,
            rect.h - inset * 2.0,
            [0.135, 0.14, 0.145],
        );
        if let Some((color, hidden_by_drag)) = item {
            if hidden_by_drag {
                return;
            }
            let item_inset = rect.w * 0.26;
            Self::push_rect_px(
                verts,
                inds,
                idx,
                w,
                h,
                rect.x + item_inset,
                rect.y + item_inset * 0.85,
                rect.w - item_inset * 2.0,
                rect.h - item_inset * 1.75,
                color,
            );
        }
    }

    pub(super) fn update_block_break_feedback(
        &mut self,
        planet: &PlanetData,
        gameplay: &PlayerGameplayState,
    ) {
        let progress = gameplay.mining.progress;
        let Some(id) = gameplay.mining.target else {
            self.break_inds = 0;
            return;
        };

        let mesh = block_break_mesh(planet, id, progress, BlockBreakStyle::default());
        if mesh.indices.is_empty() {
            self.break_inds = 0;
            return;
        }

        self.queue
            .write_buffer(&self.break_v_buf, 0, bytemuck::cast_slice(&mesh.vertices));
        self.queue
            .write_buffer(&self.break_i_buf, 0, bytemuck::cast_slice(&mesh.indices));
        self.break_inds = mesh.indices.len() as u32;
    }

    pub(super) fn update_dropped_item_mesh(
        &mut self,
        gameplay: &PlayerGameplayState,
        content: &CompiledContent,
    ) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let mut idx = 0u32;
        for drop in gameplay.dropped_items.iter().take(128) {
            let color = self.item_color(drop.stack.item, content);
            Self::push_cube(&mut verts, &mut inds, &mut idx, drop.position, 0.28, color);
        }
        if !verts.is_empty() {
            self.queue
                .write_buffer(&self.drop_v_buf, 0, bytemuck::cast_slice(&verts));
            self.queue
                .write_buffer(&self.drop_i_buf, 0, bytemuck::cast_slice(&inds));
        }
        self.drop_inds = inds.len() as u32;
    }

    pub(super) fn push_gameplay_text(
        &mut self,
        controller: &Controller,
        gameplay: &PlayerGameplayState,
        text_buffers: &mut Vec<(Buffer, f32, f32)>,
    ) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let layout = GameplayUiLayout::new(w, h, &gameplay.inventory, gameplay.inventory_open);

        if !gameplay.inventory_open {
            for slot in &layout.hotbar_slots {
                if gameplay.inventory_drag.source_slot == Some(slot.index) {
                    continue;
                }
                if let Some(stack) = gameplay.inventory.slots()[slot.index].stack {
                    if stack.count > 1 {
                        let x = slot.rect.x + slot.rect.w - 18.0 * layout.scale;
                        let y = slot.rect.y + slot.rect.h - 20.0 * layout.scale;
                        self.push_text(
                            text_buffers,
                            &stack.count.to_string(),
                            x,
                            y,
                            15.0 * layout.scale,
                        );
                    }
                }
            }
        }

        if gameplay.inventory_open {
            if let Some(panel) = layout.inventory_panel {
                self.push_text(
                    text_buffers,
                    "Backpack",
                    panel.x + 16.0 * layout.scale,
                    panel.y + 9.0 * layout.scale,
                    16.0 * layout.scale,
                );
                self.push_text(
                    text_buffers,
                    "Hotbar",
                    panel.x + 16.0 * layout.scale,
                    panel.y + panel.h - layout.slot - 18.0 * layout.scale,
                    13.0 * layout.scale,
                );
            }
            for slot in &layout.inventory_slots {
                if gameplay.inventory_drag.source_slot == Some(slot.index) {
                    continue;
                }
                let Some(stack) = gameplay.inventory.slots()[slot.index].stack else {
                    continue;
                };
                if stack.count <= 1 {
                    continue;
                }
                let x = slot.rect.x + slot.rect.w - 18.0 * layout.scale;
                let y = slot.rect.y + slot.rect.h - 20.0 * layout.scale;
                self.push_text(
                    text_buffers,
                    &stack.count.to_string(),
                    x,
                    y,
                    15.0 * layout.scale,
                );
            }
        }

        if let Some(stack) = gameplay.inventory_drag.stack {
            if stack.count > 1 {
                self.push_text(
                    text_buffers,
                    &stack.count.to_string(),
                    controller.mouse_pos.x + 8.0 * layout.scale,
                    controller.mouse_pos.y + 8.0 * layout.scale,
                    15.0 * layout.scale,
                );
            }
        }

        if gameplay.pickup_notice_timer > 0.0 {
            self.push_text(text_buffers, "Picked up", w * 0.5 - 42.0, h - 92.0, 16.0);
        }
        if gameplay.placement_blocked_timer > 0.0 {
            self.push_text(text_buffers, "Cannot place", w * 0.5 - 48.0, h * 0.58, 16.0);
        }
    }

    fn push_text(
        &mut self,
        text_buffers: &mut Vec<(Buffer, f32, f32)>,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
    ) {
        let mut buf = Buffer::new(&mut self.font_system, Metrics::new(size, size + 4.0));
        buf.set_size(
            &mut self.font_system,
            self.config.width as f32,
            self.config.height as f32,
        );
        buf.set_text(
            &mut self.font_system,
            text,
            Attrs::new()
                .family(Family::Monospace)
                .color(glyphon::Color::rgb(255, 255, 255)),
            Shaping::Advanced,
        );
        text_buffers.push((buf, x, y));
    }

    fn item_color(&self, item: ItemId, content: &CompiledContent) -> [f32; 3] {
        let Some(item) = content.items.get(item) else {
            return [0.75, 0.75, 0.75];
        };
        match item.kind {
            CompiledItemKind::Block { block } => self
                .block_content
                .block_render(block)
                .map(|render| render.color)
                .unwrap_or([0.75, 0.75, 0.75]),
            CompiledItemKind::Placeable { .. } => [0.95, 0.72, 0.35],
            CompiledItemKind::Tool { .. } => [0.72, 0.78, 0.85],
            CompiledItemKind::Armor => [0.62, 0.72, 0.9],
            CompiledItemKind::Food => [0.72, 0.9, 0.48],
            CompiledItemKind::Resource => [0.72, 0.68, 0.58],
        }
    }

    fn push_rect_px(
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        screen_w: f32,
        screen_h: f32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 3],
    ) {
        let x0 = x / screen_w * 2.0 - 1.0;
        let x1 = (x + width) / screen_w * 2.0 - 1.0;
        let y0 = 1.0 - y / screen_h * 2.0;
        let y1 = 1.0 - (y + height) / screen_h * 2.0;
        let normal = [0.0, 0.0, 1.0];
        let base = *idx;
        verts.extend_from_slice(&[
            Vertex::untextured([x0, y0, 0.0], color, normal),
            Vertex::untextured([x1, y0, 0.0], color, normal),
            Vertex::untextured([x1, y1, 0.0], color, normal),
            Vertex::untextured([x0, y1, 0.0], color, normal),
        ]);
        inds.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
        *idx += 4;
    }

    fn push_cube(
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        center: Vec3,
        size: f32,
        color: [f32; 3],
    ) {
        let h = size * 0.5;
        let p = [
            center + Vec3::new(-h, -h, -h),
            center + Vec3::new(h, -h, -h),
            center + Vec3::new(h, h, -h),
            center + Vec3::new(-h, h, -h),
            center + Vec3::new(-h, -h, h),
            center + Vec3::new(h, -h, h),
            center + Vec3::new(h, h, h),
            center + Vec3::new(-h, h, h),
        ];
        let faces = [
            ([0, 1, 2, 3], [0.0, 0.0, -1.0]),
            ([5, 4, 7, 6], [0.0, 0.0, 1.0]),
            ([4, 0, 3, 7], [-1.0, 0.0, 0.0]),
            ([1, 5, 6, 2], [1.0, 0.0, 0.0]),
            ([3, 2, 6, 7], [0.0, 1.0, 0.0]),
            ([4, 5, 1, 0], [0.0, -1.0, 0.0]),
        ];
        for (face, normal) in faces {
            let base = *idx;
            for i in face {
                verts.push(Vertex::untextured(p[i].to_array(), color, normal));
            }
            inds.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
            *idx += 4;
        }
    }

    pub(super) fn update_console_mesh(&mut self, t: f32) {
        if t <= 0.001 {
            self.console_inds = 0;
            return;
        }
        let bottom_y = 1.0 - t;
        let color = [0.1, 0.1, 0.15];
        let normal = [0.0, 0.0, 1.0];
        let verts = vec![
            Vertex::untextured([-1.0, 1.0, 0.0], color, normal),
            Vertex::untextured([1.0, 1.0, 0.0], color, normal),
            Vertex::untextured([-1.0, bottom_y, 0.0], color, normal),
            Vertex::untextured([1.0, bottom_y, 0.0], color, normal),
        ];
        let inds = vec![0u32, 2, 1, 1, 2, 3];
        self.queue
            .write_buffer(&self.console_v_buf, 0, bytemuck::cast_slice(&verts));
        self.queue
            .write_buffer(&self.console_i_buf, 0, bytemuck::cast_slice(&inds));
        self.console_inds = 6;
    }
}