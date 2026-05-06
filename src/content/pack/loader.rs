use crate::content::schema::{RawBiomeDef, RawBlockDef};
use std::path::Path;

/// A loaded but uncompiled pack — raw definitions with their derived keys.
pub struct LoadedPack {
    pub namespace: String,
    pub blocks: Vec<(String, RawBlockDef)>,
    pub biomes: Vec<(String, RawBiomeDef)>,
}

pub struct PackLoader;

impl PackLoader {
    /// Load a pack from a directory on disk.
    /// The namespace is derived from the directory name (e.g. `packs/core` → `core`).
    /// Each `.ron` file in `blocks/` becomes `namespace:stem`, same for `biomes/`.
    pub fn load_from_dir(pack_dir: &Path) -> Result<LoadedPack, String> {
        let namespace = pack_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid pack directory path: {}", pack_dir.display()))?
            .to_string();

        let blocks = Self::load_typed_dir::<RawBlockDef>(&pack_dir.join("blocks"), &namespace)?;
        let biomes = Self::load_typed_dir::<RawBiomeDef>(&pack_dir.join("biomes"), &namespace)?;

        Ok(LoadedPack { namespace, blocks, biomes })
    }

    /// Generic helper: load all `.ron` files in a directory as type `T`.
    fn load_typed_dir<T: serde::de::DeserializeOwned>(
        dir: &Path,
        namespace: &str,
    ) -> Result<Vec<(String, T)>, String> {
        let mut result = Vec::new();

        if !dir.exists() {
            return Ok(result);
        }

        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| format!("Cannot read {}: {}", dir.display(), e))?
            .collect::<Result<_, _>>()
            .map_err(|e: std::io::Error| format!("Dir entry error: {}", e))?;

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

            let def: T = ron::from_str(&text)
                .map_err(|e| format!("Parse error in {}:\n  {}", path.display(), e))?;

            result.push((format!("{}:{}", namespace, stem), def));
        }

        Ok(result)
    }
}

