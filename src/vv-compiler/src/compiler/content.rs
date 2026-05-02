use super::helpers::*;
use super::prelude::*;

use super::*;

impl ContentCompiler {
    pub(super) fn compile_item(
        &mut self,
        doc: &RawDocument<vv_schema::item::ItemDef>,
        index: &ReferenceIndex,
    ) -> CompiledItem {
        let kind = match &doc.value.kind {
            ItemKind::Block { block } => CompiledItemKind::Block {
                block: self
                    .resolve_block("item", doc, block, index)
                    .unwrap_or(BlockId::new(0)),
            },
            ItemKind::Resource => CompiledItemKind::Resource,
            ItemKind::Tool {
                tool_type,
                tool_tier,
                durability,
                mining_speed,
                attack_damage,
                ..
            } => CompiledItemKind::Tool {
                tool_type: compiled_tool_kind(*tool_type),
                tool_tier: *tool_tier,
                durability: *durability,
                mining_speed: *mining_speed,
                attack_damage: *attack_damage,
            },
            ItemKind::Armor { .. } => CompiledItemKind::Armor,
            ItemKind::Food { .. } => CompiledItemKind::Food,
            ItemKind::Placeable { placeable } => CompiledItemKind::Placeable {
                placeable: self
                    .resolve_placeable("item", doc, placeable, index)
                    .unwrap_or(PlaceableId::new(0)),
            },
        };

        CompiledItem {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            stack_max: doc.value.stack_max,
            tags: self.resolve_tags("item", doc, &doc.value.tags, index),
            kind,
        }
    }

    pub(super) fn compile_entity(
        &mut self,
        doc: &RawDocument<vv_schema::entity::EntityDef>,
        index: &ReferenceIndex,
    ) -> CompiledEntity {
        self.validate_drop_spec("entity", doc, &doc.value.drops, index);
        CompiledEntity {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            tags: self.resolve_tags("entity", doc, &doc.value.tags, index),
            health: doc.value.health,
            light_level: doc.value.light_level,
        }
    }

    pub(super) fn compile_placeable(
        &mut self,
        doc: &RawDocument<vv_schema::placeable::PlaceableDef>,
        index: &ReferenceIndex,
    ) -> CompiledPlaceable {
        self.validate_drop_spec("placeable", doc, &doc.value.drops, index);
        CompiledPlaceable {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            tags: self.resolve_tags("placeable", doc, &doc.value.tags, index),
            light_level: doc.value.light_level,
        }
    }
}
