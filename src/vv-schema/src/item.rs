use crate::common::tool::ToolKind;
use crate::common::{BlockRef, ItemRef, LangKey, PlaceableRef, TagRef};
use serde::{Deserialize, Serialize};

/// Raw item definition. ID is derived from the file path.
/// Deserialized from defs/items/<name>.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ItemDef {
    /// Lang key override. If absent: auto-derived as "item.<ns>.<name>".
    pub display_key: Option<LangKey>,
    #[serde(default = "default_stack_max")]
    pub stack_max: u8,
    pub weight: f32,
    #[serde(default)]
    pub trade_value: Option<u32>,
    pub tags: Vec<TagRef>,
    pub kind: ItemKind,
}

fn default_stack_max() -> u8 {
    64
}

impl Default for ItemDef {
    fn default() -> Self {
        ItemDef {
            display_key: None,
            stack_max: 64,
            weight: 0.1,
            trade_value: None,
            tags: vec![],
            kind: ItemKind::Resource,
        }
    }
}

/// Functional variant of an item.
/// Avoids a flat god-struct with all optional fields side by side.
/// `ToolKind` imported from `common::tool` — same type as in `BlockMiningDef`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ItemKind {
    /// Item that places a terrain block in the world.
    Block { block: BlockRef },
    /// Raw or processed resource (ore, ingot, plank, etc.).
    Resource,
    /// Tool with mining and attack stats.
    Tool {
        /// Same type as BlockMiningDef.tool — single source of truth.
        tool_type: ToolKind,
        tool_tier: u8,
        durability: u32,
        mining_speed: f32,
        attack_damage: f32,
        #[serde(default)]
        repair_with: Option<ItemRef>,
    },
    /// Wearable armor piece.
    Armor {
        slot: ArmorSlot,
        defense: f32,
        durability: u32,
        #[serde(default)]
        repair_with: Option<ItemRef>,
    },
    /// Consumable food.
    Food {
        hunger_restore: f32,
        #[serde(default)]
        saturation: f32,
    },
    /// Item that spawns a PlaceableDef entity in the world (non-voxel placed object).
    Placeable { placeable: PlaceableRef },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArmorSlot {
    Helmet,
    Chestplate,
    Leggings,
    Boots,
}
