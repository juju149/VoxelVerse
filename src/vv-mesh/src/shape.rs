use glam::Vec3;
use vv_core::BlockId;
use vv_planet::CoordSystem;
use vv_world_runtime::PlanetData;

use crate::MeshGen;

#[derive(Clone, Copy, Debug)]
pub(crate) struct VoxelOcclusion {
    pub top: bool,
    pub bottom: bool,
    pub front: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
}

impl VoxelOcclusion {
    #[inline]
    pub(crate) fn all_occluded(self) -> bool {
        self.top && self.bottom && self.front && self.back && self.left && self.right
    }

    #[inline]
    pub(crate) fn visible_array(self) -> [bool; 6] {
        [
            !self.top,
            !self.bottom,
            !self.front,
            !self.back,
            !self.left,
            !self.right,
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct VoxelCorners {
    pub i_bl: Vec3,
    pub i_br: Vec3,
    pub i_tl: Vec3,
    pub i_tr: Vec3,
    pub o_bl: Vec3,
    pub o_br: Vec3,
    pub o_tl: Vec3,
    pub o_tr: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct VoxelFaceNormals {
    pub top: Vec3,
    pub bottom: Vec3,
    pub front: Vec3,
    pub back: Vec3,
    pub left: Vec3,
    pub right: Vec3,
}

impl VoxelFaceNormals {
    #[inline]
    pub(crate) fn as_array(self) -> [Vec3; 6] {
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
    pub(crate) fn voxel_corners(id: BlockId, data: &PlanetData) -> VoxelCorners {
        let p = |u_off: u32, v_off: u32, l_off: u32| {
            CoordSystem::get_vertex_pos(
                id.face,
                id.u + u_off,
                id.v + v_off,
                id.layer + l_off,
                data.geometry,
            )
        };

        VoxelCorners {
            i_bl: p(0, 0, 0),
            i_br: p(1, 0, 0),
            i_tl: p(0, 1, 0),
            i_tr: p(1, 1, 0),
            o_bl: p(0, 0, 1),
            o_br: p(1, 0, 1),
            o_tl: p(0, 1, 1),
            o_tr: p(1, 1, 1),
        }
    }

    pub(crate) fn voxel_face_normals(c: VoxelCorners) -> VoxelFaceNormals {
        let top = safe_normalize((c.o_bl + c.o_br + c.o_tr + c.o_tl) * 0.25);
        let bottom = safe_normalize((c.i_tl + c.i_tr + c.i_br + c.i_bl) * 0.25);

        VoxelFaceNormals {
            top,
            bottom,
            front: Self::face_normal([c.i_bl, c.i_br, c.o_br, c.o_bl]),
            back: Self::face_normal([c.o_tl, c.o_tr, c.i_tr, c.i_tl]),
            left: Self::face_normal([c.i_tl, c.i_bl, c.o_bl, c.o_tl]),
            right: Self::face_normal([c.i_br, c.i_tr, c.o_tr, c.o_br]),
        }
    }

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

    pub(crate) fn top_corner_normals(
        n: VoxelFaceNormals,
        occ: VoxelOcclusion,
        strength: f32,
    ) -> [Vec3; 4] {
        Self::rounded_corner_normals(
            n.top,
            [
                [(!occ.left, n.left), (!occ.front, n.front)],
                [(!occ.right, n.right), (!occ.front, n.front)],
                [(!occ.right, n.right), (!occ.back, n.back)],
                [(!occ.left, n.left), (!occ.back, n.back)],
            ],
            strength,
        )
    }

    pub(crate) fn bottom_corner_normals(
        n: VoxelFaceNormals,
        occ: VoxelOcclusion,
        strength: f32,
    ) -> [Vec3; 4] {
        Self::rounded_corner_normals(
            n.bottom,
            [
                [(!occ.left, n.left), (!occ.back, n.back)],
                [(!occ.right, n.right), (!occ.back, n.back)],
                [(!occ.right, n.right), (!occ.front, n.front)],
                [(!occ.left, n.left), (!occ.front, n.front)],
            ],
            strength,
        )
    }

    pub(crate) fn front_corner_normals(
        n: VoxelFaceNormals,
        occ: VoxelOcclusion,
        strength: f32,
    ) -> [Vec3; 4] {
        Self::rounded_corner_normals(
            n.front,
            [
                [(!occ.bottom, n.bottom), (!occ.left, n.left)],
                [(!occ.bottom, n.bottom), (!occ.right, n.right)],
                [(!occ.top, n.top), (!occ.right, n.right)],
                [(!occ.top, n.top), (!occ.left, n.left)],
            ],
            strength,
        )
    }

    pub(crate) fn back_corner_normals(
        n: VoxelFaceNormals,
        occ: VoxelOcclusion,
        strength: f32,
    ) -> [Vec3; 4] {
        Self::rounded_corner_normals(
            n.back,
            [
                [(!occ.top, n.top), (!occ.left, n.left)],
                [(!occ.top, n.top), (!occ.right, n.right)],
                [(!occ.bottom, n.bottom), (!occ.right, n.right)],
                [(!occ.bottom, n.bottom), (!occ.left, n.left)],
            ],
            strength,
        )
    }

    pub(crate) fn left_corner_normals(
        n: VoxelFaceNormals,
        occ: VoxelOcclusion,
        strength: f32,
    ) -> [Vec3; 4] {
        Self::rounded_corner_normals(
            n.left,
            [
                [(!occ.bottom, n.bottom), (!occ.back, n.back)],
                [(!occ.bottom, n.bottom), (!occ.front, n.front)],
                [(!occ.top, n.top), (!occ.front, n.front)],
                [(!occ.top, n.top), (!occ.back, n.back)],
            ],
            strength,
        )
    }

    pub(crate) fn right_corner_normals(
        n: VoxelFaceNormals,
        occ: VoxelOcclusion,
        strength: f32,
    ) -> [Vec3; 4] {
        Self::rounded_corner_normals(
            n.right,
            [
                [(!occ.bottom, n.bottom), (!occ.front, n.front)],
                [(!occ.bottom, n.bottom), (!occ.back, n.back)],
                [(!occ.top, n.top), (!occ.back, n.back)],
                [(!occ.top, n.top), (!occ.front, n.front)],
            ],
            strength,
        )
    }
}

#[inline]
fn safe_normalize(v: Vec3) -> Vec3 {
    let len_sq = v.length_squared();
    if len_sq <= 1e-8 {
        Vec3::Y
    } else {
        v / len_sq.sqrt()
    }
}
