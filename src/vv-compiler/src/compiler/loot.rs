use super::helpers::*;
use super::prelude::*;

use super::*;

impl ContentCompiler {
    pub(super) fn compile_loot_table(
        &mut self,
        doc: &RawDocument<LootTableDef>,
        index: &ReferenceIndex,
    ) -> CompiledLootTable {
        let pools = doc
            .value
            .pools
            .iter()
            .map(|pool| CompiledLootPool {
                rolls: pool.rolls,
                bonus_rolls: pool.bonus_rolls,
                entries: pool
                    .entries
                    .iter()
                    .filter_map(|entry| {
                        self.resolve_item("loot_table", doc, &entry.item, index)
                            .map(|item| CompiledLootEntry {
                                item,
                                weight: entry.weight,
                                count_min: entry.count.min,
                                count_max: entry.count.max,
                            })
                    })
                    .collect(),
            })
            .collect();
        CompiledLootTable { pools }
    }

    pub(super) fn compile_drop_spec<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        drops: &DropSpec,
        index: &ReferenceIndex,
    ) -> CompiledDrops {
        match drops {
            DropSpec::None => CompiledDrops::None,
            DropSpec::Table(table) => self
                .resolve_loot_table(owner, doc, table, index)
                .map(CompiledDrops::Table)
                .unwrap_or(CompiledDrops::None),
            DropSpec::Inline(pools) => CompiledDrops::Inline(
                pools
                    .iter()
                    .map(|pool| CompiledLootPool {
                        rolls: pool.rolls,
                        bonus_rolls: pool.bonus_rolls,
                        entries: pool
                            .entries
                            .iter()
                            .filter_map(|entry| {
                                self.resolve_item(owner, doc, &entry.item, index)
                                    .map(|item| CompiledLootEntry {
                                        item,
                                        weight: entry.weight,
                                        count_min: entry.count.min,
                                        count_max: entry.count.max,
                                    })
                            })
                            .collect(),
                    })
                    .collect(),
            ),
        }
    }

    pub(super) fn validate_drop_spec<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        drops: &DropSpec,
        index: &ReferenceIndex,
    ) {
        match drops {
            DropSpec::None => {}
            DropSpec::Inline(pools) => {
                for pool in pools {
                    for entry in &pool.entries {
                        self.resolve_item(owner, doc, &entry.item, index);
                    }
                }
            }
            DropSpec::Table(table) => {
                self.resolve_loot_table(owner, doc, table, index);
            }
        }
    }
}
