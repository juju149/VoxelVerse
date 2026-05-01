use vv_registry::{BiomeId, BlockId as ContentBlockId};

use super::PlanetTerrain;

impl PlanetTerrain {
    pub fn get_height(&self, face: u8, u: u32, v: u32) -> u32 {
        self.column(face, u, v).height as u32
    }

    pub fn get_surface_block(&self, face: u8, u: u32, v: u32) -> ContentBlockId {
        let column = self.column(face, u, v);
        self.get_block(face, u, v, column.height as u32)
    }

    pub fn get_biome(&self, face: u8, u: u32, v: u32) -> BiomeId {
        let column = self.column(face, u, v);
        self.biomes[column.biome_index as usize].id
    }

    pub fn get_block(&self, face: u8, u: u32, v: u32, layer: u32) -> ContentBlockId {
        let column = self.column(face, u, v);
        let biome = &self.biomes[column.biome_index as usize];

        let depth_m =
            column.height.saturating_sub(layer as u16) as f32 * self.geometry.voxel_size_m;

        if let Some(ore) = self.ore_block(face, u, v, layer, depth_m, biome) {
            return ore;
        }

        let mut accumulated_depth = 0.0;

        for surface_layer in &biome.data.surface_layers {
            match surface_layer.depth_m {
                Some(layer_depth) => {
                    accumulated_depth += layer_depth.max(0.0);

                    if depth_m <= accumulated_depth {
                        return surface_layer.block;
                    }
                }
                None => return surface_layer.block,
            }
        }

        biome
            .data
            .surface_layers
            .last()
            .expect("terrain biome should have surface layers")
            .block
    }
}
