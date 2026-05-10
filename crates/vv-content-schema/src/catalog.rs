use crate::ContentRef;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RawLootTableDef {
    pub rolls: u32,
    pub entries: Vec<RawLootEntryDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawLootEntryDef {
    pub item: ContentRef,
    pub count: (u32, u32),
    pub chance: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawSkeletonDef {
    pub display_name: String,
    pub coordinate_space: RawSkeletonCoordinateSpace,
    pub scale: f32,
    #[serde(default)]
    pub slots: Vec<RawSkeletonSlotDef>,
    #[serde(default)]
    pub animation_sets: Vec<ContentRef>,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawSkeletonCoordinateSpace {
    VoxelModelLocal,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawSkeletonSlotDef {
    pub name: String,
    #[serde(default)]
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawTagSetDef {
    pub tags: Vec<RawTagGroupDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawTagGroupDef {
    pub id_hint: String,
    pub values: Vec<ContentRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawPackManifest {
    pub format_version: u32,
    pub namespace: String,
    pub display_name: String,
    pub version: String,
    pub kind: RawPackKind,
    pub description: String,
    pub authors: Vec<String>,
    pub license: String,
    pub load_priority: i32,
    #[serde(default)]
    pub dependencies: Vec<ContentRef>,
    #[serde(default)]
    pub features: Vec<String>,
    pub content_roots: RawPackContentRoots,
    pub rules: RawPackRules,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawPackKind {
    Builtin,
    Mod,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawPackContentRoots {
    pub definitions: String,
    pub media: String,
    pub generated: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawPackRules {
    pub identity: RawIdentityMode,
    pub id_style: String,
    pub runtime_loads_raw_files: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawIdentityMode {
    PathDerived,
}
