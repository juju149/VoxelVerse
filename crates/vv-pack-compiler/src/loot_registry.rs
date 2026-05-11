//! Runtime loot table registry — resolves block drops to item stacks.
//!
//! Loot tables are compiled after `ItemRegistry` so that every `item`
//! reference in a loot entry resolves to a stable `ItemId`.

use crate::item_registry::ItemId;
use std::collections::HashMap;
use vv_content_schema::RawLootTableDef;

// ─── LootTableId ─────────────────────────────────────────────────────────────

/// Compact, stable identifier for a compiled loot table.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct LootTableId(u32);

impl LootTableId {
    pub fn raw(self) -> u32 {
        self.0
    }
}

// ─── Compiled data model ─────────────────────────────────────────────────────

/// One possible item drop in a loot table roll.
#[derive(Clone, Debug)]
pub struct CompiledLootEntry {
    pub item_id: ItemId,
    /// `(min, max)` inclusive count range.
    pub count_min: u32,
    pub count_max: u32,
    /// Drop probability in [0.0, 1.0].
    pub chance: f32,
}

/// A compiled loot table ready for runtime use.
#[derive(Clone, Debug)]
pub struct CompiledLootTable {
    pub id: LootTableId,
    /// Namespaced key (`namespace:loot/path`).
    pub key: String,
    /// How many independent rolls to perform.
    pub rolls: u32,
    pub entries: Vec<CompiledLootEntry>,
}

impl CompiledLootTable {
    /// Roll this table using the provided RNG callback.
    /// `rng()` must return a value in [0.0, 1.0).
    /// Returns a list of `(ItemId, count)` drops.
    pub fn roll<R: FnMut() -> f32>(&self, mut rng: R) -> Vec<(ItemId, u32)> {
        let mut drops: Vec<(ItemId, u32)> = Vec::new();
        for _ in 0..self.rolls {
            for entry in &self.entries {
                if rng() < entry.chance {
                    let range = entry.count_max.saturating_sub(entry.count_min);
                    let count = if range == 0 {
                        entry.count_min
                    } else {
                        entry.count_min + (rng() * (range + 1) as f32) as u32
                    };
                    if count > 0 {
                        // Merge identical items from multiple entries.
                        if let Some(existing) = drops.iter_mut().find(|(id, _)| *id == entry.item_id) {
                            existing.1 = existing.1.saturating_add(count);
                        } else {
                            drops.push((entry.item_id, count));
                        }
                    }
                }
            }
        }
        drops
    }
}

// ─── LootRegistry ────────────────────────────────────────────────────────────

/// Runtime registry of all compiled loot tables.
#[derive(Debug, Default)]
pub struct LootRegistry {
    tables: Vec<CompiledLootTable>,
    key_to_id: HashMap<String, LootTableId>,
}

impl LootRegistry {
    pub(crate) fn new(tables: Vec<CompiledLootTable>) -> Self {
        let key_to_id = tables
            .iter()
            .map(|t| (t.key.clone(), t.id))
            .collect::<HashMap<_, _>>();
        Self { tables, key_to_id }
    }

    pub fn lookup(&self, key: &str) -> Option<LootTableId> {
        self.key_to_id.get(key).copied()
    }

    pub fn get(&self, id: LootTableId) -> Option<&CompiledLootTable> {
        self.tables.get(id.raw() as usize)
    }

    pub fn get_by_key(&self, key: &str) -> Option<&CompiledLootTable> {
        self.get(self.lookup(key)?)
    }

    pub fn tables(&self) -> &[CompiledLootTable] {
        &self.tables
    }

    pub fn len(&self) -> usize {
        self.tables.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }
}

// ─── Compilation ─────────────────────────────────────────────────────────────

/// Compile raw loot table definitions into a `LootRegistry`.
///
/// Every `item` reference is resolved against `item_registry`. Unknown items
/// produce errors rather than silent fallbacks.
pub fn compile_loot_tables(
    mut raw: Vec<(String, RawLootTableDef)>,
    item_registry: &crate::ItemRegistry,
) -> Result<LootRegistry, Vec<String>> {
    raw.sort_by(|(a, _), (b, _)| a.cmp(b));

    let mut errors = Vec::new();
    let mut tables: Vec<CompiledLootTable> = Vec::with_capacity(raw.len());

    for (idx, (key, def)) in raw.into_iter().enumerate() {
        if def.rolls == 0 {
            errors.push(format!("loot table '{}': rolls must be ≥ 1", key));
            continue;
        }

        let mut entries = Vec::with_capacity(def.entries.len());
        for entry in def.entries {
            let item_key = &entry.item.0;
            match item_registry.lookup(item_key) {
                Some(item_id) => {
                    let (count_min, count_max) = entry.count;
                    if count_min > count_max {
                        errors.push(format!(
                            "loot table '{}': entry '{}' has count_min {} > count_max {}",
                            key, item_key, count_min, count_max
                        ));
                        continue;
                    }
                    let chance = entry.chance.clamp(0.0, 1.0);
                    entries.push(CompiledLootEntry {
                        item_id,
                        count_min,
                        count_max,
                        chance,
                    });
                }
                None => {
                    errors.push(format!(
                        "loot table '{}': unknown item ref '{}'",
                        key, item_key
                    ));
                }
            }
        }

        tables.push(CompiledLootTable {
            id: LootTableId(idx as u32),
            key,
            rolls: def.rolls,
            entries,
        });
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(LootRegistry::new(tables))
}
