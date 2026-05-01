use std::{path::PathBuf, str::FromStr};

use vv_compiler::{compile_packs, CompileDiagnostic};
use vv_pack::load_packs_from_assets;
use vv_registry::{
    BiomeSource, BlockRenderSource, BlockRuntimeSource, CompiledIngredient, ContentKey,
    PlanetTypeSource, TaggedContent, WorldSettingsSource, WorldgenSettingsSource,
};
use vv_schema::{
    block::{BlockDef, LegacyBlockColor},
    common::HexColor,
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
    let stone_render = blocks.block_render(stone_block).expect("stone render data");
    assert!(blocks.block_visual(stone_render.visual_id).is_some());
    assert!(!blocks.block_visual_palette().is_empty());

    let world = content.world_content();
    assert_eq!(world.world_settings().chunk_size, 32);
    assert_eq!(world.world_settings().voxel_size_m, 0.5);

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

#[test]
fn parses_raw_block_render_def_with_procedural_material() {
    let source = r##"(
        render: (
            material: "terrain_layered",
            base_color: "#6F4A2B",
            palette: ["#8A5A32", "#4D321F"],
            roughness: 0.85,
            metallic: 0.0,
            alpha: 1.0,
            bevel: 0.035,
            normal_strength: 0.45,
            variation: (
                per_voxel_tint: 0.12,
                per_face_tint: 0.08,
                macro_noise_scale: 0.75,
                macro_noise_strength: 0.18,
                micro_noise_scale: 8.0,
                micro_noise_strength: 0.08,
                edge_darkening: 0.12,
                biome_tint_strength: 0.25,
            ),
            faces: (
                top: Some((color_bias: Some("#78A83A"), detail_bias: ["grass_blades"])),
            ),
            details: [
                (
                    kind: "pebbles",
                    density: 0.08,
                    color: Some("#9A8F7A"),
                    min_size: 0.015,
                    max_size: 0.055,
                    slope_bias: 0.2,
                ),
            ],
        ),
    )"##;
    let block: BlockDef = ron::from_str(source).expect("new block render schema should parse");
    assert_eq!(block.render.palette.len(), 2);
    assert_eq!(block.render.details[0].kind, "pebbles");
}

#[test]
fn reports_invalid_visual_range() {
    let mut load_order = load_core();
    let mut packs = load_order.into_packs();
    packs[0].content.blocks[0].value.render.roughness = 2.0;
    load_order = vv_pack::PackLoadOrder::new(packs);

    let error = compile_packs(&load_order).expect_err("invalid roughness should fail");
    assert!(error.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic,
            CompileDiagnostic::InvalidValue { field, .. } if field == "render.roughness"
        )
    }));
}

#[test]
fn reports_invalid_hex_color() {
    let mut load_order = load_core();
    let mut packs = load_order.into_packs();
    packs[0].content.blocks[0].value.render.color = LegacyBlockColor::None;
    packs[0].content.blocks[0].value.render.base_color = HexColor("#GG00FF".to_owned());
    load_order = vv_pack::PackLoadOrder::new(packs);

    let error = compile_packs(&load_order).expect_err("invalid hex color should fail");
    assert!(error.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic,
            CompileDiagnostic::InvalidValue { field, .. } if field == "render.base_color"
        )
    }));
}

#[test]
fn compiles_block_visuals_into_runtime_ids_and_palettes() {
    let load_order = load_core();
    let content = compile_packs(&load_order).expect("voxelverse_core should compile");
    let grass = content
        .blocks
        .id(&ContentKey::from_str("voxelverse:grass").unwrap())
        .expect("grass block id");
    let blocks = content.block_content();
    let render = blocks.block_render(grass).unwrap();
    let visual = blocks
        .block_visual(render.visual_id)
        .expect("grass visual runtime");

    assert_eq!(render.visual_id.raw(), grass.raw());
    assert!(visual.palette_len >= 1);
    assert!(!content.block_visual_palettes.is_empty());
    assert!(render.material.variation.per_voxel_tint > 0.0);
}

#[test]
fn voxel_variation_seed_is_deterministic() {
    let voxel = vv_core::BlockId {
        face: 2,
        layer: 7,
        u: 11,
        v: 13,
    };
    let block = vv_registry::BlockId::new(3);
    let a = vv_mesh::MeshGen::stable_variation_seed(voxel, block, 4, 42);
    let b = vv_mesh::MeshGen::stable_variation_seed(voxel, block, 4, 42);
    let c = vv_mesh::MeshGen::stable_variation_seed(voxel, block, 5, 42);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn world_runtime_stores_compact_content_block_ids() {
    let geometry = vv_planet::PlanetGeometry::with_resolution(8.0, 0.5, 8);
    let load_order = load_core();
    let content = compile_packs(&load_order).expect("voxelverse_core should compile");
    let terrain = vv_world_gen::PlanetTerrain::generate_for_geometry(
        geometry,
        &vv_config::WorldGenConfig::default(),
        &content.worldgen_content(),
    )
    .expect("terrain should generate");
    let mut planet = vv_world_runtime::PlanetData::new(geometry, terrain, 0);
    let block = content
        .blocks
        .id(&ContentKey::from_str("voxelverse:stone").unwrap())
        .unwrap();
    let voxel = vv_core::BlockId {
        face: 0,
        layer: 2,
        u: 2,
        v: 2,
    };
    planet.add_block(voxel, block);
    assert_eq!(planet.block_at(voxel), Some(block));
}
