use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockVariationDef {
    pub per_voxel_tint: f32,
    pub per_face_tint: f32,
    pub macro_noise_scale: f32,
    pub macro_noise_strength: f32,
    pub micro_noise_scale: f32,
    pub micro_noise_strength: f32,
    pub edge_darkening: f32,
    pub ao_influence: f32,
}

pub type RawBlockVisualVariation = BlockVariationDef;

impl Default for BlockVariationDef {
    fn default() -> Self {
        Self {
            per_voxel_tint: 0.0,
            per_face_tint: 0.0,
            macro_noise_scale: 1.0,
            macro_noise_strength: 0.0,
            micro_noise_scale: 1.0,
            micro_noise_strength: 0.0,
            edge_darkening: 0.0,
            ao_influence: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockEnvironmentDef {
    pub biome_tint_strength: f32,
    pub wetness_response: f32,
    pub snow_response: f32,
    pub dust_response: f32,
}

pub type RawBlockEnvironmentResponseDef = BlockEnvironmentDef;

impl Default for BlockEnvironmentDef {
    fn default() -> Self {
        Self {
            biome_tint_strength: 0.0,
            wetness_response: 0.0,
            snow_response: 0.0,
            dust_response: 0.0,
        }
    }
}
