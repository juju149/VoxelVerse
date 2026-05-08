#![allow(dead_code)]

use crate::content::schema::{RawCurve, RawNoiseKind, RawTreeShapeKind};
use crate::content::CompiledPlanet;
use crate::voxel::VoxelId;

#[derive(Clone, Copy, Debug)]
pub enum CompiledNoiseKind {
    Perlin,
    Simplex,
    Ridged,
    Cellular,
    Constant,
}

impl From<&RawNoiseKind> for CompiledNoiseKind {
    fn from(value: &RawNoiseKind) -> Self {
        match value {
            RawNoiseKind::Perlin => Self::Perlin,
            RawNoiseKind::Simplex => Self::Simplex,
            RawNoiseKind::Ridged => Self::Ridged,
            RawNoiseKind::Cellular => Self::Cellular,
            RawNoiseKind::Constant => Self::Constant,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CompiledCurve {
    Linear,
    Smoothstep,
}

impl From<&RawCurve> for CompiledCurve {
    fn from(value: &RawCurve) -> Self {
        match value {
            RawCurve::Linear => Self::Linear,
            RawCurve::Smoothstep => Self::Smoothstep,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CompiledNoiseRemap {
    pub in_min: f32,
    pub in_max: f32,
    pub out_min: f32,
    pub out_max: f32,
    pub curve: CompiledCurve,
}

#[derive(Clone, Debug)]
pub struct CompiledNoiseField {
    pub key: String,
    pub kind: CompiledNoiseKind,
    pub frequency: f32,
    pub amplitude: f32,
    pub octaves: u32,
    pub persistence: f32,
    pub lacunarity: f32,
    pub seed_salt: u32,
    pub domain_warp: Option<(usize, f32)>,
    pub remap: Option<CompiledNoiseRemap>,
}

#[derive(Clone, Debug, Default)]
pub struct CompiledClimateAxis {
    pub latitude_bias: f32,
    pub fields: Vec<(usize, f32)>,
    pub ocean_bias: f32,
}

#[derive(Clone, Debug)]
pub struct CompiledClimate {
    pub key: String,
    pub temperature: CompiledClimateAxis,
    pub humidity: CompiledClimateAxis,
    pub continentality: CompiledClimateAxis,
    pub erosion: CompiledClimateAxis,
    pub weirdness: CompiledClimateAxis,
}

#[derive(Clone, Debug)]
pub struct CompiledBiomeSelector {
    pub biome: usize,
    pub temperature: (f32, f32),
    pub humidity: (f32, f32),
    pub roughness: (f32, f32),
    pub weight: f32,
    /// Optional climate windows. `None` means "no constraint on this axis".
    /// When present, behaves like `temperature`/`humidity`/`roughness`.
    pub continentality: Option<(f32, f32)>,
    pub erosion: Option<(f32, f32)>,
    pub weirdness: Option<(f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct CompiledBiomeSet {
    pub key: String,
    pub blend_radius: f32,
    pub selectors: Vec<CompiledBiomeSelector>,
}

#[derive(Clone, Debug)]
pub struct CompiledBiomeSurface {
    pub top: VoxelId,
    pub under: VoxelId,
    pub depth: (u32, u32),
}

#[derive(Clone, Debug)]
pub struct CompiledBiomeTerrain {
    pub base_height: f32,
    pub amplitude: f32,
    pub flatness: f32,
    pub hill_field: usize,
    pub ridge_field: Option<usize>,
    pub terrace_strength: f32,
}

#[derive(Clone, Debug)]
pub struct CompiledBiomeColorTint {
    pub grass: [f32; 3],
    pub foliage: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct CompiledProceduralBiome {
    pub id: u8,
    pub key: String,
    pub display_name: String,
    pub surface: CompiledBiomeSurface,
    pub terrain: CompiledBiomeTerrain,
    pub color_tint: CompiledBiomeColorTint,
    pub vegetation_tags: Vec<String>,
    pub fauna_tags: Vec<String>,
    /// If set, this is a sub-biome appearing only at the border of the
    /// referenced biome (resolved index).  Used for beaches, stony shores…
    pub edge_of: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct CompiledTerrainLayer {
    pub name: String,
    pub block: VoxelId,
    pub depth: Option<(u32, u32)>,
    pub depth_from_center: Option<(u32, u32)>,
    pub all_biomes: bool,
    pub biomes: Vec<usize>,
    pub noise_variation: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct CompiledTerrainLayerSet {
    pub key: String,
    pub layers: Vec<CompiledTerrainLayer>,
}

#[derive(Clone, Debug)]
pub struct CompiledOre {
    pub key: String,
    pub block: VoxelId,
    pub replace: Vec<VoxelId>,
    pub depth: (u32, u32),
    pub density: f32,
    pub vein_size: (u32, u32),
    pub field: usize,
    pub biome_tags: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct CompiledCaveCarver {
    pub kind: String,
    pub field: usize,
    pub threshold: f32,
    pub radius: (u32, u32),
    pub depth: (u32, u32),
}

#[derive(Clone, Debug)]
pub struct CompiledCave {
    pub key: String,
    pub carvers: Vec<CompiledCaveCarver>,
    pub surface_break_chance: f32,
    pub fill_below_sea: Option<VoxelId>,
}

#[derive(Clone, Debug)]
pub struct CompiledFeaturePlacement {
    pub surface_blocks: Vec<VoxelId>,
    pub slope_max: f32,
    pub density: f32,
    pub field: usize,
    /// Biome tag whitelist.  Empty = match everything; otherwise the placement
    /// only fires in biomes whose `vegetation_tags` (or `fauna_tags` for
    /// fauna) intersect this set.  `*` is treated as "any".
    pub biome_tags: Vec<String>,
}

impl CompiledFeaturePlacement {
    /// True iff this placement is allowed in `biome` based on its `biome_tags`
    /// whitelist.  Empty whitelist (or `["*"]`) means "any biome".
    /// The tags are matched against the biome's `vegetation_tags` plus its
    /// `fauna_tags` so a single declarative list works for both.
    pub fn allowed_in_biome(&self, biome: &CompiledProceduralBiome) -> bool {
        if self.biome_tags.is_empty() {
            return true;
        }
        if self.biome_tags.iter().any(|t| t == "*") {
            return true;
        }
        self.biome_tags.iter().any(|tag| {
            biome.vegetation_tags.iter().any(|vt| vt == tag)
                || biome.fauna_tags.iter().any(|ft| ft == tag)
        })
    }
}

/// Mirrors [`RawTreeShapeKind`].  Plumbed into the bakery so each tree picks
/// the right silhouette family at stamp time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompiledTreeShapeKind {
    #[default]
    BroadLeaf,
    Conical,
    Tall,
    JungleCanopy,
    FlatTop,
    DenseDark,
}

impl From<&RawTreeShapeKind> for CompiledTreeShapeKind {
    fn from(value: &RawTreeShapeKind) -> Self {
        match value {
            RawTreeShapeKind::BroadLeaf => Self::BroadLeaf,
            RawTreeShapeKind::Conical => Self::Conical,
            RawTreeShapeKind::Tall => Self::Tall,
            RawTreeShapeKind::JungleCanopy => Self::JungleCanopy,
            RawTreeShapeKind::FlatTop => Self::FlatTop,
            RawTreeShapeKind::DenseDark => Self::DenseDark,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CompiledVegetation {
    pub key: String,
    pub placement: CompiledFeaturePlacement,
    pub trunk: VoxelId,
    pub leaves: VoxelId,
    pub height: (u32, u32),
    pub canopy_radius: (u32, u32),
    pub trunk_thickness: (u32, u32),
    pub branch_count: (u32, u32),
    pub branch_length: (u32, u32),
    /// Vertical squash factor for the canopy ellipsoid.
    pub canopy_vertical_squash: f32,
    /// Branch slope range (rise per horizontal voxel).
    pub branch_slope: (f32, f32),
    /// Number of overlapping canopy lobes.
    pub canopy_lobe_count: (u32, u32),
    /// Max trunk lean as a fraction of tree height.
    pub trunk_lean_max: f32,
    /// Silhouette family — drives which `TreeShape::compute_*` path runs.
    pub shape_kind: CompiledTreeShapeKind,
    /// 0..=1 keep-rate for canopy leaf voxels (lower = airier).
    pub canopy_density: f32,
}

#[derive(Clone, Debug)]
pub struct CompiledStructure {
    pub key: String,
    pub density: f32,
    pub min_spacing: u32,
    pub biomes: Vec<usize>,
    pub slope_max: f32,
    pub footprint_radius: u32,
    pub priority: i32,
    pub stamp: String,
}

#[derive(Clone, Debug)]
pub struct CompiledFauna {
    pub key: String,
    pub entity: String,
    pub biome_tags: Vec<String>,
    pub density: f32,
    pub group_size: (u32, u32),
    pub light: (f32, f32),
    pub despawn_distance: u32,
    pub sim_distance: u32,
}

#[derive(Clone, Debug)]
pub struct CompiledVisualDetailItem {
    pub block: VoxelId,
    pub weight: u32,
}

#[derive(Clone, Debug)]
pub struct CompiledVisualDetail {
    pub key: String,
    pub placement: CompiledFeaturePlacement,
    pub details: Vec<CompiledVisualDetailItem>,
}

#[derive(Clone, Debug)]
pub struct CompiledProceduralPlanet {
    pub key: String,
    pub base: CompiledPlanet,
    pub sea_level_offset: i32,
    pub climate: usize,
    pub biome_set: usize,
    pub terrain_layers: usize,
    pub caves: Vec<usize>,
    pub ore_sets: Vec<usize>,
    pub vegetation_sets: Vec<usize>,
    pub structure_sets: Vec<usize>,
    pub fauna_sets: Vec<usize>,
    pub visual_detail_sets: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct ProceduralRegistry {
    pub planets: Vec<CompiledProceduralPlanet>,
    pub fields: Vec<CompiledNoiseField>,
    pub climates: Vec<CompiledClimate>,
    pub biome_sets: Vec<CompiledBiomeSet>,
    pub biomes: Vec<CompiledProceduralBiome>,
    pub terrain_layers: Vec<CompiledTerrainLayerSet>,
    pub ores: Vec<CompiledOre>,
    pub caves: Vec<CompiledCave>,
    pub vegetation: Vec<CompiledVegetation>,
    pub structures: Vec<CompiledStructure>,
    pub fauna: Vec<CompiledFauna>,
    pub visual_details: Vec<CompiledVisualDetail>,
}

impl ProceduralRegistry {
    pub fn first_planet(&self) -> Option<&CompiledProceduralPlanet> {
        self.planets.first()
    }

    pub fn biome(&self, id: usize) -> &CompiledProceduralBiome {
        &self.biomes[id]
    }
}
