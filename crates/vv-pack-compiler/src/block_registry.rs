use std::collections::HashMap;
use vv_voxel::VoxelId;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MaterialTextureSet {
    pub albedo: String,
    pub normal: String,
    pub roughness: String,
}

/// Material atlas indices per cube face, in renderer-axis order.
///
/// The compiler resolves a block's `materials` map into these slots based
/// on the referenced model's mesh kind:
/// - `Cube`: `top = materials["py"]`, `bottom = materials["ny"]`,
///   `front = materials["pz"]`, `back = materials["nz"]`,
///   `right = materials["px"]`, `left = materials["nx"]`.
/// - `CubeColumn` (default Y-axis): top/bottom = `end`, others = `side`.
/// - `CrossPlane`: all six slots replicate `plane` for compatibility with
///   the existing mesher; only one slot is actually consumed at draw time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockMaterialLayers {
    pub top: u32,
    pub bottom: u32,
    pub front: u32,
    pub back: u32,
    pub left: u32,
    pub right: u32,
}

/// Dense identifier of a `CompiledBlockModel` in the `BlockModelRegistry`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct BlockModelId(u32);

impl BlockModelId {
    pub fn raw(self) -> u32 {
        self.0
    }

    pub(crate) fn from_raw(id: u32) -> Self {
        Self(id)
    }
}

/// Compiled mesh kind, retained on the model for future state transforms
/// and mesher inspection. The mesher currently dispatches on
/// `CompiledBlockVisual::shape` for performance — this enum will become the
/// single source of truth in a later sprint step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompiledMesh {
    None,
    Cube { ambient_occlusion: bool },
    CubeColumn { ambient_occlusion: bool },
    CrossPlane,
}

/// Compiled collision shape attached to a model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompiledCollision {
    None,
    FullCube,
    SoftCube,
    LeafVolume,
}

#[derive(Clone, Debug)]
pub struct CompiledBlockModel {
    pub id: BlockModelId,
    pub key: String,
    pub mesh: CompiledMesh,
    pub collision: CompiledCollision,
    /// Stable face-layer slot names declared by the model — the contract that
    /// referencing blocks must satisfy in their `materials` map.
    pub face_layers: Vec<String>,
}

impl CompiledBlockModel {
    /// True for meshes that fully fill their unit cell (used for
    /// neighbour-occlusion in the mesher).
    pub fn is_full_cube(&self) -> bool {
        matches!(
            self.mesh,
            CompiledMesh::Cube { .. } | CompiledMesh::CubeColumn { .. }
        )
    }
}

/// Registry of all compiled block models. Indexed densely by `BlockModelId`.
pub struct BlockModelRegistry {
    models: Vec<CompiledBlockModel>,
    key_to_id: HashMap<String, BlockModelId>,
}

impl BlockModelRegistry {
    pub(crate) fn new(models: Vec<CompiledBlockModel>) -> Self {
        let key_to_id = models
            .iter()
            .map(|m| (m.key.clone(), m.id))
            .collect::<HashMap<_, _>>();
        Self { models, key_to_id }
    }

    pub fn lookup(&self, key: &str) -> Option<BlockModelId> {
        self.key_to_id.get(key).copied()
    }

    pub fn get(&self, id: BlockModelId) -> Option<&CompiledBlockModel> {
        self.models.get(id.raw() as usize)
    }

    pub fn models(&self) -> &[CompiledBlockModel] {
        &self.models
    }

    pub fn len(&self) -> usize {
        self.models.len()
    }

    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }
}

/// Resolved visual representation used at runtime.
///
/// All geometry/collision questions go through the `BlockModelRegistry`
/// keyed by `model_id`. There is no shape cache — the model is the single
/// source of truth.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CompiledBlockVisual {
    /// Material layers per cube face. 0 = neutral fallback material.
    pub layers: BlockMaterialLayers,
    /// RGB tint multiplied over the material. `[1,1,1]` = no tint.
    pub tint: [f32; 3],
    /// RGB flat color fallback used when no atlas is present.
    pub flat_color: [f32; 3],
    /// Reference into the `BlockModelRegistry`.
    pub model_id: BlockModelId,
}

impl Default for CompiledBlockVisual {
    fn default() -> Self {
        Self {
            layers: BlockMaterialLayers::default(),
            tint: [1.0; 3],
            flat_color: [1.0, 0.0, 1.0],
            model_id: BlockModelId(0),
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
    material_colors: Vec<[f32; 4]>,
    default_place: VoxelId,
    planet_core: VoxelId,
    models: BlockModelRegistry,
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
        models: BlockModelRegistry,
    ) -> Self {
        Self {
            blocks,
            key_to_id,
            material_sets,
            material_colors,
            default_place,
            planet_core,
            models,
        }
    }

    pub fn lookup(&self, key: &str) -> Option<VoxelId> {
        self.key_to_id.get(key).copied()
    }

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

    pub fn is_opaque_cube(&self, id: VoxelId) -> bool {
        if id == VoxelId::AIR {
            return false;
        }
        let Some(block) = self.block(id) else {
            return false;
        };
        if !block.solid {
            return false;
        }
        self.models
            .get(block.visual.model_id)
            .is_some_and(|m| m.is_full_cube())
    }

    pub fn is_renderable(&self, id: VoxelId) -> bool {
        id != VoxelId::AIR
    }

    /// Resolve the compiled model behind a runtime `VoxelId`. Panics on
    /// unknown ids (engine bug, not a content issue).
    pub fn model_of(&self, id: VoxelId) -> &CompiledBlockModel {
        let visual = self.visual(id);
        self.models
            .get(visual.model_id)
            .unwrap_or_else(|| panic!("Block runtime id {} references unknown model", id.raw()))
    }

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

    pub fn material_colors(&self) -> &[[f32; 4]] {
        &self.material_colors
    }

    pub fn default_place_voxel(&self) -> VoxelId {
        self.default_place
    }

    pub fn planet_core_voxel(&self) -> VoxelId {
        self.planet_core
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn models(&self) -> &BlockModelRegistry {
        &self.models
    }
}
