use glam::Vec3;

use crate::{MeshGen, Vertex};

impl MeshGen {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn quad(
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        pos: [Vec3; 4],
        colors: [[f32; 3]; 4],
        texture_id: i32,
        block_id: i32,
        block_visual_id: u32,
        face_id: u32,
        voxel_pos: [i32; 3],
        variation_seed: u32,
        force_radial: bool,
        normals: Option<[Vec3; 4]>,
    ) {
        let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

        Self::quad_with_uvs(
            verts,
            inds,
            idx,
            pos,
            colors,
            uvs,
            texture_id,
            block_id,
            block_visual_id,
            face_id,
            voxel_pos,
            variation_seed,
            force_radial,
            normals,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn quad_with_uvs(
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        pos: [Vec3; 4],
        colors: [[f32; 3]; 4],
        uvs: [[f32; 2]; 4],
        texture_id: i32,
        block_id: i32,
        block_visual_id: u32,
        face_id: u32,
        voxel_pos: [i32; 3],
        variation_seed: u32,
        force_radial: bool,
        normals: Option<[Vec3; 4]>,
    ) {
        let fallback_normal = if force_radial {
            ((pos[0] + pos[1] + pos[2] + pos[3]) * 0.25).normalize_or_zero()
        } else {
            (pos[1] - pos[0]).cross(pos[2] - pos[0]).normalize_or_zero()
        };

        let normals = normals.unwrap_or([fallback_normal; 4]);
        let base = *idx;

        for (((p, c), uv), normal) in pos.iter().zip(colors.iter()).zip(uvs).zip(normals) {
            verts.push(Vertex {
                pos: p.to_array(),
                color: *c,
                normal: normal.to_array(),
                uv,
                texture_id,
                block_id,
                block_visual_id,
                face_id,
                voxel_pos,
                variation_seed,
                ao: c[0].max(c[1]).max(c[2]).clamp(0.0, 1.0),
            });
        }

        inds.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
        *idx += 4;
    }
}
