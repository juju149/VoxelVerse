#![allow(dead_code)]

use crate::CompiledPlanet;
use vv_content_schema::{RawCurve, RawHeightCurveDef, RawNoiseKind, RawTreeShapeKind};
use vv_voxel::VoxelId;

#[derive(Clone, Copy, Debug)]
pub enum CompiledNoiseKind {
    Perlin,
    Simplex,
    OpenSimplex2S,
    Ridged,
    Cellular,
    Constant,
}

impl From<&RawNoiseKind> for CompiledNoiseKind {
    fn from(value: &RawNoiseKind) -> Self {
        match value {
            RawNoiseKind::Perlin => Self::Perlin,
            RawNoiseKind::Simplex => Self::Simplex,
            RawNoiseKind::OpenSimplex2S => Self::OpenSimplex2S,
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
    pub height_curve: CompiledHeightCurve,
    /// 0..=2 — strength of the mountain boost layer for this biome.
    pub mountain_intensity: f32,
    /// 0..=1 — weight blended into the region slope-smoothing pass.
    pub slope_smoothing: f32,
}

/// Non-linear height transfer curve.  All variants accept `t ∈ [-1, 1]`
/// and return a value of similar magnitude — never expanding the range
/// beyond ~[-1.3, 1.3] so per-biome amplitudes stay predictable.
#[derive(Clone, Copy, Debug, Default)]
pub enum CompiledHeightCurve {
    #[default]
    Linear,
    Smoothstep,
    Power {
        exponent: f32,
    },
    Plateau {
        flatness: f32,
        threshold: f32,
    },
    MountainSpike {
        sharpness: f32,
    },
}

impl CompiledHeightCurve {
    /// Apply the curve.  Input is expected in `[-1, 1]`; output stays in
    /// roughly the same range.  Pure odd functions where possible so
    /// symmetric noise stays symmetric.
    pub fn evaluate(self, t: f32) -> f32 {
        match self {
            Self::Linear => t,
            Self::Smoothstep => {
                let u = (t * 0.5 + 0.5).clamp(0.0, 1.0);
                let s = u * u * (3.0 - 2.0 * u);
                s * 2.0 - 1.0
            }
            Self::Power { exponent } => {
                let e = exponent.max(0.05);
                t.signum() * t.abs().powf(e)
            }
            Self::Plateau {
                flatness,
                threshold,
            } => {
                let th = threshold.clamp(0.01, 0.95);
                let f = flatness.clamp(0.0, 1.0);
                if t.abs() <= th {
                    t * (1.0 - f)
                } else {
                    let sign = t.signum();
                    let outer = (t.abs() - th) / (1.0 - th);
                    sign * (th * (1.0 - f) + outer * (1.0 - th))
                }
            }
            Self::MountainSpike { sharpness } => {
                // Only sharpens positives.  Negatives (valleys, lowlands)
                // stay linear so flatlands keep their feel.
                let s = sharpness.max(0.0);
                if t > 0.0 {
                    let exp = 1.0 + s;
                    t.powf(exp)
                } else {
                    t
                }
            }
        }
    }
}

impl From<&RawHeightCurveDef> for CompiledHeightCurve {
    fn from(value: &RawHeightCurveDef) -> Self {
        match value {
            RawHeightCurveDef::Linear => Self::Linear,
            RawHeightCurveDef::Smoothstep => Self::Smoothstep,
            RawHeightCurveDef::Power { exponent } => Self::Power {
                exponent: *exponent,
            },
            RawHeightCurveDef::Plateau {
                flatness,
                threshold,
            } => Self::Plateau {
                flatness: *flatness,
                threshold: *threshold,
            },
            RawHeightCurveDef::MountainSpike { sharpness } => Self::MountainSpike {
                sharpness: *sharpness,
            },
        }
    }
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

/// Where in cave geometry a compiled placement targets its props.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CaveSurface {
    /// Normal above-ground placement (default).
    #[default]
    TopSurface,
    /// Solid block directly below cave air — floors and ledges.
    CaveFloor,
    /// Solid block directly above cave air — ceilings and overhangs.
    CaveCeiling,
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
    /// Minimum spacing in voxels between two instances (drives the
    /// placement grid cell size).  0 = no spacing enforced.
    pub min_spacing: f32,
    /// 0..1 fraction of the placement cell randomly offset per candidate.
    pub jitter_strength: f32,
    /// Optional clump-modulator noise field index.  When set the effective
    /// density is `base × lerp(1, clump, clump_strength)`.
    pub clump_field: Option<usize>,
    pub clump_strength: f32,
    /// Optional altitude window in voxels relative to sea level.
    pub altitude_range: Option<(f32, f32)>,
    /// Optional humidity / temperature windows in normalized climate space.
    pub humidity_range: Option<(f32, f32)>,
    pub temperature_range: Option<(f32, f32)>,
    /// Optional minimum slope as a normalized 0..1 value (sin of degrees).
    pub slope_min: f32,
    /// Per-instance scale variance bounds (min, max).  (1.0, 1.0) = none.
    pub scale_variance: (f32, f32),
    /// 0..1 — fraction of 2π randomised per instance.
    pub rotation_variance: f32,
    /// Which cave surface this placement targets.
    pub cave_surface: CaveSurface,
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

    /// Returns a 0..1 weight for how much this biome matches the placement.
    /// Unlike `allowed_in_biome`, this is continuous — used to feather
    /// forest edges across biome transitions instead of cutting in a
    /// straight line.  Currently binary at the per-biome level, but
    /// callers fold it with the biome-blend weights to get smooth edges.
    pub fn biome_match_weight(&self, biome: &CompiledProceduralBiome) -> f32 {
        if self.allowed_in_biome(biome) {
            1.0
        } else {
            0.0
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CompiledPlanetStreaming {
    pub near_voxel_lod_radius: u32,
    pub far_surface_lod_radius: u32,
    pub upload_budget_chunks_per_frame: u32,
    pub region_cell_voxels: u32,
    pub feature_budget_per_chunk: u32,
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
    /// Surface root flare radius in voxels (0 = no roots).
    pub root_radius: f32,
    /// Probability 0..1 that a placement becomes a fallen trunk instead.
    pub fallen_chance: f32,
    /// 0..1 mid-trunk S-curve strength applied on top of lean.
    pub trunk_curve_strength: f32,
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

/// Item dropped when a vox prop (or its support block) is destroyed.
#[derive(Clone, Debug)]
pub struct CompiledVoxPropDrop {
    pub item: String,
    pub count: (u32, u32),
    pub chance: f32,
}

/// One selectable variant in a vox prop scatter — a specific .vox asset key,
/// its relative weight, optional drops, and vertical offset.
#[derive(Clone, Debug)]
pub struct CompiledVoxPropVariant {
    /// Full content ref to a .vox asset, e.g.
    /// `"core:voxel/vegetation/flowers/flower_blue_1"`.
    pub model_key: String,
    pub weight: u32,
    pub drops: Vec<CompiledVoxPropDrop>,
    /// Extra voxels above the surface block (0 = touching the surface).
    pub y_offset: i32,
}

/// A compiled scatter rule: biome-filtered placement parameters + a list of
/// weighted .vox variants to instantiate.
#[derive(Clone, Debug)]
pub struct CompiledVoxPropScatter {
    pub key: String,
    pub placement: CompiledFeaturePlacement,
    pub variants: Vec<CompiledVoxPropVariant>,

    /// Cached total weight for O(1) selection.
    pub total_weight: u32,
}

impl CompiledVoxPropScatter {
    /// Pick a variant index using a pre-hashed integer in `[0, u32::MAX]`.
    pub fn pick_variant(&self, hash: u32) -> Option<&CompiledVoxPropVariant> {
        if self.total_weight == 0 || self.variants.is_empty() {
            return None;
        }
        let mut rem = hash % self.total_weight;
        for v in &self.variants {
            if rem < v.weight {
                return Some(v);
            }
            rem -= v.weight;
        }
        self.variants.last()
    }
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
    pub vox_prop_scatters: Vec<usize>,
    pub streaming: CompiledPlanetStreaming,
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
    pub vox_prop_scatters: Vec<CompiledVoxPropScatter>,
}

impl ProceduralRegistry {
    pub fn first_planet(&self) -> Option<&CompiledProceduralPlanet> {
        self.planets.first()
    }

    pub fn biome(&self, id: usize) -> &CompiledProceduralBiome {
        &self.biomes[id]
    }
}
