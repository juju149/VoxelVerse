use crate::{BlockId, ItemId, PlaceableId, RegistryTable, TagId};

#[derive(Debug, Clone)]
pub struct CompiledItem {
    pub display_key: Option<String>,
    pub stack_max: u8,
    pub tags: Vec<TagId>,
    pub kind: CompiledItemKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompiledItemKind {
    Block {
        block: BlockId,
    },
    Resource,
    Tool {
        tool_type: CompiledToolKind,
        tool_tier: u8,
        durability: u32,
        mining_speed: f32,
        attack_damage: f32,
    },
    Armor,
    Food,
    Placeable {
        placeable: PlaceableId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledToolKind {
    Hand,
    Pickaxe,
    Axe,
    Shovel,
    Sword,
    Shears,
    Hoe,
}

pub type ItemRegistry = RegistryTable<ItemId, CompiledItem>;
