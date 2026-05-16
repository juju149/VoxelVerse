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
    #[serde(alias = "name")]
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
    /// Side length (in voxels) of one cell in the lazy surface cache.  The
    /// height/biome arrays are populated on demand cell-by-cell instead of
    /// pre-baking the whole planet.  64 ≈ 4 KiB per cell — fits in L1.
    #[serde(default = "default_region_cell_voxels")]
    pub region_cell_voxels: u32,
    /// Soft cap on placed features per chunk (vegetation + props combined).
    /// 0 disables the cap.  Default 384 ≈ Minecraft-style forest density.
    #[serde(default = "default_feature_budget_per_chunk")]
    pub feature_budget_per_chunk: u32,
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
    #[serde(alias = "name")]
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
    #[serde(alias = "depth")]
    pub depth_voxels: (u32, u32),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeSlopeOverrideDef {
    #[serde(alias = "above")]
    pub above_degrees: u32,
    #[serde(alias = "use")]
    pub top: ContentRef,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeTerrainDef {
    #[serde(alias = "height")]
    pub base_height: f32,
    pub amplitude: f32,
    pub flatness: f32,
    #[serde(alias = "noise")]
    pub hill_field: ContentRef,
    #[serde(default, alias = "ridges")]
    pub ridge_field: Option<ContentRef>,
    #[serde(default, alias = "terraces")]
    pub terrace_strength: f32,
    #[serde(default, alias = "curve")]
    pub height_curve: Option<RawHeightCurveDef>,
    #[serde(default = "default_mountain_intensity", alias = "mountain")]
    pub mountain_intensity: f32,
    #[serde(default, alias = "slope_smooth")]
    pub slope_smoothing: f32,
}

/// Per-biome height transfer curve applied to the unit-range height
/// contribution before scaling.  Each variant maps `t ∈ [-1, 1]` to a
/// reshaped value of similar range, controlling the silhouette of the
/// biome — plateaus, mountain spikes, soft hills, etc.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawHeightCurveDef {
    Linear,
    Smoothstep,
    /// `t.signum() * t.abs().powf(exponent)` — exponent < 1 widens valleys,
    /// > 1 sharpens peaks.
    Power {
        exponent: f32,
    },
    /// Pushes mid values toward 0 (flat plateau between -threshold and
    /// +threshold) and clamps slopes outside.  Good for badlands / mesas.
    Plateau {
        flatness: f32,
        threshold: f32,
    },
    /// Sharpens the upper end aggressively (mountain spikes) while leaving
    /// the lower end alone (plains stay flat).
    MountainSpike {
        sharpness: f32,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomePaletteDef {
    pub grass: (f32, f32, f32),
    pub foliage: (f32, f32, f32),
    #[serde(default)]
    pub fog_bias: (f32, f32, f32),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeProceduralDef {
    #[serde(alias = "name")]
    pub display_name: String,
    pub surface: RawBiomeSurfaceDef,
    /// Slope override block — written at biome top-level as
    /// `slope: (above: 30, use: stone)` in compact files.
    #[serde(default)]
    pub slope_override: Option<RawBiomeSlopeOverrideDef>,
    pub terrain: RawBiomeTerrainDef,
    pub palette: RawBiomePaletteDef,
    #[serde(default, alias = "vegetation")]
    pub vegetation_tags: Vec<ContentRef>,
    #[serde(default, alias = "fauna")]
    pub fauna_tags: Vec<ContentRef>,
    #[serde(default, alias = "structures")]
    pub structure_tags: Vec<ContentRef>,
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
    #[serde(alias = "name")]
    pub display_name: String,
    pub layers: Vec<RawTerrainLayerDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawOreDef {
    pub block: ContentRef,
    #[serde(alias = "replaces")]
    pub replace: Vec<ContentRef>,
    #[serde(alias = "depth")]
    pub depth_voxels: (u32, u32),
    pub density: f32,
    #[serde(alias = "vein")]
    pub vein_size: (u32, u32),
    pub field: ContentRef,
    #[serde(default)]
    pub biome_tags: Vec<ContentRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawCaveDef {
    #[serde(alias = "name")]
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

/// Where in cave geometry a prop scatter should be placed.
/// `top_surface` (default) keeps all existing above-ground placements unchanged.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawCaveSurface {
    /// Normal above-ground top-surface placement (default).
    #[default]
    TopSurface,
    /// Solid block directly below cave air — floors and ledges.
    CaveFloor,
    /// Solid block directly above cave air — ceilings and overhangs.
    CaveCeiling,
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
    /// Minimum spacing in voxels between two instances of this feature.
    /// 0 disables the spacing check.  Drives the placement grid cell size
    /// so authors can mix dense flower scatter (1-2 voxels) with sparse
    /// tree placement (4-8 voxels) in the same biome.
    #[serde(default)]
    pub min_spacing_voxels: f32,
    /// 0..1 — strength of the deterministic sub-cell jitter applied to
    /// every candidate position.  0 = perfectly aligned to placement
    /// cell centres (visible grid); 1 = candidate can land anywhere
    /// inside its cell.  Default 0.75 hides the grid without overlapping
    /// neighbours.
    #[serde(default = "default_jitter_strength")]
    pub jitter_strength: f32,
    /// Optional low-frequency noise field used as a clump/clearing modulator
    /// on top of the scatter field.  Multiplies the effective density,
    /// driving forest masses + clearings without authoring a custom
    /// scatter noise per biome.
    #[serde(default)]
    pub clump_field: Option<ContentRef>,
    /// 0..1 — strength of the clump modulator.  0 leaves density flat;
    /// 1 collapses density entirely outside clump centres.
    #[serde(default)]
    pub clump_strength: f32,
    /// Optional altitude window in voxels relative to sea level.  When
    /// `Some((min, max))`, candidates whose surface height falls outside
    /// the window are rejected.  None = no altitude gate.
    #[serde(default)]
    pub altitude_range: Option<(f32, f32)>,
    /// Optional humidity window in normalized climate space (0..1).
    #[serde(default)]
    pub humidity_range: Option<(f32, f32)>,
    /// Optional temperature window in normalized climate space (0..1).
    #[serde(default)]
    pub temperature_range: Option<(f32, f32)>,
    /// Optional minimum slope (in degrees).  Pairs with `slope_max_degrees`
    /// to author cliff-only or scree-only placements.
    #[serde(default)]
    pub slope_min_degrees: Option<u32>,
    /// Optional per-instance scale variance `(min, max)`.  Both bounds
    /// must be >0; defaults to `(1.0, 1.0)` (no variance) when absent.
    #[serde(default)]
    pub scale_variance: Option<(f32, f32)>,
    /// 0..1 — fraction of the full 2π rotation that is randomised per
    /// instance.  0 = no rotation (always aligned), 1 = full random.
    /// Default 1.0 for organic placement.
    #[serde(default = "default_rotation_variance")]
    pub rotation_variance: f32,
    /// Which cave surface this scatter targets.  Defaults to `top_surface`
    /// so all existing placements are unaffected.
    #[serde(default)]
    pub cave_surface: RawCaveSurface,
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
    #[serde(alias = "tree")]
    ProceduralTree,
}

/// A vegetation entry — describes a procedurally-generated tree or plant.
/// Fields from the old `stamp` and `placement` sub-structs are now flat for
/// compact authoring.  Nested forms (`stamp: (...)`, `placement: (...)`) are
/// still accepted for backwards compatibility via an ignored outer wrapper.
#[derive(Debug, Clone, Deserialize)]
pub struct RawVegetationDef {
    #[serde(alias = "name")]
    pub display_name: String,
    #[serde(alias = "type")]
    pub kind: RawVegetationPlacementKind,
    // --- stamp fields ---
    pub trunk: ContentRef,
    pub leaves: ContentRef,
    pub height: (u32, u32),
    #[serde(default = "default_canopy_radius")]
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
    // --- placement fields ---
    pub density: f32,
    #[serde(default = "default_veg_slope_max")]
    pub slope_max_degrees: u32,
    #[serde(alias = "noise")]
    pub scatter_field: ContentRef,
    #[serde(default, alias = "biomes")]
    pub biome_tags: Vec<ContentRef>,
    #[serde(default)]
    pub allowed_surface_tags: Vec<ContentRef>,
    #[serde(default)]
    pub allowed_surface_blocks: Vec<ContentRef>,
    #[serde(default)]
    pub min_spacing_voxels: f32,
    #[serde(default = "default_jitter_strength")]
    pub jitter_strength: f32,
    #[serde(default)]
    pub clump_field: Option<ContentRef>,
    #[serde(default)]
    pub clump_strength: f32,
    #[serde(default)]
    pub altitude_range: Option<(f32, f32)>,
    #[serde(default)]
    pub humidity_range: Option<(f32, f32)>,
    #[serde(default)]
    pub temperature_range: Option<(f32, f32)>,
    #[serde(default)]
    pub slope_min_degrees: Option<u32>,
    #[serde(default)]
    pub scale_variance: Option<(f32, f32)>,
    #[serde(default = "default_rotation_variance")]
    pub rotation_variance: f32,
    /// Surface root flare radius in voxels.  0 = no roots (default).
    #[serde(default)]
    pub root_radius: f32,
    /// 0..1 probability that a placement becomes a fallen trunk.  0 = never (default).
    #[serde(default)]
    pub fallen_chance: f32,
    /// 0..1 mid-trunk S-curve strength on top of lean.  0 = straight (default).
    #[serde(default)]
    pub trunk_curve_strength: f32,
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
    #[serde(alias = "name")]
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

fn default_canopy_radius() -> (u32, u32) {
    (3, 5)
}

fn default_veg_slope_max() -> u32 {
    35
}

fn default_jitter_strength() -> f32 {
    0.75
}

fn default_rotation_variance() -> f32 {
    1.0
}

fn default_mountain_intensity() -> f32 {
    1.0
}

fn default_region_cell_voxels() -> u32 {
    64
}

fn default_feature_budget_per_chunk() -> u32 {
    384
}
