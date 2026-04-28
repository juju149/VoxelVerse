use crate::common::{ContentRef, TagRef};
use serde::{Deserialize, Serialize};

/// Tag definition. Tags are ALWAYS namespaced: "namespace:tag_name".
/// A tag is a named set of resource references within a given domain.
/// Deserialized from defs/tags/<name>.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagDef {
    /// Content domain of this tag. Validated by vv-compiler for consistency.
    #[serde(default)]
    pub kind: TagContentKind,
    /// Member references. Uses ContentRef to support mixed-domain tags.
    /// In practice, prefer single-domain tags with matching `kind`.
    pub values: Vec<ContentRef>,
    /// Tags whose values are inherited (composition). References to other TagDefs.
    #[serde(default)]
    pub extends: Vec<TagRef>,
}

/// Content domain of a tag's values.
/// Declared here, validated by the compiler (vv-compiler).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagContentKind {
    Block,
    Item,
    Entity,
    Placeable,
    /// Mixed or unconstrained domain.
    Any,
}

impl Default for TagContentKind {
    fn default() -> Self {
        TagContentKind::Any
    }
}
