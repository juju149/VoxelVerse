use crate::common::{LangKey, ScriptRef, TagRef};
use crate::loot::DropSpec;
use serde::{Deserialize, Serialize};

pub mod interaction;
pub mod mining;
pub mod physics;
pub mod placement;
pub mod render;
pub mod states;

pub use interaction::*;
pub use mining::*;
pub use physics::*;
pub use placement::*;
pub use render::*;
pub use states::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockDef {
    pub display_key: Option<LangKey>,
    pub tags: Vec<TagRef>,
    pub mining: BlockMiningDef,
    pub render: BlockRenderDef,
    pub physics: BlockPhysicsDef,
    pub drops: DropSpec,

    #[serde(default = "default_stack_max")]
    pub stack_max: u8,

    #[serde(default)]
    pub states: Option<BlockStatesDef>,

    #[serde(default)]
    pub placement: Option<BlockPlacementDef>,

    #[serde(default)]
    pub interaction: Option<BlockInteractionDef>,
}

fn default_stack_max() -> u8 {
    64
}

impl Default for BlockDef {
    fn default() -> Self {
        Self {
            display_key: None,
            tags: Vec::new(),
            mining: BlockMiningDef::default(),
            render: BlockRenderDef::default(),
            physics: BlockPhysicsDef::default(),
            drops: DropSpec::default(),
            stack_max: 64,
            states: None,
            placement: None,
            interaction: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BlockUseAction {
    CraftingGrid { width: u8, height: u8 },
    Smelting,
    Storage { slots: u32, rows: u8 },
    Custom { script: ScriptRef },
}
