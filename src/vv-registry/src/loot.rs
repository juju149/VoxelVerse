use crate::{ItemId, LootTableId, RegistryTable};

#[derive(Debug, Clone)]
pub struct CompiledLootTable {
    pub pools: Vec<CompiledLootPool>,
}

#[derive(Debug, Clone)]
pub struct CompiledLootPool {
    pub rolls: u32,
    pub bonus_rolls: u32,
    pub entries: Vec<CompiledLootEntry>,
}

#[derive(Debug, Clone)]
pub struct CompiledLootEntry {
    pub item: ItemId,
    pub weight: u32,
    pub count_min: i32,
    pub count_max: i32,
}

pub type LootTableRegistry = RegistryTable<LootTableId, CompiledLootTable>;
