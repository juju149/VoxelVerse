pub mod content_key;
pub mod ids;
pub mod registry_table;
pub mod runtime_api;

pub mod block;
pub mod entity;
pub mod item;
pub mod loot;
pub mod placeable;
pub mod recipe;
pub mod settings;
pub mod tag;
pub mod worldgen;

pub use block::{
    BlockRegistry, CompiledBlock, CompiledBlockFace, CompiledBlockMining, CompiledBlockPhysics,
    CompiledBlockRender, CompiledBlockTextures, CompiledDrops, CompiledMaterialPhase,
    CompiledStylizedMaterial, CompiledTextureLayout, CompiledTextureResource, CompiledTintMode,
    CompiledVisualMaterialType, TextureRegistry,
};
pub use content_key::{ContentKey, ContentKeyParseError};
pub use entity::{CompiledEntity, EntityRegistry};
pub use ids::{
    BiomeId, BlockId, EntityId, FaunaId, FloraId, ItemId, LootTableId, OreId, PlaceableId,
    PlanetTypeId, RecipeId, StructureId, TagId, TextureId, WeatherId,
};
pub use item::{CompiledItem, CompiledItemKind, CompiledToolKind, ItemRegistry};
pub use loot::{CompiledLootEntry, CompiledLootPool, CompiledLootTable, LootTableRegistry};
pub use placeable::{CompiledPlaceable, PlaceableRegistry};
pub use recipe::{CompiledIngredient, CompiledRecipe, CompiledRecipePattern, RecipeRegistry};
pub use registry_table::RegistryTable;
pub use runtime_api::{
    BiomeSource, BiomeView, BlockContent, BlockContentView, BlockRenderSource, BlockRuntimeSource,
    BlockRuntimeView, FloraSource, FloraView, OreSource, OreView, PlanetTypeSource, PlanetTypeView,
    WorldContentView, WorldSettingsSource, WorldgenContentView, WorldgenSettingsSource,
};
pub use settings::CompiledWorldSettings;
pub use tag::{CompiledTag, TagDomain, TagRegistry, TaggedContent};
pub use worldgen::{
    BiomeRegistry, CompiledBiome, CompiledBiomeRelief, CompiledClimateCurves, CompiledClimateRange,
    CompiledClimateSampleRanges, CompiledClimateTags, CompiledDerivedTagRule, CompiledFauna,
    CompiledFloatRange, CompiledFlora, CompiledFloraFeature, CompiledFloraPlacement,
    CompiledIdealRange, CompiledOre, CompiledOreVein, CompiledPlanetType, CompiledStructure,
    CompiledSurfaceLayer, CompiledWeather, FaunaRegistry, FloraRegistry, OreRegistry,
    PlanetTypeRegistry, StructureRegistry, WeatherRegistry,
};

#[derive(Debug, Clone, Default)]
pub struct CompiledContent {
    pub world: CompiledWorldSettings,
    pub textures: TextureRegistry,
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
