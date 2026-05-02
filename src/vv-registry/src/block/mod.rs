pub mod drops;
pub mod mining;
pub mod physics;
pub mod program;
pub mod render;
pub mod runtime;
pub mod visual;

pub use drops::*;
pub use mining::*;
pub use physics::*;
pub use program::*;
pub use render::*;
pub use runtime::*;
pub use visual::*;

use crate::{BlockId, RegistryTable, TagId};

#[derive(Debug, Clone)]
pub struct CompiledBlock {
    pub display_key: Option<String>,
    pub stack_max: u8,
    pub tags: Vec<TagId>,
    pub mining: CompiledBlockMining,
    pub physics: CompiledBlockPhysics,
    pub render: CompiledBlockRender,
    pub drops: CompiledDrops,
}

pub type BlockRegistry = RegistryTable<BlockId, CompiledBlock>;
