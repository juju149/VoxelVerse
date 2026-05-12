//! Format version constants for content packs.
//!
//! Each kind of definition embeds a `format_version: u32` so the compiler can
//! reject content authored against an older or future schema. There is no
//! migration layer yet — content must match the current version exactly.

pub const PACK_FORMAT_VERSION: u32 = 1;
pub const BLOCK_FORMAT_VERSION: u32 = 1;
pub const BLOCK_MODEL_FORMAT_VERSION: u32 = 1;
pub const MATERIAL_FORMAT_VERSION: u32 = 1;
pub const ITEM_FORMAT_VERSION: u32 = 1;
pub const ENTITY_FORMAT_VERSION: u32 = 1;
pub const LOOT_FORMAT_VERSION: u32 = 1;
pub const SKELETON_FORMAT_VERSION: u32 = 1;
pub const SOUND_EVENT_FORMAT_VERSION: u32 = 1;
pub const TAG_FORMAT_VERSION: u32 = 1;
pub const RECIPE_FORMAT_VERSION: u32 = 1;
pub const WORLDGEN_FORMAT_VERSION: u32 = 1;

/// Checks a `format_version` field against the expected constant.
/// Returns a structured error string if the version mismatches.
pub fn check_format_version(
    actual: u32,
    expected: u32,
    kind: &str,
    key: &str,
) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{kind} '{key}': format_version {actual} unsupported (expected {expected})"
        ))
    }
}

use serde::Deserialize;
use crate::ContentRef;

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
