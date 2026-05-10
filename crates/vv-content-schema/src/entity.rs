use crate::ContentRef;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RawEntityDef {
    pub display_name: String,
    pub category: RawEntityCategory,
    pub body: RawEntityBodyDef,
    pub gameplay: RawEntityGameplayDef,
    #[serde(default)]
    pub spawn: Option<ContentRef>,
    #[serde(default)]
    pub tags: Vec<ContentRef>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawEntityCategory {
    Player,
    Animal,
    Monster,
    Boss,
    Humanoid,
}

#[derive(Debug, Clone, Deserialize)]
pub enum RawEntityBodyDef {
    CharacterBody(RawCharacterBodyDef),
    ModularVoxelBody(RawModularVoxelBodyDef),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawCharacterBodyDef {
    pub skeleton: ContentRef,
    pub model_root: ContentRef,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawModularVoxelBodyDef {
    pub skeleton: ContentRef,
    pub model_root: ContentRef,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawEntityGameplayDef {
    pub health: u32,
    pub movement: ContentRef,
    #[serde(default)]
    pub behavior: Option<ContentRef>,
    #[serde(default)]
    pub drops: Option<ContentRef>,
    #[serde(default)]
    pub inventory: Option<ContentRef>,
    #[serde(default)]
    pub interaction_reach_voxels: Option<f32>,
}
