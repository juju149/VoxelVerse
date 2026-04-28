use std::path::PathBuf;

use vv_schema::{
    block::BlockDef,
    entity::EntityDef,
    item::ItemDef,
    lang::LangFileDef,
    loot::LootTableDef,
    manifest::PackManifest,
    placeable::PlaceableDef,
    recipe::RecipeDef,
    settings::{balance::BalanceSettings, gameplay::GameplaySettings, world::WorldSettings},
    tag::TagDef,
    ui::UiThemeDef,
    worldgen::{
        biome::BiomeDef,
        climate::{ClimateTagsDef, ClimateTransitionsDef, GlobalClimateCurvesDef},
        fauna::FaunaDef,
        flora::FloraDef,
        noise::NoiseGraph,
        ore::OreDef,
        planet::PlanetTypeDef,
        structure::StructureDef,
        universe::UniverseDef,
        weather::WeatherDef,
    },
};

#[derive(Debug, Clone)]
pub struct RawDocument<T> {
    pub pack_namespace: String,
    pub source_path: PathBuf,
    pub relative_path: PathBuf,
    pub value: T,
}

#[derive(Debug, Clone)]
pub struct RawContentSet {
    pub manifest: PackManifest,
    pub pack_root: PathBuf,
    pub blocks: Vec<RawDocument<BlockDef>>,
    pub items: Vec<RawDocument<ItemDef>>,
    pub entities: Vec<RawDocument<EntityDef>>,
    pub placeables: Vec<RawDocument<PlaceableDef>>,
    pub recipes: Vec<RawDocument<RecipeDef>>,
    pub loot_tables: Vec<RawDocument<LootTableDef>>,
    pub tags: Vec<RawDocument<TagDef>>,
    pub lang: Vec<RawDocument<LangFileDef>>,
    pub gameplay_settings: Vec<RawDocument<GameplaySettings>>,
    pub balance_settings: Vec<RawDocument<BalanceSettings>>,
    pub world_settings: Vec<RawDocument<WorldSettings>>,
    pub ui_themes: Vec<RawDocument<UiThemeDef>>,
    pub universes: Vec<RawDocument<UniverseDef>>,
    pub climate_tags: Vec<RawDocument<ClimateTagsDef>>,
    pub climate_curves: Vec<RawDocument<GlobalClimateCurvesDef>>,
    pub climate_transitions: Vec<RawDocument<ClimateTransitionsDef>>,
    pub planet_types: Vec<RawDocument<PlanetTypeDef>>,
    pub biomes: Vec<RawDocument<BiomeDef>>,
    pub flora: Vec<RawDocument<FloraDef>>,
    pub fauna: Vec<RawDocument<FaunaDef>>,
    pub ores: Vec<RawDocument<OreDef>>,
    pub structures: Vec<RawDocument<StructureDef>>,
    pub weather: Vec<RawDocument<WeatherDef>>,
    pub noise: Vec<RawDocument<NoiseGraph>>,
}
