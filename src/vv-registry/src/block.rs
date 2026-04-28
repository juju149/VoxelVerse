use crate::{BlockId, CompiledLootPool, LootTableId, RegistryTable, TagId};

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

#[derive(Debug, Clone, Copy)]
pub struct CompiledBlockMining {
    pub hardness: f32,
    pub tool_tier_min: u8,
    pub drop_xp: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledMaterialPhase {
    Solid,
    Liquid,
    Passable,
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledBlockPhysics {
    pub phase: CompiledMaterialPhase,
    pub density: f32,
    pub friction: f32,
    pub drag: f32,
}

#[derive(Debug, Clone)]
pub struct CompiledBlockRender {
    pub color: [f32; 3],
    pub roughness: f32,
    pub translucent: bool,
    pub emits_light: u8,
    pub texture_layout: CompiledTextureLayout,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledTextureLayout {
    Single,
    Sides,
    Custom,
}

#[derive(Debug, Clone)]
pub enum CompiledDrops {
    None,
    Inline(Vec<CompiledLootPool>),
    Table(LootTableId),
}
