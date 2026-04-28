pub mod content_key;
pub mod ids;
pub mod registry_table;

pub mod block;
pub mod entity;
pub mod item;
pub mod loot;
pub mod placeable;
pub mod recipe;
pub mod tag;
pub mod worldgen;

pub use block::{
    BlockRegistry, CompiledBlock, CompiledBlockMining, CompiledBlockPhysics, CompiledBlockRender,
    CompiledDrops, CompiledMaterialPhase, CompiledTextureLayout,
};
pub use content_key::{ContentKey, ContentKeyParseError};
pub use entity::{CompiledEntity, EntityRegistry};
pub use ids::{
    BiomeId, BlockId, EntityId, FaunaId, FloraId, ItemId, LootTableId, OreId, PlaceableId,
    PlanetTypeId, RecipeId, StructureId, TagId, WeatherId,
};
pub use item::{CompiledItem, CompiledItemKind, ItemRegistry};
pub use loot::{CompiledLootEntry, CompiledLootPool, CompiledLootTable, LootTableRegistry};
pub use placeable::{CompiledPlaceable, PlaceableRegistry};
pub use recipe::{CompiledIngredient, CompiledRecipe, CompiledRecipePattern, RecipeRegistry};
pub use registry_table::RegistryTable;
pub use tag::{CompiledTag, TagDomain, TagRegistry, TaggedContent};
pub use worldgen::{
    BiomeRegistry, CompiledBiome, CompiledBiomeRelief, CompiledClimateCurves, CompiledClimateRange,
    CompiledClimateSampleRanges, CompiledClimateTags, CompiledDerivedTagRule, CompiledFauna,
    CompiledFloatRange, CompiledFlora, CompiledFloraFeature, CompiledFloraPlacement, CompiledIdealRange,
    CompiledOre, CompiledOreVein, CompiledPlanetType, CompiledStructure, CompiledSurfaceLayer,
    CompiledWeather, FaunaRegistry, FloraRegistry, OreRegistry, PlanetTypeRegistry,
    StructureRegistry, WeatherRegistry,
};

#[derive(Debug, Clone, Default)]
pub struct CompiledContent {
    pub blocks: BlockRegistry,
    pub items: ItemRegistry,
    pub entities: EntityRegistry,
    pub placeables: PlaceableRegistry,
    pub recipes: RecipeRegistry,
    pub loot_tables: LootTableRegistry,
    pub tags: TagRegistry,
    pub planet_types: PlanetTypeRegistry,
    pub biomes: BiomeRegistry,
    pub flora: FloraRegistry,
    pub fauna: FaunaRegistry,
    pub ores: OreRegistry,
    pub structures: StructureRegistry,
    pub weather: WeatherRegistry,
    pub default_planet_type: Option<PlanetTypeId>,
    pub climate_tags: CompiledClimateTags,
    pub climate_curves: CompiledClimateCurves,
}
