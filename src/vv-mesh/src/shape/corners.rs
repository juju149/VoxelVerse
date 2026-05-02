use glam::Vec3;
use vv_core::BlockId;
use vv_planet::CoordSystem;
use vv_world_runtime::PlanetData;

use crate::MeshGen;

use super::math::safe_normalize;

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
}
