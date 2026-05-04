use crate::common::HexColor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockDetailDef {
    pub kind: BlockDetailKind,
    pub color: HexColor,
    pub density: f32,
    pub min_size: f32,
    pub max_size: f32,
    pub slope_bias: f32,
    pub faces: Vec<BlockDetailFace>,
    pub seed: u32,
}

impl Default for BlockDetailDef {
    fn default() -> Self {
        Self {
            kind: BlockDetailKind::Speckle,
            color: HexColor("#FFFFFF80".to_owned()),
            density: 0.0,
            min_size: 0.04,
            max_size: 0.12,
            slope_bias: 0.0,
            faces: vec![BlockDetailFace::All],
            seed: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockDetailKind {
    Pebble,
    Root,
    LeafLobe,
    Grain,
    Speckle,
    Stain,
    Crack,
}

impl Default for BlockDetailKind {
    fn default() -> Self {
        Self::Speckle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockDetailFace {
    Top,
    Bottom,
    Side,
    North,
    South,
    East,
    West,
    All,
}
