use vv_registry::BlockId as ContentBlockId;

use crate::{climate::TerrainBiome, hash01, tags_match};

use super::{types::TerrainOre, PlanetTerrain};

impl PlanetTerrain {
    pub(super) fn ore_block(
        &self,
        face: u8,
        u: u32,
        v: u32,
        layer: u32,
        depth_m: f32,
        biome: &TerrainBiome,
    ) -> Option<ContentBlockId> {
        if depth_m <= 0.0 {
            return None;
        }

        for ore in self.ores.iter() {
            if !self.ore_matches_biome(ore, biome) {
                continue;
            }

            let vein = ore.data.vein;

            if depth_m < vein.depth_min_m || depth_m > vein.depth_max_m {
                continue;
            }

            let voxel_volume_m3 = self.geometry.voxel_size_m.powi(3);
            let chance = (vein.frequency * 0.035 * voxel_volume_m3).clamp(0.0, 0.35);

            if hash01(face, u, v, layer, ore.index) < chance {
                return Some(ore.data.block);
            }
        }

        None
    }

    fn ore_matches_biome(&self, ore: &TerrainOre, biome: &TerrainBiome) -> bool {
        tags_match(
            &ore.data.required_tags,
            &ore.data.forbidden_tags,
            &biome.data.provided_tags,
        )
    }
}
