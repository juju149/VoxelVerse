//! Filesystem scan + tolerant parse of a content pack.
//!
//! Everything the rest of Pack Doctor inspects is collected here in a single
//! pass. The scan never fails on a bad file; failures are turned into
//! `ParseError`s and propagated to the diagnostics layer so the report keeps
//! going.

use std::path::{Path, PathBuf};

use vv_content_schema::{RawObjectDef, RawPackManifest, RawVoxelModelManifest};

use crate::parse::{pack_relative, parse_string, parse_value, read_typed, ParseError};

pub struct PackScan {
    pub pack_root: PathBuf,
    pub manifest: Option<RawPackManifest>,
    pub namespace: String,

    pub objects: Vec<ParsedObject>,
    pub voxel_models: Vec<ParsedVoxelModel>,
    pub world_files: Vec<ParsedWorldFile>,
    pub render: RenderScan,

    pub texture_files: Vec<TextureFile>,
    pub voxel_files: Vec<MediaFile>,
    pub wgsl_files: Vec<MediaFile>,

    pub all_ron_files: Vec<PathBuf>,
    pub parse_errors: Vec<ParseError>,
}

pub struct ParsedObject {
    pub id: String,
    pub rel_path: String,
    pub def: RawObjectDef,
}

pub struct ParsedVoxelModel {
    pub id: String,
    pub rel_path: String,
    pub def: RawVoxelModelManifest,
}

/// Files under `defs/world/...` use a small custom schema that is still being
/// stabilised. We keep them as untyped values so reference checks can still
/// look up short names while the typed schemas catch up.
pub struct ParsedWorldFile {
    pub id: String,
    pub rel_path: String,
    pub category: WorldCategory,
    pub value: ron::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorldCategory {
    Biome,
    BiomeSet,
    Caves,
    Climate,
    Noise,
    Ores,
    Planets,
    Props,
    Structures,
    Terrain,
    Vegetation,
    PropScatter,
    Other,
}

#[derive(Default)]
pub struct RenderScan {
    pub wgsl_files: Vec<MediaFile>,
    pub ron_files: Vec<String>,
}

pub struct TextureFile {
    pub rel_path: String,
    pub abs_path: PathBuf,
}

pub struct MediaFile {
    pub rel_path: String,
    pub abs_path: PathBuf,
}

impl PackScan {
    pub fn scan(pack_root: &Path) -> Result<Self, String> {
        let pack_root = pack_root
            .canonicalize()
            .map_err(|e| format!("Cannot resolve pack root '{}': {}", pack_root.display(), e))?;

        let mut errors = Vec::new();
        let manifest = load_manifest(&pack_root, &mut errors);
        let namespace = manifest
            .as_ref()
            .map(|m| m.namespace.clone())
            .unwrap_or_else(|| derive_namespace(&pack_root));

        let mut all_ron = Vec::new();
        collect_files(&pack_root, "ron", &mut all_ron, SOURCE_SKIP);

        let objects = load_objects(&pack_root, &mut errors);
        let voxel_models = load_voxel_models(&pack_root, &mut errors);
        let world_files = load_world(&pack_root, &mut errors);
        let render = load_render(&pack_root, &namespace, &mut errors);

        let mut texture_files = Vec::new();
        collect_textures(
            &pack_root.join("media").join("textures"),
            &pack_root,
            &mut texture_files,
        );
        let mut voxel_files = Vec::new();
        collect_media(
            &pack_root.join("media").join("voxel"),
            "vox",
            &pack_root,
            &mut voxel_files,
        );
        let mut wgsl_files = Vec::new();
        collect_media(
            &pack_root.join("render"),
            "wgsl",
            &pack_root,
            &mut wgsl_files,
        );

        Ok(Self {
            pack_root,
            manifest,
            namespace,
            objects,
            voxel_models,
            world_files,
            render,
            texture_files,
            voxel_files,
            wgsl_files,
            all_ron_files: all_ron,
            parse_errors: errors,
        })
    }

    pub fn relative(&self, abs_path: &Path) -> String {
        pack_relative(&self.pack_root, abs_path)
    }
}

const SOURCE_SKIP: &[&str] = &["source", "generated/reports", "target"];

fn load_manifest(pack_root: &Path, errors: &mut Vec<ParseError>) -> Option<RawPackManifest> {
    let path = pack_root.join("pack.ron");
    if !path.exists() {
        errors.push(ParseError {
            rel_path: "pack.ron".to_string(),
            line: 0,
            column: 0,
            message: "missing pack manifest".to_string(),
            suggestion: Some("create pack.ron at the pack root".to_string()),
        });
        return None;
    }
    match read_typed::<RawPackManifest>(pack_root, &path) {
        Ok(m) => Some(m),
        Err(e) => {
            errors.push(e);
            None
        }
    }
}

fn load_objects(pack_root: &Path, errors: &mut Vec<ParseError>) -> Vec<ParsedObject> {
    let root = pack_root.join("defs").join("objects");
    let mut paths = Vec::new();
    collect_files(&root, "ron", &mut paths, &[]);
    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        let rel = pack_relative(pack_root, &path);
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                errors.push(ParseError {
                    rel_path: rel,
                    line: 0,
                    column: 0,
                    message: format!("cannot read file: {e}"),
                    suggestion: None,
                });
                continue;
            }
        };
        match parse_string::<RawObjectDef>(rel.clone(), &text) {
            Ok(def) => {
                let id = derive_object_id(pack_root, &path);
                out.push(ParsedObject {
                    id,
                    rel_path: rel,
                    def,
                });
            }
            Err(e) => errors.push(e),
        }
    }
    out
}

fn load_voxel_models(pack_root: &Path, errors: &mut Vec<ParseError>) -> Vec<ParsedVoxelModel> {
    let root = pack_root.join("defs").join("voxel_models");
    let mut paths = Vec::new();
    collect_files(&root, "ron", &mut paths, &[]);
    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        let rel = pack_relative(pack_root, &path);
        match read_typed::<RawVoxelModelManifest>(pack_root, &path) {
            Ok(def) => {
                let id = derive_voxel_model_id(pack_root, &path);
                out.push(ParsedVoxelModel {
                    id,
                    rel_path: rel,
                    def,
                });
            }
            Err(e) => errors.push(e),
        }
    }
    out
}

fn load_world(pack_root: &Path, errors: &mut Vec<ParseError>) -> Vec<ParsedWorldFile> {
    let root = pack_root.join("defs").join("world");
    let mut paths = Vec::new();
    collect_files(&root, "ron", &mut paths, &[]);
    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        let rel = pack_relative(pack_root, &path);
        let category = classify_world_path(&rel);
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                errors.push(ParseError {
                    rel_path: rel,
                    line: 0,
                    column: 0,
                    message: format!("cannot read file: {e}"),
                    suggestion: None,
                });
                continue;
            }
        };
        match parse_value(rel.clone(), &text) {
            Ok(value) => {
                let id = derive_world_id(pack_root, &path);
                out.push(ParsedWorldFile {
                    id,
                    rel_path: rel,
                    category,
                    value,
                });
            }
            Err(e) => errors.push(e),
        }
    }
    out
}

fn load_render(pack_root: &Path, namespace: &str, errors: &mut Vec<ParseError>) -> RenderScan {
    let root = pack_root.join("render");
    let mut scan = RenderScan::default();
    if !root.exists() {
        return scan;
    }
    let mut wgsl = Vec::new();
    collect_media(&root.join("shaders"), "wgsl", pack_root, &mut wgsl);
    scan.wgsl_files = wgsl;

    let mut ron_paths = Vec::new();
    collect_files(&root, "ron", &mut ron_paths, &[]);
    ron_paths.sort();
    scan.ron_files = ron_paths
        .iter()
        .map(|path| pack_relative(pack_root, path))
        .collect();
    let _ = (namespace, errors);
    scan
}

fn collect_files(root: &Path, ext: &str, out: &mut Vec<PathBuf>, skip: &[&str]) {
    if !root.exists() {
        return;
    }
    walk(root, root, ext, out, skip);
}

fn walk(base: &Path, dir: &Path, ext: &str, out: &mut Vec<PathBuf>, skip: &[&str]) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if is_private_dir(&path) {
                continue;
            }
            let rel = pack_relative(base, &path);
            if skip
                .iter()
                .any(|s| rel == *s || rel.starts_with(&format!("{s}/")))
            {
                continue;
            }
            walk(base, &path, ext, out, skip);
        } else if path.extension().and_then(|s| s.to_str()) == Some(ext) {
            out.push(path);
        }
    }
}

fn collect_textures(textures_root: &Path, pack_root: &Path, out: &mut Vec<TextureFile>) {
    if !textures_root.exists() {
        return;
    }
    let mut paths = Vec::new();
    walk(textures_root, textures_root, "png", &mut paths, &[]);
    paths.sort();
    for path in paths {
        let rel = pack_relative(pack_root, &path);
        out.push(TextureFile {
            rel_path: rel,
            abs_path: path,
        });
    }
}

fn collect_media(root: &Path, ext: &str, pack_root: &Path, out: &mut Vec<MediaFile>) {
    if !root.exists() {
        return;
    }
    let mut paths = Vec::new();
    walk(root, root, ext, &mut paths, &[]);
    paths.sort();
    for path in paths {
        let rel = pack_relative(pack_root, &path);
        out.push(MediaFile {
            rel_path: rel,
            abs_path: path,
        });
    }
}

fn derive_namespace(pack_root: &Path) -> String {
    pack_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("core")
        .to_string()
}

fn derive_object_id(pack_root: &Path, path: &Path) -> String {
    let rel = path
        .strip_prefix(pack_root.join("defs").join("objects"))
        .ok();
    let Some(rel) = rel else {
        return pack_relative(pack_root, path);
    };
    let mut parts: Vec<String> = rel
        .iter()
        .filter_map(|p| p.to_str())
        .map(str::to_string)
        .collect();
    if let Some(last) = parts.pop() {
        let stem = last
            .trim_end_matches(".ron")
            .split('.')
            .next()
            .unwrap_or(&last)
            .to_string();
        parts.push(stem);
    }
    format!("object/{}", parts.join("/"))
}

fn derive_world_id(pack_root: &Path, path: &Path) -> String {
    let rel = match path.strip_prefix(pack_root.join("defs").join("world")) {
        Ok(r) => r,
        Err(_) => return pack_relative(pack_root, path),
    };
    let mut parts: Vec<String> = rel
        .iter()
        .filter_map(|p| p.to_str())
        .map(str::to_string)
        .collect();
    if let Some(last) = parts.pop() {
        let stem = last
            .trim_end_matches(".ron")
            .split('.')
            .next()
            .unwrap_or(&last)
            .to_string();
        parts.push(stem);
    }
    format!("world/{}", parts.join("/"))
}

fn derive_voxel_model_id(pack_root: &Path, path: &Path) -> String {
    let rel = match path.strip_prefix(pack_root.join("defs").join("voxel_models")) {
        Ok(r) => r,
        Err(_) => return pack_relative(pack_root, path),
    };
    let mut parts: Vec<String> = rel
        .iter()
        .filter_map(|p| p.to_str())
        .map(str::to_string)
        .collect();
    if let Some(last) = parts.pop() {
        let stem = last
            .trim_end_matches(".ron")
            .split('.')
            .next()
            .unwrap_or(&last)
            .to_string();
        parts.push(stem);
    }
    format!("voxel_model/{}", parts.join("/"))
}

fn classify_world_path(rel: &str) -> WorldCategory {
    let parts: Vec<&str> = rel.split('/').collect();
    let sub = parts.get(2).copied().unwrap_or("");
    let leaf = parts.last().copied().unwrap_or("");
    if leaf.ends_with(".prop_scatter.ron") {
        return WorldCategory::PropScatter;
    }
    match sub {
        "biomes" => WorldCategory::Biome,
        "biome_sets" => WorldCategory::BiomeSet,
        "caves" => WorldCategory::Caves,
        "climate" => WorldCategory::Climate,
        "noise" => WorldCategory::Noise,
        "ores" => WorldCategory::Ores,
        "planets" => WorldCategory::Planets,
        "props" => WorldCategory::Props,
        "structures" => WorldCategory::Structures,
        "terrain_layers" => WorldCategory::Terrain,
        "vegetation" => WorldCategory::Vegetation,
        "fauna" => WorldCategory::Other,
        _ => WorldCategory::Other,
    }
}

fn is_private_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| name.starts_with('_'))
}
