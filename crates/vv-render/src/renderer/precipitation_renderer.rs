//! Procedural screen-space precipitation overlay.
//!
//! Phase 3.B of the weather/cosmos roadmap. Reads `weather_params` from the
//! global uniform and draws rain streaks / snow flakes additively over the
//! current scene colour. Cheap by construction: one full-screen triangle, no
//! depth read, early-out in the shader when `intensity == 0`.

use super::Renderer;
use crate::render_pipeline_desc::PipelineId;

impl<'a> Renderer<'a> {
    pub(super) fn render_precipitation(&self, enc: &mut wgpu::CommandEncoder) {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Precipitation Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.scene.view,
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
        pass.set_pipeline(self.pipeline(PipelineId::Precipitation));
        pass.set_bind_group(0, &self.global_bind, &[]);
        pass.draw(0..3, 0..1);
    }
}
