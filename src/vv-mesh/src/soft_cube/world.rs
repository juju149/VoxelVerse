use glam::Vec3;

use crate::shape::VoxelCorners;

#[derive(Debug, Clone, Copy)]
pub(crate) struct SoftCubeWorldFrame {
    pub axis_x: Vec3,
    pub axis_y: Vec3,
    pub axis_z: Vec3,
}

impl SoftCubeWorldFrame {
    pub(crate) fn from_corners(c: VoxelCorners) -> Self {
        let axis_x =
            ((c.i_br - c.i_bl) + (c.i_tr - c.i_tl) + (c.o_br - c.o_bl) + (c.o_tr - c.o_tl)) * 0.25;

        let axis_y =
            ((c.o_bl - c.i_bl) + (c.o_br - c.i_br) + (c.o_tl - c.i_tl) + (c.o_tr - c.i_tr)) * 0.25;

        let axis_z =
            ((c.i_tl - c.i_bl) + (c.i_tr - c.i_br) + (c.o_tl - c.o_bl) + (c.o_tr - c.o_br)) * 0.25;

        Self {
            axis_x: axis_x.normalize_or_zero(),
            axis_y: axis_y.normalize_or_zero(),
            axis_z: axis_z.normalize_or_zero(),
        }
    }

    pub(crate) fn normal_to_world(self, normal: Vec3) -> Vec3 {
        (self.axis_x * normal.x + self.axis_y * normal.y + self.axis_z * normal.z)
            .normalize_or_zero()
    }
}

pub(crate) fn local_to_world(c: VoxelCorners, local: Vec3) -> Vec3 {
    let u = (local.x + 0.5).clamp(0.0, 1.0);
    let l = (local.y + 0.5).clamp(0.0, 1.0);
    let v = (local.z + 0.5).clamp(0.0, 1.0);

    let i_v0 = c.i_bl.lerp(c.i_br, u);
    let i_v1 = c.i_tl.lerp(c.i_tr, u);
    let inner = i_v0.lerp(i_v1, v);

    let o_v0 = c.o_bl.lerp(c.o_br, u);
    let o_v1 = c.o_tl.lerp(c.o_tr, u);
    let outer = o_v0.lerp(o_v1, v);

    inner.lerp(outer, l)
}
