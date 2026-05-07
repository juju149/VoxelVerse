use crate::generation::CoordSystem;
use crate::math::Ray;
use crate::voxel::VoxelCoord;
use crate::world::VoxelRead;

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
        let min_radius = 0.5;
        let step = 0.25;

        while distance < reach {
            let point = ray.point_at(distance);
            if point.length() < min_radius {
                break;
            }

            if let Some(id) = CoordSystem::pos_to_id(point, planet.resolution()) {
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
        use crate::content::compile::ContentCompiler;
        use crate::content::pack::PackLoader;
        use std::sync::Arc;
        let pack = PackLoader::load_from_dir(std::path::Path::new("packs/core"))
            .expect("packs/core must exist for tests");
        let registry = Arc::new(ContentCompiler::compile_blocks(pack.blocks).expect("blocks"));
        let biomes =
            Arc::new(ContentCompiler::compile_biomes(pack.biomes, &registry).expect("biomes"));
        let planet_def = ContentCompiler::compile_planets(pack.planets)
            .expect("planets")
            .into_iter()
            .next()
            .expect("planet")
            .with_resolution(16);
        let planet = PlanetData::new(planet_def, registry, biomes);
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
