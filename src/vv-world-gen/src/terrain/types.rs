use std::{
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc, RwLock},
};

use vv_config::WorldGenConfig;
use vv_planet::PlanetGeometry;
use vv_registry::{CompiledClimateCurves, CompiledFlora, CompiledOre, CompiledPlanetType};

use crate::{climate::TerrainBiome, NoiseGenerator};

#[derive(Clone, Copy, Debug)]
pub struct TerrainColumn {
    pub height: u16,
    pub(crate) biome_index: u16,
}

#[derive(Clone, Debug)]
pub(crate) struct TerrainFlora {
    pub index: u32,
    pub data: CompiledFlora,
}

#[derive(Clone, Debug)]
pub(crate) struct TerrainOre {
    pub index: u32,
    pub data: CompiledOre,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TerrainNoiseConfig {
    pub octaves: u32,
    pub persistence: f32,
    pub lacunarity: f32,
}

impl From<&WorldGenConfig> for TerrainNoiseConfig {
    fn from(cfg: &WorldGenConfig) -> Self {
        Self {
            octaves: cfg.noise_octaves,
            persistence: cfg.noise_persistence,
            lacunarity: cfg.noise_lacunarity,
        }
    }
}

pub struct PlanetTerrain {
    pub(crate) columns: Arc<RwLock<HashMap<u64, TerrainColumn>>>,
    pub(crate) cache_hits: Arc<AtomicU64>,
    pub(crate) cache_misses: Arc<AtomicU64>,
    pub(crate) cache_compute_micros: Arc<AtomicU64>,

    pub(crate) biomes: Arc<Vec<TerrainBiome>>,
    pub(crate) flora: Arc<Vec<TerrainFlora>>,
    pub(crate) ores: Arc<Vec<TerrainOre>>,

    pub(crate) planet: CompiledPlanetType,
    pub(crate) climate_curves: CompiledClimateCurves,
    pub(crate) generator: Arc<NoiseGenerator>,
    pub(crate) noise: TerrainNoiseConfig,

    pub(crate) geometry: PlanetGeometry,
    pub(crate) world_seed: u32,

    pub(crate) max_feature_height_m: f32,
    pub(crate) max_feature_radius_m: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TerrainCacheStats {
    pub cached_columns: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub compute_micros: u64,
}
