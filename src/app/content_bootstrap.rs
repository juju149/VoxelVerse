use crate::content::CompiledPlanet;
use crate::content::{compile::ContentCompiler, pack::PackLoader, BiomeRegistry, BlockRegistry};
use std::path::Path;
use std::sync::Arc;

pub struct LoadedCoreContent {
    pub blocks: Arc<BlockRegistry>,
    pub biomes: Arc<BiomeRegistry>,
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

    let compiled_biomes = ContentCompiler::compile_biomes(pack.biomes, &compiled_blocks)
        .unwrap_or_else(|errors| {
            for e in &errors {
                eprintln!("[content error] {}", e);
            }
            panic!("Biome compilation failed; see errors above.");
        });

    let compiled_planets =
        ContentCompiler::compile_planets(pack.planets).unwrap_or_else(|errors| {
            for e in &errors {
                eprintln!("[content error] {}", e);
            }
            panic!("Planet compilation failed; see errors above.");
        });

    let planet = compiled_planets
        .into_iter()
        .next()
        .expect("compile_planets guarantees at least one planet");

    println!(
        "Loaded {} blocks, {} biomes, planet '{}' ({}) from pack 'core'.",
        compiled_blocks.block_count(),
        compiled_biomes.biome_count(),
        planet.key,
        planet.display_name,
    );

    LoadedCoreContent {
        blocks: Arc::new(compiled_blocks),
        biomes: Arc::new(compiled_biomes),
        planet,
    }
}
