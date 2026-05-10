use std::path::{Path, PathBuf};
use vv_content_schema::*;

/// A loaded but uncompiled pack. IDs are path-derived and stable across hosts.
#[allow(dead_code)]
pub struct LoadedPack {
    pub manifest: RawPackManifest,
    pub blocks: Vec<(String, RawBlockDef)>,
    pub materials: Vec<(String, RawMaterialDef)>,
    pub items: Vec<(String, RawItemDef)>,
    pub entities: Vec<(String, RawEntityDef)>,
    pub loot: Vec<(String, RawLootTableDef)>,
    pub skeletons: Vec<(String, RawSkeletonDef)>,
    pub prop_collections: Vec<(String, RawPropCollectionDef)>,
    pub vegetation_catalogs: Vec<(String, RawVegetationCatalogDef)>,
    pub tag_sets: Vec<(String, RawTagSetDef)>,
    pub voxel_assets: Option<RawVoxelAssetRegistry>,
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
    pub fn load_from_dir(pack_dir: &Path) -> Result<LoadedPack, String> {
        let namespace = namespace_from_dir(pack_dir)?;
        let manifest = load_file::<RawPackManifest>(&pack_dir.join("pack.ron"))?;
        if manifest.namespace != namespace {
            return Err(format!(
                "Pack namespace '{}' does not match directory '{}'",
                manifest.namespace, namespace
            ));
        }
        check_format_version(
            manifest.format_version,
            PACK_FORMAT_VERSION,
            "pack",
            &manifest.namespace,
        )?;

        let defs = pack_dir.join(&manifest.content_roots.definitions);
        if !defs.exists() {
            return Err(format!(
                "Pack is missing defs directory: {}",
                defs.display()
            ));
        }
        let voxel_assets_path = pack_dir
            .join(&manifest.content_roots.generated)
            .join("registries")
            .join("voxel_assets.ron");

        Ok(LoadedPack {
            manifest,
            blocks: load_typed_tree(&defs.join("blocks"), &defs, &namespace)?,
            materials: load_typed_tree(&defs.join("materials"), &defs, &namespace)?,
            items: load_typed_tree(&defs.join("items"), &defs, &namespace)?,
            entities: load_typed_tree(&defs.join("entities"), &defs, &namespace)?,
            loot: load_typed_tree(&defs.join("loot"), &defs, &namespace)?,
            skeletons: load_typed_tree(&defs.join("skeletons"), &defs, &namespace)?,
            prop_collections: load_typed_tree(&defs.join("props"), &defs, &namespace)?,
            vegetation_catalogs: load_typed_tree(&defs.join("vegetation"), &defs, &namespace)?,
            tag_sets: load_typed_tree(&defs.join("tags"), &defs, &namespace)?,
            voxel_assets: load_optional_file(&voxel_assets_path)?,
        })
    }

    pub fn load_procedural_from_dir(pack_dir: &Path) -> Result<RawProceduralPack, String> {
        let namespace = namespace_from_dir(pack_dir)?;
        let root = pack_dir.join("defs").join("worldgen");
        if !root.exists() {
            return Ok(RawProceduralPack::default());
        }

        Ok(RawProceduralPack {
            planets: load_typed_tree(
                &root.join("planet_profiles"),
                &pack_dir.join("defs"),
                &namespace,
            )?,
            fields: load_typed_tree(
                &root.join("noise_fields"),
                &pack_dir.join("defs"),
                &namespace,
            )?,
            climates: load_typed_tree(
                &root.join("climate_profiles"),
                &pack_dir.join("defs"),
                &namespace,
            )?,
            biome_sets: load_typed_tree(
                &root.join("biome_sets"),
                &pack_dir.join("defs"),
                &namespace,
            )?,
            biomes: load_typed_tree(&root.join("biomes"), &pack_dir.join("defs"), &namespace)?,
            terrain_layers: load_typed_tree(
                &root.join("terrain_layers"),
                &pack_dir.join("defs"),
                &namespace,
            )?,
            ores: load_typed_tree(&root.join("ores"), &pack_dir.join("defs"), &namespace)?,
            caves: load_typed_tree(&root.join("caves"), &pack_dir.join("defs"), &namespace)?,
            vegetation: load_typed_tree(
                &root.join("vegetation"),
                &pack_dir.join("defs"),
                &namespace,
            )?,
            structures: load_typed_tree(
                &root.join("structures"),
                &pack_dir.join("defs"),
                &namespace,
            )?,
            fauna: load_typed_tree(&root.join("spawns"), &pack_dir.join("defs"), &namespace)?,
            visual_details: load_typed_tree(
                &root.join("visual_details"),
                &pack_dir.join("defs"),
                &namespace,
            )?,
        })
    }
}

fn namespace_from_dir(pack_dir: &Path) -> Result<String, String> {
    pack_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("Invalid pack directory path: {}", pack_dir.display()))
        .map(str::to_string)
}

fn load_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    ron::from_str(&text)
        .or_else(|_| ron::from_str(strip_outer_type_name(&text)))
        .map_err(|e| format!("Parse error in {}:\n  {}", path.display(), e))
}

fn load_optional_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>, String> {
    if path.exists() {
        load_file(path).map(Some)
    } else {
        Ok(None)
    }
}

fn strip_outer_type_name(text: &str) -> &str {
    let trimmed = text.trim_start_matches('\u{feff}').trim_start();
    let Some(open) = trimmed.find('(') else {
        return text;
    };
    if trimmed[..open]
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        &trimmed[open..]
    } else {
        text
    }
}

fn load_typed_tree<T: serde::de::DeserializeOwned>(
    dir: &Path,
    defs_root: &Path,
    namespace: &str,
) -> Result<Vec<(String, T)>, String> {
    let mut paths = Vec::new();
    collect_ron_files(dir, &mut paths)?;
    paths.sort();

    paths
        .into_iter()
        .map(|path| {
            let key = derive_key(namespace, defs_root, &path)?;
            let def = load_file::<T>(&path)?;
            Ok((key, def))
        })
        .collect()
}

fn collect_ron_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in
        std::fs::read_dir(dir).map_err(|e| format!("Cannot read {}: {}", dir.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Dir entry error in {}: {}", dir.display(), e))?;
        let path = entry.path();
        if path.is_dir() {
            collect_ron_files(&path, out)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("ron") {
            out.push(path);
        }
    }
    Ok(())
}

fn derive_key(namespace: &str, defs_root: &Path, path: &Path) -> Result<String, String> {
    let rel = path
        .strip_prefix(defs_root)
        .map_err(|_| format!("{} is outside {}", path.display(), defs_root.display()))?;
    let parts: Vec<_> = rel
        .iter()
        .filter_map(|p| p.to_str())
        .map(str::to_string)
        .collect();
    if parts.len() < 2 {
        return Err(format!(
            "Definition path is too shallow: {}",
            path.display()
        ));
    }

    let root = parts[0].as_str();
    let stem = strip_def_suffix(parts.last().unwrap());
    let dirs = &parts[1..parts.len() - 1];
    let id_path = match root {
        "blocks" => join_domain("block", dirs, &stem),
        "materials" => join_domain("material", dirs, &stem),
        "items" => join_domain("item", &singular_item_dirs(dirs), &stem),
        "entities" => join_domain("entity", dirs, &stem),
        "loot" => join_domain("loot", dirs, &stem),
        "skeletons" => format!("skeleton/{}", stem),
        "props" => format!("prop_collection/{}", stem),
        "vegetation" => format!("vegetation_catalog/{}", stem),
        "tags" => format!("tags/{}", stem),
        "worldgen" => derive_worldgen_key(dirs, &stem)?,
        other => {
            return Err(format!(
                "Unknown definition root '{}': {}",
                other,
                path.display()
            ));
        }
    };

    Ok(format!("{}:{}", namespace, id_path))
}

fn derive_worldgen_key(dirs: &[String], stem: &str) -> Result<String, String> {
    let Some(kind) = dirs.first().map(String::as_str) else {
        return Err(format!("Worldgen definition '{}' has no category", stem));
    };
    let domain = match kind {
        "biomes" => "biome",
        "biome_sets" => "biome_set",
        "caves" => "cave",
        "climate_profiles" => "climate",
        "noise_fields" => "field",
        "ores" => "ore",
        "planet_profiles" => "planet_profile",
        "spawns" => "spawn",
        "structures" => "structure",
        "terrain_layers" => "terrain_layers",
        "vegetation" => "vegetation",
        "visual_details" => "visual_detail",
        other => return Err(format!("Unknown worldgen category '{}'", other)),
    };
    Ok(format!("{}/{}", domain, stem))
}

fn strip_def_suffix(file_name: &str) -> String {
    let stem = file_name.strip_suffix(".ron").unwrap_or(file_name);
    stem.split('.').next().unwrap_or(stem).to_string()
}

fn singular_item_dirs(dirs: &[String]) -> Vec<String> {
    dirs.iter()
        .map(|dir| match dir.as_str() {
            "blocks" => "block".to_string(),
            "resources" => "resource".to_string(),
            "tools" => "tool".to_string(),
            "weapons" => "weapon".to_string(),
            "consumables" => "consumable".to_string(),
            other => other.to_string(),
        })
        .collect()
}

fn join_domain(domain: &str, dirs: &[String], stem: &str) -> String {
    let mut parts = Vec::with_capacity(dirs.len() + 2);
    parts.push(domain.to_string());
    parts.extend(dirs.iter().cloned());
    parts.push(stem.to_string());
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::PackLoader;
    use std::path::Path;

    #[test]
    fn core_pack_parses_all_schema_groups() {
        let core_pack_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core");
        let pack = PackLoader::load_from_dir(&core_pack_dir).expect("core pack");
        let procedural =
            PackLoader::load_procedural_from_dir(&core_pack_dir).expect("procedural pack");

        assert!(pack.blocks.len() >= 20);
        assert!(pack.materials.len() >= 10);
        assert!(pack.items.len() >= 20);
        assert!(!pack.entities.is_empty());
        assert!(!pack.loot.is_empty());
        assert!(!pack.skeletons.is_empty());
        assert!(!pack.prop_collections.is_empty());
        assert!(!pack.vegetation_catalogs.is_empty());
        assert!(!pack.tag_sets.is_empty());
        let voxel_assets = pack.voxel_assets.expect("voxel asset registry");
        assert_eq!(voxel_assets.asset_count as usize, voxel_assets.assets.len());
        assert_eq!(voxel_assets.generated_from, "media/voxel");
        for asset in &voxel_assets.assets {
            assert!(
                asset.id.0.starts_with("core:voxel/"),
                "bad voxel id {}",
                asset.id.0
            );
            assert!(
                core_pack_dir.join(&asset.path).exists(),
                "missing voxel asset {}",
                asset.path
            );
        }

        assert!(!procedural.planets.is_empty());
        assert!(!procedural.fields.is_empty());
        assert!(!procedural.biomes.is_empty());
        assert!(!procedural.vegetation.is_empty());
    }
}
