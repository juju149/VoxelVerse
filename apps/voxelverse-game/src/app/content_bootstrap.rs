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

    let compiled_blocks = ContentCompiler::compile_blocks(pack.blocks, pack.materials)
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

    println!(
        "Loaded {} blocks, {} procedural biomes, {} material layers, planet '{}' ({}) from pack 'core'.",
        compiled_blocks.block_count(),
        procedural.biomes.len(),
        texture_registry.materials().len(),
        planet.key,
        planet.display_name,
    );

    LoadedCoreContent {
        blocks: Arc::new(compiled_blocks),
        procedural: Arc::new(procedural),
        procedural_planet_index,
        textures: Arc::new(texture_registry),
        planet,
    }
}
