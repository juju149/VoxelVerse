use crate::common::{BlockRef, IdealRange, LangKey, RgbColor, TagRef};
use serde::{Deserialize, Serialize};

/// Biome definition. Deserialized from defs/worldgen/biomes/<name>.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BiomeDef {
    pub display_key: Option<LangKey>,
    #[serde(default = "one")]
    pub weight: f32,
    #[serde(default)]
    pub required_tags: Vec<TagRef>,
    #[serde(default)]
    pub forbidden_tags: Vec<TagRef>,
    #[serde(default)]
    pub preferred_tags: Vec<TagRef>,
    #[serde(default)]
    pub provided_tags: Vec<TagRef>,
    pub climate: BiomeClimate,
    pub relief: BiomeRelief,
    pub surface: Vec<SurfaceLayer>,
    #[serde(default)]
    pub lod: BiomeLod,
}

fn one() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BiomeClimate {
    pub temperature: IdealRange,
    pub humidity: IdealRange,
    pub altitude: IdealRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BiomeRelief {
    pub base_height_m: f32,
    pub height_variance_m: f32,
    #[serde(default)]
    pub roughness: f32,
    #[serde(default)]
    pub river_probability: f32,
}

/// Surface layer. `depth_m: None` = infinite bottom layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceLayer {
    pub block: BlockRef,
    pub depth_m: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BiomeLod {
    pub horizon_color: RgbColor,
    pub fog_density: f32,
}

impl Default for BiomeLod {
    fn default() -> Self {
        BiomeLod {
            horizon_color: RgbColor {
                r: 0.6,
                g: 0.7,
                b: 0.8,
            },
            fog_density: 0.002,
        }
    }
}
