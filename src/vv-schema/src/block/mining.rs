use crate::common::tool::ToolKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockMiningDef {
    pub hardness: f32,
    pub tool: ToolKind,
    pub tool_tier_min: u8,
    pub sound_material: SoundMaterial,
    pub drop_xp: u8,
}

impl Default for BlockMiningDef {
    fn default() -> Self {
        Self {
            hardness: 1.0,
            tool: ToolKind::Hand,
            tool_tier_min: 0,
            sound_material: SoundMaterial::Stone,
            drop_xp: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundMaterial {
    Stone,
    Dirt,
    Gravel,
    Sand,
    Wood,
    Grass,
    Water,
    Lava,
    Glass,
    Metal,
    Cloth,
}

impl Default for SoundMaterial {
    fn default() -> Self {
        Self::Stone
    }
}
