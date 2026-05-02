use glam::Vec3;

use crate::{MeshGen, Vertex};

use super::normals::slerp_normal;

const EDGE_STEPS: usize = 5;
const EDGE_BULGE: f32 = 0.18;
const CORNER_BULGE: f32 = 0.24;

impl MeshGen {
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

        let mut edge = |a: usize, b: usize, a0: usize, a1: usize, b0: usize, b1: usize| {
            if !(visible[a] && visible[b]) {
                return;
            }

            let n_a = face_normals[a].normalize_or_zero();
            let n_b = face_normals[b].normalize_or_zero();

            if n_a.length_squared() < 1e-8 || n_b.length_squared() < 1e-8 {
                return;
            }

            let (texture_id, colors_a) = face_style[a];
            let (_, colors_b) = face_style[b];

            let pa0 = face_pos[a][a0];
            let pa1 = face_pos[a][a1];
            let pb0 = face_pos[b][b0];
            let pb1 = face_pos[b][b1];

            let chord0 = (pb0 - pa0).length();
            let chord1 = (pb1 - pa1).length();

            let mut strip_a = [Vec3::ZERO; EDGE_STEPS + 1];
            let mut strip_b = [Vec3::ZERO; EDGE_STEPS + 1];
            let mut normals = [Vec3::Y; EDGE_STEPS + 1];
            let mut colors_0 = [[0.0; 3]; EDGE_STEPS + 1];
            let mut colors_1 = [[0.0; 3]; EDGE_STEPS + 1];

            for i in 0..=EDGE_STEPS {
                let t = i as f32 / EDGE_STEPS as f32;

                let n = slerp_normal(n_a, n_b, t).normalize_or_zero();
                let bulge = (std::f32::consts::PI * t).sin();

                strip_a[i] = pa0.lerp(pb0, t) + n * chord0 * EDGE_BULGE * bulge;
                strip_b[i] = pa1.lerp(pb1, t) + n * chord1 * EDGE_BULGE * bulge;

                normals[i] = n;

                colors_0[i] = Self::mix_color(colors_a[a0], colors_b[b0], t);
                colors_1[i] = Self::mix_color(colors_a[a1], colors_b[b1], t);
            }

            for i in 0..EDGE_STEPS {
                Self::quad(
                    verts,
                    inds,
                    idx,
                    [strip_a[i], strip_b[i], strip_b[i + 1], strip_a[i + 1]],
                    [colors_0[i], colors_1[i], colors_1[i + 1], colors_0[i + 1]],
                    texture_id,
                    block_id,
                    block_visual_id,
                    a as u32,
                    voxel_pos,
                    variation_seed,
                    false,
                    Some([normals[i], normals[i], normals[i + 1], normals[i + 1]]),
                );
            }
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

            let n0 = face_normals[faces[0]].normalize_or_zero();
            let n1 = face_normals[faces[1]].normalize_or_zero();
            let n2 = face_normals[faces[2]].normalize_or_zero();

            if n0.length_squared() < 1e-8
                || n1.length_squared() < 1e-8
                || n2.length_squared() < 1e-8
            {
                return;
            }

            let n01 = slerp_normal(n0, n1, 0.5).normalize_or_zero();
            let n12 = slerp_normal(n1, n2, 0.5).normalize_or_zero();
            let n20 = slerp_normal(n2, n0, 0.5).normalize_or_zero();
            let n_center = (n0 + n1 + n2).normalize_or_zero();

            let avg_len = ((p0 - p1).length() + (p1 - p2).length() + (p2 - p0).length()) / 3.0;

            let m01 = (p0 + p1) * 0.5 + n01 * avg_len * EDGE_BULGE;
            let m12 = (p1 + p2) * 0.5 + n12 * avg_len * EDGE_BULGE;
            let m20 = (p2 + p0) * 0.5 + n20 * avg_len * EDGE_BULGE;
            let center = (p0 + p1 + p2) / 3.0 + n_center * avg_len * CORNER_BULGE;

            let c0 = face_style[points[0].0].1[points[0].1];
            let c1 = face_style[points[1].0].1[points[1].1];
            let c2 = face_style[points[2].0].1[points[2].1];

            let c01 = Self::mix_color(c0, c1, 0.5);
            let c12 = Self::mix_color(c1, c2, 0.5);
            let c20 = Self::mix_color(c2, c0, 0.5);
            let cc = Self::mix_color(Self::mix_color(c0, c1, 0.5), c2, 0.333);

            Self::tri(
                verts,
                inds,
                idx,
                [p0, m01, m20],
                [c0, c01, c20],
                texture_id,
                block_id,
                block_visual_id,
                faces[0] as u32,
                voxel_pos,
                variation_seed,
                Some([n0, n01, n20]),
            );

            Self::tri(
                verts,
                inds,
                idx,
                [m01, p1, m12],
                [c01, c1, c12],
                texture_id,
                block_id,
                block_visual_id,
                faces[1] as u32,
                voxel_pos,
                variation_seed,
                Some([n01, n1, n12]),
            );

            Self::tri(
                verts,
                inds,
                idx,
                [m20, m12, p2],
                [c20, c12, c2],
                texture_id,
                block_id,
                block_visual_id,
                faces[2] as u32,
                voxel_pos,
                variation_seed,
                Some([n20, n12, n2]),
            );

            Self::tri(
                verts,
                inds,
                idx,
                [m01, m12, center],
                [c01, c12, cc],
                texture_id,
                block_id,
                block_visual_id,
                faces[0] as u32,
                voxel_pos,
                variation_seed,
                Some([n01, n12, n_center]),
            );

            Self::tri(
                verts,
                inds,
                idx,
                [m12, m20, center],
                [c12, c20, cc],
                texture_id,
                block_id,
                block_visual_id,
                faces[1] as u32,
                voxel_pos,
                variation_seed,
                Some([n12, n20, n_center]),
            );

            Self::tri(
                verts,
                inds,
                idx,
                [m20, m01, center],
                [c20, c01, cc],
                texture_id,
                block_id,
                block_visual_id,
                faces[2] as u32,
                voxel_pos,
                variation_seed,
                Some([n20, n01, n_center]),
            );
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
