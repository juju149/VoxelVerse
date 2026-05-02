use glam::Vec3;

use crate::MeshGen;

use super::{VoxelCorners, VoxelOcclusion};

#[derive(Clone, Copy, Debug)]
pub(crate) struct VoxelFacePositions {
    pub top: [Vec3; 4],
    pub bottom: [Vec3; 4],
    pub front: [Vec3; 4],
    pub back: [Vec3; 4],
    pub left: [Vec3; 4],
    pub right: [Vec3; 4],
}

impl VoxelFacePositions {
    #[inline]
    pub(crate) fn as_array(self) -> [[Vec3; 4]; 6] {
        [
            self.top,
            self.bottom,
            self.front,
            self.back,
            self.left,
            self.right,
        ]
    }
}

impl MeshGen {
    pub(crate) fn sculpted_face_positions(
        c: VoxelCorners,
        occ: VoxelOcclusion,
        edge_width: f32,
    ) -> VoxelFacePositions {
        let w = edge_width.clamp(0.0, 0.22);

        let top_visible = !occ.top;
        let bottom_visible = !occ.bottom;
        let front_visible = !occ.front;
        let back_visible = !occ.back;
        let left_visible = !occ.left;
        let right_visible = !occ.right;

        VoxelFacePositions {
            top: Self::inset_face(
                [c.o_bl, c.o_br, c.o_tr, c.o_tl],
                [front_visible, right_visible, back_visible, left_visible],
                w,
            ),
            bottom: Self::inset_face(
                [c.i_tl, c.i_tr, c.i_br, c.i_bl],
                [back_visible, right_visible, front_visible, left_visible],
                w,
            ),
            front: Self::inset_face(
                [c.i_bl, c.i_br, c.o_br, c.o_bl],
                [bottom_visible, right_visible, top_visible, left_visible],
                w,
            ),
            back: Self::inset_face(
                [c.o_tl, c.o_tr, c.i_tr, c.i_tl],
                [top_visible, right_visible, bottom_visible, left_visible],
                w,
            ),
            left: Self::inset_face(
                [c.i_tl, c.i_bl, c.o_bl, c.o_tl],
                [bottom_visible, front_visible, top_visible, back_visible],
                w,
            ),
            right: Self::inset_face(
                [c.i_br, c.i_tr, c.o_tr, c.o_br],
                [bottom_visible, back_visible, top_visible, front_visible],
                w,
            ),
        }
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
}
