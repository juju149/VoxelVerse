use crate::common::{FloatRange, LangKey, LootTableRef, ResourceRef, TagRef};
use serde::{Deserialize, Serialize};

/// Structure worldgen feature. Deserialized from defs/worldgen/structures/<name>.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructureDef {
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
    pub placement: StructurePlacement,
    /// Template file reference (e.g. resources/structures/ruined_temple.vox).
    #[serde(default)]
    pub template: Option<ResourceRef>,
    /// Loot table for integrated chests.
    #[serde(default)]
    pub loot_table: Option<LootTableRef>,
}

fn one() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructurePlacement {
    /// Structures generated per normalized chunk.
    pub frequency: f32,
    #[serde(default)]
    pub altitude_range: Option<FloatRange>,
    #[serde(default = "default_slope_max")]
    pub slope_max: f32,
}

fn default_slope_max() -> f32 {
    0.3
}
