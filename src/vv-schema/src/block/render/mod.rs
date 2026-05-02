pub mod faces;
pub mod material;
pub mod meshing;
pub mod patch;
pub mod program;
pub mod shape;
pub mod variation;

pub use faces::*;
pub use material::*;
pub use meshing::*;
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
    pub program: BlockSurfaceProgramDef,
    pub variation: BlockVariationDef,
    pub environment: BlockEnvironmentDef,
    pub faces: BlockFaceOverrides,
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
            variation: BlockVariationDef::default(),
            environment: BlockEnvironmentDef::default(),
            faces: BlockFaceOverrides::default(),
            meshing: BlockMeshingDef::default(),
        }
    }
}
