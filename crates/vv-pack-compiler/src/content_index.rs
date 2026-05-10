//! Cross-domain index of every content key declared by a loaded pack.
//!
//! The compiler builds this index once before any compilation pass so that any
//! `ContentRef` encountered later (drops, audio events, materials, tags, …) can
//! be checked for existence, not just for syntactic validity. A reference that
//! does not resolve to a real definition is a hard error.
//!
//! Domains currently indexed:
//!  - `core:block/...`       (block defs)
//!  - `core:material/...`    (material defs)
//!  - `core:item/...`        (item defs)
//!  - `core:entity/...`      (entity defs)
//!  - `core:loot/...`        (loot tables)
//!  - `core:skeleton/...`    (skeleton defs)
//!  - `core:props/...`       (prop collection defs)
//!  - `core:vegetation/...`  (vegetation catalog defs)
//!  - `core:voxel/...`       (generated voxel asset registry)
//!
//! Domains *not yet* indexed (deferred to later sprint steps because they
//! require their own refactor):
//!  - `core:sound/...`  → arrives in step 3 with `RawSoundEventDef`.
//!  - `core:tag/...`    → tags are currently anonymous `id_hint` strings; will
//!     be reworked into addressable defs.
//!  - `core:texture/...` → resolved by `TextureRegistry` from media paths.

use std::collections::HashSet;
use vv_content_schema::ContentRef;
use vv_pack_loader::LoadedPack;

#[derive(Debug, Default)]
pub struct ContentIndex {
    keys: HashSet<String>,
}

impl ContentIndex {
    /// Builds the index from every key in `LoadedPack`. Generated voxel asset
    /// IDs are also included so blocks/items can reference voxel models.
    pub fn build(pack: &LoadedPack) -> Self {
        let mut keys = HashSet::new();

        for (k, _) in &pack.blocks {
            keys.insert(k.clone());
        }
        for (k, _) in &pack.materials {
            keys.insert(k.clone());
        }
        for (k, _) in &pack.items {
            keys.insert(k.clone());
        }
        for (k, _) in &pack.entities {
            keys.insert(k.clone());
        }
        for (k, _) in &pack.loot {
            keys.insert(k.clone());
        }
        for (k, _) in &pack.skeletons {
            keys.insert(k.clone());
        }
        for (k, _) in &pack.prop_collections {
            keys.insert(k.clone());
        }
        for (k, _) in &pack.vegetation_catalogs {
            keys.insert(k.clone());
        }
        if let Some(reg) = &pack.voxel_assets {
            for asset in &reg.assets {
                keys.insert(asset.id.0.clone());
            }
        }

        Self { keys }
    }

    /// Returns true if the given fully-qualified key is declared in the pack.
    pub fn contains(&self, key: &str) -> bool {
        self.keys.contains(key)
    }

    /// Asserts that `r` resolves to a known key. On failure, appends a
    /// descriptive error to `errors`. `ctx` should describe where the
    /// reference was encountered (e.g. "block 'core:block/.../grass' drops").
    pub fn require(&self, r: &ContentRef, ctx: &str, errors: &mut Vec<String>) {
        if !self.keys.contains(&r.0) {
            errors.push(format!("dangling reference '{}' in {ctx}", r.0));
        }
    }

    /// Same as `require` but only enforced when the reference is `Some`.
    pub fn require_opt(&self, r: Option<&ContentRef>, ctx: &str, errors: &mut Vec<String>) {
        if let Some(r) = r {
            self.require(r, ctx, errors);
        }
    }

    /// Number of indexed keys — useful for diagnostics.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Convenience constructor for tests and synthetic compile pipelines.
    pub fn from_keys<I, S>(keys: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            keys: keys.into_iter().map(Into::into).collect(),
        }
    }
}
