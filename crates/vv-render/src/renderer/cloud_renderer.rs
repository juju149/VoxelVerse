use super::Renderer;
use crate::pipeline::desc::PipelineId;

impl<'a> Renderer<'a> {
    pub(super) fn render_clouds(&self, enc: &mut wgpu::CommandEncoder) {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clouds Pass"),
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
        pass.set_pipeline(self.pipeline(PipelineId::Clouds));
        pass.set_bind_group(0, &self.global_bind, &[]);
        pass.draw(0..3, 0..1);
    }
}
