use vv_math::CoordSystem;
use vv_math::Ray;
use vv_voxel::VoxelCoord;
use vv_world::VoxelRead;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockSelectionMode {
    HitSolid,
    Placement,
}

pub struct BlockSelection;

impl BlockSelection {
    pub fn trace(
        ray: Ray,
        reach: f32,
        planet: &impl VoxelRead,
        mode: BlockSelectionMode,
    ) -> Option<(VoxelCoord, f32)> {
        let mut distance = 0.0;
        let mut last_empty = None;
        let profile = planet.profile();
        let min_radius = profile.voxel_size_meters;
        let step = (profile.voxel_size_meters * 0.5).max(0.05);

        while distance < reach {
            let point = ray.point_at(distance);
            if point.length() < min_radius {
                break;
            }

            if let Some(id) = CoordSystem::pos_to_id(point, profile) {
                let exists = planet.exists(id);
                match mode {
                    BlockSelectionMode::HitSolid if exists => return Some((id, distance)),
                    BlockSelectionMode::HitSolid => {}
                    BlockSelectionMode::Placement if exists => {
                        return last_empty.map(|candidate| (candidate, distance));
                    }
                    BlockSelectionMode::Placement => last_empty = Some(id),
                }
            }

            distance += step;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockSelection, BlockSelectionMode};
    use glam::Vec3;
    use vv_math::Ray;
    use vv_voxel::{VoxelCoord, VoxelId};
    use vv_world::{PlanetProfile, VoxelRead};

    struct EmptyPlanet {
        profile: PlanetProfile,
    }

    impl EmptyPlanet {
        fn new() -> Self {
            Self {
                profile: PlanetProfile {
                    resolution: 32,
                    surface_layer: 16,
                    core_layers: 4,
                    voxel_size_meters: 1.0,
                    edge_rounding_radius_voxels: 0.16,
                    inner_radius: 4.0,
                    surface_radius: 16.0,
                    layer_height: 1.0,
                    max_terrain_offset: 6,
                    spawn_clearance_layers: 8.0,
                    seed: 1,
                },
            }
        }
    }

    impl VoxelRead for EmptyPlanet {
        fn resolution(&self) -> u32 {
            self.profile.resolution
        }

        fn profile(&self) -> PlanetProfile {
            self.profile
        }

        fn get_voxel(&self, _coord: VoxelCoord) -> VoxelId {
            VoxelId::AIR
        }

        fn exists(&self, _coord: VoxelCoord) -> bool {
            false
        }
    }

    #[test]
    fn empty_ray_returns_no_hit() {
        let planet = EmptyPlanet::new();
        let ray = Ray {
            origin: Vec3::new(0.0, 0.0, 10_000.0),
            direction: Vec3::Z,
        };
        assert_eq!(
            BlockSelection::trace(ray, 8.0, &planet, BlockSelectionMode::HitSolid),
            None
        );
    }
}
