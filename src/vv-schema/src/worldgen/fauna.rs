use crate::common::{EntityRef, FloatRange, LangKey, TagRef};
use serde::{Deserialize, Serialize};

/// Fauna spawn definition. Deserialized from defs/worldgen/fauna/<name>.ron.
/// Categories (predator, prey, passive) live in entity tags + AiBehavior, not filesystem paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaunaDef {
    pub display_key: Option<LangKey>,
    /// Reference to the entity definition in defs/entities/.
    pub entity: EntityRef,
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
    pub spawn: FaunaSpawn,
}

fn one() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaunaSpawn {
    pub group_min: u32,
    pub group_max: u32,
    pub density: f32,
    pub altitude_range: FloatRange,
    #[serde(default)]
    pub time_of_day: Option<FloatRange>,
}
