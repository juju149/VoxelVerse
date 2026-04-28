use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    error::{PackLoadError, PackLoadResult},
    loaded_pack::{LoadedPack, PackLoadOrder},
    reader::load_pack,
};

pub fn load_packs_from_assets(assets_root: &Path) -> PackLoadResult<PackLoadOrder> {
    let pack_dirs = discover_packs(&assets_root.join("packs"))?;
    let mut packs = Vec::with_capacity(pack_dirs.len());
    for pack_dir in pack_dirs {
        packs.push(load_pack(&pack_dir)?);
    }
    Ok(PackLoadOrder::new(order_packs(packs)))
}

pub fn discover_packs(packs_root: &Path) -> PackLoadResult<Vec<PathBuf>> {
    if !packs_root.exists() {
        return Ok(Vec::new());
    }

    let mut packs = Vec::new();
    let entries = fs::read_dir(packs_root).map_err(|source| PackLoadError::Io {
        path: packs_root.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| PackLoadError::Io {
            path: packs_root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() && path.join("pack.ron").is_file() {
            packs.push(path);
        }
    }

    packs.sort();
    Ok(packs)
}

fn order_packs(mut packs: Vec<LoadedPack>) -> Vec<LoadedPack> {
    packs.sort_by(|a, b| {
        a.content
            .manifest
            .namespace
            .cmp(&b.content.manifest.namespace)
    });
    packs
}
