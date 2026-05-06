use crate::content::schema::RawBlockDef;
use std::path::Path;

/// A loaded but uncompiled pack — raw block definitions with their derived keys.
pub struct LoadedPack {
    pub namespace: String,
    pub blocks: Vec<(String, RawBlockDef)>,
}

pub struct PackLoader;

impl PackLoader {
    /// Load a pack from a directory on disk.
    /// The namespace is derived from the directory name (e.g. `packs/core` → `core`).
    /// Each `.ron` file in `blocks/` becomes a block with key `namespace:stem`.
    pub fn load_from_dir(pack_dir: &Path) -> Result<LoadedPack, String> {
        let namespace = pack_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid pack directory path: {}", pack_dir.display()))?
            .to_string();

        let blocks_dir = pack_dir.join("blocks");
        let mut blocks = Vec::new();

        if blocks_dir.exists() {
            let mut entries: Vec<_> = std::fs::read_dir(&blocks_dir)
                .map_err(|e| format!("Cannot read {}: {}", blocks_dir.display(), e))?
                .collect::<Result<_, _>>()
                .map_err(|e: std::io::Error| format!("Dir entry error: {}", e))?;

            // Sort entries for deterministic load order.
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("ron") {
                    continue;
                }

                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| format!("Invalid filename: {}", path.display()))?
                    .to_string();

                let text = std::fs::read_to_string(&path)
                    .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;

                let def: RawBlockDef = ron::from_str(&text).map_err(|e| {
                    format!("Parse error in {}:\n  {}", path.display(), e)
                })?;

                let key = format!("{}:{}", namespace, stem);
                blocks.push((key, def));
            }
        }

        Ok(LoadedPack { namespace, blocks })
    }
}
