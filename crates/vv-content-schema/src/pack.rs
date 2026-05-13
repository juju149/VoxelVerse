//! Pack manifest (`pack.ron`).
//!
//! Describes the pack as a whole: identity, dependencies, content roots, and
//! identity rules. Lives in its own module so the manifest can evolve without
//! pulling unrelated bits along with it.

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
