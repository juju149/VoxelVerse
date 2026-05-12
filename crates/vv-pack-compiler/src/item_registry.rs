//! Runtime item registry — compact `ItemId` indices into a flat table of
//! compiled item definitions.
//!
//! Items are sorted **alphabetically by key** during compilation so that IDs
//! are deterministic across reloads and packs applied in the same order.

use std::collections::HashMap;

// ─── ItemId ─────────────────────────────────────────────────────────────────

/// Compact, stable identifier for a compiled item.
///
/// IDs start at 0, are assigned alphabetically, and remain stable for a given
/// pack load order. The sentinel `NONE = u32::MAX` is reserved.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct ItemId(u32);

impl ItemId {
    pub const NONE: Self = Self(u32::MAX);

    pub fn raw(self) -> u32 {
        self.0
    }

    pub fn from_raw(id: u32) -> Self {
        Self(id)
    }
}

// ─── Compiled data model ─────────────────────────────────────────────────────

/// Maximum stack size an item slot can hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StackSize(pub u32);

/// Tool attributes needed for mining speed and tier checks.
#[derive(Clone, Debug)]
pub struct CompiledToolData {
    /// Tag keys this tool satisfies (e.g. `"core:tag/item/tool/pickaxe"`).
    pub tool_tag_keys: Vec<String>,
    pub tier: u32,
    pub mining_speed: f32,
    pub durability: u32,
}

/// Weapon attributes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompiledWeaponClass {
    Bow,
    Sword,
}

#[derive(Clone, Debug)]
pub struct CompiledWeaponData {
    pub class: CompiledWeaponClass,
    pub damage: f32,
    pub attack_speed: f32,
    pub durability: u32,
    /// Content key of the projectile entity (if any).
    pub projectile_key: Option<String>,
}

/// Food attributes.
#[derive(Clone, Copy, Debug)]
pub struct CompiledFoodData {
    pub nutrition: u32,
    pub saturation: f32,
    pub eat_seconds: f32,
}

/// Consumable attributes.
#[derive(Clone, Debug)]
pub struct CompiledConsumableData {
    pub effect_key: String,
    pub magnitude: f32,
    pub use_seconds: f32,
}

/// Crafting ingredient metadata.
#[derive(Clone, Debug, Default)]
pub struct CompiledIngredientData {
    /// Fuel ticks provided when used as furnace fuel.
    pub fuel_value: Option<u32>,
    /// Key of the item this smelts into (e.g. iron ore → iron ingot).
    pub smelts_to_key: Option<String>,
}

/// Discriminated gameplay role of an item.
#[derive(Clone, Debug)]
pub enum CompiledItemGameplay {
    /// When used, places the block with this family key.
    PlaceBlock { block_key: String },
    CraftingIngredient(CompiledIngredientData),
    Tool(CompiledToolData),
    Weapon(CompiledWeaponData),
    Food(CompiledFoodData),
    Consumable(CompiledConsumableData),
}

/// Visual representation of an item.
#[derive(Clone, Debug)]
pub struct CompiledItemVisual {
    /// Content key of the inventory icon texture.
    pub icon_key: String,
    /// Which model to show when dropped in the world.
    pub world_model: CompiledItemWorldModel,
    /// Model shown in the player's hand (optional).
    pub hand_model_key: Option<String>,
}

#[derive(Clone, Debug)]
pub enum CompiledItemWorldModel {
    None,
    /// Uses the block's mesh — content key of the block family.
    BlockItem(String),
    /// A .vox model — content key.
    Voxel(String),
}

/// Fully compiled item, ready for runtime use.
#[derive(Clone, Debug)]
pub struct CompiledItem {
    pub id: ItemId,
    /// Namespaced key (`namespace:item/path`).
    pub key: String,
    /// Human-readable name for UI.
    pub display_name: String,
    /// Category string (e.g. `"block"`, `"resource"`, `"tool"`, `"food"`).
    pub category: String,
    pub stack_size: StackSize,
    pub visual: CompiledItemVisual,
    pub gameplay: CompiledItemGameplay,
    /// All tag keys this item carries (resolved by `TagRegistry`).
    pub tag_keys: Vec<String>,
}

// ─── ItemRegistry ────────────────────────────────────────────────────────────

/// Runtime registry of all compiled items.
///
/// Indexed densely by `ItemId`. Lookup by content key via `lookup`.
#[derive(Debug)]
pub struct ItemRegistry {
    items: Vec<CompiledItem>,
    key_to_id: HashMap<String, ItemId>,
}

impl ItemRegistry {
    pub(crate) fn new(items: Vec<CompiledItem>) -> Self {
        let key_to_id = items
            .iter()
            .map(|i| (i.key.clone(), i.id))
            .collect::<HashMap<_, _>>();
        Self { items, key_to_id }
    }

    /// Returns `None` for unknown keys.
    pub fn lookup(&self, key: &str) -> Option<ItemId> {
        self.key_to_id.get(key).copied()
    }

    pub fn get(&self, id: ItemId) -> Option<&CompiledItem> {
        if id == ItemId::NONE {
            return None;
        }
        self.items.get(id.raw() as usize)
    }

    pub fn get_by_key(&self, key: &str) -> Option<&CompiledItem> {
        self.get(self.lookup(key)?)
    }

    pub fn items(&self) -> &[CompiledItem] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
