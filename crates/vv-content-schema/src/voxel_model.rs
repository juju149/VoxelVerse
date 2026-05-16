//! Authored voxel model manifests.
//!
//! Gameplay definitions never point directly at `media/voxel/**/*.vox`.
//! They reference a manifest under `defs/voxel_models/**.voxel_model.ron`,
//! and this manifest owns scale, pivot, collision and usage metadata.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawVoxelModelManifest {
    #[serde(default = "default_format_version")]
    pub format_version: u32,
    pub source: String,
    pub usage: Vec<RawVoxelModelUsage>,
    pub scale: (f32, f32, f32),
    pub pivot: (f32, f32, f32),
    pub orientation: (f32, f32, f32),
    pub bounds: (f32, f32, f32),
    pub collision: RawVoxelModelCollision,
    pub render: RawVoxelModelRender,
    pub inventory: RawVoxelModelInventoryUse,
    pub hand: RawVoxelModelHandUse,
    pub world: RawVoxelModelWorldUse,
    pub structure: RawVoxelModelStructureUse,
    #[serde(default)]
    pub lod: Option<RawVoxelModelLod>,
}

fn default_format_version() -> u32 {
    crate::OBJECT_FORMAT_VERSION
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawVoxelModelUsage {
    Inventory,
    Hand,
    WorldItem,
    Entity,
    Prop,
    Structure,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawVoxelModelCollision {
    None,
    VoxelBounds,
    BlockAligned,
    CustomBounds,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawVoxelModelRender {
    Opaque,
    Cutout,
    Emissive,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawVoxelModelInventoryUse {
    Disabled,
    RenderPreview,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawVoxelModelHandUse {
    Disabled,
    FirstPerson,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawVoxelModelWorldUse {
    Disabled,
    DroppedItem,
    Entity,
    Prop,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawVoxelModelStructureUse {
    Disabled,
    StaticProp,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawVoxelModelLod {
    pub strategy: RawVoxelModelLodStrategy,
    pub max_distance_voxels: f32,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawVoxelModelLodStrategy {
    None,
    FadeOut,
    Impostor,
}
