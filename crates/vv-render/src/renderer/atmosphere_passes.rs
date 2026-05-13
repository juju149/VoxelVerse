use super::Renderer;

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
        pass.set_pipeline(&self.pipeline_clouds);
        pass.set_bind_group(0, &self.sky_global_bind, &[]);
        pass.draw(0..3, 0..1);
    }

    pub(super) fn render_volumetric_fog(&self, enc: &mut wgpu::CommandEncoder) {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Volumetric Fog Pass"),
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
        pass.set_pipeline(&self.pipeline_volumetric_fog);
        pass.set_bind_group(0, &self.sky_global_bind, &[]);
        pass.draw(0..3, 0..1);
    }

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
        pass.set_pipeline(&self.pipeline_post);
        pass.set_bind_group(0, &self.sky_global_bind, &[]);
        pass.set_bind_group(1, &self.post_bind, &[]);
        pass.draw(0..3, 0..1);
    }
}
