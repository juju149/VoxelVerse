//! Skeleton definitions (rigs for characters, animals and props).
//!
//! Skeletons live under `defs/skeletons/*.skeleton.ron`. They describe a
//! hierarchy of named bones with rest-pose transforms. Animations,
//! attachments, and equipment slots reference bones by name, so the data
//! authored here is the source of truth for everything downstream.
//!
//! Sprint scope is intentionally minimal: enough to parse files cleanly, no
//! runtime evaluator yet.

use serde::Deserialize;

/// One rig.
#[derive(Debug, Clone, Deserialize)]
pub struct RawSkeletonDef {
    pub format_version: u32,
    pub display_name: String,
    /// Bones declared in *parent-before-children* order. The compiler relies
    /// on this property — it does not topologically sort.
    pub bones: Vec<RawSkeletonBone>,
    /// Named attachment points (slots) used by equipment / accessories.
    #[serde(default)]
    pub slots: Vec<RawSkeletonSlot>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawSkeletonBone {
    pub name: String,
    /// Parent bone name. `None` only for the root.
    #[serde(default)]
    pub parent: Option<String>,
    pub rest: RawBoneTransform,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawSkeletonSlot {
    pub name: String,
    pub bone: String,
    /// Optional local-space offset from the bone's origin.
    #[serde(default)]
    pub offset: Option<(f32, f32, f32)>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBoneTransform {
    pub translation: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    #[serde(default = "default_scale")]
    pub scale: (f32, f32, f32),
}

fn default_scale() -> (f32, f32, f32) {
    (1.0, 1.0, 1.0)
}
