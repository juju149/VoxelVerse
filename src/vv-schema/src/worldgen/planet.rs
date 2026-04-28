use crate::common::{LangKey, TagRef};
use serde::{Deserialize, Serialize};

/// Domain-typed reference to a planet type definition.
/// Format: "namespace:name". E.g. "voxelverse:temperate".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlanetTypeRef(pub String);

/// Planet type definition. Deserialized from defs/worldgen/planet_types/<name>.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanetTypeDef {
    pub display_key: Option<LangKey>,
    #[serde(default)]
    pub global_tags: Vec<TagRef>,
    #[serde(default)]
    pub global_forbidden_tags: Vec<TagRef>,
    #[serde(default)]
    pub climate_bias: ClimateBias,
    #[serde(default = "one")]
    pub altitude_variance_multiplier: f32,
    pub ocean_coverage: f32,
    #[serde(default = "half")]
    pub climate_transition_speed: f32,
    pub size: PlanetSize,
    #[serde(default)]
    pub size_climate_effect: Option<PlanetSizeClimateEffect>,
}

fn one() -> f32 {
    1.0
}
fn half() -> f32 {
    0.5
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ClimateBias {
    pub temperature: f32,
    pub humidity: f32,
}

impl Default for ClimateBias {
    fn default() -> Self {
        ClimateBias {
            temperature: 0.0,
            humidity: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanetSize {
    pub min_km: f32,
    pub max_km: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanetSizeClimateEffect {
    pub small_humidity_delta: f32,
    pub large_temperature_range_expansion: f32,
}
