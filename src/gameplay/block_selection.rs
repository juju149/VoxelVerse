use crate::generation::CoordSystem;
use crate::math::Ray;
use crate::voxel::VoxelCoord;
use crate::world::PlanetData;

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
        planet: &PlanetData,
        mode: BlockSelectionMode,
    ) -> Option<(VoxelCoord, f32)> {
        let mut distance = 0.0;
        let mut last_empty = None;
        let min_radius = 0.5;
        let step = 0.25;

        while distance < reach {
            let point = ray.point_at(distance);
            if point.length() < min_radius {
                break;
            }

            if let Some(id) = CoordSystem::pos_to_id(point, planet.resolution) {
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
    use crate::math::Ray;
    use crate::world::PlanetData;
    use glam::Vec3;

    #[test]
    fn empty_ray_returns_no_hit() {
        let planet = PlanetData::new(16);
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
