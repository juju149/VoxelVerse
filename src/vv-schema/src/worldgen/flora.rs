use crate::common::{BlockRef, LangKey, TagRef};
use serde::{Deserialize, Serialize};

/// Flora worldgen feature. Deserialized from defs/worldgen/flora/<name>.ron.
/// Categories (tree, flower, cluster) live in tags, not filesystem subdirectories.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FloraDef {
    pub display_key: Option<LangKey>,
    #[serde(default = "one")]
    pub weight: f32,
    #[serde(default)]
    pub required_tags: Vec<TagRef>,
    #[serde(default)]
    pub forbidden_tags: Vec<TagRef>,
    #[serde(default)]
    pub optional_tags: Vec<TagRef>,
    #[serde(default)]
    pub provided_tags: Vec<TagRef>,
    pub placement: FloraPlacement,
    pub feature: FloraFeature,
}

fn one() -> f32 {
    1.0
}

fn default_canopy_start_t() -> f32 {
    0.75
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FloraPlacement {
    /// Spawn density per square meter of surface.
    pub density_base: f32,
    /// Maximum altitude above the authored planet radius, in meters.
    pub altitude_max_m: Option<f32>,
    pub slope_max: f32,
    pub near_water_bonus: f32,
    /// Cluster radius in meters; voxel coverage is derived at generation time.
    pub cluster_radius_m: f32,
    pub cluster_min: u32,
    pub cluster_max: u32,
}

impl Default for FloraPlacement {
    fn default() -> Self {
        FloraPlacement {
            density_base: 0.05,
            altitude_max_m: None,
            slope_max: 0.5,
            near_water_bonus: 1.0,
            cluster_radius_m: 3.0,
            cluster_min: 1,
            cluster_max: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum FloraFeature {
    Plant {
        block: BlockRef,
        /// Physical plant height range in meters.
        height_min_m: f32,
        height_max_m: f32,
    },
    Tree {
        log_block: BlockRef,
        leaf_block: BlockRef,
        /// Physical trunk height range in meters.
        trunk_height_min_m: f32,
        trunk_height_max_m: f32,
        /// Physical canopy dimensions in meters.
        canopy_radius_m: f32,
        canopy_height_m: f32,
        /// Fraction of trunk height at which foliage begins (0.0 = ground, 1.0 = top).
        /// Default 0.75 produces a crown that starts near the top of the trunk.
        #[serde(default = "default_canopy_start_t")]
        canopy_start_t: f32,
        /// Girth multiplier for the trunk cross-section (0.0 = single-voxel, 1.0 = widest).
        /// Values above 0.5 enable multi-voxel trunks on tall trees.
        #[serde(default)]
        trunk_girth: f32,
        /// Silhouette bias: negative values favour a columnar crown, positive values
        /// favour a spreading crown. Clamped to [-1.0, 1.0].
        #[serde(default)]
        crown_bias: f32,
    },
    Cluster {
        block: BlockRef,
        /// Physical cluster radius range in meters.
        radius_min_m: f32,
        radius_max_m: f32,
    },
}
