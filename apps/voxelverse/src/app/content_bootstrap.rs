use std::collections::HashMap;
use std::fmt;
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

/// Error returned when the core content pack fails to load.
///
/// Each variant corresponds to one compilation stage. The error contains a
/// human-readable message suitable for display in a pre-game error screen.
#[derive(Debug)]
pub enum LoadContentError {
    PackLoad(String),
    ObjectCompilation(Vec<String>),
    ProceduralLoad(String),
    ProceduralCompilation(Vec<String>),
    TextureLoad(Vec<String>),
    NoPlanet,
}

impl fmt::Display for LoadContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PackLoad(msg) => write!(f, "Pack load failed: {msg}"),
            Self::ObjectCompilation(errs) => {
                write!(f, "Object compilation failed:\n{}", errs.join("\n"))
            }
            Self::ProceduralLoad(msg) => write!(f, "Procedural pack load failed: {msg}"),
            Self::ProceduralCompilation(errs) => {
                write!(f, "Procedural compilation failed:\n{}", errs.join("\n"))
            }
            Self::TextureLoad(errs) => write!(f, "Texture load failed:\n{}", errs.join("\n")),
            Self::NoPlanet => write!(f, "Procedural pack defines no planet"),
        }
    }
}

pub fn asset_pack_root() -> PathBuf {
    let cwd_path = PathBuf::from("assets/packs");
    if cwd_path.exists() {
        cwd_path
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs")
    }
}

pub fn load_core_content() -> Result<LoadedCoreContent, LoadContentError> {
    let pack_root = asset_pack_root();
    let core_pack_dir = pack_root.join("core");
    let pack = PackLoader::load_from_dir(&core_pack_dir)
        .map_err(|e| LoadContentError::PackLoad(format!("{}: {e}", core_pack_dir.display())))?;

    // ── Unified object compilation (.object.ron format) ─────────────────────
    let compiled = compile_objects(pack.objects).map_err(|errs| {
        LoadContentError::ObjectCompilation(errs.iter().map(|e| e.to_string()).collect())
    })?;

    let compiled_blocks = compiled.blocks;
    let compiled_items = compiled.items;
    let compiled_tags = compiled.tags;
    let compiled_loot = compiled.loot;
    let compiled_recipes = compiled.recipes;

    let procedural_pack = PackLoader::load_procedural_from_dir(&core_pack_dir).map_err(|e| {
        LoadContentError::ProceduralLoad(format!("{}/worldgen: {e}", core_pack_dir.display()))
    })?;
    let procedural = ContentCompiler::compile_procedural(procedural_pack, &compiled_blocks)
        .map_err(|errs| {
            LoadContentError::ProceduralCompilation(errs.iter().map(|e| e.to_string()).collect())
        })?;
    let procedural_planet_index = 0;
    let planet = procedural
        .first_planet()
        .ok_or(LoadContentError::NoPlanet)?
        .base
        .clone();

    let texture_registry = TextureRegistry::load(&pack_root, &compiled_blocks).map_err(|errs| {
        LoadContentError::TextureLoad(errs.iter().map(|e| e.to_string()).collect())
    })?;
    let terrain_visuals = TerrainVisualPalette::from_textures(&compiled_blocks, &texture_registry);

    // Build vox asset path index from the registry (content_ref -> relative path).
    // Only explicit entries from voxel_assets.ron are accepted — no auto-scan fallback.
    let vox_asset_paths: HashMap<String, String> = pack
        .voxel_assets
        .iter()
        .flat_map(|reg| reg.assets.iter())
        .map(|def| (def.id.0.clone(), def.path.clone()))
        .collect();

    // Collect only the model keys actually referenced by scatter variant defs.
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

    Ok(LoadedCoreContent {
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
    })
}
