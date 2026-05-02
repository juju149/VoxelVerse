use crate::common::ResourceRef;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockShapeDef {
    pub kind: BlockShape,
    pub profile: BlockShapeProfile,
    pub bevel: f32,
    pub roundness: f32,
    pub face_depth: f32,
    pub normal_strength: f32,
}

pub type RawBlockGeometryDef = BlockShapeDef;

impl Default for BlockShapeDef {
    fn default() -> Self {
        Self {
            kind: BlockShape::Cube,
            profile: BlockShapeProfile::HardCube,
            bevel: 0.0,
            roundness: 0.0,
            face_depth: 0.0,
            normal_strength: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockShape {
    Cube,
    Cross,
    Fluid,
    Custom { model: ResourceRef },
}

impl Default for BlockShape {
    fn default() -> Self {
        Self::Cube
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockShapeProfile {
    HardCube,
    SoftCube,
    PillowCube,
    ChunkyCube,
    NaturalRock,
    LeafMass,
    Crystal,
    LiquidSoft,
}

pub type BlockGeometryProfile = BlockShapeProfile;

impl Default for BlockShapeProfile {
    fn default() -> Self {
        Self::HardCube
    }
}
