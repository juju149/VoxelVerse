use std::path::{Path, PathBuf};
use vv_content_schema::*;

pub struct LoadedPack {
    pub manifest: RawPackManifest,
    pub objects: Vec<(String, RawObjectDef)>,
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
    pub vox_prop_scatters: Vec<(String, RawVoxPropScatterDef)>,
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
            objects: load_typed_tree(&defs.join("objects"), &defs, &namespace)?,
            voxel_assets: load_optional_file(&voxel_assets_path)?,
        })
    }

    pub fn load_procedural_from_dir(pack_dir: &Path) -> Result<RawProceduralPack, String> {
        let namespace = namespace_from_dir(pack_dir)?;
        let defs = pack_dir.join("defs");

        // Support both the old `defs/worldgen/` layout and the new `defs/world/` layout.
        // Each sub-directory is tried in order; the first that exists wins.
        let root_old = defs.join("worldgen");
        let root_new = defs.join("world");

        // Helper: pick the directory that exists, or fall back to the other.
        let pick = |old_sub: &str, new_sub: &str| -> PathBuf {
            let old = root_old.join(old_sub);
            let new = root_new.join(new_sub);
            if new.exists() {
                new
            } else {
                old
            }
        };

        if !root_old.exists() && !root_new.exists() {
            return Ok(RawProceduralPack::default());
        }

        Ok(RawProceduralPack {
            planets: load_typed_tree(&pick("planet_profiles", "planets"), &defs, &namespace)?,
            fields: load_typed_tree(&pick("noise_fields", "noise"), &defs, &namespace)?,
            climates: load_typed_tree(&pick("climate_profiles", "climate"), &defs, &namespace)?,
            biome_sets: load_typed_tree(&pick("biome_sets", "biome_sets"), &defs, &namespace)?,
            biomes: load_typed_tree(&pick("biomes", "biomes"), &defs, &namespace)?,
            terrain_layers: load_typed_tree(&pick("terrain_layers", "terrain"), &defs, &namespace)?,
            ores: load_typed_tree(&pick("ores", "ores"), &defs, &namespace)?,
            caves: load_typed_tree(&pick("caves", "caves"), &defs, &namespace)?,
            vegetation: load_typed_tree_filtered(
                &pick("vegetation", "vegetation"),
                &defs,
                &namespace,
                Some(".vegetation."),
            )?,
            structures: load_typed_tree(&pick("structures", "structures"), &defs, &namespace)?,
            fauna: load_typed_tree(&pick("spawns", "spawns"), &defs, &namespace)?,
            vox_prop_scatters: load_typed_tree_filtered(
                &pick("prop_scatters", "vegetation"),
                &defs,
                &namespace,
                Some(".prop_scatter."),
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
    let opts =
        ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
    opts.from_str(&text)
        .or_else(|_| opts.from_str(strip_outer_type_name(&text)))
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
    // Skip BOM then skip leading line comments and whitespace to find the type name.
    let mut cursor = text.trim_start_matches('\u{feff}');
    loop {
        cursor = cursor.trim_start_matches(|c: char| c.is_whitespace());
        if cursor.starts_with("//") {
            cursor = match cursor.find('\n') {
                Some(nl) => &cursor[nl + 1..],
                None => return text,
            };
        } else {
            break;
        }
    }
    let Some(open) = cursor.find('(') else {
        return text;
    };
    if cursor[..open]
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
        && open > 0
    {
        &cursor[open..]
    } else {
        text
    }
}

fn load_typed_tree<T: serde::de::DeserializeOwned>(
    dir: &Path,
    defs_root: &Path,
    namespace: &str,
) -> Result<Vec<(String, T)>, String> {
    load_typed_tree_filtered(dir, defs_root, namespace, None)
}

/// Like `load_typed_tree` but only processes files whose name contains
/// `required_suffix` (e.g. `".vegetation.ron"`, `".prop_scatter.ron"`).
fn load_typed_tree_filtered<T: serde::de::DeserializeOwned>(
    dir: &Path,
    defs_root: &Path,
    namespace: &str,
    required_suffix: Option<&str>,
) -> Result<Vec<(String, T)>, String> {
    let mut paths = Vec::new();
    collect_ron_files(dir, &mut paths, required_suffix)?;
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

fn collect_ron_files(
    dir: &Path,
    out: &mut Vec<PathBuf>,
    suffix_filter: Option<&str>,
) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in
        std::fs::read_dir(dir).map_err(|e| format!("Cannot read {}: {}", dir.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Dir entry error in {}: {}", dir.display(), e))?;
        let path = entry.path();
        if path.is_dir() {
            collect_ron_files(&path, out, suffix_filter)?;
        } else {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let is_ron = name.ends_with(".ron");
            let passes_filter = suffix_filter.map(|s| name.contains(s)).unwrap_or(true);
            if is_ron && passes_filter {
                out.push(path);
            }
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
        "objects" => join_domain("object", dirs, &stem),
        "worldgen" => derive_world_key(dirs, &stem, false)?,
        "world" => derive_world_key(dirs, &stem, true)?,
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

fn derive_world_key(dirs: &[String], stem: &str, new_layout: bool) -> Result<String, String> {
    let Some(kind) = dirs.first().map(String::as_str) else {
        return Err(format!("Worldgen definition '{}' has no category", stem));
    };
    let domain = match kind {
        // Old layout sub-dirs
        "biomes" => "biome",
        "biome_sets" => "biome_set",
        "caves" => "cave",
        "climate_profiles" => "climate", // old name
        "noise_fields" => "field",       // old name
        "ores" => "ore",
        "planet_profiles" => "planet_profile", // old name
        "spawns" => "spawn",
        "structures" => "structure",
        "terrain_layers" => "terrain_layers", // old name
        "vegetation" => "vegetation",
        "visual_details" => {
            return Err(
                "'visual_details' directory is obsolete; rename to 'prop_scatters'".to_string(),
            )
        }
        "prop_scatters" => "prop_scatter",
        // New layout sub-dirs (defs/world/)
        "climate" => "climate",
        "noise" => "field",
        "planets" => "planet_profile",
        "terrain" => "terrain_layers",
        "props" => "prop_collection",
        other => {
            if new_layout {
                // Unknown sub-dir in new layout — skip gracefully.
                return Ok(format!("world/{}/{}", other, stem));
            }
            return Err(format!("Unknown worldgen category '{}'", other));
        }
    };
    Ok(format!("{}/{}", domain, stem))
}

fn strip_def_suffix(file_name: &str) -> String {
    let stem = file_name.strip_suffix(".ron").unwrap_or(file_name);
    stem.split('.').next().unwrap_or(stem).to_string()
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

        assert!(!pack.objects.is_empty(), "core pack must load objects");
        assert!(
            pack.objects.len() >= 20,
            "expected >= 20 objects, got {}",
            pack.objects.len()
        );

        if let Some(voxel_assets) = &pack.voxel_assets {
            assert_eq!(voxel_assets.asset_count as usize, voxel_assets.assets.len());
            for asset in &voxel_assets.assets {
                assert!(asset.id.0.starts_with("core:voxel/"));
            }
        }

        assert!(
            !procedural.planets.is_empty(),
            "must have at least one planet"
        );
        assert!(!procedural.fields.is_empty(), "must have noise fields");
        assert!(!procedural.biomes.is_empty(), "must have biomes");
    }
}
