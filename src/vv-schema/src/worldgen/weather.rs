use crate::common::{FloatRange, LangKey, ResourceRef, TagRef};
use serde::{Deserialize, Serialize};

/// Weather phenomenon definition. Deserialized from defs/worldgen/weather/<name>.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeatherDef {
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
    pub intensity: FloatRange,
    pub duration_seconds: FloatRange,
    #[serde(default)]
    pub particle: Option<ResourceRef>,
    #[serde(default)]
    pub sound: Option<ResourceRef>,
}

fn one() -> f32 {
    1.0
}
