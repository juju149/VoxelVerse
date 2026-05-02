use glam::Vec3;

use crate::{MeshGen, Vertex};

impl MeshGen {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn tri(
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        pos: [Vec3; 3],
        colors: [[f32; 3]; 3],
        texture_id: i32,
        block_id: i32,
        block_visual_id: u32,
        face_id: u32,
        voxel_pos: [i32; 3],
        variation_seed: u32,
        normals: Option<[Vec3; 3]>,
    ) {
        let fallback = (pos[1] - pos[0]).cross(pos[2] - pos[0]).normalize();
        let normals = normals.unwrap_or([fallback; 3]);
        let base = *idx;
        let uvs = [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];

        for (((p, color), uv), normal) in pos.iter().zip(colors).zip(uvs).zip(normals) {
            verts.push(Vertex {
                pos: p.to_array(),
                color,
                normal: normal.to_array(),
                uv,
                texture_id,
                block_id,
                block_visual_id,
                face_id,
                voxel_pos,
                variation_seed,
                ao: color[0].max(color[1]).max(color[2]).clamp(0.0, 1.0),
            });
        }

        inds.extend_from_slice(&[base, base + 1, base + 2]);
        *idx += 3;
    }
}
