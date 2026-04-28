use crate::common::{BlockRef, FloatRange, IntRange, LangKey, TagRef};
use serde::{Deserialize, Serialize};

/// Ore vein worldgen feature. Deserialized from defs/worldgen/ores/<name>.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OreDef {
    pub display_key: Option<LangKey>,
    /// Block to place for the ore vein.
    pub block: BlockRef,
    #[serde(default = "one")]
    pub weight: f32,
    #[serde(default)]
    pub required_tags: Vec<TagRef>,
    #[serde(default)]
    pub forbidden_tags: Vec<TagRef>,
    #[serde(default)]
    pub optional_tags: Vec<TagRef>,
    pub vein: VeinSpec,
}

fn one() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VeinSpec {
    /// Vein size in blocks.
    pub size: IntRange,
    /// Depth below the surface in meters (positive values = deeper).
    pub depth_m: FloatRange,
    /// Generation frequency: veins per normalized chunk.
    pub frequency: f32,
}
