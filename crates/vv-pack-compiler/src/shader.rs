//! Shared shader-pack resolver.
//!
//! Both `vv-render` (at runtime, to load the engine-required shaders) and
//! `vv-pack-doctor` (at validation time, to compile every WGSL file in a
//! pack and naga-parse it) use this module so a single source of truth
//! drives include expansion, pack-stack override resolution and overrun
//! reporting.
//!
//! Stack convention: lowest priority first (typically `core`), then mod
//! packs on top. File resolution walks the stack from highest to lowest
//! priority — the first pack that contains a given relative path wins.

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

/// Identity of a single pack in the shader-resolution stack.
///
/// `root` is the pack directory (the parent of `render/shaders`), not the
/// shader directory itself. `name` is a short human label used in override
/// reports and error messages (`core`, `better_clouds`, ...).
#[derive(Debug, Clone)]
pub struct PackShaderRoot {
    pub name: String,
    pub root: PathBuf,
}

impl PackShaderRoot {
    pub fn new(name: impl Into<String>, root: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            root: root.into(),
        }
    }
}

/// One shadowed override entry: a higher-priority pack provided a file that
/// also existed in one or more lower packs.
#[derive(Debug, Clone)]
pub struct ShaderOverride {
    pub relative_path: PathBuf,
    pub winner: String,
    pub shadowed: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ShaderOverrideReport {
    pub overrides: Vec<ShaderOverride>,
}

impl ShaderOverrideReport {
    pub fn is_empty(&self) -> bool {
        self.overrides.is_empty()
    }
}

/// One WGSL file discovered through the pack stack, with the pack that wins
/// for that relative path.
#[derive(Debug, Clone)]
pub struct EnumeratedShader {
    pub relative_path: PathBuf,
    pub winner_pack: String,
    pub winner_abs_path: PathBuf,
}

#[derive(Debug, Clone)]
struct ResolvedPack {
    name: String,
    shader_root: PathBuf,
}

#[derive(Debug, Clone)]
struct FileResolution {
    winner: String,
    shadowed: Vec<String>,
}

/// Stateful resolver that expands shader sources across a pack stack.
///
/// Call `expand` for each top-level shader you need, then read the override
/// report via `override_report` (or consume it via `into_override_report`).
pub struct ShaderResolver {
    packs: Vec<ResolvedPack>,
    resolutions: HashMap<PathBuf, FileResolution>,
}

impl ShaderResolver {
    pub fn new(packs: &[PackShaderRoot]) -> Result<Self, String> {
        if packs.is_empty() {
            return Err("shader pack stack is empty".to_string());
        }

        let resolved: Vec<ResolvedPack> = packs
            .iter()
            .map(|p| ResolvedPack {
                name: p.name.clone(),
                shader_root: p.root.join("render").join("shaders"),
            })
            .collect();

        let base = &resolved[0];
        if !base.shader_root.is_dir() {
            return Err(format!(
                "base shader pack '{}' missing render/shaders at {}",
                base.name,
                base.shader_root.display()
            ));
        }

        Ok(Self {
            packs: resolved,
            resolutions: HashMap::new(),
        })
    }

    /// Expand a shader at the given relative path (rooted at the per-pack
    /// `render/shaders/`) into a single WGSL string with `#include` directives
    /// inlined. Each include is itself resolved through the pack stack.
    pub fn expand(&mut self, rel_path: &Path) -> Result<String, String> {
        let mut include_stack = Vec::new();
        let mut seen = HashSet::new();
        expand_file(
            &self.packs,
            rel_path,
            &mut include_stack,
            &mut seen,
            &mut self.resolutions,
        )
    }

    pub fn override_report(&self) -> ShaderOverrideReport {
        build_override_report(&self.resolutions)
    }

    pub fn into_override_report(self) -> ShaderOverrideReport {
        build_override_report(&self.resolutions)
    }

    /// Every relative path that has been resolved through this resolver so
    /// far (whether the file came from the base pack or was overridden).
    /// Useful for "which includes did my passes actually reach" queries.
    pub fn resolved_paths(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = self.resolutions.keys().cloned().collect();
        paths.sort();
        paths
    }

    /// Walk every pack in the stack and list every `.wgsl` file in
    /// `render/shaders/**`, deduplicated across packs (the highest-priority
    /// pack wins for each relative path). Results are sorted by relative
    /// path for deterministic output.
    pub fn enumerate_wgsl(&self) -> Result<Vec<EnumeratedShader>, String> {
        let mut seen: HashMap<PathBuf, EnumeratedShader> = HashMap::new();

        for pack in self.packs.iter().rev() {
            if !pack.shader_root.is_dir() {
                continue;
            }
            let mut stack = vec![pack.shader_root.clone()];
            while let Some(dir) = stack.pop() {
                let entries = std::fs::read_dir(&dir)
                    .map_err(|e| format!("cannot read shader dir {}: {}", dir.display(), e))?;
                for entry in entries {
                    let entry = entry
                        .map_err(|e| format!("cannot read entry in {}: {}", dir.display(), e))?;
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                        continue;
                    }
                    if path.extension().and_then(|e| e.to_str()) != Some("wgsl") {
                        continue;
                    }
                    let rel = path
                        .strip_prefix(&pack.shader_root)
                        .map_err(|e| format!("strip prefix failed: {e}"))?
                        .to_path_buf();
                    // Higher-priority packs were enumerated first because we
                    // iterate `packs.iter().rev()`; do not overwrite.
                    seen.entry(rel.clone()).or_insert(EnumeratedShader {
                        relative_path: rel,
                        winner_pack: pack.name.clone(),
                        winner_abs_path: path,
                    });
                }
            }
        }

        let mut out: Vec<EnumeratedShader> = seen.into_values().collect();
        out.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        Ok(out)
    }

    pub fn pack_names(&self) -> Vec<&str> {
        self.packs.iter().map(|p| p.name.as_str()).collect()
    }
}

fn build_override_report(resolutions: &HashMap<PathBuf, FileResolution>) -> ShaderOverrideReport {
    let mut overrides: Vec<ShaderOverride> = resolutions
        .iter()
        .filter(|(_, r)| !r.shadowed.is_empty())
        .map(|(rel, r)| ShaderOverride {
            relative_path: rel.clone(),
            winner: r.winner.clone(),
            shadowed: r.shadowed.clone(),
        })
        .collect();
    overrides.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    ShaderOverrideReport { overrides }
}

fn resolve_file(
    packs: &[ResolvedPack],
    rel_path: &Path,
) -> Result<(PathBuf, FileResolution), String> {
    let mut winner: Option<(String, PathBuf)> = None;
    let mut shadowed: Vec<String> = Vec::new();

    for pack in packs.iter().rev() {
        let abs = pack.shader_root.join(rel_path);
        if abs.is_file() {
            if winner.is_none() {
                winner = Some((pack.name.clone(), abs));
            } else {
                shadowed.push(pack.name.clone());
            }
        }
    }

    let (winner_name, winner_path) = winner.ok_or_else(|| {
        let stack = packs
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "shader file '{}' not found in any pack ({})",
            rel_path.display(),
            stack
        )
    })?;

    Ok((
        winner_path,
        FileResolution {
            winner: winner_name,
            shadowed,
        },
    ))
}

fn expand_file(
    packs: &[ResolvedPack],
    rel_path: &Path,
    include_stack: &mut Vec<PathBuf>,
    seen: &mut HashSet<PathBuf>,
    resolutions: &mut HashMap<PathBuf, FileResolution>,
) -> Result<String, String> {
    let rel_path = normalize_relative(rel_path)?;
    if !seen.insert(rel_path.clone()) {
        return Ok(String::new());
    }
    if include_stack.contains(&rel_path) {
        return Err(format!(
            "cyclic WGSL include: {} -> {}",
            include_stack
                .iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" -> "),
            rel_path.display()
        ));
    }

    let (abs, resolution) = resolve_file(packs, &rel_path)?;
    resolutions.entry(rel_path.clone()).or_insert(resolution);

    let source = std::fs::read_to_string(&abs)
        .map_err(|e| format!("cannot read shader {}: {}", abs.display(), e))?;
    include_stack.push(rel_path.clone());

    let parent = rel_path.parent().unwrap_or_else(|| Path::new(""));
    let mut expanded = String::with_capacity(source.len() + 512);
    for (line_index, line) in source.lines().enumerate() {
        if let Some(include) = parse_include(line) {
            let include_rel = if include.starts_with("include/") || include.starts_with("passes/") {
                PathBuf::from(include)
            } else {
                parent.join(include)
            };
            let nested = expand_file(packs, &include_rel, include_stack, seen, resolutions)
                .map_err(|e| {
                    format!(
                        "{}:{} include '{}': {}",
                        rel_path.display(),
                        line_index + 1,
                        include,
                        e
                    )
                })?;
            expanded.push_str(&nested);
            expanded.push('\n');
        } else {
            expanded.push_str(line);
            expanded.push('\n');
        }
    }

    include_stack.pop();
    Ok(expanded)
}

fn parse_include(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("#include")?.trim();
    rest.strip_prefix('"')?
        .split_once('"')
        .map(|(path, _)| path)
}

fn normalize_relative(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        return Err(format!(
            "absolute include path is forbidden: {}",
            path.display()
        ));
    }
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    return Err(format!("include escapes shader root: {}", path.display()));
                }
            }
            other => {
                return Err(format!(
                    "unsupported shader path component {:?} in {}",
                    other,
                    path.display()
                ));
            }
        }
    }
    Ok(out)
}

/// Run naga's WGSL front-end on an already-expanded shader source. Returns
/// a human-readable error string with naga's full diagnostic on failure.
pub fn validate_wgsl(source: &str, label: &str) -> Result<(), String> {
    naga::front::wgsl::parse_str(source)
        .map(|_| ())
        .map_err(|e| {
            format!(
                "WGSL parse failed for {label}:\n{}",
                e.emit_to_string(source)
            )
        })
}
