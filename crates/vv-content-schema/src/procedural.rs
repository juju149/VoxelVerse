use crate::ContentRef;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawNoiseKind {
    Perlin,
    Simplex,
    #[serde(rename = "opensimplex2s")]
    OpenSimplex2S,
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
    pub field: ContentRef,
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
    pub seed_salt: String,
    #[serde(default)]
    pub domain_warp: Option<RawDomainWarpDef>,
    #[serde(default)]
    pub remap: Option<RawNoiseRemapDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawClimateDef {
    pub display_name: String,
    pub fields: RawClimateFieldsDef,
    pub atmosphere: RawAtmosphereDef,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawClimateFieldsDef {
    pub temperature: ContentRef,
    pub humidity: ContentRef,
    pub continentality: ContentRef,
    pub erosion: ContentRef,
    pub weirdness: ContentRef,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawAtmosphereDef {
    pub fog_color: (f32, f32, f32),
    pub horizon_fog_density: f32,
    pub sky_scatter_strength: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawPlanetProceduralDef {
    pub display_name: String,
    pub seed: u32,
    pub shape: RawPlanetShapeDef,
    pub climate: ContentRef,
    pub biome_set: ContentRef,
    pub terrain_layers: ContentRef,
    #[serde(default)]
    pub caves: Vec<ContentRef>,
    #[serde(default)]
    pub ores: Vec<ContentRef>,
    #[serde(default)]
    pub vegetation: Vec<ContentRef>,
    #[serde(default)]
    pub structures: Vec<ContentRef>,
    #[serde(default)]
    pub spawns: Vec<ContentRef>,
    #[serde(default)]
    pub visual_details: Vec<ContentRef>,
    pub streaming: RawPlanetStreamingDef,
}

#[derive(Debug, Clone, Deserialize)]
pub enum RawPlanetShapeDef {
    SphericalVoxelPlanet(RawSphericalVoxelPlanetDef),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawSphericalVoxelPlanetDef {
    pub resolution: u32,
    pub surface_layer: u32,
    pub voxel_size_meters: f32,
    pub edge_rounding_radius_voxels: f32,
    pub core_layers: u32,
    pub sea_level_offset: i32,
    pub max_terrain_offset: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawPlanetStreamingDef {
    pub near_voxel_lod_radius: u32,
    pub far_surface_lod_radius: u32,
    pub upload_budget_chunks_per_frame: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeSelectorDef {
    pub biome: ContentRef,
    pub temperature: (f32, f32),
    pub humidity: (f32, f32),
    #[serde(default)]
    pub elevation: Option<(f32, f32)>,
    pub weight: f32,
    #[serde(default)]
    pub continentality: Option<(f32, f32)>,
    #[serde(default)]
    pub erosion: Option<(f32, f32)>,
    #[serde(default)]
    pub weirdness: Option<(f32, f32)>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum RawBiomeSelectionDef {
    ClimateMap(RawBiomeClimateMapDef),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeClimateMapDef {
    pub entries: Vec<RawBiomeSelectorDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeSetDef {
    pub display_name: String,
    pub selection: RawBiomeSelectionDef,
    #[serde(default = "default_blend_radius")]
    pub blend_radius: f32,
}

fn default_blend_radius() -> f32 {
    0.08
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeSurfaceDef {
    pub top: ContentRef,
    pub under: ContentRef,
    pub depth_voxels: (u32, u32),
    #[serde(default)]
    pub slope_override: Option<RawBiomeSlopeOverrideDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeSlopeOverrideDef {
    pub above_degrees: u32,
    pub top: ContentRef,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeTerrainDef {
    pub base_height: f32,
    pub amplitude: f32,
    pub flatness: f32,
    pub hill_field: ContentRef,
    #[serde(default)]
    pub ridge_field: Option<ContentRef>,
    pub terrace_strength: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomePaletteDef {
    pub grass: (f32, f32, f32),
    pub foliage: (f32, f32, f32),
    pub fog_bias: (f32, f32, f32),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomePlacementDef {
    #[serde(default)]
    pub vegetation_tags: Vec<ContentRef>,
    #[serde(default)]
    pub fauna_tags: Vec<ContentRef>,
    #[serde(default)]
    pub structure_tags: Vec<ContentRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeProceduralDef {
    pub display_name: String,
    pub surface: RawBiomeSurfaceDef,
    pub terrain: RawBiomeTerrainDef,
    pub palette: RawBiomePaletteDef,
    pub placement: RawBiomePlacementDef,
    #[serde(default)]
    pub tags: Vec<ContentRef>,
    #[serde(default)]
    pub edge_of: Option<ContentRef>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawTerrainLayerRange {
    Surface,
    Subsurface,
    Crust,
    DeepCrust,
    Core,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawTerrainLayerDef {
    pub range: RawTerrainLayerRange,
    pub block: ContentRef,
    #[serde(default)]
    pub thickness: Option<(u32, u32)>,
    #[serde(default)]
    pub noise_variation: Option<ContentRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawTerrainLayerSetDef {
    pub display_name: String,
    pub layers: Vec<RawTerrainLayerDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawOreDef {
    pub block: ContentRef,
    pub replace: Vec<ContentRef>,
    pub depth_voxels: (u32, u32),
    pub density: f32,
    pub vein_size: (u32, u32),
    pub field: ContentRef,
    #[serde(default)]
    pub biome_tags: Vec<ContentRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawCaveDef {
    pub display_name: String,
    pub fields: Vec<ContentRef>,
    pub carve: RawCaveCarveDef,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawCaveCarveDef {
    pub min_depth_voxels: u32,
    pub max_depth_voxels: u32,
    pub tunnel_radius: (f32, f32),
    pub chamber_radius: (f32, f32),
    pub air_block: ContentRef,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawFeaturePlacementDef {
    #[serde(default)]
    pub allowed_surface_tags: Vec<ContentRef>,
    #[serde(default)]
    pub allowed_surface_blocks: Vec<ContentRef>,
    #[serde(default)]
    pub biome_tags: Vec<ContentRef>,
    pub density: f32,
    pub slope_max_degrees: u32,
    pub scatter_field: ContentRef,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawTreeShapeKind {
    #[default]
    BroadLeaf,
    Conical,
    Tall,
    JungleCanopy,
    FlatTop,
    DenseDark,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawVegetationPlacementKind {
    ProceduralTree,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawVegetationStampDef {
    pub trunk: ContentRef,
    pub leaves: ContentRef,
    pub height: (u32, u32),
    pub canopy_radius: (u32, u32),
    #[serde(default)]
    pub shape_kind: RawTreeShapeKind,
    #[serde(default = "default_canopy_density")]
    pub canopy_density: f32,
    #[serde(default = "default_trunk_thickness")]
    pub trunk_thickness: (u32, u32),
    #[serde(default)]
    pub branch_count: (u32, u32),
    #[serde(default = "default_branch_length")]
    pub branch_length: (u32, u32),
    #[serde(default = "default_canopy_squash")]
    pub canopy_vertical_squash: f32,
    #[serde(default = "default_branch_slope")]
    pub branch_slope: (f32, f32),
    #[serde(default = "default_canopy_lobe_count")]
    pub canopy_lobe_count: (u32, u32),
    #[serde(default = "default_trunk_lean_max")]
    pub trunk_lean_max: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawVegetationDef {
    pub display_name: String,
    pub kind: RawVegetationPlacementKind,
    pub placement: RawFeaturePlacementDef,
    pub stamp: RawVegetationStampDef,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawStructureRarity {
    Common,
    Uncommon,
    Rare,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawStructureDef {
    pub display_name: String,
    pub structure: ContentRef,
    pub biome_tags: Vec<ContentRef>,
    pub spacing_voxels: (u32, u32),
    pub slope_max_degrees: u32,
    pub rarity: RawStructureRarity,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawSpawnLightDef {
    Daylight,
    Night,
    Any,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawFaunaDef {
    pub entity: ContentRef,
    pub biome_tags: Vec<ContentRef>,
    pub density: f32,
    pub group_size: (u32, u32),
    pub light: RawSpawnLightDef,
    pub surface_tags: Vec<ContentRef>,
}

/// Drop entry for a vox prop — item dropped when the prop is destroyed.
#[derive(Debug, Clone, Deserialize)]
pub struct RawVoxPropDropDef {
    pub item: ContentRef,
    #[serde(default = "default_drop_count")]
    pub count: (u32, u32),
    #[serde(default = "default_drop_chance")]
    pub chance: f32,
}

/// One model variant inside a vox prop scatter — a specific .vox file to place,
/// with its weight and optional drops.
#[derive(Debug, Clone, Deserialize)]
pub struct RawVoxPropVariantDef {
    /// Content ref to a specific .vox asset, e.g.
    /// `"core:voxel/vegetation/flowers/flower_blue_1"`.
    pub model: ContentRef,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub drops: Vec<RawVoxPropDropDef>,
    /// Vertical offset in voxels above the surface (0 = touching surface).
    #[serde(default)]
    pub y_offset: i32,
}

/// A scatter rule: describes which .vox props to place, where, and how dense.
/// Files live in `defs/worldgen/prop_scatters/*.prop_scatter.ron`.
#[derive(Debug, Clone, Deserialize)]
pub struct RawVoxPropScatterDef {
    pub display_name: String,
    pub placement: RawFeaturePlacementDef,
    pub variants: Vec<RawVoxPropVariantDef>,
}

fn default_drop_count() -> (u32, u32) {
    (1, 1)
}
fn default_drop_chance() -> f32 {
    1.0
}
fn default_weight() -> u32 {
    1
}

fn default_trunk_thickness() -> (u32, u32) {
    (1, 1)
}

fn default_branch_length() -> (u32, u32) {
    (1, 2)
}

fn default_canopy_squash() -> f32 {
    0.85
}

fn default_branch_slope() -> (f32, f32) {
    (0.25, 0.80)
}

fn default_canopy_lobe_count() -> (u32, u32) {
    (3, 6)
}

fn default_trunk_lean_max() -> f32 {
    0.12
}

fn default_canopy_density() -> f32 {
    1.0
}
