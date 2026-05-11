//! Runtime item registry — compact `ItemId` indices into a flat table of
//! compiled item definitions.
//!
//! Items are sorted **alphabetically by key** during compilation so that IDs
//! are deterministic across reloads and packs applied in the same order.

use std::collections::HashMap;
use vv_content_schema::{
    RawConsumableDef, RawFoodDef, RawItemDef, RawItemGameplayDef, RawItemWorldModel, RawToolDef,
    RawWeaponClass, RawWeaponDef,
};

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

    pub(crate) fn from_raw(id: u32) -> Self {
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

// ─── Compilation helpers ─────────────────────────────────────────────────────

/// Compile a sorted list of raw item definitions into an `ItemRegistry`.
/// Errors are collected and returned all at once.
pub fn compile_items(mut raw: Vec<(String, RawItemDef)>) -> Result<ItemRegistry, Vec<String>> {
    raw.sort_by(|(a, _), (b, _)| a.cmp(b));

    let mut errors = Vec::new();
    let mut items = Vec::with_capacity(raw.len());

    for (idx, (key, def)) in raw.into_iter().enumerate() {
        let visual = compile_visual(&key, def.visual, &mut errors);
        let gameplay = compile_gameplay(&key, def.gameplay, &mut errors);

        items.push(CompiledItem {
            id: ItemId::from_raw(idx as u32),
            key,
            display_name: def.display_name,
            category: def.category,
            stack_size: StackSize(def.stack_size),
            visual,
            gameplay,
            tag_keys: def.tags.into_iter().map(|r| r.0).collect(),
        });
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(ItemRegistry::new(items))
}

fn compile_visual(
    key: &str,
    raw: vv_content_schema::RawItemVisualDef,
    _errors: &mut Vec<String>,
) -> CompiledItemVisual {
    let world_model = match raw.world_model {
        RawItemWorldModel::None => CompiledItemWorldModel::None,
        RawItemWorldModel::BlockItem(r) => CompiledItemWorldModel::BlockItem(r.0),
        RawItemWorldModel::Voxel(r) => CompiledItemWorldModel::Voxel(r.0),
    };
    CompiledItemVisual {
        icon_key: raw.inventory_icon.0,
        world_model,
        hand_model_key: raw.hand_model.map(|r| r.0),
    }
}

fn compile_gameplay(
    key: &str,
    raw: RawItemGameplayDef,
    errors: &mut Vec<String>,
) -> CompiledItemGameplay {
    match raw {
        RawItemGameplayDef::PlaceBlock(block_ref) => CompiledItemGameplay::PlaceBlock {
            block_key: block_ref.0,
        },
        RawItemGameplayDef::CraftingIngredient(raw_ci) => {
            CompiledItemGameplay::CraftingIngredient(CompiledIngredientData {
                fuel_value: raw_ci.fuel_value,
                smelts_to_key: raw_ci.smelts_to.map(|r| r.0),
            })
        }
        RawItemGameplayDef::Tool(raw_tool) => {
            if raw_tool.mining_speed <= 0.0 {
                errors.push(format!(
                    "item '{}': tool mining_speed must be > 0 (got {})",
                    key, raw_tool.mining_speed
                ));
            }
            CompiledItemGameplay::Tool(CompiledToolData {
                tool_tag_keys: raw_tool.tool_tags.into_iter().map(|r| r.0).collect(),
                tier: raw_tool.tier,
                mining_speed: raw_tool.mining_speed,
                durability: raw_tool.durability,
            })
        }
        RawItemGameplayDef::Weapon(raw_w) => {
            let class = match raw_w.class {
                RawWeaponClass::Bow => CompiledWeaponClass::Bow,
                RawWeaponClass::Sword => CompiledWeaponClass::Sword,
            };
            CompiledItemGameplay::Weapon(CompiledWeaponData {
                class,
                damage: raw_w.damage,
                attack_speed: raw_w.attack_speed,
                durability: raw_w.durability,
                projectile_key: raw_w.projectile.map(|r| r.0),
            })
        }
        RawItemGameplayDef::Food(raw_f) => {
            CompiledItemGameplay::Food(CompiledFoodData {
                nutrition: raw_f.nutrition,
                saturation: raw_f.saturation,
                eat_seconds: raw_f.eat_seconds,
            })
        }
        RawItemGameplayDef::Consumable(raw_c) => {
            CompiledItemGameplay::Consumable(CompiledConsumableData {
                effect_key: raw_c.effect.0,
                magnitude: raw_c.magnitude,
                use_seconds: raw_c.use_seconds,
            })
        }
    }
}
