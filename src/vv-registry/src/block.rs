use crate::{
    BlockId, BlockVisualId, CompiledLootPool, CompiledToolKind, ContentKey, LootTableId,
    MaterialId, RegistryTable, TagId, TextureId,
};
use bytemuck::{Pod, Zeroable};
use smallvec::SmallVec;

#[derive(Debug, Clone)]
pub struct CompiledBlock {
    pub display_key: Option<String>,
    pub stack_max: u8,
    pub tags: Vec<TagId>,
    pub mining: CompiledBlockMining,
    pub physics: CompiledBlockPhysics,
    pub render: CompiledBlockRender,
    pub drops: CompiledDrops,
}

pub type BlockRegistry = RegistryTable<BlockId, CompiledBlock>;

#[derive(Debug, Clone, Copy)]
pub struct CompiledBlockMining {
    pub hardness: f32,
    pub tool: CompiledToolKind,
    pub tool_tier_min: u8,
    pub drop_xp: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledMaterialPhase {
    Solid,
    Liquid,
    Passable,
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledBlockPhysics {
    pub phase: CompiledMaterialPhase,
    pub density: f32,
    pub friction: f32,
    pub drag: f32,
}

#[derive(Debug, Clone)]
pub struct CompiledBlockRender {
    pub visual_id: BlockVisualId,
    pub color: [f32; 3],
    pub roughness: f32,
    pub metallic: f32,
    pub emission: Option<[f32; 4]>,
    pub alpha: f32,
    pub render_mode: CompiledRenderMode,
    pub emits_light: u8,
    pub tint: CompiledTintMode,
    pub shape: CompiledBlockShape,
    pub meshing: CompiledBlockMeshing,
    pub material: CompiledBlockVisual,
    pub texture_layout: CompiledTextureLayout,
    pub textures: CompiledBlockTextures,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompiledBlockShape {
    Cube,
    Cross,
    Fluid,
    Custom { model: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledRenderMode {
    Opaque,
    Cutout,
    Transparent,
    Additive,
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledBlockMeshing {
    pub occludes: bool,
    pub greedy_merge: bool,
    pub casts_shadow: bool,
    pub receives_ao: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledTintMode {
    None,
    GrassColor,
    FoliageColor,
    WaterColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledTextureLayout {
    Single,
    Sides,
    Custom,
}

#[derive(Debug, Clone, Default)]
pub struct CompiledBlockTextures {
    pub single: Option<TextureId>,
    pub side: Option<TextureId>,
    pub top: Option<TextureId>,
    pub bottom: Option<TextureId>,
    pub north: Option<TextureId>,
    pub south: Option<TextureId>,
    pub east: Option<TextureId>,
    pub west: Option<TextureId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledBlockFace {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

impl CompiledBlockRender {
    pub fn texture_for_face(&self, face: CompiledBlockFace) -> Option<TextureId> {
        let textures = &self.textures;
        match self.texture_layout {
            CompiledTextureLayout::Single => textures.single,
            CompiledTextureLayout::Sides => match face {
                CompiledBlockFace::Top => textures.top.or(textures.single),
                CompiledBlockFace::Bottom => textures.bottom.or(textures.single),
                CompiledBlockFace::North
                | CompiledBlockFace::South
                | CompiledBlockFace::East
                | CompiledBlockFace::West => textures.side.or(textures.single),
            },
            CompiledTextureLayout::Custom => match face {
                CompiledBlockFace::Top => textures.top,
                CompiledBlockFace::Bottom => textures.bottom,
                CompiledBlockFace::North => textures.north.or(textures.side),
                CompiledBlockFace::South => textures.south.or(textures.side),
                CompiledBlockFace::East => textures.east.or(textures.side),
                CompiledBlockFace::West => textures.west.or(textures.side),
            }
            .or(textures.single),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompiledTextureResource;

pub type TextureRegistry = RegistryTable<TextureId, CompiledTextureResource>;
pub type BlockVisualRegistry = RegistryTable<BlockVisualId, RuntimeBlockVisual>;
pub type MaterialRegistry = RegistryTable<MaterialId, CompiledMaterialShader>;

#[derive(Debug, Clone)]
pub struct CompiledMaterialShader {
    pub shader_key: ContentKey,
}

#[derive(Debug, Clone)]
pub struct CompiledBlockVisual {
    pub material_key: ContentKey,
    pub base_color: [f32; 4],
    pub palette: SmallVec<[[f32; 4]; 8]>,
    pub roughness: f32,
    pub metallic: f32,
    pub emission: Option<[f32; 4]>,
    pub alpha: f32,
    pub bevel: f32,
    pub normal_strength: f32,
    pub variation: CompiledBlockVisualVariation,
    pub procedural: BlockProceduralConfig,
    pub faces: CompiledBlockFaceVisuals,
    pub details: SmallVec<[CompiledBlockDetail; 8]>,
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledBlockVisualVariation {
    pub per_voxel_tint: f32,
    pub per_face_tint: f32,
    pub macro_noise_scale: f32,
    pub macro_noise_strength: f32,
    pub micro_noise_scale: f32,
    pub micro_noise_strength: f32,
    pub edge_darkening: f32,
    pub ao_influence: f32,
    pub biome_tint_strength: f32,
    pub wetness_response: f32,
    pub snow_response: f32,
    pub dust_response: f32,
}

#[derive(Debug, Clone, Default)]
pub struct CompiledBlockFaceVisuals {
    pub top: Option<CompiledBlockFaceVisual>,
    pub side: Option<CompiledBlockFaceVisual>,
    pub bottom: Option<CompiledBlockFaceVisual>,
    pub north: Option<CompiledBlockFaceVisual>,
    pub south: Option<CompiledBlockFaceVisual>,
    pub east: Option<CompiledBlockFaceVisual>,
    pub west: Option<CompiledBlockFaceVisual>,
}

#[derive(Debug, Clone)]
pub struct CompiledBlockFaceVisual {
    pub color_bias: [f32; 4],
    pub detail_bias: SmallVec<[String; 4]>,
}

#[derive(Debug, Clone)]
pub struct CompiledBlockDetail {
    pub kind: String,
    pub density: f32,
    pub color: [f32; 4],
    pub min_size: f32,
    pub max_size: f32,
    pub slope_bias: f32,
}

pub const BLOCK_VISUAL_FACE_COUNT: usize = 6;
pub const BLOCK_VISUAL_DETAIL_COUNT: usize = 8;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct BlockProceduralConfig {
    pub grid_size: u32,
    pub face_blend: u32,
    pub _padding: [u32; 2],
}

impl BlockProceduralConfig {
    pub fn new(grid_size: u32, face_blend: bool) -> Self {
        Self {
            grid_size,
            face_blend: u32::from(face_blend),
            _padding: [0; 2],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RuntimeBlockFaceVisual {
    pub color_bias: [f32; 4],
    pub detail_mask: u32,
    pub _padding: [u32; 3],
}

impl Default for RuntimeBlockFaceVisual {
    fn default() -> Self {
        Self {
            color_bias: [1.0, 1.0, 1.0, 1.0],
            detail_mask: 0,
            _padding: [0; 3],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RuntimeBlockDetail {
    pub color: [f32; 4],
    pub params: [f32; 4],
    pub meta: [u32; 4],
}

impl Default for RuntimeBlockDetail {
    fn default() -> Self {
        Self {
            color: [0.0; 4],
            params: [0.0; 4],
            meta: [0; 4],
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct BlockVisualFlags(pub u32);

impl BlockVisualFlags {
    pub const TRANSPARENT: u32 = 1 << 0;
    pub const EMISSIVE: u32 = 1 << 1;
    pub const BIOME_TINTED: u32 = 1 << 2;
    pub const OCCLUDES: u32 = 1 << 3;
    pub const RECEIVES_AO: u32 = 1 << 4;

    pub fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    pub fn contains(self, bit: u32) -> bool {
        self.0 & bit != 0
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RuntimeBlockVisual {
    pub base_color: [f32; 4],
    pub emission: [f32; 4],
    pub surface: [f32; 4],
    pub shape: [f32; 4],
    pub variation_a: [f32; 4],
    pub variation_b: [f32; 4],
    pub response: [f32; 4],
    pub palette: [u32; 4],
    pub procedural: [u32; 4],
    pub faces: [RuntimeBlockFaceVisual; BLOCK_VISUAL_FACE_COUNT],
    pub details: [RuntimeBlockDetail; BLOCK_VISUAL_DETAIL_COUNT],
}

impl RuntimeBlockVisual {
    pub fn fallback() -> Self {
        Self {
            base_color: [0.55, 0.55, 0.55, 1.0],
            emission: [0.0; 4],
            surface: [1.0, 0.0, 1.0, 0.0],
            shape: [0.0; 4],
            variation_a: [0.0, 0.0, 1.0, 0.0],
            variation_b: [1.0, 0.0, 0.0, 1.0],
            response: [0.0; 4],
            palette: [0, 1, 0, 0],
            procedural: [10, 0, 0, 0],
            faces: [RuntimeBlockFaceVisual::default(); BLOCK_VISUAL_FACE_COUNT],
            details: [RuntimeBlockDetail::default(); BLOCK_VISUAL_DETAIL_COUNT],
        }
    }
}

#[derive(Debug, Clone)]
pub enum CompiledDrops {
    None,
    Inline(Vec<CompiledLootPool>),
    Table(LootTableId),
}
