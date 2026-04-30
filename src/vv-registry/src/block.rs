use crate::{
    BlockId, CompiledLootPool, CompiledToolKind, LootTableId, RegistryTable, TagId, TextureId,
};

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
    pub color: [f32; 3],
    pub roughness: f32,
    pub translucent: bool,
    pub emits_light: u8,
    pub tint: CompiledTintMode,
    pub material: CompiledStylizedMaterial,
    pub texture_layout: CompiledTextureLayout,
    pub textures: CompiledBlockTextures,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledTintMode {
    None,
    GrassColor,
    FoliageColor,
    WaterColor,
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledStylizedMaterial {
    pub visual_type: CompiledVisualMaterialType,
    pub secondary_color: [f32; 3],
    pub texture_influence: f32,
    pub block_variation: f32,
    pub face_variation: f32,
    pub macro_variation: f32,
    pub detail_strength: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledVisualMaterialType {
    Generic,
    Grass,
    Dirt,
    Stone,
    Sand,
    Wood,
    Leaves,
    CutStone,
    Planks,
    Ore,
    Water,
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

#[derive(Debug, Clone)]
pub enum CompiledDrops {
    None,
    Inline(Vec<CompiledLootPool>),
    Table(LootTableId),
}
