use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use vv_pack_compiler::{
    BlockRegistry, CompiledPlanet, ContentCompiler, ProceduralRegistry, TextureRegistry,
};
use vv_pack_loader::PackLoader;

pub struct LoadedCoreContent {
    pub blocks: Arc<BlockRegistry>,
    pub procedural: Arc<ProceduralRegistry>,
    pub procedural_planet_index: usize,
    pub textures: Arc<TextureRegistry>,
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

    let content_index = vv_pack_compiler::ContentIndex::build(&pack);
    let compiled_models =
        ContentCompiler::compile_block_models(pack.block_models).unwrap_or_else(|errors| {
            for e in &errors {
                eprintln!("[content error] {}", e);
            }
            panic!("Block model compilation failed; see errors above.");
        });
    let compiled_blocks =
        ContentCompiler::compile_blocks(pack.blocks, pack.materials, compiled_models, &content_index)
            .unwrap_or_else(|errors| {
                for e in &errors {
                    eprintln!("[content error] {}", e);
                }
                panic!("Block compilation failed; see errors above.");
            });

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
        "Loaded {} blocks, {} procedural biomes, {} material layers, {} vox models needed, planet '{}' ({}) from pack 'core'.",
        compiled_blocks.block_count(),
        procedural.biomes.len(),
        texture_registry.materials().len(),
        needed_vox_keys.len(),
        planet.key,
        planet.display_name,
    );

    LoadedCoreContent {
        blocks: Arc::new(compiled_blocks),
        procedural: Arc::new(procedural),
        procedural_planet_index,
        textures: Arc::new(texture_registry),
        planet,
        vox_asset_paths,
        needed_vox_keys,
        core_pack_dir,
    }
}
