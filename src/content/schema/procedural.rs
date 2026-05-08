#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawNoiseKind {
    Perlin,
    Simplex,
    Ridged,
    Cellular,
    Constant,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawCurve {
    Linear,
    Smoothstep,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawDomainWarpDef {
    pub field: String,
    pub strength: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawNoiseRemapDef {
    pub in_min: f32,
    pub in_max: f32,
    pub out_min: f32,
    pub out_max: f32,
    pub curve: RawCurve,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawNoiseFieldDef {
    pub kind: RawNoiseKind,
    pub frequency: f32,
    pub amplitude: f32,
    pub octaves: u32,
    pub persistence: f32,
    pub lacunarity: f32,
    pub seed_salt: u32,
    #[serde(default)]
    pub domain_warp: Option<RawDomainWarpDef>,
    #[serde(default)]
    pub remap: Option<RawNoiseRemapDef>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawClimateAxisDef {
    #[serde(default)]
    pub latitude_bias: f32,
    #[serde(default)]
    pub fields: Vec<(String, f32)>,
    #[serde(default)]
    pub ocean_bias: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawClimateDef {
    pub temperature: RawClimateAxisDef,
    pub humidity: RawClimateAxisDef,
    pub continentality: RawClimateAxisDef,
    pub erosion: RawClimateAxisDef,
    pub weirdness: RawClimateAxisDef,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawPlanetProceduralDef {
    pub display_name: String,
    pub seed: u32,
    pub resolution: u32,
    pub surface_layer: u32,
    pub voxel_size_meters: f32,
    pub core_layers: u32,
    pub inner_radius_fraction: f32,
    pub sea_level_offset: i32,
    pub max_terrain_offset: i32,
    pub climate: String,
    pub biome_set: String,
    pub terrain_layers: String,
    #[serde(default)]
    pub caves: Vec<String>,
    #[serde(default)]
    pub ore_sets: Vec<String>,
    #[serde(default)]
    pub vegetation_sets: Vec<String>,
    #[serde(default)]
    pub structure_sets: Vec<String>,
    #[serde(default)]
    pub fauna_sets: Vec<String>,
    #[serde(default)]
    pub visual_detail_sets: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeSelectorDef {
    pub biome: String,
    pub temperature: (f32, f32),
    pub humidity: (f32, f32),
    pub roughness: (f32, f32),
    pub weight: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeSetDef {
    pub blend_radius: f32,
    pub selectors: Vec<RawBiomeSelectorDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeSurfaceDef {
    pub top: String,
    pub under: String,
    pub depth: (u32, u32),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeTerrainDef {
    pub base_height: f32,
    pub amplitude: f32,
    pub flatness: f32,
    pub hill_field: String,
    #[serde(default)]
    pub ridge_field: Option<String>,
    pub terrace_strength: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeColorTintDef {
    pub grass: [f32; 3],
    pub foliage: [f32; 3],
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeProceduralDef {
    pub display_name: String,
    pub surface: RawBiomeSurfaceDef,
    pub terrain: RawBiomeTerrainDef,
    pub color_tint: RawBiomeColorTintDef,
    #[serde(default)]
    pub vegetation_tags: Vec<String>,
    #[serde(default)]
    pub fauna_tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawTerrainLayerDef {
    pub name: String,
    pub block: String,
    #[serde(default)]
    pub depth: Option<(u32, u32)>,
    #[serde(default)]
    pub depth_from_center: Option<(u32, u32)>,
    #[serde(default)]
    pub biomes: Vec<String>,
    #[serde(default)]
    pub noise_variation: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawTerrainLayerSetDef {
    pub layers: Vec<RawTerrainLayerDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawOreDef {
    pub block: String,
    pub replace: Vec<String>,
    pub depth: (u32, u32),
    pub density: f32,
    pub vein_size: (u32, u32),
    pub field: String,
    #[serde(default)]
    pub biome_tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawCaveCarverDef {
    pub kind: String,
    pub field: String,
    pub threshold: f32,
    pub radius: (u32, u32),
    pub depth: (u32, u32),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawCaveDef {
    pub carvers: Vec<RawCaveCarverDef>,
    pub surface_break_chance: f32,
    #[serde(default)]
    pub fill_below_sea: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawFeaturePlacementDef {
    #[serde(default)]
    pub surface_blocks: Vec<String>,
    #[serde(default)]
    pub slope_max: f32,
    pub density: f32,
    pub field: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawVegetationStampDef {
    pub kind: String,
    pub trunk: String,
    pub leaves: String,
    pub height: (u32, u32),
    pub canopy_radius: (u32, u32),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawVegetationDef {
    pub placement: RawFeaturePlacementDef,
    pub stamp: RawVegetationStampDef,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawStructurePlacementDef {
    pub density: f32,
    pub min_spacing: u32,
    #[serde(default)]
    pub biomes: Vec<String>,
    pub slope_max: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawStructureDef {
    pub placement: RawStructurePlacementDef,
    pub footprint_radius: u32,
    pub priority: i32,
    pub stamp: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawFaunaSpawnDef {
    #[serde(default)]
    pub biome_tags: Vec<String>,
    pub density: f32,
    pub group_size: (u32, u32),
    pub light: (f32, f32),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawFaunaRuntimeDef {
    pub despawn_distance: u32,
    pub sim_distance: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawFaunaDef {
    pub entity: String,
    pub spawn: RawFaunaSpawnDef,
    pub runtime: RawFaunaRuntimeDef,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawVisualDetailItemDef {
    pub block: String,
    pub weight: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawVisualDetailDef {
    pub placement: RawFeaturePlacementDef,
    pub details: Vec<RawVisualDetailItemDef>,
}
