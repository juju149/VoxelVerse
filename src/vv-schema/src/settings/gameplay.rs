use serde::{Deserialize, Serialize};

/// Core gameplay parameters. Deserialized from defs/settings/gameplay.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GameplaySettings {
    pub reach_m: f32,
    pub gravity_m_s2: f32,
    pub day_length_seconds: u32,
    pub fall_damage_threshold_m: f32,
    pub hunger_depletion_rate: f32,
}

impl Default for GameplaySettings {
    fn default() -> Self {
        GameplaySettings {
            reach_m: 4.5,
            gravity_m_s2: 9.81,
            day_length_seconds: 1200,
            fall_damage_threshold_m: 3.0,
            hunger_depletion_rate: 1.0,
        }
    }
}
