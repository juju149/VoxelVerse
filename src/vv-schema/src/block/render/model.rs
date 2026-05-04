use std::collections::BTreeMap;

use crate::common::HexColor;
use serde::{Deserialize, Serialize};

use super::details::BlockDetailFace;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockProceduralModelDef {
    pub seed: u32,
    pub palette: BlockModelPaletteDef,
    pub layers: Vec<BlockLayerDef>,
    pub details: Vec<BlockModelDetailDef>,
    pub instances: Vec<BlockInstanceDef>,
    pub lod: BlockModelLodDef,
}

impl Default for BlockProceduralModelDef {
    fn default() -> Self {
        Self {
            seed: 0,
            palette: BlockModelPaletteDef::default(),
            layers: Vec::new(),
            details: Vec::new(),
            instances: Vec::new(),
            lod: BlockModelLodDef::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockModelPaletteDef {
    pub colors: BTreeMap<String, HexColor>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockModelLodDef {
    pub shader_distance: f32,
    pub height_distance: f32,
    pub instance_distance: f32,
}

impl Default for BlockModelLodDef {
    fn default() -> Self {
        Self {
            shader_distance: 96.0,
            height_distance: 32.0,
            instance_distance: 16.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockLayerDef {
    pub id: String,
    pub enabled: bool,
    pub faces: Vec<BlockDetailFace>,
    pub mask: BlockMaskDef,
    pub operator: BlockLayerOperatorDef,
    pub blend: BlockLayerBlendDef,
    pub seed: u32,
}

impl Default for BlockLayerDef {
    fn default() -> Self {
        Self {
            id: String::new(),
            enabled: true,
            faces: vec![BlockDetailFace::All],
            mask: BlockMaskDef::All,
            operator: BlockLayerOperatorDef::default(),
            blend: BlockLayerBlendDef::default(),
            seed: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockMaskDef {
    All,
    HeightBand {
        #[serde(default)]
        from: f32,
        #[serde(default = "one")]
        to: f32,
        #[serde(default)]
        softness: f32,
    },
    Cap {
        #[serde(default = "default_cap_thickness")]
        thickness: f32,
        #[serde(default)]
        side_falloff: f32,
        #[serde(default)]
        drip_amount: f32,
        #[serde(default = "one")]
        drip_noise: f32,
    },
    Noise {
        #[serde(default = "one")]
        scale: f32,
        #[serde(default = "half")]
        threshold: f32,
        #[serde(default)]
        softness: f32,
        #[serde(default)]
        seed: u32,
    },
    Scatter {
        #[serde(default)]
        density: f32,
        #[serde(default = "default_min_size")]
        min_size: f32,
        #[serde(default = "default_max_size")]
        max_size: f32,
        #[serde(default)]
        clustering: f32,
        #[serde(default)]
        seed: u32,
    },
    CellGaps {
        source: String,
        #[serde(default = "default_gap_width")]
        width: f32,
    },
    And {
        masks: Vec<BlockMaskDef>,
    },
    Or {
        masks: Vec<BlockMaskDef>,
    },
    Subtract {
        base: Box<BlockMaskDef>,
        subtract: Box<BlockMaskDef>,
    },
}

impl Default for BlockMaskDef {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockLayerBlendDef {
    pub mode: BlockLayerBlendMode,
    pub opacity: f32,
}

impl Default for BlockLayerBlendDef {
    fn default() -> Self {
        Self {
            mode: BlockLayerBlendMode::Replace,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockLayerBlendMode {
    Replace,
    Add,
    Overlay,
    Multiply,
    Emissive,
}

impl Default for BlockLayerBlendMode {
    fn default() -> Self {
        Self::Replace
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockLayerOperatorDef {
    Flat {
        #[serde(default)]
        color: String,
    },
    Cells {
        #[serde(default)]
        color: String,
        #[serde(default)]
        shadow: String,
        #[serde(default = "default_cell_size")]
        cell_size: f32,
        #[serde(default = "half")]
        irregularity: f32,
        #[serde(default)]
        bevel: f32,
        #[serde(default)]
        height: f32,
    },
    Bricks {
        #[serde(default)]
        color: String,
        #[serde(default)]
        shadow: String,
        #[serde(default = "default_rows")]
        rows: u8,
        #[serde(default = "default_columns")]
        columns: u8,
        #[serde(default)]
        stagger: bool,
        #[serde(default = "default_gap_width")]
        mortar_width: f32,
        #[serde(default = "default_gap_depth")]
        mortar_depth: f32,
        #[serde(default)]
        bevel: f32,
        #[serde(default)]
        height: f32,
    },
    Tiles {
        #[serde(default)]
        color: String,
        #[serde(default)]
        shadow: String,
        #[serde(default = "default_rows")]
        rows: u8,
        #[serde(default = "default_columns")]
        columns: u8,
        #[serde(default = "default_gap_width")]
        gap_width: f32,
        #[serde(default = "default_gap_depth")]
        gap_depth: f32,
        #[serde(default)]
        bevel: f32,
        #[serde(default)]
        height: f32,
    },
    Rings {
        #[serde(default)]
        color: String,
        #[serde(default)]
        shadow: String,
        #[serde(default = "default_rows")]
        rings: u8,
        #[serde(default = "half")]
        wobble: f32,
        #[serde(default)]
        height: f32,
    },
    Strips {
        #[serde(default)]
        color: String,
        #[serde(default)]
        shadow: String,
        #[serde(default = "default_columns")]
        count: u8,
        #[serde(default)]
        vertical: bool,
        #[serde(default = "half")]
        wobble: f32,
        #[serde(default)]
        height: f32,
    },
    Waves {
        #[serde(default)]
        color: String,
        #[serde(default)]
        shadow: String,
        #[serde(default = "default_rows")]
        count: u8,
        #[serde(default = "half")]
        amplitude: f32,
        #[serde(default = "one")]
        frequency: f32,
        #[serde(default)]
        height: f32,
    },
    Cracks {
        #[serde(default)]
        color: String,
        #[serde(default = "default_crack_density")]
        density: f32,
        #[serde(default = "default_gap_depth")]
        depth: f32,
        #[serde(default = "one")]
        scale: f32,
    },
    Veins {
        #[serde(default)]
        color: String,
        #[serde(default)]
        vein_color: String,
        #[serde(default = "one")]
        scale: f32,
        #[serde(default = "half")]
        strength: f32,
    },
    Blobs {
        #[serde(default)]
        color: String,
        #[serde(default)]
        shadow: String,
        #[serde(default = "half")]
        density: f32,
        #[serde(default = "half")]
        roundness: f32,
        #[serde(default)]
        height: f32,
    },
    Granules {
        #[serde(default)]
        color: String,
        #[serde(default)]
        shadow: String,
        #[serde(default = "one")]
        density: f32,
        #[serde(default = "default_min_size")]
        min_size: f32,
        #[serde(default = "default_max_size")]
        max_size: f32,
        #[serde(default)]
        height: f32,
    },
    Inclusions {
        #[serde(default)]
        color: String,
        #[serde(default)]
        shadow: String,
        #[serde(default = "default_inclusion_density")]
        density: f32,
        #[serde(default = "default_max_size")]
        size: f32,
        #[serde(default)]
        height: f32,
    },
    EmissiveFill {
        #[serde(default)]
        color: String,
        #[serde(default)]
        hot_color: String,
        #[serde(default)]
        pulse: f32,
    },
    OrnamentalLine {
        #[serde(default)]
        color: String,
        #[serde(default)]
        pattern: String,
        #[serde(default = "default_gap_depth")]
        depth: f32,
        #[serde(default = "default_columns")]
        repeat: u8,
    },
}

impl Default for BlockLayerOperatorDef {
    fn default() -> Self {
        Self::Flat {
            color: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockModelDetailDef {
    pub kind: BlockModelDetailKind,
    pub layer: Option<String>,
    pub color: String,
    pub density: f32,
    pub size: BlockSizeRangeDef,
    pub slope_bias: f32,
    pub faces: Vec<BlockDetailFace>,
    pub seed: u32,
}

impl Default for BlockModelDetailDef {
    fn default() -> Self {
        Self {
            kind: BlockModelDetailKind::Speckles,
            layer: None,
            color: String::new(),
            density: 0.0,
            size: BlockSizeRangeDef::default(),
            slope_bias: 0.0,
            faces: vec![BlockDetailFace::All],
            seed: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockModelDetailKind {
    Pebbles,
    Roots,
    LeafLobes,
    Grain,
    Speckles,
    Stains,
    Cracks,
    Flowers,
    Crystals,
    Spikes,
    Droplets,
}

impl Default for BlockModelDetailKind {
    fn default() -> Self {
        Self::Speckles
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockSizeRangeDef {
    pub min: f32,
    pub max: f32,
}

impl Default for BlockSizeRangeDef {
    fn default() -> Self {
        Self {
            min: 0.04,
            max: 0.12,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockInstanceDef {
    pub kind: BlockInstanceKind,
    pub layer: Option<String>,
    pub color: String,
    pub colors: Vec<String>,
    pub center_color: String,
    pub density: f32,
    pub size: BlockSizeRangeDef,
    pub orientation: BlockInstanceOrientation,
    pub faces: Vec<BlockDetailFace>,
    pub lod: BlockInstanceLod,
    pub seed: u32,
}

impl Default for BlockInstanceDef {
    fn default() -> Self {
        Self {
            kind: BlockInstanceKind::PebbleBlob,
            layer: None,
            color: String::new(),
            colors: Vec::new(),
            center_color: String::new(),
            density: 0.0,
            size: BlockSizeRangeDef::default(),
            orientation: BlockInstanceOrientation::SurfaceNormalRandom,
            faces: vec![BlockDetailFace::All],
            lod: BlockInstanceLod::Near,
            seed: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockInstanceKind {
    LeafCard,
    FlowerCard,
    CrystalPrism,
    SpikeCone,
    PebbleBlob,
    RootCurve,
    DropletBlob,
}

impl Default for BlockInstanceKind {
    fn default() -> Self {
        Self::PebbleBlob
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockInstanceOrientation {
    SurfaceNormal,
    SurfaceNormalRandom,
    Vertical,
    Horizontal,
    Random,
}

impl Default for BlockInstanceOrientation {
    fn default() -> Self {
        Self::SurfaceNormalRandom
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockInstanceLod {
    Near,
    Mid,
    Far,
}

impl Default for BlockInstanceLod {
    fn default() -> Self {
        Self::Near
    }
}

fn one() -> f32 {
    1.0
}

fn half() -> f32 {
    0.5
}

fn default_cap_thickness() -> f32 {
    0.20
}

fn default_cell_size() -> f32 {
    0.18
}

fn default_rows() -> u8 {
    4
}

fn default_columns() -> u8 {
    4
}

fn default_gap_width() -> f32 {
    0.035
}

fn default_gap_depth() -> f32 {
    0.025
}

fn default_crack_density() -> f32 {
    0.08
}

fn default_inclusion_density() -> f32 {
    0.12
}

fn default_min_size() -> f32 {
    0.035
}

fn default_max_size() -> f32 {
    0.12
}
