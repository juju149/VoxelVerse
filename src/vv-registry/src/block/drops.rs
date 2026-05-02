use crate::{CompiledLootPool, LootTableId};

#[derive(Debug, Clone)]
pub enum CompiledDrops {
    None,
    Inline(Vec<CompiledLootPool>),
    Table(LootTableId),
}
