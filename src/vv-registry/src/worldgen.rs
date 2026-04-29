use crate::{
    BiomeId, BlockId, EntityId, FaunaId, FloraId, LootTableId, OreId, PlanetTypeId, RegistryTable,
    StructureId, TagId, WeatherId,
};

#[derive(Debug, Clone, Copy)]
pub struct CompiledIdealRange {
    pub min: f32,
    pub ideal_min: f32,
    pub ideal_max: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledFloatRange {
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledClimateSampleRanges {
    pub temperature: CompiledIdealRange,
    pub humidity: CompiledIdealRange,
    pub altitude: CompiledIdealRange,
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledBiomeRelief {
    pub base_height_m: f32,
    pub height_variance_m: f32,
    pub roughness: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledSurfaceLayer {
    pub block: BlockId,
    pub depth_m: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledClimateRange {
    pub tag: TagId,
    pub range: CompiledFloatRange,
}

#[derive(Debug, Clone)]
pub struct CompiledDerivedTagRule {
    pub requires: Vec<TagId>,
    pub produces: Vec<TagId>,
}

#[derive(Debug, Clone, Default)]
pub struct CompiledClimateTags {
    pub temperature: Vec<CompiledClimateRange>,
    pub humidity: Vec<CompiledClimateRange>,
    pub altitude: Vec<CompiledClimateRange>,
    pub slope: Vec<CompiledClimateRange>,
    pub latitude: Vec<CompiledClimateRange>,
    pub depth: Vec<CompiledClimateRange>,
    pub derived: Vec<CompiledDerivedTagRule>,
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledClimateCurves {
    pub temperature_noise_scale: f32,
    pub humidity_noise_scale: f32,
    pub minimum_biome_transition_m: f32,
}

impl Default for CompiledClimateCurves {
    fn default() -> Self {
        Self {
            temperature_noise_scale: 7.7,
            humidity_noise_scale: 3.1,
            minimum_biome_transition_m: 20.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompiledPlanetType {
    pub display_key: Option<String>,
    pub tags: Vec<TagId>,
    pub forbidden_tags: Vec<TagId>,
    pub temperature_bias: f32,
    pub humidity_bias: f32,
    pub altitude_variance_multiplier: f32,
    pub climate_transition_speed: f32,
    pub min_radius_km: f32,
    pub max_radius_km: f32,
    pub ocean_coverage: f32,
}

pub type PlanetTypeRegistry = RegistryTable<PlanetTypeId, CompiledPlanetType>;

#[derive(Debug, Clone)]
pub struct CompiledBiome {
    pub display_key: Option<String>,
    pub weight: f32,
    pub required_tags: Vec<TagId>,
    pub forbidden_tags: Vec<TagId>,
    pub preferred_tags: Vec<TagId>,
    pub provided_tags: Vec<TagId>,
    pub climate: CompiledClimateSampleRanges,
    pub relief: CompiledBiomeRelief,
    pub surface_layers: Vec<CompiledSurfaceLayer>,
}

pub type BiomeRegistry = RegistryTable<BiomeId, CompiledBiome>;

#[derive(Debug, Clone, Copy)]
pub struct CompiledFloraPlacement {
    pub density_base: f32,
    pub altitude_max_m: Option<f32>,
    pub slope_max: f32,
    pub cluster_radius_m: f32,
    pub cluster_min: u32,
    pub cluster_max: u32,
}

#[derive(Debug, Clone)]
pub enum CompiledFloraFeature {
    Plant {
        block: BlockId,
        height_min_m: f32,
        height_max_m: f32,
    },
    Tree {
        log_block: BlockId,
        leaf_block: BlockId,
        trunk_height_min_m: f32,
        trunk_height_max_m: f32,
        canopy_radius_m: f32,
        canopy_height_m: f32,
    },
    Cluster {
        block: BlockId,
        radius_min_m: f32,
        radius_max_m: f32,
    },
}

#[derive(Debug, Clone)]
pub struct CompiledFlora {
    pub display_key: Option<String>,
    pub weight: f32,
    pub required_tags: Vec<TagId>,
    pub forbidden_tags: Vec<TagId>,
    pub provided_tags: Vec<TagId>,
    pub placement: CompiledFloraPlacement,
    pub feature: CompiledFloraFeature,
}

pub type FloraRegistry = RegistryTable<FloraId, CompiledFlora>;

#[derive(Debug, Clone)]
pub struct CompiledFauna {
    pub display_key: Option<String>,
    pub entity: EntityId,
    pub required_tags: Vec<TagId>,
    pub provided_tags: Vec<TagId>,
}

pub type FaunaRegistry = RegistryTable<FaunaId, CompiledFauna>;

#[derive(Debug, Clone, Copy)]
pub struct CompiledOreVein {
    pub size_min: u32,
    pub size_max: u32,
    pub depth_min_m: f32,
    pub depth_max_m: f32,
    pub frequency: f32,
}

#[derive(Debug, Clone)]
pub struct CompiledOre {
    pub display_key: Option<String>,
    pub weight: f32,
    pub block: BlockId,
    pub required_tags: Vec<TagId>,
    pub forbidden_tags: Vec<TagId>,
    pub vein: CompiledOreVein,
}

pub type OreRegistry = RegistryTable<OreId, CompiledOre>;

#[derive(Debug, Clone)]
pub struct CompiledStructure {
    pub display_key: Option<String>,
    pub required_tags: Vec<TagId>,
    pub provided_tags: Vec<TagId>,
    pub loot_table: Option<LootTableId>,
}

pub type StructureRegistry = RegistryTable<StructureId, CompiledStructure>;

#[derive(Debug, Clone)]
pub struct CompiledWeather {
    pub display_key: Option<String>,
    pub required_tags: Vec<TagId>,
    pub provided_tags: Vec<TagId>,
}

pub type WeatherRegistry = RegistryTable<WeatherId, CompiledWeather>;
