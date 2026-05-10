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
    use crate::app::content_bootstrap::asset_pack_root;
    use crate::math::Ray;
    use crate::world::PlanetData;
    use glam::Vec3;

    #[test]
    fn empty_ray_returns_no_hit() {
        use crate::content::compile::ContentCompiler;
        use crate::content::pack::PackLoader;
        use std::sync::Arc;
        let core_pack_dir = asset_pack_root().join("core");
        let pack = PackLoader::load_from_dir(&core_pack_dir)
            .expect("assets/packs/core must exist for tests");
        let index = vv_pack_compiler::ContentIndex::build(&pack);
        let models =
            ContentCompiler::compile_block_models(pack.block_models).expect("block_models");
        let registry = Arc::new(
            ContentCompiler::compile_blocks(pack.blocks, pack.materials, models, &index)
                .expect("blocks"),
        );
        let procedural_pack =
            PackLoader::load_procedural_from_dir(&core_pack_dir).expect("procedural pack");
        let procedural = Arc::new(
            ContentCompiler::compile_procedural(procedural_pack, &registry).expect("procedural"),
        );
        let planet_def = procedural
            .first_planet()
            .expect("procedural planet")
            .base
            .with_resolution(16);
        let planet = PlanetData::new(planet_def, registry, procedural, 0);
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
