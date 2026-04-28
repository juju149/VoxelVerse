use std::collections::HashMap;

use vv_registry::{
    BiomeId, BlockId, ContentKey, EntityId, FaunaId, FloraId, ItemId, LootTableId, OreId,
    PlaceableId, PlanetTypeId, RecipeId, StructureId, TagId, WeatherId,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct ReferenceIndex {
    pub blocks: HashMap<ContentKey, BlockId>,
    pub items: HashMap<ContentKey, ItemId>,
    pub entities: HashMap<ContentKey, EntityId>,
    pub placeables: HashMap<ContentKey, PlaceableId>,
    pub recipes: HashMap<ContentKey, RecipeId>,
    pub loot_tables: HashMap<ContentKey, LootTableId>,
    pub tags: HashMap<ContentKey, TagId>,
    pub planet_types: HashMap<ContentKey, PlanetTypeId>,
    pub biomes: HashMap<ContentKey, BiomeId>,
    pub flora: HashMap<ContentKey, FloraId>,
    pub fauna: HashMap<ContentKey, FaunaId>,
    pub ores: HashMap<ContentKey, OreId>,
    pub structures: HashMap<ContentKey, StructureId>,
    pub weather: HashMap<ContentKey, WeatherId>,
}
