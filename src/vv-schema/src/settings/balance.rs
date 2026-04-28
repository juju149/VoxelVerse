use serde::{Deserialize, Serialize};

/// Gameplay balance tuning knobs. Deserialized from defs/settings/balance.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BalanceSettings {
    pub base_mining_speed: f32,
    pub inventory_weight_speed_penalty: f32,
    pub tool_damage_multiplier: f32,
    pub xp_multiplier: f32,
}

impl Default for BalanceSettings {
    fn default() -> Self {
        BalanceSettings {
            base_mining_speed: 1.0,
            inventory_weight_speed_penalty: 0.15,
            tool_damage_multiplier: 1.0,
            xp_multiplier: 1.0,
        }
    }
}
