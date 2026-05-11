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
use vv_content_schema::RawTagSetDef;

// ─── TagId ───────────────────────────────────────────────────────────────────

/// Compact, stable identifier for a compiled tag.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct TagId(u32);

impl TagId {
    pub fn raw(self) -> u32 {
        self.0
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

// ─── Compilation ─────────────────────────────────────────────────────────────

/// Compile all `RawTagSetDef` files into a flat `TagRegistry`.
///
/// Each tag set file declares one or more tag groups.  Each group has an
/// `id_hint` (e.g. `"stone"`) that is used verbatim as the tag key suffix
/// under the pack namespace convention (`namespace:tag/<sub-path>/<id_hint>`).
/// The tag file's own content key provides the sub-path context.
///
/// Because `id_hint` is a free string and multiple files may define tags in
/// the same namespace, duplicate keys are merged (values are unioned).
/// References to unknown content are currently warnings, not hard errors —
/// content authoring is iterative and tags often lead defs.
pub fn compile_tags(raw: Vec<(String, RawTagSetDef)>) -> TagRegistry {
    // key → accumulated value set
    let mut accumulator: HashMap<String, HashSet<String>> = HashMap::new();

    for (file_key, def) in raw {
        // Derive the tag namespace from the file key. The file key is
        // `namespace:tag/<sub-path>/<filename>` — we strip the final component
        // and use `namespace:tag/<sub-path>` as the prefix.
        let prefix = derive_tag_prefix(&file_key);

        for group in def.tags {
            let tag_key = if prefix.is_empty() {
                group.id_hint.clone()
            } else {
                format!("{}/{}", prefix, group.id_hint)
            };

            let entry = accumulator.entry(tag_key).or_default();
            for value in group.values {
                entry.insert(value.0);
            }
        }
    }

    // Sort for deterministic IDs.
    let mut sorted: Vec<(String, HashSet<String>)> = accumulator.into_iter().collect();
    sorted.sort_by(|(a, _), (b, _)| a.cmp(b));

    let tags = sorted
        .into_iter()
        .enumerate()
        .map(|(idx, (key, values))| CompiledTag {
            id: TagId(idx as u32),
            key,
            values,
        })
        .collect();

    TagRegistry::new(tags)
}

/// Extract the canonical tag prefix from a tag-set file content key.
///
/// File key:  `core:tag/blocks/core_block_tags`
/// Prefix:    `core:tag/blocks`
fn derive_tag_prefix(file_key: &str) -> String {
    // Split on ':' to separate namespace from path.
    let Some((ns, path)) = file_key.split_once(':') else {
        return file_key.to_string();
    };

    // Remove the last path component (the filename).
    let prefix_path = match path.rfind('/') {
        Some(idx) => &path[..idx],
        None => path,
    };

    format!("{}:{}", ns, prefix_path)
}
