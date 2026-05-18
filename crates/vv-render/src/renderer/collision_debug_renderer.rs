use super::Renderer;
use crate::pipeline::desc::PipelineId;
use crate::types::Vertex;
use glam::Vec3;
use vv_meshing::{CpuMesh, CpuVertex};
use vv_world::{PlanetData, PlanetGeometry};

/// Generate a wireframe mesh visualising the solid voxels around `player_pos`.
fn build_collision_debug_mesh(player_pos: Vec3, planet: &PlanetData) -> CpuMesh {
    let mut verts = Vec::new();
    let mut inds = Vec::new();
    let res = planet.resolution();
    let profile = planet.profile();
    let color = [1.0_f32, 0.0, 0.0];
    let normal = [0.0_f32, 1.0, 0.0];
    let range = 2_i32;

    if let Some((center_id, _)) = PlanetGeometry::get_local_coords(player_pos, profile) {
        let start_u = (center_id.u as i32 - range).max(0);
        let end_u = (center_id.u as i32 + range).min(res as i32 - 1);
        let start_v = (center_id.v as i32 - range).max(0);
        let end_v = (center_id.v as i32 + range).min(res as i32 - 1);
        let start_l = (center_id.layer as i32 - range).max(0);
        let end_l = (center_id.layer as i32 + range).min(res as i32 - 1);
        let mut idx = 0u32;

        for l in start_l..=end_l {
            for v in start_v..=end_v {
                for u in start_u..=end_u {
                    use vv_voxel::VoxelCoord;
                    let id = VoxelCoord {
                        face: center_id.face,
                        layer: l as u32,
                        u: u as u32,
                        v: v as u32,
                    };
                    let block_pos =
                        PlanetGeometry::get_block_center(id.face, id.u, id.v, id.layer, profile);
                    if !vv_physics::Physics::is_solid(block_pos, planet) {
                        continue;
                    }

                    let get_p = |uu, vv, ll| {
                        PlanetGeometry::get_vertex_pos(
                            id.face,
                            id.u + uu,
                            id.v + vv,
                            id.layer + ll,
                            profile,
                        )
                    };
                    let c000 = get_p(0, 0, 0);
                    let c100 = get_p(1, 0, 0);
                    let c010 = get_p(0, 1, 0);
                    let c110 = get_p(1, 1, 0);
                    let c001 = get_p(0, 0, 1);
                    let c101 = get_p(1, 0, 1);
                    let c011 = get_p(0, 1, 1);
                    let c111 = get_p(1, 1, 1);
                    let center = (c000 + c100 + c010 + c110 + c001 + c101 + c011 + c111) * 0.125;
                    let shrink = 0.90_f32;
                    let v = |p: Vec3| CpuVertex {
                        pos: (center + (p - center) * shrink).to_array(),
                        uv: [0.0, 0.0],
                        color,
                        normal,
                        tex_index: 0,
                    };
                    let corners = [
                        v(c000),
                        v(c100),
                        v(c110),
                        v(c010),
                        v(c001),
                        v(c101),
                        v(c111),
                        v(c011),
                    ];
                    for c in &corners {
                        verts.push(*c);
                    }
                    let base = idx;
                    for (s, e) in [
                        (0, 1),
                        (1, 2),
                        (2, 3),
                        (3, 0),
                        (4, 5),
                        (5, 6),
                        (6, 7),
                        (7, 4),
                        (0, 4),
                        (1, 5),
                        (2, 6),
                        (3, 7),
                    ] {
                        inds.push(base + s);
                        inds.push(base + e);
                    }
                    idx += 8;
                }
            }
        }
    }
    CpuMesh::new(verts, inds)
}

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

        let mesh = build_collision_debug_mesh(player_pos, planet);
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
