use super::Renderer;
use crate::pipeline::desc::PipelineId;

impl<'a> Renderer<'a> {
    pub(super) fn render_final_composite(
        &self,
        enc: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Final Composite Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(self.pipeline(PipelineId::FinalComposite));
        pass.set_bind_group(0, &self.global_bind, &[]);
        pass.set_bind_group(1, &self.post_bind, &[]);
        pass.draw(0..3, 0..1);
    }
}
