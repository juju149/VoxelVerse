//! Pack scanning: parses content via the regular `PackLoader` and walks the
//! filesystem to enumerate raw media. Other check modules consume the
//! resulting `PackScan` and never re-read the pack from disk.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use vv_content_schema::{
    ContentRef, RawBlockDef, RawItemDef, RawLootTableDef, RawMaterialDef, RawPackManifest,
    RawRecipeDef, RawTagSetDef,
};
use vv_pack_loader::{LoadedPack, PackLoader, RawProceduralPack};

/// A scanned pack, ready for inspection.
pub struct PackScan {
    pub pack_root: PathBuf,
    pub manifest: RawPackManifest,
    pub blocks: Vec<(String, RawBlockDef)>,
    pub materials: Vec<(String, RawMaterialDef)>,
    pub items: Vec<(String, RawItemDef)>,
    pub loot: Vec<(String, RawLootTableDef)>,
    pub recipes: Vec<(String, RawRecipeDef)>,
    pub tag_sets: Vec<(String, RawTagSetDef)>,
    pub procedural: RawProceduralPack,
    pub block_model_ids: HashSet<String>,
    pub texture_files: Vec<TextureFile>,
    pub voxel_files: Vec<PathBuf>,
    pub all_ron_files: Vec<PathBuf>,
}

pub struct TextureFile {
    /// Pack-relative path with forward slashes.
    pub rel_path: String,
    pub abs_path: PathBuf,
    /// `core:texture/...` reference id, derived from the path under `media/textures/`.
    pub texture_ref: String,
}

impl PackScan {
    pub fn scan(pack_root: &Path) -> Result<Self, String> {
        let pack_root = pack_root
            .canonicalize()
            .map_err(|e| format!("Cannot resolve pack root '{}': {}", pack_root.display(), e))?;

        let loaded: LoadedPack = PackLoader::load_from_dir(&pack_root)?;
        let procedural = PackLoader::load_procedural_from_dir(&pack_root)
            .unwrap_or_else(|_| RawProceduralPack::default());

        let block_model_ids = loaded
            .block_models
            .iter()
            .map(|(id, _)| id.clone())
            .collect::<HashSet<_>>();

        let texture_files = collect_texture_files(&pack_root, &loaded.manifest.namespace)?;
        let voxel_files = collect_files_with_ext(&pack_root.join("media").join("voxel"), "vox")?;
        let all_ron_files = collect_files_with_ext(&pack_root, "ron")?;

        Ok(Self {
            pack_root,
            manifest: loaded.manifest,
            blocks: loaded.blocks,
            materials: loaded.materials,
            items: loaded.items,
            loot: loaded.loot,
            recipes: loaded.recipes,
            tag_sets: loaded.tag_sets,
            procedural,
            block_model_ids,
            texture_files,
            voxel_files,
            all_ron_files,
        })
    }

    /// `true` if the given content reference resolves to a known content id.
    /// Tag and icon references are recognized as namespaced even when they
    /// are not declared, since the loader does not load them explicitly.
    pub fn block_id_exists(&self, id: &str) -> bool {
        self.blocks.iter().any(|(known, _)| known == id)
    }

    pub fn item_id_exists(&self, id: &str) -> bool {
        self.items.iter().any(|(known, _)| known == id)
    }

    pub fn material_id_exists(&self, id: &str) -> bool {
        self.materials.iter().any(|(known, _)| known == id)
    }

    pub fn loot_id_exists(&self, id: &str) -> bool {
        self.loot.iter().any(|(known, _)| known == id)
    }

    pub fn block_model_id_exists(&self, id: &str) -> bool {
        self.block_model_ids.contains(id)
    }

    pub fn texture_id_exists(&self, id: &str) -> bool {
        self.texture_files.iter().any(|t| t.texture_ref == id)
    }

    pub fn relative(&self, path: &Path) -> String {
        path.strip_prefix(&self.pack_root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
    }
}

/// Trait alias for content references so we can pass `ContentRef` or `&str`
/// interchangeably to the check modules.
pub trait AsRefStr {
    fn as_str(&self) -> &str;
}

impl AsRefStr for ContentRef {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRefStr for str {
    fn as_str(&self) -> &str {
        self
    }
}

fn collect_texture_files(pack_root: &Path, namespace: &str) -> Result<Vec<TextureFile>, String> {
    let textures_root = pack_root.join("media").join("textures");
    if !textures_root.exists() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    walk_dir(&textures_root, &mut |path| {
        if path.extension().and_then(|s| s.to_str()) != Some("png") {
            return Ok(());
        }
        let rel = path
            .strip_prefix(&textures_root)
            .map_err(|_| format!("texture path outside root: {}", path.display()))?;
        let rel_no_ext = rel.with_extension("");
        let id_path = rel_no_ext.to_string_lossy().replace('\\', "/");
        let texture_ref = format!("{}:texture/{}", namespace, id_path);
        let rel_full = path
            .strip_prefix(pack_root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        out.push(TextureFile {
            rel_path: rel_full,
            abs_path: path.to_path_buf(),
            texture_ref,
        });
        Ok(())
    })?;
    out.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    Ok(out)
}

fn collect_files_with_ext(root: &Path, ext: &str) -> Result<Vec<PathBuf>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    walk_dir(root, &mut |path| {
        if path.extension().and_then(|s| s.to_str()) == Some(ext) {
            out.push(path.to_path_buf());
        }
        Ok(())
    })?;
    out.sort();
    Ok(out)
}

fn walk_dir(
    dir: &Path,
    visit: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Cannot read directory {}: {}", dir.display(), e))?;
    for entry in entries {
        let entry =
            entry.map_err(|e| format!("Dir entry error in {}: {}", dir.display(), e))?;
        let path = entry.path();
        if path.is_dir() {
            walk_dir(&path, visit)?;
        } else if path.is_file() {
            visit(&path)?;
        }
    }
    Ok(())
}
