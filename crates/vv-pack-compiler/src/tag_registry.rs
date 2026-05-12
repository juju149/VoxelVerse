//! Runtime tag registry — resolves tag keys to sets of item keys and block keys.
//!
//! Tags are defined in `TagSetDef` files and compiled into dense lookup tables.
//! Both item tags (e.g. `"core:tag/item/tool/pickaxe"`) and block tags
//! (e.g. `"core:tag/blocks/stone"`) live in the same registry since they
//! share the same namespace convention.
//!
//! The compiler builds this registry from loaded tag sets and validates that
//! every referenced content key exists in the known content index.

use std::collections::{HashMap, HashSet};

// ─── TagId ───────────────────────────────────────────────────────────────────

/// Compact, stable identifier for a compiled tag.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct TagId(u32);

impl TagId {
    pub fn raw(self) -> u32 {
        self.0
    }
    pub(crate) fn from_raw(id: u32) -> Self {
        Self(id)
    }
}

// ─── Compiled data model ─────────────────────────────────────────────────────

/// A compiled tag: a named set of content keys.
#[derive(Clone, Debug)]
pub struct CompiledTag {
    pub id: TagId,
    /// Full namespaced key (`namespace:tag/...`).
    pub key: String,
    /// All content keys (items or blocks) that carry this tag.
    pub values: HashSet<String>,
}

impl CompiledTag {
    pub fn contains(&self, content_key: &str) -> bool {
        self.values.contains(content_key)
    }
}

// ─── TagRegistry ─────────────────────────────────────────────────────────────

/// Runtime registry of all compiled tags.
#[derive(Debug, Default)]
pub struct TagRegistry {
    tags: Vec<CompiledTag>,
    key_to_id: HashMap<String, TagId>,
    /// Reverse mapping: content_key → all tag IDs that include it.
    content_to_tags: HashMap<String, Vec<TagId>>,
}

impl TagRegistry {
    pub(crate) fn new(tags: Vec<CompiledTag>) -> Self {
        let key_to_id = tags
            .iter()
            .map(|t| (t.key.clone(), t.id))
            .collect::<HashMap<_, _>>();

        let mut content_to_tags: HashMap<String, Vec<TagId>> = HashMap::new();
        for tag in &tags {
            for value in &tag.values {
                content_to_tags.entry(value.clone()).or_default().push(tag.id);
            }
        }

        Self {
            tags,
            key_to_id,
            content_to_tags,
        }
    }

    pub fn lookup(&self, key: &str) -> Option<TagId> {
        self.key_to_id.get(key).copied()
    }

    pub fn get(&self, id: TagId) -> Option<&CompiledTag> {
        self.tags.get(id.raw() as usize)
    }

    pub fn get_by_key(&self, key: &str) -> Option<&CompiledTag> {
        self.get(self.lookup(key)?)
    }

    /// Returns all tag IDs that the given content key belongs to.
    pub fn tags_for(&self, content_key: &str) -> &[TagId] {
        self.content_to_tags
            .get(content_key)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Returns `true` if `content_key` is a member of `tag_key`.
    pub fn has_tag(&self, content_key: &str, tag_key: &str) -> bool {
        match self.get_by_key(tag_key) {
            Some(tag) => tag.contains(content_key),
            None => false,
        }
    }

    pub fn tags(&self) -> &[CompiledTag] {
        &self.tags
    }

    pub fn len(&self) -> usize {
        self.tags.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }
}
