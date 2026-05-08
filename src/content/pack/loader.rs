use crate::content::schema::{
    RawBiomeProceduralDef, RawBiomeSetDef, RawBlockDef, RawCaveDef, RawClimateDef, RawFaunaDef,
    RawNoiseFieldDef, RawOreDef, RawPlanetProceduralDef, RawStructureDef, RawTerrainLayerSetDef,
    RawVegetationDef, RawVisualDetailDef,
};
use std::path::Path;

/// A loaded but uncompiled pack — raw definitions with their derived keys.
#[allow(dead_code)]
pub struct LoadedPack {
    pub blocks: Vec<(String, RawBlockDef)>,
}

#[derive(Default)]
pub struct RawProceduralPack {
    pub planets: Vec<(String, RawPlanetProceduralDef)>,
    pub fields: Vec<(String, RawNoiseFieldDef)>,
    pub climates: Vec<(String, RawClimateDef)>,
    pub biome_sets: Vec<(String, RawBiomeSetDef)>,
    pub biomes: Vec<(String, RawBiomeProceduralDef)>,
    pub terrain_layers: Vec<(String, RawTerrainLayerSetDef)>,
    pub ores: Vec<(String, RawOreDef)>,
    pub caves: Vec<(String, RawCaveDef)>,
    pub vegetation: Vec<(String, RawVegetationDef)>,
    pub structures: Vec<(String, RawStructureDef)>,
    pub fauna: Vec<(String, RawFaunaDef)>,
    pub visual_details: Vec<(String, RawVisualDetailDef)>,
}

pub struct PackLoader;

impl PackLoader {
    /// Load a pack from a directory on disk.
    /// The namespace is derived from the directory name (e.g. `packs/core` → `core`).
    /// Each `.ron` file in `blocks/` becomes `namespace:stem`.
    pub fn load_from_dir(pack_dir: &Path) -> Result<LoadedPack, String> {
        let namespace = pack_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid pack directory path: {}", pack_dir.display()))?
            .to_string();

        let blocks = Self::load_typed_dir::<RawBlockDef>(&pack_dir.join("blocks"), &namespace)?;
        Ok(LoadedPack { blocks })
    }

    pub fn load_procedural_from_dir(pack_dir: &Path) -> Result<RawProceduralPack, String> {
        let namespace = pack_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid pack directory path: {}", pack_dir.display()))?
            .to_string();

        let root = pack_dir.join("procedurale");
        if !root.exists() {
            return Ok(RawProceduralPack::default());
        }

        Ok(RawProceduralPack {
            planets: Self::load_typed_dir::<RawPlanetProceduralDef>(
                &root.join("planets"),
                &namespace,
            )?,
            fields: Self::load_typed_dir::<RawNoiseFieldDef>(&root.join("fields"), &namespace)?,
            climates: Self::load_typed_dir::<RawClimateDef>(&root.join("climates"), &namespace)?,
            biome_sets: Self::load_typed_dir::<RawBiomeSetDef>(
                &root.join("biome_sets"),
                &namespace,
            )?,
            biomes: Self::load_typed_dir::<RawBiomeProceduralDef>(
                &root.join("biomes"),
                &namespace,
            )?,
            terrain_layers: Self::load_typed_dir::<RawTerrainLayerSetDef>(
                &root.join("terrain_layers"),
                &namespace,
            )?,
            ores: Self::load_typed_dir::<RawOreDef>(&root.join("ores"), &namespace)?,
            caves: Self::load_typed_dir::<RawCaveDef>(&root.join("caves"), &namespace)?,
            vegetation: Self::load_typed_dir::<RawVegetationDef>(
                &root.join("vegetation"),
                &namespace,
            )?,
            structures: Self::load_typed_dir::<RawStructureDef>(
                &root.join("structures"),
                &namespace,
            )?,
            fauna: Self::load_typed_dir::<RawFaunaDef>(&root.join("fauna"), &namespace)?,
            visual_details: Self::load_typed_dir::<RawVisualDetailDef>(
                &root.join("visual_details"),
                &namespace,
            )?,
        })
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
