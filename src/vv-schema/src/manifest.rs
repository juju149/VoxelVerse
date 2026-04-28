use serde::{Deserialize, Serialize};

/// Contents of pack.ron at the root of a pack.
///
/// Convention: the pack folder name MUST match `namespace`.
/// Exception: `voxelverse_core` is the official pack with namespace `"voxelverse"`.
/// For all other packs, folder_name == namespace is REQUIRED.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackManifest {
    /// Unique namespace prefix for all resources in this pack. E.g. "voxelverse", "mymod".
    pub namespace: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<PackDependency>,
    #[serde(default)]
    pub override_policy: OverridePolicy,
    /// Namespaces for which this pack provides compatibility patches in compat/.
    #[serde(default)]
    pub compat_targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackDependency {
    pub namespace: String,
    pub version_req: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverridePolicy {
    /// Error if two packs define the same resource without an explicit override.
    Error,
    /// Overrides must be declared explicitly in the overriding file.
    Explicit,
    /// This pack silently overwrites any conflicting resource.
    Override,
}

impl Default for OverridePolicy {
    fn default() -> Self {
        OverridePolicy::Explicit
    }
}
