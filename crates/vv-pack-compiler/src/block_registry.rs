use crate::block_family::{BlockStateValue, CompiledBlockFamily};
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

    #[doc(hidden)]
    pub fn for_tests(id: u32) -> Self {
        Self(id)
    }
}

/// Compiled mesh kind, retained on the model for future state transforms
/// and mesher inspection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompiledMesh {
    None,
    Cube { ambient_occlusion: bool },
    CubeColumn { ambient_occlusion: bool },
}

/// Compiled collision shape attached to a model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompiledCollision {
    None,
    FullCube,
    SoftCube,
    LeafVolume,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompiledMeshClass {
    #[default]
    OpaqueCube,
    Cutout,
    Prop,
    Water,
    Foliage,
    Emissive,
    Invisible,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompiledSoundKind {
    #[default]
    None,
    Grass,
    Stone,
    Wood,
    Sand,
    Snow,
    Dirt,
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
#[derive(Debug)]
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
pub struct CompiledBlockVisual {
    /// Material layers per cube face. 0 = neutral fallback material.
    pub layers: BlockMaterialLayers,
    /// RGB tint multiplied over the material. `[1,1,1]` = no tint.
    pub tint: [f32; 3],
    /// Reference into the `BlockModelRegistry`.
    pub model_id: BlockModelId,
}

impl Default for CompiledBlockVisual {
    fn default() -> Self {
        Self {
            layers: BlockMaterialLayers::default(),
            tint: [1.0; 3],
            model_id: BlockModelId(0),
        }
    }
}

/// A compiled block definition ready for runtime use.
///
/// One `CompiledBlock` is generated per state-variant of a family. All
/// variants of the same block share `family_key`; their `state` field
/// disambiguates them.
#[derive(Clone, Debug)]
pub struct CompiledBlock {
    pub id: VoxelId,
    /// Namespaced key of the source block (the family). Multiple variants
    /// share this key; their `state` distinguishes them.
    pub family_key: String,
    /// State assignment for this variant. Empty for stateless blocks.
    /// State value for interaction systems that inspect block variants.
    pub state: BlockStateValue,
    /// Human-readable name for UI and diagnostics.
    pub display_name: String,
    pub solid: bool,
    /// RGB color used for voxel rendering until a texture atlas is in place.
    pub color: [f32; 3],
    /// Hits needed to break this block. 0.0 = unbreakable.
    pub hardness: f32,
    /// Visual data — ready for the renderer without further lookup.
    pub visual: CompiledBlockVisual,
    /// Category string from the raw block definition (e.g. "terrain", "ore",
    /// "tool", "food"). Used for inventory filtering.
    pub category: String,
    /// Maximum number of this block that can stack in one inventory slot.
    pub max_stack: u32,
    /// Content key of the loot table to roll when this block is broken.
    /// Resolved at runtime against `LootRegistry`.
    pub drops_key: String,
    /// Tag key of the preferred tool (e.g. `"core:tag/item/tool/pickaxe"`).
    /// `None` means any tool (or bare hand) works equally.
    pub preferred_tool_tag: Option<String>,
    /// Minimum tool tier required for reliable drops.
    pub required_tool_tier: u32,
    /// Logical block sound category compiled from data.
    pub sound_kind: CompiledSoundKind,
    /// Renderer/mesher routing class. This decides batching strategy; it is
    /// content-owned, not inferred from concrete block names at runtime.
    pub mesh_class: CompiledMeshClass,
}

/// Runtime registry of all compiled blocks.
/// IDs are compact `u16` indices; `VoxelId(0)` is always air.
#[derive(Debug)]
pub struct BlockRegistry {
    blocks: Vec<CompiledBlock>,
    /// Maps a family_key (e.g. `"core:block/.../oak_log"`) to its
    /// default-variant `VoxelId`. Used by `lookup` to keep the existing
    /// "give me this block" call sites working transparently.
    family_default: HashMap<String, VoxelId>,
    families: Vec<CompiledBlockFamily>,
    family_for_voxel: HashMap<VoxelId, usize>,
    family_by_key: HashMap<String, usize>,
    material_sets: Vec<MaterialTextureSet>,
    material_colors: Vec<[f32; 4]>,
    default_place: VoxelId,
    planet_core: VoxelId,
    models: BlockModelRegistry,
}

impl BlockRegistry {
    /// Constructed by `ContentCompiler::compile_blocks` — not by hand.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        blocks: Vec<CompiledBlock>,
        families: Vec<CompiledBlockFamily>,
        material_sets: Vec<MaterialTextureSet>,
        material_colors: Vec<[f32; 4]>,
        default_place: VoxelId,
        planet_core: VoxelId,
        models: BlockModelRegistry,
    ) -> Self {
        let mut family_default = HashMap::with_capacity(families.len());
        let mut family_for_voxel = HashMap::with_capacity(blocks.len());
        let mut family_by_key = HashMap::with_capacity(families.len());
        for (idx, fam) in families.iter().enumerate() {
            family_default.insert(fam.family_key.clone(), fam.default_variant);
            family_by_key.insert(fam.family_key.clone(), idx);
            for variant_id in &fam.variants {
                family_for_voxel.insert(*variant_id, idx);
            }
        }
        Self {
            blocks,
            family_default,
            families,
            family_for_voxel,
            family_by_key,
            material_sets,
            material_colors,
            default_place,
            planet_core,
            models,
        }
    }

    pub fn lookup(&self, key: &str) -> Option<VoxelId> {
        self.family_default.get(key).copied()
    }

    pub fn lookup_default(&self, family_key: &str) -> Option<VoxelId> {
        self.family_default.get(family_key).copied()
    }

    pub fn lookup_stem(&self, stem: &str) -> Option<VoxelId> {
        self.family_default.iter().find_map(|(key, &id)| {
            if key.rsplit('/').next() == Some(stem) {
                Some(id)
            } else {
                None
            }
        })
    }

    /// Look up a specific variant of a family by its state assignment.
    pub fn lookup_variant(&self, family_key: &str, state: &BlockStateValue) -> Option<VoxelId> {
        let idx = *self.family_by_key.get(family_key)?;
        self.families[idx].lookup(state)
    }

    /// Resolve which family a runtime `VoxelId` belongs to.
    pub fn family_of(&self, id: VoxelId) -> Option<&CompiledBlockFamily> {
        let idx = *self.family_for_voxel.get(&id)?;
        self.families.get(idx)
    }

    /// Resolve a runtime `VoxelId` to its `BlockStateValue`.
    pub fn state_of(&self, id: VoxelId) -> Option<&BlockStateValue> {
        let fam = self.family_of(id)?;
        fam.state_of(id)
    }

    pub fn families(&self) -> &[CompiledBlockFamily] {
        &self.families
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

    pub fn uses_greedy_opaque_meshing(&self, id: VoxelId) -> bool {
        if id == VoxelId::AIR {
            return false;
        }
        let Some(block) = self.block(id) else {
            return false;
        };
        if !block.solid || block.mesh_class != CompiledMeshClass::OpaqueCube {
            return false;
        }
        self.models
            .get(block.visual.model_id)
            .is_some_and(|m| matches!(m.mesh, CompiledMesh::Cube { .. }))
    }

    pub fn mesh_class(&self, id: VoxelId) -> CompiledMeshClass {
        self.block(id).map(|b| b.mesh_class).unwrap_or_default()
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

    /// Return the category string for a block (e.g. `"terrain"`, `"ore"`,
    /// `"tool"`). Returns `""` for unknown ids.
    pub fn category(&self, id: VoxelId) -> &str {
        self.block(id).map(|b| b.category.as_str()).unwrap_or("")
    }

    /// Return the maximum stack size for a block. Defaults to 99 for unknown ids.
    pub fn max_stack(&self, id: VoxelId) -> u32 {
        self.block(id).map(|b| b.max_stack).unwrap_or(99)
    }
}
