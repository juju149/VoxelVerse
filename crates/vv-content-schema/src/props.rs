use crate::ContentRef;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RawPropCollectionDef {
    pub props: Vec<RawPropEntryDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawPropEntryDef {
    pub id_hint: String,
    pub model: ContentRef,
    pub collision: RawPropCollision,
    pub interaction: RawPropInteraction,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawPropCollision {
    None,
    VoxelBounds,
    Ladder,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawPropInteraction {
    None,
    Shelter,
    LightSource,
    Signal,
    Climbable,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawVegetationCatalogDef {
    pub groups: Vec<RawVegetationGroupDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawVegetationGroupDef {
    pub id_hint: String,
    pub models: Vec<ContentRef>,
    pub placement: RawVegetationCatalogPlacement,
    pub render: RawPropRenderMode,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawVegetationCatalogPlacement {
    SurfaceOnly,
    CaveSurfaceOrCeiling,
    UnderwaterSurface,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawPropRenderMode {
    InstancedVoxelProp,
}
