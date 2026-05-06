use crate::content::block_registry::{BlockRegistry, CompiledBlock};
use crate::content::schema::RawBlockDef;
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

            // The first solid non-air block named "dirt" (or fallback: first solid block) is the
            // default placement block. Simple heuristic until tool/hotbar system exists.
            if key.ends_with(":dirt") {
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
            });
        }

        // Fallback: if no dirt found, use first solid block.
        if default_place == VoxelId::AIR {
            if let Some(solid) = blocks.iter().find(|b| b.solid) {
                default_place = solid.id;
            }
        }

        Ok(BlockRegistry::new(blocks, key_to_id, default_place))
    }
}
