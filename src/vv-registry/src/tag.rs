use crate::{BlockId, EntityId, ItemId, PlaceableId, RegistryTable, TagId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagDomain {
    Block,
    Item,
    Entity,
    Placeable,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaggedContent {
    Block(BlockId),
    Item(ItemId),
    Entity(EntityId),
    Placeable(PlaceableId),
}

#[derive(Debug, Clone)]
pub struct CompiledTag {
    pub domain: TagDomain,
    pub values: Vec<TaggedContent>,
    pub extends: Vec<TagId>,
}

pub type TagRegistry = RegistryTable<TagId, CompiledTag>;
