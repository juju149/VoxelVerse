//! Celestial overlay pass (Phase 5.B of the weather/cosmos roadmap).
//!
//! Runs between the sky pass and the clouds pass so:
//! - the base sky gradient + sun (from the sky pass) sits underneath,
//! - clouds drawn afterwards naturally occlude stars and aurora.
//!
//! Reads `celestial_params` and `celestial_moon` from the global uniform.
//! The shader self-skips when every weighted source is zero, so the pass
//! costs near nothing without a `CelestialState` snapshot.

use super::Renderer;
use crate::pipeline::desc::PipelineId;

impl<'a> Renderer<'a> {
    pub(super) fn render_celestial(&self, enc: &mut wgpu::CommandEncoder) {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Celestial Pass"),
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
        pass.set_pipeline(self.pipeline(PipelineId::Celestial));
        pass.set_bind_group(0, &self.global_bind, &[]);
        pass.draw(0..3, 0..1);
    }
}
