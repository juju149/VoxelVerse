use crate::voxel::VoxelId;

/// A compiled biome definition — block IDs already resolved from the registry.
#[derive(Clone, Debug)]
pub struct CompiledBiome {
    pub id: u8,
    /// Namespaced key (e.g. `"core:plains"`).
    pub key: String,
    pub display_name: String,
    /// Block placed on the top surface layer.
    pub surface_block: VoxelId,
    /// Block placed on all layers below the surface.
    pub subsurface_block: VoxelId,
    /// Climate temperature center (0.0 = arctic, 1.0 = tropical).
    pub temperature_center: f32,
    /// Terrain roughness center (0.0 = flat, 1.0 = mountainous).
    pub roughness_center: f32,
    /// Scales the planet-wide terrain amplitude for this biome (0 = flat, 1 = full).
    pub terrain_amplitude: f32,
    /// Bias toward flat surface (0 = natural, 1 = fully flat).
    pub terrain_flatness: f32,
}

impl CompiledBiome {
    /// 2D climate coordinate used for nearest-biome Voronoi selection.
    #[inline]
    pub fn climate_point(&self) -> (f32, f32) {
        (self.temperature_center, self.roughness_center)
    }
}

/// Runtime registry of all compiled biomes.
pub struct BiomeRegistry {
    biomes: Vec<CompiledBiome>,
}

impl BiomeRegistry {
    pub(crate) fn new(biomes: Vec<CompiledBiome>) -> Self {
        assert!(!biomes.is_empty(), "BiomeRegistry must have at least one biome");
        Self { biomes }
    }

    /// Get a biome by its compact runtime id.
    /// Falls back to biome 0 if the id is out of range.
    pub fn biome(&self, id: u8) -> &CompiledBiome {
        self.biomes
            .get(id as usize)
            .unwrap_or(&self.biomes[0])
    }

    pub fn biomes(&self) -> &[CompiledBiome] {
        &self.biomes
    }

    pub fn biome_count(&self) -> usize {
        self.biomes.len()
    }
}
