use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

use crate::pipeline::graph::ShaderPath;

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

#[derive(Debug, Clone)]
struct ResolvedPack {
    name: String,
    shader_root: PathBuf,
}

pub(crate) struct ShaderLibrary {
    packs: Vec<ResolvedPack>,
    sources: HashMap<ShaderPath, String>,
}

impl ShaderLibrary {
    /// Load the engine-required shaders from an ordered pack stack.
    ///
    /// Stack convention: lowest priority first (typically `core`), then mod
    /// packs on top. File resolution walks the stack from highest to lowest
    /// priority — the first pack that contains a given relative path wins.
    /// Lower packs that also contain the same path are recorded as
    /// `shadowed` in the override report.
    pub fn load_stack(
        packs: &[PackShaderRoot],
    ) -> Result<(Self, ShaderOverrideReport), String> {
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

        // The base pack (lowest priority) must exist — it's the fallback for
        // every required file. Higher-priority packs may omit the shaders
        // directory entirely if they don't override anything.
        let base = &resolved[0];
        if !base.shader_root.is_dir() {
            return Err(format!(
                "base shader pack '{}' missing render/shaders at {}",
                base.name,
                base.shader_root.display()
            ));
        }

        let mut sources = HashMap::with_capacity(ShaderPath::REQUIRED.len());
        let mut resolutions: HashMap<PathBuf, FileResolution> = HashMap::new();

        for shader in ShaderPath::REQUIRED {
            let mut include_stack = Vec::new();
            let mut seen = HashSet::new();
            let source = expand_file(
                &resolved,
                Path::new(shader.relative()),
                &mut include_stack,
                &mut seen,
                &mut resolutions,
            )?;
            sources.insert(*shader, source);
        }

        let report = build_override_report(&resolutions);

        Ok((
            Self {
                packs: resolved,
                sources,
            },
            report,
        ))
    }

    pub fn source(&self, shader: ShaderPath) -> Result<&str, String> {
        self.sources.get(&shader).map(String::as_str).ok_or_else(|| {
            format!(
                "shader '{}' was not loaded from pack stack [{}]",
                shader.relative(),
                self.pack_names_csv()
            )
        })
    }

    fn pack_names_csv(&self) -> String {
        self.packs
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Debug, Clone)]
struct FileResolution {
    winner: String,
    shadowed: Vec<String>,
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

/// Walk the pack stack from highest to lowest priority and return the
/// absolute path of the first match, plus the names of any lower packs
/// that also contain the file.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn core_only_stack() -> Vec<PackShaderRoot> {
        let core = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core");
        vec![PackShaderRoot::new("core", core)]
    }

    #[test]
    fn all_required_core_shaders_expand_and_parse_as_wgsl() {
        let (lib, report) =
            ShaderLibrary::load_stack(&core_only_stack()).expect("shader library loads");
        assert!(
            report.is_empty(),
            "core-only stack should produce no overrides, got {:?}",
            report.overrides
        );
        for shader in ShaderPath::REQUIRED {
            let source = lib.source(*shader).expect("shader present");
            naga::front::wgsl::parse_str(source).unwrap_or_else(|error| {
                panic!("WGSL parse failed for {}: {:?}", shader.relative(), error)
            });
        }
    }

    #[test]
    fn missing_base_pack_is_a_hard_error() {
        let bogus = vec![PackShaderRoot::new(
            "ghost",
            Path::new("c:/does/not/exist"),
        )];
        let err = match ShaderLibrary::load_stack(&bogus) {
            Ok(_) => panic!("expected error for missing base pack"),
            Err(e) => e,
        };
        assert!(
            err.contains("missing render/shaders"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn empty_stack_is_a_hard_error() {
        let err = match ShaderLibrary::load_stack(&[]) {
            Ok(_) => panic!("expected error for empty stack"),
            Err(e) => e,
        };
        assert!(err.contains("empty"), "unexpected error: {err}");
    }
}

#[cfg(test)]
mod shader_interface_contract_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn shader_interface_contract_smoke_test_parses() {
        let core_pack = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core");
        let shader_root = core_pack.join("render").join("shaders");

        let pack = ResolvedPack {
            name: "core".to_string(),
            shader_root,
        };
        let mut include_stack = Vec::new();
        let mut seen = HashSet::new();
        let mut resolutions = HashMap::new();
        let source = expand_file(
            std::slice::from_ref(&pack),
            Path::new("passes/debug/shader_contract_smoke.frag.wgsl"),
            &mut include_stack,
            &mut seen,
            &mut resolutions,
        )
        .expect("shader contract smoke shader expands");

        naga::front::wgsl::parse_str(&source).expect("shader contract smoke shader parses");
    }

    #[test]
    fn material_atlas_contract_declares_canonical_bindings() {
        let core_pack = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core");
        let shader_root = core_pack.join("render").join("shaders");

        let source =
            std::fs::read_to_string(shader_root.join("include/interface/material_atlas.wgsl"))
                .expect("material atlas include is readable");

        for binding in [
            "@group(2) @binding(0) var vv_material_albedo: texture_2d_array<f32>;",
            "@group(2) @binding(1) var vv_material_normal: texture_2d_array<f32>;",
            "@group(2) @binding(2) var vv_material_roughness: texture_2d_array<f32>;",
            "@group(2) @binding(3) var vv_material_sampler: sampler;",
            "@group(2) @binding(4) var<storage, read> vv_material_flat_colors: array<vec4<f32>>;",
        ] {
            assert!(
                source.contains(binding),
                "material atlas include missing binding: {binding}"
            );
        }
    }

    #[test]
    fn textured_mesh_fragments_use_material_atlas_sampler() {
        let core_pack = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core");
        let shader_root = core_pack.join("render").join("shaders");

        for rel in ["passes/terrain/terrain.frag.wgsl", "passes/ui/ui.frag.wgsl"] {
            let source = std::fs::read_to_string(shader_root.join(rel))
                .unwrap_or_else(|error| panic!("cannot read {rel}: {error}"));
            assert!(
                source.contains("include/interface/material_atlas.wgsl"),
                "{rel} must include the material atlas contract"
            );
            assert!(
                source.contains("vv_sample_material("),
                "{rel} must sample block materials through the canonical helper"
            );
        }
    }
}
