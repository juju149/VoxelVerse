use crate::ContentRef;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RawVoxelAssetRegistry {
    pub generated_from: String,
    pub asset_count: u32,
    pub assets: Vec<RawVoxelAssetDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawVoxelAssetDef {
    pub id: ContentRef,
    pub path: String,
    pub kind: RawGeneratedAssetKind,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawGeneratedAssetKind {
    VoxelModel,
}
