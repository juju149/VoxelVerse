//! Media types — texture sets, material descriptors, sampling modes.
//!
//! These are the "shape" of authored media references shared by the block,
//! item, and render layers. Concrete file existence lives in the doctor; this
//! module only defines what authors write in RON.

use serde::Deserialize;

use crate::ContentRef;

/// Alias kept for legacy compatibility — a texture is just another content ref.
pub type TextureRef = ContentRef;

/// PBR-lite material slot triplet.
#[derive(Debug, Clone, Deserialize)]
pub struct RawMaterialTextureSet {
    pub albedo: TextureRef,
    pub normal: TextureRef,
    pub roughness: TextureRef,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RawRenderMode {
    Invisible,
    #[default]
    Opaque,
    AlphaTest,
}

#[derive(Debug, Clone, Deserialize)]
pub enum RawMaterialTint {
    BiomeTint(String),
    Fixed([f32; 3]),
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawMaterialCategory {
    BlockSurface,
    Item,
    Prop,
    Creature,
    Ui,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawTextureSampling {
    PixelArtNearest,
    Linear,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawAuthoringDef {
    pub source: String,
    pub generated_by: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawMaterialDef {
    pub display_name: String,
    pub category: RawMaterialCategory,
    pub albedo: TextureRef,
    #[serde(default)]
    pub normal: Option<TextureRef>,
    #[serde(default)]
    pub roughness: Option<TextureRef>,
    #[serde(default)]
    pub tint: Option<RawMaterialTint>,
    pub render: RawRenderMode,
    pub sampling: RawTextureSampling,
    pub atlas: ContentRef,
    pub authoring: RawAuthoringDef,
}
