use std::collections::HashMap;
use vv_content_schema::RawBlockShape;
use vv_voxel::VoxelId;

/// Geometric shape used to dispatch the mesher.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlockShape {
    #[default]
    Cube,
    CrossPlane,
}

impl From<RawBlockShape> for BlockShape {
    fn from(value: RawBlockShape) -> Self {
        match value {
            RawBlockShape::None => Self::Cube,
            RawBlockShape::Cube => Self::Cube,
            RawBlockShape::CrossPlane => Self::CrossPlane,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MaterialTextureSet {
    pub albedo: String,
    pub normal: String,
    pub roughness: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockMaterialLayers {
    pub top: u32,
    pub bottom: u32,
    pub front: u32,
    pub back: u32,
    pub left: u32,
    pub right: u32,
}

/// Resolved visual representation used at runtime.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CompiledBlockVisual {
    /// Material layers per cube face. 0 = neutral fallback material.
    pub layers: BlockMaterialLayers,
    /// RGB tint multiplied over the material. `[1,1,1]` = no tint.
    pub tint: [f32; 3],
    /// RGB flat color fallback used when no atlas is present.
    pub flat_color: [f32; 3],
    /// Mesh dispatch shape.
    pub shape: BlockShape,
}

impl Default for CompiledBlockVisual {
    fn default() -> Self {
        Self {
            layers: BlockMaterialLayers::default(),
            tint: [1.0; 3],
            flat_color: [1.0, 0.0, 1.0],
            shape: BlockShape::Cube,
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
    /// One RGBA color per atlas layer (length = material_sets.len() + 1).
    /// Index 0 is the fallback layer (white). Index L+1 is the representative
    /// flat color of `material_sets[L]` — taken from the first block that uses
    /// that material. Consumed by the renderer's color-only debug toggle.
    material_colors: Vec<[f32; 4]>,
    default_place: VoxelId,
    planet_core: VoxelId,
}

impl BlockRegistry {
    /// Constructed by `ContentCompiler::compile_blocks` — not by hand.
    pub(crate) fn new(
        blocks: Vec<CompiledBlock>,
        key_to_id: HashMap<String, VoxelId>,
        material_sets: Vec<MaterialTextureSet>,
        material_colors: Vec<[f32; 4]>,
        default_place: VoxelId,
        planet_core: VoxelId,
    ) -> Self {
        Self {
            blocks,
            key_to_id,
            material_sets,
            material_colors,
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

    #[cfg(test)]
    pub fn blocks(&self) -> &[CompiledBlock] {
        &self.blocks
    }

    pub fn is_solid(&self, id: VoxelId) -> bool {
        self.block(id).is_some_and(|b| b.solid)
    }

    /// Whether the block fully fills its 1×1×1 voxel cell. Cross-planes and
    /// air do not — used by the mesher to decide if neighbour faces are occluded.
    pub fn is_opaque_cube(&self, id: VoxelId) -> bool {
        if id == VoxelId::AIR {
            return false;
        }
        self.block(id)
            .is_some_and(|b| b.solid && b.visual.shape == BlockShape::Cube)
    }

    /// Anything that is not air should be visited by the mesher (cross-planes
    /// included), even if not solid for collision purposes.
    pub fn is_renderable(&self, id: VoxelId) -> bool {
        id != VoxelId::AIR
    }

    #[allow(dead_code)]
    pub fn shape(&self, id: VoxelId) -> BlockShape {
        self.block(id).map(|b| b.visual.shape).unwrap_or_default()
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

    /// Per-atlas-layer flat colors. Same indexing as `material_layer` packed
    /// in vertex `tex_index` (layer 0 = fallback, layer L+1 = material L).
    pub fn material_colors(&self) -> &[[f32; 4]] {
        &self.material_colors
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
