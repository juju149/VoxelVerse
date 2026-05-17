use glam::Vec3;
use vv_math::{CoordSystem, GridCoord, SphericalGrid};
use vv_voxel::{PlanetProfile, VoxelCoord};

pub struct PlanetGeometry;

impl PlanetGeometry {
    pub fn grid(profile: PlanetProfile) -> SphericalGrid {
        SphericalGrid::new(
            profile.resolution,
            profile.inner_radius,
            profile.layer_height,
        )
    }

    pub fn get_local_coords(pos: Vec3, profile: PlanetProfile) -> Option<(VoxelCoord, Vec3)> {
        CoordSystem::get_local_coords(pos, Self::grid(profile))
            .map(|(coord, local)| (voxel_coord(coord), local))
    }

    pub fn get_direction(face: u8, u: u32, v: u32, res: u32) -> Vec3 {
        CoordSystem::get_direction(face, u, v, res)
    }

    pub fn get_vertex_pos(face: u8, u: u32, v: u32, layer: u32, profile: PlanetProfile) -> Vec3 {
        CoordSystem::get_vertex_pos(face, u, v, layer, Self::grid(profile))
    }

    pub fn get_vertex_pos_f32(
        face: u8,
        u: f32,
        v: f32,
        layer: f32,
        profile: PlanetProfile,
    ) -> Vec3 {
        CoordSystem::get_vertex_pos_f32(face, u, v, layer, Self::grid(profile))
    }

    pub fn get_block_center(face: u8, u: u32, v: u32, layer: u32, profile: PlanetProfile) -> Vec3 {
        CoordSystem::get_block_center(face, u, v, layer, Self::grid(profile))
    }

    pub fn pos_to_id(pos: Vec3, profile: PlanetProfile) -> Option<VoxelCoord> {
        CoordSystem::pos_to_id(pos, Self::grid(profile)).map(voxel_coord)
    }
}

fn voxel_coord(coord: GridCoord) -> VoxelCoord {
    VoxelCoord {
        face: coord.face,
        layer: coord.layer,
        u: coord.u,
        v: coord.v,
    }
}
