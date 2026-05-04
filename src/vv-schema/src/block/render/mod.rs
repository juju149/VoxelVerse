pub mod details;
pub mod faces;
pub mod material;
pub mod meshing;
pub mod model;
pub mod patch;
pub mod program;
pub mod shape;
pub mod variation;

pub use details::*;
pub use faces::*;
pub use material::*;
pub use meshing::*;
pub use model::*;
pub use patch::*;
pub use program::*;
pub use shape::*;
pub use variation::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockRenderDef {
    pub material: BlockMaterialDef,
    pub lighting: BlockLightingDef,
    pub shape: BlockShapeDef,

    // Legacy/simple bridge. Kept so old .ron still works.
    // New authored blocks should prefer `model`.
    pub program: BlockSurfaceProgramDef,

    // VoxelForge procedural surface model.
    // A complete .ron can now sculpt layers, masks, operators, details and near-camera instances.
    pub model: Option<BlockProceduralModelDef>,

    pub variation: BlockVariationDef,
    pub environment: BlockEnvironmentDef,
    pub faces: BlockFaceOverrides,
    pub details: Vec<BlockDetailDef>,
    pub meshing: BlockMeshingDef,
}

pub type RawBlockRenderDef = BlockRenderDef;

impl Default for BlockRenderDef {
    fn default() -> Self {
        Self {
            material: BlockMaterialDef::default(),
            lighting: BlockLightingDef::default(),
            shape: BlockShapeDef::default(),
            program: BlockSurfaceProgramDef::default(),
            model: None,
            variation: BlockVariationDef::default(),
            environment: BlockEnvironmentDef::default(),
            faces: BlockFaceOverrides::default(),
            details: Vec::new(),
            meshing: BlockMeshingDef::default(),
        }
    }
}
