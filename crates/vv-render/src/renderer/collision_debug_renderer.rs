use super::Renderer;
use crate::pipeline::desc::PipelineId;
use crate::types::Vertex;
use vv_meshing::MeshGen;
use vv_world::PlanetData;

impl<'a> Renderer<'a> {
    pub(super) fn update_collision_debug_mesh(
        &mut self,
        enabled: bool,
        player_pos: glam::Vec3,
        planet: &PlanetData,
    ) {
        if !enabled {
            self.collision_inds = 0;
            return;
        }

        let mesh = MeshGen::generate_collision_debug(player_pos, planet);
        let gpu_v: Vec<Vertex> = mesh.vertices.iter().copied().map(Vertex::from).collect();
        self.queue
            .write_buffer(&self.collision_v_buf, 0, bytemuck::cast_slice(&gpu_v));
        self.queue.write_buffer(
            &self.collision_i_buf,
            0,
            bytemuck::cast_slice(&mesh.indices),
        );
        self.collision_inds = mesh.indices.len() as u32;
    }

    pub(super) fn draw_collision_debug<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
    ) -> usize {
        if self.collision_inds == 0 {
            return 0;
        }

        pass.set_pipeline(self.pipeline(PipelineId::DebugLine));
        pass.set_bind_group(0, &self.global_bind, &[]);
        pass.set_bind_group(1, &self.local_bind_identity, &[]);
        pass.set_vertex_buffer(0, self.collision_v_buf.slice(..));
        pass.set_index_buffer(self.collision_i_buf.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..self.collision_inds, 0, 0..1);
        1
    }
}
