use std::{path::PathBuf, str::FromStr};

use vv_compiler::{compile_packs, CompileDiagnostic};
use vv_pack::load_packs_from_assets;
use vv_registry::{
    BiomeSource, BlockRenderSource, BlockRuntimeSource, CompiledIngredient, ContentKey,
    PlanetTypeSource, TaggedContent, WorldSettingsSource, WorldgenSettingsSource,
};

fn load_core() -> vv_pack::PackLoadOrder {
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    load_packs_from_assets(&assets).expect("voxelverse_core should load")
}

#[test]
fn compiles_voxelverse_core_into_runtime_registries() {
    let load_order = load_core();
    let content = compile_packs(&load_order).expect("voxelverse_core should compile");

    let stone_key = ContentKey::from_str("voxelverse:stone").unwrap();
    let stone_block = content.blocks.id(&stone_key).expect("stone block id");
    let cobblestone_key = ContentKey::from_str("voxelverse:cobblestone").unwrap();
    let cobblestone_item = content
        .items
        .id(&cobblestone_key)
        .expect("cobblestone item id");

    let pickaxe_key = ContentKey::from_str("voxelverse:pickaxe_stone").unwrap();
    let pickaxe_item = content.items.id(&pickaxe_key).expect("pickaxe item id");
    let recipe = content
        .recipes
        .get(content.recipes.id(&pickaxe_key).expect("pickaxe recipe id"))
        .expect("pickaxe recipe");
    assert_eq!(recipe.result_item, pickaxe_item);
    assert!(matches!(
        recipe.ingredients.as_slice(),
        [CompiledIngredient::Item { item, count: 3 }, CompiledIngredient::Item { .. }]
            if *item == cobblestone_item
    ));

    let meadow_key = ContentKey::from_str("voxelverse:meadow").unwrap();
    let meadow = content
        .biomes
        .get(content.biomes.id(&meadow_key).expect("meadow biome id"))
        .expect("meadow biome");
    let grass_block = content
        .blocks
        .id(&ContentKey::from_str("voxelverse:grass").unwrap())
        .expect("grass block id");
    assert_eq!(meadow.surface_layers[0].block, grass_block);

    let structure_key = ContentKey::from_str("voxelverse:waystone_circle").unwrap();
    let structure = content
        .structures
        .get(content.structures.id(&structure_key).expect("structure id"))
        .expect("structure");
    assert_eq!(structure.loot_table, None);

    let weather_key = ContentKey::from_str("voxelverse:clear").unwrap();
    assert!(content.weather.id(&weather_key).is_some());

    let solid_key = ContentKey::from_str("voxelverse:solid").unwrap();
    let solid = content
        .tags
        .get(content.tags.id(&solid_key).expect("solid tag id"))
        .expect("solid tag");
    assert!(solid.values.contains(&TaggedContent::Block(stone_block)));
}

#[test]
fn exposes_narrow_runtime_content_views() {
    let load_order = load_core();
    let content = compile_packs(&load_order).expect("voxelverse_core should compile");

    let stone_key = ContentKey::from_str("voxelverse:stone").unwrap();
    let stone_block = content.blocks.id(&stone_key).expect("stone block id");

    let blocks = content.block_content();
    let stone_runtime = blocks
        .block_runtime(stone_block)
        .expect("stone runtime block view");
    assert_eq!(stone_runtime.key, &stone_key);
    assert!(stone_runtime.physics.density > 0.0);
    assert_eq!(
        blocks
            .block_render(stone_block)
            .expect("stone render data")
            .emits_light,
        0
    );

    let world = content.world_content();
    assert_eq!(world.world_settings().chunk_size, 32);
    assert_eq!(world.world_settings().voxel_size_m, 0.05);

    let worldgen = content.worldgen_content();
    let default_planet = worldgen
        .default_planet_type()
        .expect("default planet type id");
    assert_eq!(
        worldgen
            .planet_type(default_planet)
            .expect("default planet type")
            .key,
        &ContentKey::from_str("voxelverse:temperate").unwrap()
    );
    assert_eq!(worldgen.biomes().count(), 4);
    let meadow_id = content
        .biomes
        .id(&ContentKey::from_str("voxelverse:meadow").unwrap())
        .expect("meadow biome id");
    assert_eq!(
        worldgen
            .biome(meadow_id)
            .expect("meadow biome")
            .data
            .surface_layers[0]
            .block,
        content
            .blocks
            .id(&ContentKey::from_str("voxelverse:grass").unwrap())
            .expect("grass block")
    );
    assert!(worldgen.climate_curves().temperature_noise_scale > 0.0);
}

#[test]
fn reports_missing_references_with_owner_and_path() {
    let mut load_order = load_core();
    let mut packs = load_order.into_packs();
    packs[0].content.recipes[0].value.result.item.0 = "voxelverse:missing_item".to_owned();
    load_order = vv_pack::PackLoadOrder::new(packs);

    let error = compile_packs(&load_order).expect_err("missing item should fail");
    assert!(error.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic,
            CompileDiagnostic::MissingReference {
                owner,
                reference,
                ..
            } if owner == "recipe" && reference == "voxelverse:missing_item"
        )
    }));
}
