use crate::voxel::VoxelId;
use std::collections::HashMap;

/// A compiled block definition ready for runtime use.
#[derive(Clone, Debug)]
pub struct CompiledBlock {
    pub id: VoxelId,
    /// Namespaced key derived from pack path, e.g. `"core:dirt"`.
    pub key: String,
    pub display_name: String,
    pub solid: bool,
    /// RGB color used for voxel rendering until a texture atlas is in place.
    pub color: [f32; 3],
    /// Hits needed to break this block. 0.0 = unbreakable.
    pub hardness: f32,
}

/// Runtime registry of all compiled blocks.
/// IDs are compact `u16` indices; `VoxelId(0)` is always air.
pub struct BlockRegistry {
    blocks: Vec<CompiledBlock>,
    key_to_id: HashMap<String, VoxelId>,
    default_place: VoxelId,
}

impl BlockRegistry {
    /// Constructed by `ContentCompiler::compile_blocks` — not by hand.
    pub(crate) fn new(
        blocks: Vec<CompiledBlock>,
        key_to_id: HashMap<String, VoxelId>,
        default_place: VoxelId,
    ) -> Self {
        Self {
            blocks,
            key_to_id,
            default_place,
        }
    }

    /// Look up a block ID by its namespaced key (e.g. `"core:dirt"`).
    pub fn lookup(&self, key: &str) -> Option<VoxelId> {
        self.key_to_id.get(key).copied()
    }

    /// Get the compiled definition for a runtime ID.
    pub fn block(&self, id: VoxelId) -> Option<&CompiledBlock> {
        self.blocks.get(id.raw() as usize)
    }

    pub fn is_solid(&self, id: VoxelId) -> bool {
        self.block(id).is_some_and(|b| b.solid)
    }

    /// Returns the block's RGB color. Falls back to dirt-brown if the ID is unknown.
    pub fn color(&self, id: VoxelId) -> [f32; 3] {
        self.block(id).map(|b| b.color).unwrap_or([0.6, 0.4, 0.2])
    }

    /// The block placed when the player uses the default slot (typically dirt).
    pub fn default_place_voxel(&self) -> VoxelId {
        self.default_place
    }

    /// Number of registered blocks (including air).
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
}
