use crate::common::HexColor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockFaceOverrides {
    pub top: Option<BlockFaceOverride>,
    pub side: Option<BlockFaceOverride>,
    pub bottom: Option<BlockFaceOverride>,
    pub north: Option<BlockFaceOverride>,
    pub south: Option<BlockFaceOverride>,
    pub east: Option<BlockFaceOverride>,
    pub west: Option<BlockFaceOverride>,
}

pub type RawBlockFaceVisuals = BlockFaceOverrides;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockFaceOverride {
    pub color_bias: Option<HexColor>,
}

pub type RawBlockFaceVisual = BlockFaceOverride;

impl Default for BlockFaceOverride {
    fn default() -> Self {
        Self { color_bias: None }
    }
}
