use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use vv_pack_compiler::{
    compile_objects, BlockRegistry, CompiledPlanet, ContentCompiler, ItemRegistry, LootRegistry,
    ProceduralRegistry, RecipeRegistry, TagRegistry, TextureRegistry,
};
use vv_pack_loader::PackLoader;

use vv_world::TerrainVisualPalette;

pub struct LoadedCoreContent {
    pub blocks: Arc<BlockRegistry>,
    pub items: Arc<ItemRegistry>,
    pub loot: Arc<LootRegistry>,
    pub tags: Arc<TagRegistry>,
    pub recipes: Arc<RecipeRegistry>,
    pub procedural: Arc<ProceduralRegistry>,
    pub procedural_planet_index: usize,
    pub textures: Arc<TextureRegistry>,
    pub terrain_visuals: Arc<TerrainVisualPalette>,
    pub planet: CompiledPlanet,
    /// Maps content ref strings to file paths relative to the core pack directory.
    pub vox_asset_paths: HashMap<String, String>,
    /// Only the model keys referenced by scatter variant defs.
    pub needed_vox_keys: std::collections::HashSet<String>,
    /// Absolute path to the core pack directory.
    pub core_pack_dir: PathBuf,
}

pub fn asset_pack_root() -> PathBuf {
    let cwd_path = PathBuf::from("assets/packs");
    if cwd_path.exists() {
        cwd_path
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs")
    }
}

pub fn load_core_content() -> LoadedCoreContent {
    let pack_root = asset_pack_root();
    let core_pack_dir = pack_root.join("core");
    let pack = PackLoader::load_from_dir(&core_pack_dir).unwrap_or_else(|e| {
        panic!("Failed to load {}: {}", core_pack_dir.display(), e);
    });

    // ── Unified object compilation (.object.ron format) ─────────────────────
    let compiled = compile_objects(pack.objects).unwrap_or_else(|errors| {
        for e in &errors {
            eprintln!("[object compilation error] {}", e);
        }
        panic!("Object compilation failed; see errors above.");
    });
    let compiled_blocks = compiled.blocks;
    let compiled_items = compiled.items;
    let compiled_tags = compiled.tags;
    let compiled_loot = compiled.loot;
    let compiled_recipes = compiled.recipes;

    let procedural_pack =
        PackLoader::load_procedural_from_dir(&core_pack_dir).unwrap_or_else(|e| {
            panic!("Failed to load {}/worldgen: {}", core_pack_dir.display(), e);
        });
    let procedural = ContentCompiler::compile_procedural(procedural_pack, &compiled_blocks)
        .unwrap_or_else(|errors| {
            for e in &errors {
                eprintln!("[procedural content error] {}", e);
            }
            panic!("Procedural compilation failed; see errors above.");
        });
    let procedural_planet_index = 0;
    let planet = procedural
        .first_planet()
        .expect("compile_procedural guarantees at least one planet")
        .base
        .clone();

    let texture_registry =
        TextureRegistry::load(&pack_root, &compiled_blocks).unwrap_or_else(|errors| {
            for e in &errors {
                eprintln!("[texture error] {}", e);
            }
            panic!("Texture loading failed; see errors above.");
        });
    let terrain_visuals = TerrainVisualPalette::from_textures(&compiled_blocks, &texture_registry);

    // Build vox asset path index from the registry (content_ref -> relative path).
    let vox_asset_paths: HashMap<String, String> = pack
        .voxel_assets
        .iter()
        .flat_map(|reg| reg.assets.iter())
        .map(|def| (def.id.0.clone(), def.path.clone()))
        .collect();

    // Collect only the model keys actually referenced by scatter variant defs so
    // we don't load thousands of character/entity .vox models at startup.
    let needed_vox_keys: std::collections::HashSet<String> = procedural
        .vox_prop_scatters
        .iter()
        .flat_map(|scatter| scatter.variants.iter())
        .map(|v| v.model_key.clone())
        .collect();

    println!(
        "Loaded {} blocks, {} items, {} loot tables, {} tags, {} recipes, \
         {} biomes, {} material layers, {} vox models needed, planet '{}' ({}) from pack 'core'.",
        compiled_blocks.block_count(),
        compiled_items.len(),
        compiled_loot.len(),
        compiled_tags.len(),
        compiled_recipes.len(),
        procedural.biomes.len(),
        texture_registry.materials().len(),
        needed_vox_keys.len(),
        planet.key,
        planet.display_name,
    );

    LoadedCoreContent {
        blocks: Arc::new(compiled_blocks),
        items: Arc::new(compiled_items),
        loot: Arc::new(compiled_loot),
        tags: Arc::new(compiled_tags),
        recipes: Arc::new(compiled_recipes),
        procedural: Arc::new(procedural),
        procedural_planet_index,
        textures: Arc::new(texture_registry),
        terrain_visuals: Arc::new(terrain_visuals),
        planet,
        vox_asset_paths,
        needed_vox_keys,
        core_pack_dir,
    }
}
