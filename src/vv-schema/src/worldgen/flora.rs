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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FloraPlacement {
    pub density_base: f32,
    pub altitude_max: Option<f32>,
    pub slope_max: f32,
    pub near_water_bonus: f32,
    pub cluster_radius: f32,
    pub cluster_min: u32,
    pub cluster_max: u32,
}

impl Default for FloraPlacement {
    fn default() -> Self {
        FloraPlacement {
            density_base: 0.05,
            altitude_max: None,
            slope_max: 0.5,
            near_water_bonus: 1.0,
            cluster_radius: 3.0,
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
        height_min: u32,
        height_max: u32,
    },
    Tree {
        log_block: BlockRef,
        leaf_block: BlockRef,
        trunk_height_min: u32,
        trunk_height_max: u32,
        canopy_radius: f32,
        canopy_height: f32,
    },
    Cluster {
        block: BlockRef,
        radius_min: f32,
        radius_max: f32,
    },
}
