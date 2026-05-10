use crate::{ContentRef, RawBlockMaterials, RawBlockShape, RawRenderMode};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlockRole {
    DefaultPlace,
    PlanetCore,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawBlockCollision {
    None,
    FullCube,
    SoftCube,
    LeafVolume,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawBlockPlacement {
    GridAligned,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBlockPhysicalDef {
    pub solid: bool,
    pub opaque: bool,
    pub collision: RawBlockCollision,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub friction: f32,
    pub restitution: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBlockVisual {
    pub shape: RawBlockShape,
    pub render: RawRenderMode,
    pub materials: RawBlockMaterials,
    pub ambient_occlusion: bool,
    pub casts_shadow: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBlockGameplayDef {
    #[serde(default)]
    pub preferred_tool: Option<ContentRef>,
    pub drops: ContentRef,
    pub placement: RawBlockPlacement,
    pub replaceable: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBlockAudioDef {
    pub footstep: ContentRef,
    #[serde(rename = "break")]
    pub break_sound: ContentRef,
    pub place: ContentRef,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawBlockRuntimeDef {
    #[serde(default)]
    pub role: Option<BlockRole>,
    #[serde(default)]
    pub reserved_id: Option<u16>,
    #[serde(default = "default_true")]
    pub can_target: bool,
    #[serde(default = "default_true")]
    pub blocks_light: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawBlockSimulationDef {
    #[serde(default)]
    pub decays_without_log: bool,
    #[serde(default)]
    pub supports_biome_tint: bool,
    #[serde(default)]
    pub surface_attached: bool,
    #[serde(default)]
    pub breaks_when_support_removed: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBlockDef {
    pub display_name: String,
    pub category: String,
    pub physical: RawBlockPhysicalDef,
    pub visual: RawBlockVisual,
    pub gameplay: RawBlockGameplayDef,
    pub audio: RawBlockAudioDef,
    #[serde(default)]
    pub tags: Vec<ContentRef>,
    #[serde(default)]
    pub runtime: RawBlockRuntimeDef,
    #[serde(default)]
    pub simulation: RawBlockSimulationDef,
}

fn default_true() -> bool {
    true
}
