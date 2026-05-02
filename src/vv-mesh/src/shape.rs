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

#[derive(Clone, Copy, Debug)]
pub(crate) struct VoxelFaceNormals {
    pub top_radial: Vec3,
    pub bottom_radial: Vec3,
    pub front: Vec3,
    pub back: Vec3,
    pub left: Vec3,
    pub right: Vec3,
}

impl VoxelFaceNormals {
    #[inline]
    pub(crate) fn as_array(self) -> [Vec3; 6] {
        [
            self.top_radial,
            self.bottom_radial,
            self.front,
            self.back,
            self.left,
            self.right,
        ]
    }
}

impl MeshGen {
    #[inline]
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

    #[inline]
    pub(crate) fn voxel_face_normals(c: VoxelCorners) -> VoxelFaceNormals {
        VoxelFaceNormals {
            top_radial: ((c.o_bl + c.o_br + c.o_tr + c.o_tl) * 0.25).normalize(),
            bottom_radial: ((c.i_tl + c.i_tr + c.i_br + c.i_bl) * 0.25).normalize(),
            front: Self::face_normal([c.i_bl, c.i_br, c.o_br, c.o_bl]),
            back: Self::face_normal([c.o_tl, c.o_tr, c.i_tr, c.i_tl]),
            left: Self::face_normal([c.i_tl, c.i_bl, c.o_bl, c.o_tl]),
            right: Self::face_normal([c.i_br, c.i_tr, c.o_tr, c.o_br]),
        }
    }

    #[inline]
    pub(crate) fn sculpted_face_positions(
        c: VoxelCorners,
        occ: VoxelOcclusion,
        bevel_width: f32,
    ) -> VoxelFacePositions {
        VoxelFacePositions {
            top: Self::inset_face(
                [c.o_bl, c.o_br, c.o_tr, c.o_tl],
                [!occ.front, !occ.right, !occ.back, !occ.left],
                bevel_width,
            ),
            bottom: Self::inset_face(
                [c.i_tl, c.i_tr, c.i_br, c.i_bl],
                [!occ.back, !occ.right, !occ.front, !occ.left],
                bevel_width,
            ),
            front: Self::inset_face(
                [c.i_bl, c.i_br, c.o_br, c.o_bl],
                [!occ.bottom, !occ.right, !occ.top, !occ.left],
                bevel_width,
            ),
            back: Self::inset_face(
                [c.o_tl, c.o_tr, c.i_tr, c.i_tl],
                [!occ.top, !occ.right, !occ.bottom, !occ.left],
                bevel_width,
            ),
            left: Self::inset_face(
                [c.i_tl, c.i_bl, c.o_bl, c.o_tl],
                [!occ.bottom, !occ.front, !occ.top, !occ.back],
                bevel_width,
            ),
            right: Self::inset_face(
                [c.i_br, c.i_tr, c.o_tr, c.o_br],
                [!occ.bottom, !occ.back, !occ.top, !occ.front],
                bevel_width,
            ),
        }
    }

    #[inline]
    pub(crate) fn top_corner_normals(
        n: VoxelFaceNormals,
        occ: VoxelOcclusion,
        strength: f32,
    ) -> [Vec3; 4] {
        Self::rounded_corner_normals(
            n.top_radial,
            [
                [(!occ.left, n.left), (!occ.front, n.front)],
                [(!occ.right, n.right), (!occ.front, n.front)],
                [(!occ.right, n.right), (!occ.back, n.back)],
                [(!occ.left, n.left), (!occ.back, n.back)],
            ],
            strength,
        )
    }

    #[inline]
    pub(crate) fn bottom_corner_normals(
        n: VoxelFaceNormals,
        occ: VoxelOcclusion,
        strength: f32,
    ) -> [Vec3; 4] {
        Self::rounded_corner_normals(
            n.bottom_radial,
            [
                [(!occ.left, n.left), (!occ.back, n.back)],
                [(!occ.right, n.right), (!occ.back, n.back)],
                [(!occ.right, n.right), (!occ.front, n.front)],
                [(!occ.left, n.left), (!occ.front, n.front)],
            ],
            strength,
        )
    }

    #[inline]
    pub(crate) fn front_corner_normals(
        n: VoxelFaceNormals,
        occ: VoxelOcclusion,
        strength: f32,
    ) -> [Vec3; 4] {
        Self::rounded_corner_normals(
            n.front,
            [
                [(!occ.bottom, n.bottom_radial), (!occ.left, n.left)],
                [(!occ.bottom, n.bottom_radial), (!occ.right, n.right)],
                [(!occ.top, n.top_radial), (!occ.right, n.right)],
                [(!occ.top, n.top_radial), (!occ.left, n.left)],
            ],
            strength,
        )
    }

    #[inline]
    pub(crate) fn back_corner_normals(
        n: VoxelFaceNormals,
        occ: VoxelOcclusion,
        strength: f32,
    ) -> [Vec3; 4] {
        Self::rounded_corner_normals(
            n.back,
            [
                [(!occ.top, n.top_radial), (!occ.left, n.left)],
                [(!occ.top, n.top_radial), (!occ.right, n.right)],
                [(!occ.bottom, n.bottom_radial), (!occ.right, n.right)],
                [(!occ.bottom, n.bottom_radial), (!occ.left, n.left)],
            ],
            strength,
        )
    }

    #[inline]
    pub(crate) fn left_corner_normals(
        n: VoxelFaceNormals,
        occ: VoxelOcclusion,
        strength: f32,
    ) -> [Vec3; 4] {
        Self::rounded_corner_normals(
            n.left,
            [
                [(!occ.bottom, n.bottom_radial), (!occ.back, n.back)],
                [(!occ.bottom, n.bottom_radial), (!occ.front, n.front)],
                [(!occ.top, n.top_radial), (!occ.front, n.front)],
                [(!occ.top, n.top_radial), (!occ.back, n.back)],
            ],
            strength,
        )
    }

    #[inline]
    pub(crate) fn right_corner_normals(
        n: VoxelFaceNormals,
        occ: VoxelOcclusion,
        strength: f32,
    ) -> [Vec3; 4] {
        Self::rounded_corner_normals(
            n.right,
            [
                [(!occ.bottom, n.bottom_radial), (!occ.front, n.front)],
                [(!occ.bottom, n.bottom_radial), (!occ.back, n.back)],
                [(!occ.top, n.top_radial), (!occ.back, n.back)],
                [(!occ.top, n.top_radial), (!occ.front, n.front)],
            ],
            strength,
        )
    }
}
