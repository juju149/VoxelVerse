use super::text_cache::TextSlot;
use super::Renderer;
use crate::snapshot::RenderFrameSnapshot;
use glyphon::{Resolution, TextArea, TextBounds};
use std::collections::HashSet;

struct PendingText {
    slot: TextSlot,
    text: String,
    size: f32,
    color: [u8; 3],
    left: f32,
    top: f32,
}

impl<'a> Renderer<'a> {
    pub(super) fn render_text_pass(
        &mut self,
        enc: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        frame: &RenderFrameSnapshot<'_>,
        rendered_chunks: usize,
        rendered_lods: usize,
    ) {
        let mut pending = Vec::new();
        self.push_console_text(&mut pending, frame);
        self.push_hotbar_text(&mut pending, frame);
        self.push_inventory_text(&mut pending, frame);
        self.push_frame_stats_text(&mut pending);
        self.push_debug_text(&mut pending, frame, rendered_chunks, rendered_lods);
        self.prepare_and_render_text(enc, view, pending);
    }

    fn push_console_text(&self, pending: &mut Vec<PendingText>, frame: &RenderFrameSnapshot<'_>) {
        let console = &frame.console;
        if console.height_fraction <= 0.0 {
            return;
        }

        let console_pixel_height = (self.config.height as f32 / 2.0) * console.height_fraction;
        let start_y = console_pixel_height - 40.0;
        let line_height = 20.0;

        for (i, (line_text, color)) in console.history.iter().rev().enumerate() {
            let y = start_y - (i as f32 * line_height);
            if y < 0.0 {
                break;
            }
            pending.push(PendingText {
                slot: TextSlot::console_history(i as u32),
                text: line_text.clone(),
                size: 16.0,
                color: [
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                ],
                left: 10.0,
                top: y,
            });
        }

        let input_y = console_pixel_height - 20.0;
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let cursor = if (time / 500).is_multiple_of(2) {
            "_"
        } else {
            " "
        };
        pending.push(PendingText {
            slot: TextSlot::console_input(),
            text: format!("> {}{}", console.input_buffer, cursor),
            size: 16.0,
            color: [255, 255, 0],
            left: 10.0,
            top: input_y,
        });
    }

    fn push_hotbar_text(&self, pending: &mut Vec<PendingText>, frame: &RenderFrameSnapshot<'_>) {
        for (i, spec) in self
            .hotbar_text_specs(&frame.hotbar)
            .into_iter()
            .enumerate()
        {
            pending.push(PendingText {
                slot: TextSlot::hotbar_quantity(i as u32),
                text: spec.text,
                size: spec.size.max(8.0),
                color: spec.color,
                left: spec.left,
                top: spec.top,
            });
        }
    }

    fn push_inventory_text(&self, pending: &mut Vec<PendingText>, frame: &RenderFrameSnapshot<'_>) {
        for (i, spec) in self
            .inventory_text_specs(
                &frame.inventory,
                &frame.hotbar,
                &frame.ui.inventory,
                frame.planet,
                &frame.craft,
            )
            .into_iter()
            .enumerate()
        {
            pending.push(PendingText {
                slot: TextSlot::inventory_spec(i as u32),
                text: spec.text,
                size: spec.size.max(8.0),
                color: spec.color,
                left: spec.left,
                top: spec.top,
            });
        }
    }

    fn push_frame_stats_text(&self, pending: &mut Vec<PendingText>) {
        pending.push(PendingText {
            slot: TextSlot::FPS,
            text: format!("FPS: {}", self.frame_stats.fps()),
            size: 20.0,
            color: [0, 255, 0],
            left: self.config.width as f32 - 120.0,
            top: 10.0,
        });
    }

    fn push_debug_text(
        &self,
        pending: &mut Vec<PendingText>,
        frame: &RenderFrameSnapshot<'_>,
        rendered_chunks: usize,
        rendered_lods: usize,
    ) {
        let debug = &frame.debug;
        let show_engine_debug = debug.debug_mode || self.engine_debug_page;
        if !show_engine_debug {
            return;
        }

        let status = if debug.freeze_culling {
            "FROZEN"
        } else {
            "ACTIVE"
        };
        let stats = self.render_stats(rendered_chunks, rendered_lods);
        let target = frame
            .camera
            .cursor_id
            .map(|id| format!("f{} l{} u{} v{}", id.face, id.layer, id.u, id.v));
        let info = stats.debug_overlay(
            status,
            self.frame_stats.frame_time_ms(),
            frame.camera.player_pos.to_array(),
            target,
        );
        pending.push(PendingText {
            slot: TextSlot::DEBUG,
            text: info,
            size: 14.0,
            color: [200, 200, 200],
            left: self.config.width as f32 - 180.0,
            top: 40.0,
        });
    }

    fn prepare_and_render_text(
        &mut self,
        enc: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        pending: Vec<PendingText>,
    ) {
        let viewport_w = self.config.width;
        let viewport_h = self.config.height;
        for item in &pending {
            self.text_cache.ensure(
                item.slot,
                &mut self.font_system,
                &item.text,
                item.size,
                item.color,
                viewport_w,
                viewport_h,
            );
        }

        let used: HashSet<TextSlot> = pending.iter().map(|p| p.slot).collect();
        self.text_cache.retain(|slot| used.contains(slot));

        let text_areas: Vec<TextArea> = pending
            .iter()
            .filter_map(|item| {
                self.text_cache.get(item.slot).map(|buffer| TextArea {
                    buffer,
                    left: item.left,
                    top: item.top,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: viewport_w as i32,
                        bottom: viewport_h as i32,
                    },
                    default_color: glyphon::Color::rgb(255, 255, 255),
                })
            })
            .collect();

        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.text_atlas,
                Resolution {
                    width: viewport_w,
                    height: viewport_h,
                },
                text_areas,
                &mut self.swash_cache,
            )
            .unwrap();

        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Text Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.text_renderer
            .render(&self.text_atlas, &mut pass)
            .unwrap();
    }
}
