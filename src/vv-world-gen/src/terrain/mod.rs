mod cache;
mod features;
mod flora;
mod generation;
mod metrics;
mod ore;
mod surface;
mod types;

pub use types::{PlanetTerrain, TerrainCacheStats, TerrainColumn};

use std::{
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc, RwLock},
};

use vv_config::WorldGenConfig;
use vv_planet::PlanetGeometry;
use vv_registry::{
    CompiledWorldSettings, PlanetTypeSource, WorldgenContentView, WorldgenSettingsSource,
};

use crate::{
    climate::TerrainBiome, error::TerrainGenerationError, noise::NoiseGenerator,
    planet::deterministic_planet_radius_m,
};

use self::{
    metrics::{max_feature_height_m, max_feature_radius_m},
    types::{TerrainFlora, TerrainNoiseConfig, TerrainOre},
};

impl PlanetTerrain {
    pub fn generate(
        cfg: &WorldGenConfig,
        content: &WorldgenContentView<'_>,
        settings: &CompiledWorldSettings,
    ) -> Result<Self, TerrainGenerationError> {
        let default_planet = content
            .default_planet_type()
            .ok_or(TerrainGenerationError::MissingDefaultPlanetType)?;

        let planet = content
            .planet_type(default_planet)
            .ok_or(TerrainGenerationError::MissingPlanetType(default_planet))?;

        let radius_m = deterministic_planet_radius_m(&planet.data, cfg.noise_seed, settings);
        let geometry = PlanetGeometry::new(radius_m, settings.voxel_size_m);

        Self::generate_for_geometry(geometry, cfg, content)
    }

    pub fn generate_for_geometry(
        geometry: PlanetGeometry,
        cfg: &WorldGenConfig,
        content: &WorldgenContentView<'_>,
    ) -> Result<Self, TerrainGenerationError> {
        let default_planet = content
            .default_planet_type()
            .ok_or(TerrainGenerationError::MissingDefaultPlanetType)?;

        let planet = content
            .planet_type(default_planet)
            .ok_or(TerrainGenerationError::MissingPlanetType(default_planet))?;

        let biome_views: Vec<_> = content.biomes().collect();

        if biome_views.is_empty() {
            return Err(TerrainGenerationError::NoBiomes);
        }

        if biome_views
            .iter()
            .any(|biome| biome.data.surface_layers.is_empty())
        {
            return Err(TerrainGenerationError::BiomeWithoutSurfaceLayer);
        }

        let biomes = biome_views
            .iter()
            .map(|biome| TerrainBiome {
                id: biome.id,
                data: biome.data.clone(),
            })
            .collect::<Vec<_>>();

        let flora = content
            .flora()
            .enumerate()
            .map(|(index, flora)| TerrainFlora {
                index: index as u32,
                data: flora.data.clone(),
            })
            .collect::<Vec<_>>();

        let ores = content
            .ores()
            .enumerate()
            .map(|(index, ore)| TerrainOre {
                index: index as u32,
                data: ore.data.clone(),
            })
            .collect::<Vec<_>>();

        let max_feature_height_m = max_feature_height_m(&flora);
        let max_feature_radius_m = max_feature_radius_m(&flora);

        Ok(Self {
            columns: Arc::new(RwLock::new(HashMap::new())),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            cache_compute_micros: Arc::new(AtomicU64::new(0)),

            biomes: Arc::new(biomes),
            flora: Arc::new(flora),
            ores: Arc::new(ores),

            planet: planet.data.clone(),
            climate_curves: *content.climate_curves(),
            generator: Arc::new(NoiseGenerator::new(cfg.noise_seed)),
            noise: TerrainNoiseConfig::from(cfg),

            geometry,
            world_seed: cfg.noise_seed,

            max_feature_height_m,
            max_feature_radius_m,
        })
    }

    pub fn world_seed(&self) -> u32 {
        self.world_seed
    }

    pub fn geometry(&self) -> PlanetGeometry {
        self.geometry
    }

    pub fn resolution(&self) -> u32 {
        self.geometry.resolution
    }
}

impl Clone for PlanetTerrain {
    fn clone(&self) -> Self {
        Self {
            columns: self.columns.clone(),
            cache_hits: self.cache_hits.clone(),
            cache_misses: self.cache_misses.clone(),
            cache_compute_micros: self.cache_compute_micros.clone(),

            biomes: self.biomes.clone(),
            flora: self.flora.clone(),
            ores: self.ores.clone(),

            planet: self.planet.clone(),
            climate_curves: self.climate_curves,
            generator: self.generator.clone(),
            noise: self.noise,

            geometry: self.geometry,
            world_seed: self.world_seed,

            max_feature_height_m: self.max_feature_height_m,
            max_feature_radius_m: self.max_feature_radius_m,
        }
    }
}
