use crate::common::{FloatRange, TagRef};
use serde::{Deserialize, Serialize};

/// Climate tags with normalized value ranges [0..1].
/// Singleton file: defs/worldgen/climate/tags.ron
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClimateTagsDef {
    pub temperature: Vec<ClimateRange>,
    pub humidity: Vec<ClimateRange>,
    pub altitude: Vec<ClimateRange>,
    pub slope: Vec<ClimateRange>,
    pub latitude: Vec<ClimateRange>,
    pub depth: Vec<ClimateRange>,
    /// Combinatorial rules: if all conditions are met, tags are produced.
    pub derived: Vec<DerivedTagRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClimateRange {
    pub tag: TagRef,
    pub range: FloatRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedTagRule {
    pub requires: Vec<TagRef>,
    pub produces: Vec<TagRef>,
}

/// Global climate curve parameters.
/// Singleton file: defs/worldgen/climate/curves.ron
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GlobalClimateCurvesDef {
    pub temperature_noise_scale: f32,
    pub humidity_noise_scale: f32,
    pub minimum_biome_transition_m: f32,
}

impl Default for GlobalClimateCurvesDef {
    fn default() -> Self {
        GlobalClimateCurvesDef {
            temperature_noise_scale: 7.7,
            humidity_noise_scale: 3.1,
            minimum_biome_transition_m: 20.0,
        }
    }
}

/// Climate transition blend rules between tags.
/// Singleton file: defs/worldgen/climate/transitions.ron
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClimateTransitionsDef {
    pub transitions: Vec<ClimateTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClimateTransition {
    pub from: TagRef,
    pub to: TagRef,
    pub blend_distance_m: f32,
}
