use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;
use vv_schema::manifest::PackManifest;

use crate::{
    error::{PackLoadError, PackLoadResult},
    loaded_pack::LoadedPack,
    raw_content::{RawContentSet, RawDocument},
};

pub fn load_pack(pack_root: &Path) -> PackLoadResult<LoadedPack> {
    if !pack_root.is_dir() {
        return Err(PackLoadError::InvalidPackDirectory {
            path: pack_root.to_path_buf(),
        });
    }

    let manifest_path = pack_root.join("pack.ron");
    if !manifest_path.is_file() {
        return Err(PackLoadError::MissingManifest {
            pack_dir: pack_root.to_path_buf(),
        });
    }

    let manifest = parse_file::<PackManifest>(&manifest_path)?;
    let namespace = manifest.namespace.clone();

    let content = RawContentSet {
        manifest,
        pack_root: pack_root.to_path_buf(),
        blocks: load_many(pack_root, &namespace, "defs/blocks")?,
        items: load_many(pack_root, &namespace, "defs/items")?,
        entities: load_many(pack_root, &namespace, "defs/entities")?,
        placeables: load_many(pack_root, &namespace, "defs/placeables")?,
        recipes: load_many(pack_root, &namespace, "defs/recipes")?,
        loot_tables: load_many(pack_root, &namespace, "defs/loot_tables")?,
        tags: load_many(pack_root, &namespace, "defs/tags")?,
        lang: load_many(pack_root, &namespace, "lang")?,
        gameplay_settings: load_many(pack_root, &namespace, "defs/settings/gameplay")?,
        balance_settings: load_many(pack_root, &namespace, "defs/settings/balance")?,
        world_settings: load_many(pack_root, &namespace, "defs/settings/world")?,
        ui_themes: load_many(pack_root, &namespace, "defs/ui")?,
        universes: load_many(pack_root, &namespace, "defs/worldgen/universe")?,
        climate_tags: load_many(pack_root, &namespace, "defs/worldgen/climate/tags")?,
        climate_curves: load_many(pack_root, &namespace, "defs/worldgen/climate/curves")?,
        climate_transitions: load_many(pack_root, &namespace, "defs/worldgen/climate/transitions")?,
        planet_types: load_many(pack_root, &namespace, "defs/worldgen/planet_types")?,
        biomes: load_many(pack_root, &namespace, "defs/worldgen/biomes")?,
        flora: load_many(pack_root, &namespace, "defs/worldgen/flora")?,
        fauna: load_many(pack_root, &namespace, "defs/worldgen/fauna")?,
        ores: load_many(pack_root, &namespace, "defs/worldgen/ores")?,
        structures: load_many(pack_root, &namespace, "defs/worldgen/structures")?,
        weather: load_many(pack_root, &namespace, "defs/worldgen/weather")?,
        noise: load_many(pack_root, &namespace, "defs/worldgen/noise")?,
    };

    Ok(LoadedPack { content })
}

fn load_many<T>(
    pack_root: &Path,
    namespace: &str,
    relative: &str,
) -> PackLoadResult<Vec<RawDocument<T>>>
where
    T: DeserializeOwned,
{
    let mut target = pack_root.join(relative);
    if !target.exists() {
        let ron_path = pack_root.join(format!("{relative}.ron"));
        if ron_path.exists() {
            target = ron_path;
        }
    }
    let files = ron_files(&target)?;
    files
        .into_iter()
        .map(|path| {
            let value = parse_file::<T>(&path)?;
            let relative_path = path
                .strip_prefix(pack_root)
                .unwrap_or(path.as_path())
                .to_path_buf();
            Ok(RawDocument {
                pack_namespace: namespace.to_owned(),
                source_path: path,
                relative_path,
                value,
            })
        })
        .collect()
}

fn ron_files(path: &Path) -> PackLoadResult<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(
            if path.extension().and_then(|ext| ext.to_str()) == Some("ron") {
                vec![path.to_path_buf()]
            } else {
                Vec::new()
            },
        );
    }

    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    collect_ron_files(path, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_ron_files(path: &Path, files: &mut Vec<PathBuf>) -> PackLoadResult<()> {
    let entries = fs::read_dir(path).map_err(|source| PackLoadError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| PackLoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_ron_files(&path, files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("ron") {
            files.push(path);
        }
    }

    Ok(())
}

fn parse_file<T>(path: &Path) -> PackLoadResult<T>
where
    T: DeserializeOwned,
{
    let source = fs::read_to_string(path).map_err(|source| PackLoadError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    ron::from_str(&source).map_err(|source| PackLoadError::Ron {
        path: path.to_path_buf(),
        source,
    })
}
