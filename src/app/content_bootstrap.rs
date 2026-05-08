use crate::content::{compile::ContentCompiler, pack::PackLoader, BlockRegistry};
use crate::content::{CompiledPlanet, ProceduralRegistry, TextureRegistry};
use std::path::Path;
use std::sync::Arc;

pub struct LoadedCoreContent {
    pub blocks: Arc<BlockRegistry>,
    pub procedural: Arc<ProceduralRegistry>,
    pub procedural_planet_index: usize,
    pub textures: Arc<TextureRegistry>,
    pub planet: CompiledPlanet,
}

pub fn load_core_content() -> LoadedCoreContent {
    let pack = PackLoader::load_from_dir(Path::new("packs/core")).expect(
        "Failed to load packs/core; make sure the directory exists next to the executable.",
    );

    let compiled_blocks = ContentCompiler::compile_blocks(pack.blocks).unwrap_or_else(|errors| {
        for e in &errors {
            eprintln!("[content error] {}", e);
        }
        panic!("Block compilation failed; see errors above.");
    });

    let procedural_pack = PackLoader::load_procedural_from_dir(Path::new("packs/core"))
        .unwrap_or_else(|e| {
            panic!("Failed to load packs/core/procedurale: {}", e);
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

    let texture_registry = TextureRegistry::load(Path::new("packs"), &compiled_blocks)
        .unwrap_or_else(|errors| {
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
