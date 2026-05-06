use super::Renderer;
use crate::content::TerrainPalette;
use crate::generation::CoordSystem;
use crate::rendering::Vertex;
use crate::voxel::VoxelCoord;
use crate::world::PlanetData;
use glam::Vec3;

impl<'a> Renderer<'a> {
    pub fn update_cursor(&mut self, planet: &PlanetData, id: Option<VoxelCoord>) {
        if let Some(id) = id {
            let res = planet.resolution;
            let p = |u, v, l| {
                CoordSystem::get_vertex_pos(id.face, id.u + u, id.v + v, id.layer + l, res)
            };

            let corners = [
                p(0, 0, 0),
                p(1, 0, 0),
                p(0, 1, 0),
                p(1, 1, 0),
                p(0, 0, 1),
                p(1, 0, 1),
                p(0, 1, 1),
                p(1, 1, 1),
            ];

            let edges = [
                (0, 1),
                (1, 3),
                (3, 2),
                (2, 0),
                (4, 5),
                (5, 7),
                (7, 6),
                (6, 4),
                (0, 4),
                (1, 5),
                (2, 6),
                (3, 7),
            ];

            let mut verts = Vec::new();
            let mut inds = Vec::new();
            let thickness = 0.025;
            let color = TerrainPalette::CURSOR;
            let mut idx_base = 0;

            for (start, end) in edges {
                let a = corners[start];
                let b = corners[end];
                let dir = (b - a).normalize();
                let ref_up = if dir.dot(Vec3::Y).abs() > 0.9 {
                    Vec3::X
                } else {
                    Vec3::Y
                };
                let right = dir.cross(ref_up).normalize() * thickness;
                let up = dir.cross(right).normalize() * thickness;
                let offsets = [(-right - up), (right - up), (right + up), (-right + up)];

                for off in offsets {
                    verts.push(Vertex {
                        pos: (a + off).to_array(),
                        uv: [0.0, 0.0],
                        color,
                        normal: [0.0; 3],
                        tex_index: 0,
                    });
                    verts.push(Vertex {
                        pos: (b + off).to_array(),
                        uv: [0.0, 0.0],
                        color,
                        normal: [0.0; 3],
                        tex_index: 0,
                    });
                }

                let faces = [(0, 1, 3, 2), (2, 3, 5, 4), (4, 5, 7, 6), (6, 7, 1, 0)];
                for (i0, i1, i2, i3) in faces {
                    inds.push(idx_base + i0);
                    inds.push(idx_base + i1);
                    inds.push(idx_base + i2);
                    inds.push(idx_base + i2);
                    inds.push(idx_base + i3);
                    inds.push(idx_base + i0);
                }
                idx_base += 8;
            }

            self.queue
                .write_buffer(&self.cursor_v_buf, 0, bytemuck::cast_slice(&verts));
            self.queue
                .write_buffer(&self.cursor_i_buf, 0, bytemuck::cast_slice(&inds));
            self.cursor_inds = inds.len() as u32;
        } else {
            self.cursor_inds = 0;
        }
    }
}
