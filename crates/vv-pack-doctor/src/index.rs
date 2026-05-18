//! Reference resolution helpers.
//!
//! Pack files use short names (`stone`, `oak_log`, `#tag.soil`) instead of
//! fully-qualified IDs. The index turns the scanned objects into a set of
//! lookup tables that the cross-reference checks can consult without having
//! to walk the object list every time.

use std::collections::{BTreeMap, BTreeSet};

use crate::scan::{PackScan, ParsedObject, ParsedWorldFile, WorldCategory};

pub struct PackIndex<'a> {
    pub scan: &'a PackScan,
    pub object_by_short: BTreeMap<String, Vec<&'a ParsedObject>>,
    pub world_by_short: BTreeMap<(WorldCategory, String), Vec<&'a ParsedWorldFile>>,
    pub tags_declared: BTreeSet<String>,
    pub stations_declared: BTreeSet<String>,
    pub voxel_model_set: BTreeSet<String>,
    pub voxel_asset_set: BTreeSet<String>,
    pub texture_set: BTreeSet<String>,
    pub voxel_set: BTreeSet<String>,
}

impl<'a> PackIndex<'a> {
    pub fn build(scan: &'a PackScan) -> Self {
        let mut object_by_short: BTreeMap<String, Vec<&ParsedObject>> = BTreeMap::new();
        let mut tags_declared: BTreeSet<String> = BTreeSet::new();
        let mut stations_declared: BTreeSet<String> = BTreeSet::new();

        for obj in &scan.objects {
            let short = short_name(&obj.id);
            object_by_short.entry(short).or_default().push(obj);
            for tag in &obj.def.tags {
                if let Some(normalized) = normalize_tag_key(tag) {
                    tags_declared.insert(normalized.clone());
                    if let Some(rest) = normalized.strip_prefix("station/") {
                        stations_declared.insert(rest.to_string());
                    }
                }
            }
            if let Some(station) = &obj.def.station {
                for tag in &station.station_tags {
                    if let Some(normalized) = normalize_tag_key(tag) {
                        tags_declared.insert(normalized.clone());
                        if let Some(rest) = normalized.strip_prefix("station/") {
                            stations_declared.insert(rest.to_string());
                        }
                    }
                }
            }
        }

        let mut world_by_short: BTreeMap<(WorldCategory, String), Vec<&ParsedWorldFile>> =
            BTreeMap::new();
        for file in &scan.world_files {
            let short = short_name(&file.id);
            world_by_short
                .entry((file.category, short))
                .or_default()
                .push(file);
        }

        let mut voxel_model_set = BTreeSet::new();
        for model in &scan.voxel_models {
            voxel_model_set.insert(format!("{}:{}", scan.namespace, model.id));
        }

        let mut voxel_asset_set = BTreeSet::new();
        if let Some(registry) = &scan.voxel_assets {
            for asset in &registry.def.assets {
                voxel_asset_set.insert(asset.id.0.clone());
            }
        }

        let mut texture_set = BTreeSet::new();
        for tex in &scan.texture_files {
            texture_set.insert(tex.rel_path.clone());
        }
        let mut voxel_set = BTreeSet::new();
        for v in &scan.voxel_files {
            voxel_set.insert(v.rel_path.clone());
        }

        Self {
            scan,
            object_by_short,
            world_by_short,
            tags_declared,
            stations_declared,
            voxel_model_set,
            voxel_asset_set,
            texture_set,
            voxel_set,
        }
    }

    /// Resolves a short item/block reference like `"stone"` or `"core:object/terrain/stone"`.
    /// Returns the canonical id (without namespace prefix) on success.
    pub fn resolve_object(&self, reference: &str) -> Option<&ParsedObject> {
        if reference.is_empty() {
            return None;
        }
        // Strip leading namespace.
        let stripped = reference.split(':').next_back().unwrap_or(reference);
        // Exact id match (with or without "object/" prefix).
        for obj in &self.scan.objects {
            if obj.id == stripped {
                return Some(obj);
            }
            if obj
                .id
                .strip_prefix("object/")
                .map(|s| s == stripped)
                .unwrap_or(false)
            {
                return Some(obj);
            }
        }
        // Short-name match.
        let short = stripped
            .split('/')
            .next_back()
            .unwrap_or(stripped)
            .to_string();
        match self.object_by_short.get(&short) {
            Some(v) if v.len() == 1 => Some(v[0]),
            Some(v) if v.len() > 1 => v.first().copied(), // first match; caller can warn
            _ => None,
        }
    }

    pub fn ambiguous_object(&self, short: &str) -> Option<Vec<&ParsedObject>> {
        self.object_by_short
            .get(short)
            .filter(|v| v.len() > 1)
            .cloned()
    }

    pub fn resolve_world(
        &self,
        category: WorldCategory,
        reference: &str,
    ) -> Option<&ParsedWorldFile> {
        let stripped = reference.split(':').next_back().unwrap_or(reference);
        let short = stripped
            .split('/')
            .next_back()
            .unwrap_or(stripped)
            .to_string();
        self.world_by_short
            .get(&(category, short))
            .and_then(|v| v.first().copied())
    }

    pub fn texture_exists(&self, rel: &str) -> bool {
        self.texture_set.contains(rel)
    }

    pub fn voxel_exists(&self, rel: &str) -> bool {
        self.voxel_set.contains(rel)
    }

    pub fn voxel_model_exists(&self, key: &str) -> bool {
        self.voxel_model_set.contains(key)
    }

    pub fn voxel_asset_registered(&self, key: &str) -> bool {
        if self.voxel_asset_set.contains(key) {
            return true;
        }
        if key.contains(':') {
            return false;
        }
        self.voxel_asset_set
            .contains(&format!("{}:{key}", self.scan.namespace))
    }
}

pub fn short_name(id: &str) -> String {
    id.rsplit('/').next().unwrap_or(id).to_string()
}

/// Parse a strict V1 tag reference.
/// Example: `#core:tag/station/construction`.
pub fn parse_tag_ref(value: &str) -> Option<(&str, &str)> {
    let stripped = value.strip_prefix('#')?;
    let (_, path) = stripped.split_once(':')?;
    let rest = path.strip_prefix("tag/")?;
    let (kind, name) = rest.split_once('/')?;
    Some((kind, name))
}

pub fn normalize_tag_key(value: &str) -> Option<String> {
    let stripped = value.strip_prefix('#')?;
    let (_, path) = stripped.split_once(':')?;
    path.strip_prefix("tag/").map(str::to_string)
}
