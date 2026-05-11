use super::Renderer;
use crate::gameplay::{Hotbar, HOTBAR_SLOT_COUNT};
use crate::rendering::Vertex;
use crate::ui::{ComponentState, UiColor, UiTheme};
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
        let mut verts = Vec::with_capacity(128);
        let mut inds = Vec::with_capacity(192);

        let panel_pad = theme.hotbar.panel_padding;
        let total_width =
            HOTBAR_SLOT_COUNT as f32 * layout.slot + (HOTBAR_SLOT_COUNT - 1) as f32 * layout.gap;
        self.add_ui_rect(
            &mut verts,
            &mut inds,
            layout.left - panel_pad,
            layout.top - panel_pad,
            layout.left + total_width + panel_pad,
            layout.top + layout.slot + panel_pad,
            theme.panel.fill.as_rgb(),
        );

        for (index, slot) in hotbar.slots().iter().enumerate() {
            let x0 = layout.left + index as f32 * (layout.slot + layout.gap);
            let y0 = layout.top;
            let x1 = x0 + layout.slot;
            let y1 = y0 + layout.slot;
            let selected = index == hotbar.selected_index();
            let state = if selected {
                ComponentState::Selected
            } else {
                ComponentState::Normal
            };

            self.add_ui_rect(
                &mut verts,
                &mut inds,
                x0,
                y0,
                x1,
                y1,
                theme.slot.border_for(state).as_rgb(),
            );
            self.add_ui_rect(
                &mut verts,
                &mut inds,
                x0 + theme.slot.inner_inset,
                y0 + theme.slot.inner_inset,
                x1 - theme.slot.inner_inset,
                y1 - theme.slot.inner_inset,
                theme.slot.fill_for(state).as_rgb(),
            );
            self.add_ui_rect(
                &mut verts,
                &mut inds,
                x0 + theme.slot.inner_inset + 2.0,
                y0 + theme.slot.inner_inset + 2.0,
                x1 - theme.slot.inner_inset - 2.0,
                y1 - theme.slot.inner_inset - 2.0,
                if slot.is_some() {
                    theme.slot.inner_fill.as_rgb()
                } else {
                    theme.inventory_grid.empty_slot_fill.as_rgb()
                },
            );

            if selected {
                let gold = theme.slot.border_selected.as_rgb();
                let stroke = theme.slot.selected_border_width;
                self.add_ui_rect(
                    &mut verts,
                    &mut inds,
                    x0 - stroke,
                    y0 - stroke,
                    x1 + stroke,
                    y0,
                    gold,
                );
                self.add_ui_rect(
                    &mut verts,
                    &mut inds,
                    x0 - stroke,
                    y1,
                    x1 + stroke,
                    y1 + stroke,
                    gold,
                );
                self.add_ui_rect(&mut verts, &mut inds, x0 - stroke, y0, x0, y1, gold);
                self.add_ui_rect(&mut verts, &mut inds, x1, y0, x1 + stroke, y1, gold);
            }

            if let Some(slot) = slot {
                let color = UiColor::rgba(
                    planet.content.color(slot.voxel)[0],
                    planet.content.color(slot.voxel)[1],
                    planet.content.color(slot.voxel)[2],
                    1.0,
                )
                .scale_rgb(1.12)
                .as_rgb();
                let inset = theme.slot.icon_inset;
                self.add_ui_rect(
                    &mut verts,
                    &mut inds,
                    x0 + inset,
                    y0 + inset,
                    x1 - inset,
                    y1 - inset,
                    color,
                );
                self.add_ui_rect(
                    &mut verts,
                    &mut inds,
                    x0 + inset,
                    y0 + inset,
                    x1 - inset,
                    y0 + inset + 4.0,
                    theme.slot.content_glint.as_rgb(),
                );
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

            let (left, top) = self.hotbar_quantity_position(index);
            specs.push(HotbarTextSpec {
                text: slot.quantity.to_string(),
                left,
                top,
                color: theme.quantity_badge.text.as_rgb8(),
            });
        }

        if let Some(notice) = hotbar.notice_text() {
            let (left, top) = self.hotbar_notice_position();
            specs.push(HotbarTextSpec {
                text: notice.to_string(),
                left,
                top,
                color: theme.quantity_badge.notice.as_rgb8(),
            });
        }

        specs
    }

    fn hotbar_quantity_position(&self, index: usize) -> (f32, f32) {
        let theme = UiTheme::VOXELVERSE;
        let layout = self.hotbar_layout();
        let x = layout.left + index as f32 * (layout.slot + layout.gap) + layout.slot
            - theme.quantity_badge.right_inset;
        let y = layout.top + layout.slot - theme.quantity_badge.bottom_inset;
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
        let gap = theme.hotbar.slot_gap;
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

    fn add_ui_rect(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: [f32; 3],
    ) {
        let width = self.config.width.max(1) as f32;
        let height = self.config.height.max(1) as f32;
        let to_ndc = |x: f32, y: f32| -> [f32; 3] {
            [(x / width) * 2.0 - 1.0, 1.0 - (y / height) * 2.0, 0.0]
        };
        let normal = [0.0, 0.0, 1.0];
        let base = verts.len() as u32;
        verts.extend([
            Vertex {
                pos: to_ndc(x0, y0),
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
            Vertex {
                pos: to_ndc(x1, y0),
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
            Vertex {
                pos: to_ndc(x0, y1),
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
            Vertex {
                pos: to_ndc(x1, y1),
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
        ]);
        inds.extend([base, base + 2, base + 1, base + 1, base + 2, base + 3]);
    }
}
