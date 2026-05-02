use glam::Vec3;

use vv_core::BlockId;
use vv_registry::{BlockId as ContentBlockId, TextureId};

use crate::{MeshGen, Vertex};

impl MeshGen {
    #[inline]
    pub(crate) fn calculate_ao(side1: bool, side2: bool, corner: bool) -> f32 {
        let mut occ = 0;

        if side1 {
            occ += 1;
        }

        if side2 {
            occ += 1;
        }

        if corner && (side1 || side2) {
            occ += 1;
        }

        match occ {
            0 => 1.0,
            1 => 0.8,
            2 => 0.6,
            _ => 0.4,
        }
    }

    #[inline]
    pub(crate) fn face_texture_id(texture: Option<TextureId>) -> i32 {
        texture.map(|id| id.raw() as i32).unwrap_or(-1)
    }

    #[inline]
    pub(crate) fn mix_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ]
    }

    #[inline]
    pub(crate) fn face_normal(pos: [Vec3; 4]) -> Vec3 {
        (pos[1] - pos[0]).cross(pos[2] - pos[0]).normalize()
    }

    pub(crate) fn rounded_corner_normals(
        base: Vec3,
        adjacent_faces: [[(bool, Vec3); 2]; 4],
        strength: f32,
    ) -> [Vec3; 4] {
        if strength <= 0.0 {
            return [base; 4];
        }

        let t = strength.clamp(0.0, 1.0);
        let mut normals = [base; 4];

        for (normal, adjacent) in normals.iter_mut().zip(adjacent_faces) {
            let mut target = Vec3::ZERO;
            let mut count = 0u32;

            for (visible, face_normal) in adjacent {
                if visible {
                    target += face_normal;
                    count += 1;
                }
            }

            if count == 0 {
                continue;
            }

            let len = target.length();
            if len < 1e-8 {
                continue;
            }

            *normal = slerp_normal(base, target / len, t);
        }

        normals
    }

    pub(crate) fn inset_face(
        mut pos: [Vec3; 4],
        exposed_edges: [bool; 4],
        width: f32,
    ) -> [Vec3; 4] {
        if width <= 0.0 {
            return pos;
        }

        let original = pos;

        if exposed_edges[0] {
            pos[0] += (original[3] - original[0]) * width;
            pos[1] += (original[2] - original[1]) * width;
        }

        if exposed_edges[1] {
            pos[1] += (original[0] - original[1]) * width;
            pos[2] += (original[3] - original[2]) * width;
        }

        if exposed_edges[2] {
            pos[2] += (original[1] - original[2]) * width;
            pos[3] += (original[0] - original[3]) * width;
        }

        if exposed_edges[3] {
            pos[3] += (original[2] - original[3]) * width;
            pos[0] += (original[1] - original[0]) * width;
        }

        pos
    }

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
        let fallback_normal = if force_radial {
            ((pos[0] + pos[1] + pos[2] + pos[3]) * 0.25).normalize()
        } else {
            (pos[1] - pos[0]).cross(pos[2] - pos[0]).normalize()
        };

        let normals = normals.unwrap_or([fallback_normal; 4]);
        let base = *idx;
        let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

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

    pub fn stable_variation_seed(
        voxel: BlockId,
        block: ContentBlockId,
        face_id: u32,
        planet_seed: u32,
    ) -> u32 {
        let mut hash = 0x811c_9dc5u32 ^ planet_seed;

        for value in [
            voxel.face as u32,
            voxel.u,
            voxel.v,
            voxel.layer,
            block.raw(),
            face_id,
        ] {
            hash ^= value.wrapping_mul(0x9e37_79b9);
            hash = hash.rotate_left(13).wrapping_mul(0x85eb_ca6b);
        }

        hash ^ (hash >> 16)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_edge_bevels(
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        block_id: i32,
        visible: [bool; 6],
        face_pos: [[Vec3; 4]; 6],
        face_style: [(i32, [[f32; 3]; 4]); 6],
        face_normals: [Vec3; 6],
        block_visual_id: u32,
        voxel_pos: [i32; 3],
        variation_seed: u32,
    ) {
        const TOP: usize = 0;
        const BOTTOM: usize = 1;
        const FRONT: usize = 2;
        const BACK: usize = 3;
        const LEFT: usize = 4;
        const RIGHT: usize = 5;

        // Sagitta-to-chord ratio for a 90° quarter-circle arc: (1/√2 − ½) ≈ 0.2071.
        // Multiplied by the chord length it gives the outward push to the arc midpoint.
        const ARC_K: f32 = 0.2071;

        let mut edge = |a: usize, b: usize, a0: usize, a1: usize, b0: usize, b1: usize| {
            if !(visible[a] && visible[b]) {
                return;
            }

            let n_a = face_normals[a];
            let n_b = face_normals[b];
            let n_mid = slerp_normal(n_a, n_b, 0.5);
            // Bisector direction: outward toward the bevel's original corner.
            let n_diag = (n_a + n_b).normalize();

            let (texture_id, colors_a) = face_style[a];
            let (_, colors_b) = face_style[b];

            let pa0 = face_pos[a][a0];
            let pa1 = face_pos[a][a1];
            let pb0 = face_pos[b][b0];
            let pb1 = face_pos[b][b1];

            // Arc midpoints: push the chord midpoint outward by the sagitta amount.
            let mid0 = (pa0 + pb0) * 0.5 + n_diag * ((pa0 - pb0).length() * ARC_K);
            let mid1 = (pa1 + pb1) * 0.5 + n_diag * ((pa1 - pb1).length() * ARC_K);

            let c_mid0 = Self::mix_color(colors_a[a0], colors_b[b0], 0.5);
            let c_mid1 = Self::mix_color(colors_a[a1], colors_b[b1], 0.5);

            // Quad A-side → arc midpoint (normals sweep from n_a to n_mid).
            Self::quad(
                verts, inds, idx,
                [pa0, pa1, mid1, mid0],
                [colors_a[a0], colors_a[a1], c_mid1, c_mid0],
                texture_id, block_id, block_visual_id, a as u32,
                voxel_pos, variation_seed, false,
                Some([n_a, n_a, n_mid, n_mid]),
            );

            // Quad arc midpoint → B-side (normals sweep from n_mid to n_b).
            Self::quad(
                verts, inds, idx,
                [mid0, mid1, pb1, pb0],
                [c_mid0, c_mid1, colors_b[b1], colors_b[b0]],
                texture_id, block_id, block_visual_id, a as u32,
                voxel_pos, variation_seed, false,
                Some([n_mid, n_mid, n_b, n_b]),
            );
        };

        edge(TOP, FRONT, 0, 1, 3, 2);
        edge(TOP, RIGHT, 1, 2, 3, 2);
        edge(TOP, BACK, 2, 3, 1, 0);
        edge(TOP, LEFT, 3, 0, 3, 2);
        edge(BOTTOM, FRONT, 2, 3, 0, 1);
        edge(BOTTOM, RIGHT, 1, 2, 0, 1);
        edge(BOTTOM, BACK, 0, 1, 3, 2);
        edge(BOTTOM, LEFT, 3, 0, 0, 1);
        edge(FRONT, LEFT, 3, 0, 2, 1);
        edge(FRONT, RIGHT, 1, 2, 0, 3);
        edge(BACK, LEFT, 3, 0, 3, 0);
        edge(BACK, RIGHT, 1, 2, 2, 1);

        let mut corner = |faces: [usize; 3], points: [(usize, usize); 3]| {
            if !faces.iter().all(|face| visible[*face]) {
                return;
            }

            let texture_id = face_style[faces[0]].0;

            let p0 = face_pos[points[0].0][points[0].1];
            let p1 = face_pos[points[1].0][points[1].1];
            let p2 = face_pos[points[2].0][points[2].1];

            let n0 = face_normals[faces[0]];
            let n1 = face_normals[faces[1]];
            let n2 = face_normals[faces[2]];
            let n_center = (n0 + n1 + n2).normalize();

            // Push the center outward to form a smooth spherical cap. The push
            // amount (~18% of average edge length) keeps continuity with the
            // adjacent round-edge arcs without over-inflating the corner.
            let avg_edge = ((p0 - p1).length() + (p1 - p2).length() + (p2 - p0).length()) / 3.0;
            let p_center = (p0 + p1 + p2) / 3.0 + n_center * (avg_edge * 0.18);

            let c0 = face_style[points[0].0].1[points[0].1];
            let c1 = face_style[points[1].0].1[points[1].1];
            let c2 = face_style[points[2].0].1[points[2].1];
            let c_center = Self::mix_color(Self::mix_color(c0, c1, 0.5), c2, 0.333);

            // 3 sub-triangles with per-vertex normals sweep toward n_center.
            Self::tri(verts, inds, idx, [p0, p1, p_center], [c0, c1, c_center],
                texture_id, block_id, block_visual_id, faces[0] as u32,
                voxel_pos, variation_seed, Some([n0, n1, n_center]));

            Self::tri(verts, inds, idx, [p1, p2, p_center], [c1, c2, c_center],
                texture_id, block_id, block_visual_id, faces[1] as u32,
                voxel_pos, variation_seed, Some([n1, n2, n_center]));

            Self::tri(verts, inds, idx, [p2, p0, p_center], [c2, c0, c_center],
                texture_id, block_id, block_visual_id, faces[2] as u32,
                voxel_pos, variation_seed, Some([n2, n0, n_center]));
        };

        corner([TOP, FRONT, LEFT], [(TOP, 0), (FRONT, 3), (LEFT, 2)]);
        corner([TOP, FRONT, RIGHT], [(TOP, 1), (RIGHT, 3), (FRONT, 2)]);
        corner([TOP, BACK, RIGHT], [(TOP, 2), (BACK, 1), (RIGHT, 2)]);
        corner([TOP, BACK, LEFT], [(TOP, 3), (LEFT, 3), (BACK, 0)]);
        corner([BOTTOM, FRONT, LEFT], [(BOTTOM, 3), (LEFT, 1), (FRONT, 0)]);
        corner(
            [BOTTOM, FRONT, RIGHT],
            [(BOTTOM, 2), (FRONT, 1), (RIGHT, 0)],
        );
        corner([BOTTOM, BACK, RIGHT], [(BOTTOM, 1), (RIGHT, 1), (BACK, 2)]);
        corner([BOTTOM, BACK, LEFT], [(BOTTOM, 0), (BACK, 3), (LEFT, 0)]);
    }
}

#[inline]
fn slerp_normal(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    let dot = a.dot(b).clamp(-1.0, 1.0);
    let theta = dot.acos();
    if theta < 1e-6 {
        return a;
    }
    let sin_theta = theta.sin();
    (a * ((1.0 - t) * theta).sin() + b * (t * theta).sin()) / sin_theta
}
