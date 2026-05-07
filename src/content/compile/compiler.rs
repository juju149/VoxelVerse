use crate::content::biome_registry::{BiomeRegistry, CompiledBiome};
use crate::content::block_registry::{BlockRegistry, CompiledBlock, CompiledBlockVisual};
use crate::content::schema::{BlockRole, RawBiomeDef, RawBlockDef};
use crate::voxel::VoxelId;
use std::collections::HashMap;

pub struct ContentCompiler;

impl ContentCompiler {
    /// Compile raw block definitions into a runtime `BlockRegistry`.
    ///
    /// Rules:
    /// - A block with key ending in `:air` must be present — it is assigned `VoxelId(0)`.
    /// - All other blocks are sorted alphabetically for deterministic ID assignment.
    /// - Returns a list of human-readable errors if validation fails.
    pub fn compile_blocks(
        mut raw: Vec<(String, RawBlockDef)>,
    ) -> Result<BlockRegistry, Vec<String>> {
        let mut errors = Vec::new();

        let air_pos = raw.iter().position(|(key, _)| key.ends_with(":air"));
        if air_pos.is_none() {
            errors.push("Pack must define a block with key ending in ':air'.".into());
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Pull air out first — it must become VoxelId(0).
        let air_pos = air_pos.unwrap();
        let air_entry = raw.remove(air_pos);

        // Sort the rest alphabetically for stable IDs across reloads.
        raw.sort_by(|(a, _), (b, _)| a.cmp(b));

        // Reinsert air at the front.
        raw.insert(0, air_entry);

        let mut blocks: Vec<CompiledBlock> = Vec::with_capacity(raw.len());
        let mut key_to_id: HashMap<String, VoxelId> = HashMap::with_capacity(raw.len());
        let mut default_place = VoxelId::AIR;

        for (idx, (key, def)) in raw.into_iter().enumerate() {
            let id = VoxelId::new(idx as u16);

            // A block that declares `role = "default_place"` is used as the
            // initial placement block.  No name heuristics needed.
            if def.role == Some(BlockRole::DefaultPlace) {
                default_place = id;
            }

            key_to_id.insert(key.clone(), id);
            blocks.push(CompiledBlock {
                id,
                key,
                display_name: def.display_name,
                solid: def.solid,
                color: def.color,
                hardness: def.hardness,
                // Visual defaults: flat color from the block's `color` field.
                // Texture atlas index will be assigned once the atlas pipeline is ready.
                visual: CompiledBlockVisual {
                    atlas_index: 0,
                    tint: [1.0, 1.0, 1.0],
                    flat_color: def.color,
                },
            });
        }

        // Fallback: if no block declared `role = "default_place"`, use first solid block.
        if default_place == VoxelId::AIR {
            if let Some(solid) = blocks.iter().find(|b| b.solid) {
                default_place = solid.id;
            }
        }

        Ok(BlockRegistry::new(blocks, key_to_id, default_place))
    }

    /// Compile raw biome definitions into a `BiomeRegistry`.
    ///
    /// Rules:
    /// - At least one biome must be defined.
    /// - Biomes are sorted by temperature_center descending (tropical first) for
    ///   stable ID assignment.
    /// - All block references are resolved against the provided `BlockRegistry`.
    /// - Returns human-readable errors on failure.
    pub fn compile_biomes(
        mut raw: Vec<(String, RawBiomeDef)>,
        block_registry: &BlockRegistry,
    ) -> Result<BiomeRegistry, Vec<String>> {
        let mut errors = Vec::new();

        if raw.is_empty() {
            errors.push("Pack must define at least one biome.".into());
            return Err(errors);
        }

        // Sort by temperature_center descending for deterministic IDs.
        raw.sort_by(|(_, a), (_, b)| {
            b.temperature_center
                .partial_cmp(&a.temperature_center)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut biomes: Vec<CompiledBiome> = Vec::with_capacity(raw.len());

        for (idx, (key, def)) in raw.into_iter().enumerate() {
            let surface = block_registry.lookup(&def.surface_block);
            let subsurface = block_registry.lookup(&def.subsurface_block);

            if surface.is_none() {
                errors.push(format!(
                    "Biome '{}': unknown surface_block '{}'",
                    key, def.surface_block
                ));
            }
            if subsurface.is_none() {
                errors.push(format!(
                    "Biome '{}': unknown subsurface_block '{}'",
                    key, def.subsurface_block
                ));
            }

            if errors.is_empty() {
                biomes.push(CompiledBiome {
                    id: idx as u8,
                    key,
                    display_name: def.display_name,
                    surface_block: surface.unwrap(),
                    subsurface_block: subsurface.unwrap(),
                    temperature_center: def.temperature_center.clamp(0.0, 1.0),
                    roughness_center: def.roughness_center.clamp(0.0, 1.0),
                    terrain_amplitude: def.terrain_amplitude.clamp(0.0, 1.0),
                    terrain_flatness: def.terrain_flatness.clamp(0.0, 1.0),
                });
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(BiomeRegistry::new(biomes))
    }
}
