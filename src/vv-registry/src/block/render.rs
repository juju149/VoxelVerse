use crate::{BlockVisualId, ContentKey, MaterialId, RegistryTable, TextureId};

use super::{CompiledBlockVisual, RuntimeBlockVisual};

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

    // CPU-side compiled visual description.
    // Kept here for now because vv-mesh/vv-render still consume this shape.
    pub material: CompiledBlockVisual,

    // Temporary bridge while the no-texture procedural pipeline is being rebuilt.
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
