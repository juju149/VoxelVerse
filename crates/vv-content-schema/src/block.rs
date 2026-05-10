use crate::{ContentRef, RawRenderMode};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlockRole {
    DefaultPlace,
    PlanetCore,
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
    pub hardness: f32,
    pub blast_resistance: f32,
    pub friction: f32,
    pub restitution: f32,
}

/// Per-block visual override. Geometry, AO, and collision now live on the
/// referenced `RawBlockModelDef`. The block keeps the render mode and the
/// resolved face-layer material map.
#[derive(Debug, Clone, Deserialize)]
pub struct RawBlockVisual {
    pub render: RawRenderMode,
    /// Map from `face_layer` slot name → material `ContentRef`.
    /// The compiler validates that the keys match the referenced model's
    /// `face_layers()` exactly — no missing slot, no extra slot.
    /// For invisible blocks (air-like) the map is empty.
    #[serde(default)]
    pub materials: HashMap<String, ContentRef>,
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
    pub format_version: u32,
    pub display_name: String,
    pub category: String,
    /// Reference to a `RawBlockModelDef` (e.g. `core:block_model/cube`).
    /// Provides geometry, face-layer slot names, and collision.
    pub model: ContentRef,
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
