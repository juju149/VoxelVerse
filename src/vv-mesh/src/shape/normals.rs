use glam::Vec3;

use crate::MeshGen;

use super::{VoxelFaceNormals, VoxelOcclusion};

impl MeshGen {
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
