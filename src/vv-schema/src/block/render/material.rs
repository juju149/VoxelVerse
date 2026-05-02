use crate::common::{HexColor, ResourceRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockMaterialDef {
    pub kind: BlockMaterialKind,
    pub base_color: HexColor,
    pub palette: Vec<HexColor>,
    pub roughness: f32,
    pub metallic: f32,
    pub alpha: f32,
    pub tint: TintMode,
}

pub type RawBlockSurfaceDef = BlockMaterialDef;

impl Default for BlockMaterialDef {
    fn default() -> Self {
        Self {
            kind: BlockMaterialKind::Generic,
            base_color: HexColor("#8A8A8A".into()),
            palette: Vec::new(),
            roughness: 0.85,
            metallic: 0.0,
            alpha: 1.0,
            tint: TintMode::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockMaterialKind {
    Generic,
    Stone,
    Dirt,
    Grass,
    Sand,
    Snow,
    Wood,
    Leaves,
    Metal,
    Glass,
    Ice,
    Liquid,
    Emissive,
    Custom { material: ResourceRef },
}

impl Default for BlockMaterialKind {
    fn default() -> Self {
        Self::Generic
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockLightingDef {
    pub emission: Option<HexColor>,
    pub emits_light: u8,
}

pub type RawBlockLightingDef = BlockLightingDef;

impl Default for BlockLightingDef {
    fn default() -> Self {
        Self {
            emission: None,
            emits_light: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TintMode {
    None,
    GrassColor,
    FoliageColor,
    WaterColor,
}

impl Default for TintMode {
    fn default() -> Self {
        Self::None
    }
}
