use super::Renderer;
use crate::gameplay::{Hotbar, HOTBAR_SLOT_COUNT};
use crate::rendering::Vertex;
use crate::ui::{ComponentState, UiRect, UiTheme};
use crate::world::PlanetData;

struct HotbarLayout {
    slot: f32,
    gap: f32,
    left: f32,
    top: f32,
}

pub(super) struct HotbarTextSpec {
    pub text: String,
    pub left: f32,
    pub top: f32,
    pub size: f32,
    pub color: [u8; 3],
}

impl<'a> Renderer<'a> {
    pub fn update_console_mesh(&mut self, t: f32) {
        if t <= 0.001 {
            self.console_inds = 0;
            return;
        }

        let height = t * 1.0;
        let bottom_y = 1.0 - height;

        let color = UiTheme::VOXELVERSE.panel.fill.as_rgb();
        let normal = [0.0, 0.0, 1.0];

        let verts = vec![
            Vertex {
                pos: [-1.0, 1.0, 0.0],
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
            Vertex {
                pos: [1.0, 1.0, 0.0],
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
            Vertex {
                pos: [-1.0, bottom_y, 0.0],
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
            Vertex {
                pos: [1.0, bottom_y, 0.0],
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
        ];

        let inds = vec![0, 2, 1, 1, 2, 3];

        self.queue
            .write_buffer(&self.console_v_buf, 0, bytemuck::cast_slice(&verts));
        self.queue
            .write_buffer(&self.console_i_buf, 0, bytemuck::cast_slice(&inds));
        self.console_inds = inds.len() as u32;
    }

    pub fn update_hotbar_mesh(&mut self, hotbar: &Hotbar, planet: &PlanetData) {
        let layout = self.hotbar_layout();
        let theme = UiTheme::VOXELVERSE;
        let mut verts = Vec::with_capacity(512);
        let mut inds = Vec::with_capacity(768);

        // Scale factor for strokes/radii — derived from height vs 1080p reference.
        let scale = (self.config.height as f32 / 1080.0).clamp(0.7, 2.0);

        // Draw only the slots — no panel background, no shadow, no border.
        // Visual separation comes from the slot borders alone.
        for (index, slot) in hotbar.slots().iter().enumerate() {
            let x0 = layout.left + index as f32 * (layout.slot + layout.gap);
            let slot_rect = UiRect { x: x0, y: layout.top, w: layout.slot, h: layout.slot };

            let selected = index == hotbar.selected_index();
            let state = if selected {
                ComponentState::Selected
            } else if slot.is_none() {
                ComponentState::Empty
            } else {
                ComponentState::Normal
            };

            self.draw_inventory_slot(
                &mut verts,
                &mut inds,
                slot_rect,
                *slot,
                state,
                planet,
                &theme,
                scale,
            );

            if let Some(s) = slot {
                if s.quantity > 1 {
                    self.draw_quantity_badge(
                        &mut verts,
                        &mut inds,
                        slot_rect,
                        s.quantity,
                        &theme,
                        scale,
                    );
                }
            }
        }

        self.queue
            .write_buffer(&self.hotbar_v_buf, 0, bytemuck::cast_slice(&verts));
        self.queue
            .write_buffer(&self.hotbar_i_buf, 0, bytemuck::cast_slice(&inds));
        self.hotbar_inds = inds.len() as u32;
    }

    pub(super) fn hotbar_text_specs(&self, hotbar: &Hotbar) -> Vec<HotbarTextSpec> {
        let theme = UiTheme::VOXELVERSE;
        let mut specs = Vec::new();
        for (index, slot) in hotbar.slots().iter().enumerate() {
            let Some(slot) = slot else {
                continue;
            };
            if slot.quantity <= 1 {
                continue;
            }

            let text = slot.quantity.to_string();
            let (left, top) = self.hotbar_quantity_position(index, &text);
            let scale = (self.config.height as f32 / 1080.0).clamp(0.7, 2.0);
            specs.push(HotbarTextSpec {
                text,
                left,
                top,
                size: theme.quantity_badge.font_size * scale,
                color: theme.quantity_badge.text.as_rgb8(),
            });
        }

        if let Some(notice) = hotbar.notice_text() {
            let (left, top) = self.hotbar_notice_position();
            specs.push(HotbarTextSpec {
                text: notice.to_string(),
                left,
                top,
                size: theme.text.body_size,
                color: theme.player_notice.info_text.as_rgb8(),
            });
        }

        specs
    }

    fn hotbar_quantity_position(&self, index: usize, text: &str) -> (f32, f32) {
        let theme = UiTheme::VOXELVERSE;
        let layout = self.hotbar_layout();
        // Same scale used by draw_quantity_badge so the text sits exactly
        // inside the badge background geometry.
        let scale = (self.config.height as f32 / 1080.0).clamp(0.7, 2.0);
        let font = theme.quantity_badge.font_size * scale;
        let digits = text.chars().count() as f32;
        let badge_w = (digits * font * 0.6 + 10.0 * scale).max(font * 1.4);
        let badge_h = font + 6.0 * scale;
        let slot_right = layout.left + index as f32 * (layout.slot + layout.gap) + layout.slot;
        let slot_bottom = layout.top + layout.slot;
        let badge_x = slot_right - 4.0 * scale - badge_w;
        let badge_y = slot_bottom - 3.0 * scale - badge_h;
        let x = badge_x + 5.0 * scale;
        let y = badge_y + 2.0 * scale;
        (x, y)
    }

    fn hotbar_notice_position(&self) -> (f32, f32) {
        let theme = UiTheme::VOXELVERSE;
        let layout = self.hotbar_layout();
        (layout.left, layout.top - theme.hotbar.notice_offset_y)
    }

    fn hotbar_layout(&self) -> HotbarLayout {
        let theme = UiTheme::VOXELVERSE;
        let height = self.config.height as f32;
        let width = self.config.width as f32;
        let slot = (height * theme.hotbar.slot_height_ratio)
            .clamp(theme.hotbar.slot_size_min, theme.hotbar.slot_size_max);
        let gap = theme.hotbar.slot_gap * (slot / 56.0).max(1.0);
        let total_width = HOTBAR_SLOT_COUNT as f32 * slot + (HOTBAR_SLOT_COUNT - 1) as f32 * gap;
        let bottom_margin = (height * 0.035).clamp(
            theme.spacing.hotbar_bottom_margin_min,
            theme.spacing.hotbar_bottom_margin_max,
        );
        HotbarLayout {
            slot,
            gap,
            left: (width - total_width) * 0.5,
            top: height - bottom_margin - slot,
        }
    }
}
