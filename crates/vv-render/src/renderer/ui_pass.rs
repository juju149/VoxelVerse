use super::Renderer;
use crate::render_pipeline_desc::PipelineId;

impl<'a> Renderer<'a> {
    pub(super) fn render_ui_mesh_pass(
        &mut self,
        enc: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        is_first_person: bool,
    ) -> usize {
        let mut draw_calls = 0usize;
        let mut ui_pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("UI Mesh Pass"),
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
        ui_pass.set_pipeline(self.pipeline(PipelineId::Ui));
        ui_pass.set_bind_group(0, &self.global_bind_identity, &[]);
        ui_pass.set_bind_group(1, &self.local_bind_identity, &[]);
        ui_pass.set_bind_group(2, &self.atlas_bind, &[]);

        if is_first_person && self.first_person_inds > 0 {
            ui_pass.set_vertex_buffer(0, self.first_person_v_buf.slice(..));
            ui_pass.set_index_buffer(self.first_person_i_buf.slice(..), wgpu::IndexFormat::Uint32);
            ui_pass.draw_indexed(0..self.first_person_inds, 0, 0..1);
            draw_calls += 1;
        }

        if self.hotbar_inds > 0 {
            ui_pass.set_vertex_buffer(0, self.hotbar_v_buf.slice(..));
            ui_pass.set_index_buffer(self.hotbar_i_buf.slice(..), wgpu::IndexFormat::Uint32);
            ui_pass.draw_indexed(0..self.hotbar_inds, 0, 0..1);
            draw_calls += 1;
        }

        if self.inventory_inds > 0 {
            ui_pass.set_vertex_buffer(0, self.inventory_v_buf.slice(..));
            ui_pass.set_index_buffer(self.inventory_i_buf.slice(..), wgpu::IndexFormat::Uint32);
            ui_pass.draw_indexed(0..self.inventory_inds, 0, 0..1);
            draw_calls += 1;
        }

        if self.console_inds > 0 {
            ui_pass.set_vertex_buffer(0, self.console_v_buf.slice(..));
            ui_pass.set_index_buffer(self.console_i_buf.slice(..), wgpu::IndexFormat::Uint32);
            ui_pass.draw_indexed(0..self.console_inds, 0, 0..1);
            draw_calls += 1;
        }

        draw_calls
    }
}
