use crate::{BlockId, ItemId, PlaceableId, RegistryTable, TagId};

#[derive(Debug, Clone)]
pub struct CompiledItem {
    pub display_key: Option<String>,
    pub stack_max: u8,
    pub tags: Vec<TagId>,
    pub kind: CompiledItemKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledItemKind {
    Block { block: BlockId },
    Resource,
    Tool,
    Armor,
    Food,
    Placeable { placeable: PlaceableId },
}

pub type ItemRegistry = RegistryTable<ItemId, CompiledItem>;
