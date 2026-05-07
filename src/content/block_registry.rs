use crate::voxel::VoxelId;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MaterialTextureSet {
    pub albedo: String,
    pub normal: String,
    pub roughness: String,
}

/// Resolved visual representation used at runtime.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CompiledBlockVisual {
    /// Material layer for top faces. 0 = neutral fallback material.
    pub top_material_layer: u32,
    /// RGB tint multiplied over the material. `[1,1,1]` = no tint.
    pub tint: [f32; 3],
    /// RGB flat color fallback used when no atlas is present.
    pub flat_color: [f32; 3],
}

impl Default for CompiledBlockVisual {
    fn default() -> Self {
        Self {
            top_material_layer: 0,
            tint: [1.0; 3],
            flat_color: [1.0, 0.0, 1.0],
        }
    }
}

/// A compiled block definition ready for runtime use.
#[derive(Clone, Debug)]
#[allow(dead_code)]
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
    /// Visual data — ready for the renderer without further lookup.
    pub visual: CompiledBlockVisual,
}

/// Runtime registry of all compiled blocks.
/// IDs are compact `u16` indices; `VoxelId(0)` is always air.
pub struct BlockRegistry {
    blocks: Vec<CompiledBlock>,
    key_to_id: HashMap<String, VoxelId>,
    material_sets: Vec<MaterialTextureSet>,
    default_place: VoxelId,
    planet_core: VoxelId,
}

impl BlockRegistry {
    /// Constructed by `ContentCompiler::compile_blocks` — not by hand.
    pub(crate) fn new(
        blocks: Vec<CompiledBlock>,
        key_to_id: HashMap<String, VoxelId>,
        material_sets: Vec<MaterialTextureSet>,
        default_place: VoxelId,
        planet_core: VoxelId,
    ) -> Self {
        Self {
            blocks,
            key_to_id,
            material_sets,
            default_place,
            planet_core,
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

    /// Returns the block's RGB color. Unknown runtime IDs are engine bugs, not content fallbacks.
    pub fn color(&self, id: VoxelId) -> [f32; 3] {
        self.block(id)
            .unwrap_or_else(|| panic!("Unknown block runtime id {}", id.raw()))
            .color
    }

    pub fn visual(&self, id: VoxelId) -> &CompiledBlockVisual {
        &self
            .block(id)
            .unwrap_or_else(|| panic!("Unknown block runtime id {}", id.raw()))
            .visual
    }

    pub fn material_sets(&self) -> &[MaterialTextureSet] {
        &self.material_sets
    }

    /// The block placed when the player uses the default slot.
    pub fn default_place_voxel(&self) -> VoxelId {
        self.default_place
    }

    /// Block used for protected deep planet layers.
    pub fn planet_core_voxel(&self) -> VoxelId {
        self.planet_core
    }

    /// Number of registered blocks (including air).
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
}
