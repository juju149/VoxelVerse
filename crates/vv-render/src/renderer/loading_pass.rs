use super::Renderer;
use glyphon::{Attrs, Buffer, Family, Metrics, Resolution, Shaping, TextArea, TextBounds};

impl<'a> Renderer<'a> {
    pub fn render_loading(&mut self, progress: f32, message: &str) {
        let progress = progress.clamp(0.0, 1.0);
        self.window
            .set_title(&format!("VoxelVerse - chargement {:.0}%", progress * 100.0));

        let Ok(out) = self.surface.get_current_texture() else {
            return;
        };
        let view = out
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let _pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Loading Clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.03,
                            g: 0.04,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        let filled = (progress * 28.0).round() as usize;
        let bar = format!(
            "[{}{}] {:.0}%",
            "#".repeat(filled),
            "-".repeat(28usize.saturating_sub(filled)),
            progress * 100.0
        );
        let text = format!("VoxelVerse\n{}\n{}", message, bar);
        let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(24.0, 34.0));
        buffer.set_size(
            &mut self.font_system,
            self.config.width as f32,
            self.config.height as f32,
        );
        buffer.set_text(
            &mut self.font_system,
            &text,
            Attrs::new()
                .family(Family::Monospace)
                .color(glyphon::Color::rgb(230, 240, 235)),
            Shaping::Advanced,
        );

        let text_area = TextArea {
            buffer: &buffer,
            left: 48.0,
            top: (self.config.height as f32 * 0.5 - 70.0).max(40.0),
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: 0,
                right: self.config.width as i32,
                bottom: self.config.height as i32,
            },
            default_color: glyphon::Color::rgb(255, 255, 255),
        };

        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.text_atlas,
                Resolution {
                    width: self.config.width,
                    height: self.config.height,
                },
                vec![text_area],
                &mut self.swash_cache,
            )
            .ok();

        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Loading Text"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
            let _ = self.text_renderer.render(&self.text_atlas, &mut pass);
        }

        self.queue.submit(std::iter::once(enc.finish()));
        out.present();
        self.device.poll(wgpu::Maintain::Wait);
    }
}
