use crate::common::{IntRange, ItemRef, LootTableRef};
use serde::{Deserialize, Serialize};

/// Named loot table file. Referenceable by LootTableRef.
/// Deserialized from defs/loot_tables/<name>.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LootTableDef {
    pub pools: Vec<LootPool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LootPool {
    #[serde(default = "default_rolls")]
    pub rolls: u32,
    #[serde(default)]
    pub bonus_rolls: u32,
    pub entries: Vec<LootEntry>,
}

fn default_rolls() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LootEntry {
    /// Always an item. Loot tables drop items, never raw blocks.
    pub item: ItemRef,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub count: IntRange,
}

fn default_weight() -> u32 {
    1
}

/// Drop specification: none, inline pools, or reference to a named loot table.
/// Used in BlockDef, EntityDef, and PlaceableDef.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DropSpec {
    /// No drops.
    None,
    /// Anonymous inline pools. Convenient for blocks with 1-2 simple drops.
    Inline(Vec<LootPool>),
    /// Reference to a defs/loot_tables/<name>.ron file.
    Table(LootTableRef),
}

impl Default for DropSpec {
    fn default() -> Self {
        DropSpec::None
    }
}
