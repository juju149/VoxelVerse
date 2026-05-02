use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum BlockSurfaceProgramDef {
    Flat,
}

pub type RawBlockSurfaceProgramDef = BlockSurfaceProgramDef;

impl Default for BlockSurfaceProgramDef {
    fn default() -> Self {
        Self::Flat
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockSurfaceProgramId {
    Flat = 0,
}

impl BlockSurfaceProgramDef {
    pub fn program_id(&self) -> BlockSurfaceProgramId {
        match self {
            Self::Flat => BlockSurfaceProgramId::Flat,
        }
    }
}
