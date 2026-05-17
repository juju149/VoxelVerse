use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

use crate::render_graph::ShaderPath;

pub(crate) struct ShaderLibrary {
    root: PathBuf,
    sources: HashMap<ShaderPath, String>,
}

impl ShaderLibrary {
    pub fn load(pack_root: &Path) -> Result<Self, String> {
        let root = pack_root.join("render").join("shaders");
        if !root.is_dir() {
            return Err(format!("render shader library missing: {}", root.display()));
        }

        let mut sources = HashMap::with_capacity(ShaderPath::REQUIRED.len());
        for shader in ShaderPath::REQUIRED {
            let mut include_stack = Vec::new();
            let source = expand_shader(&root, Path::new(shader.relative()), &mut include_stack)?;
            sources.insert(*shader, source);
        }

        Ok(Self { root, sources })
    }

    pub fn source(&self, shader: ShaderPath) -> Result<&str, String> {
        self.sources
            .get(&shader)
            .map(String::as_str)
            .ok_or_else(|| {
                format!(
                    "shader '{}' was not loaded from {}",
                    shader.relative(),
                    self.root.display()
                )
            })
    }
}

fn expand_shader(
    root: &Path,
    rel_path: &Path,
    include_stack: &mut Vec<PathBuf>,
) -> Result<String, String> {
    let mut seen = HashSet::new();
    expand_file(root, rel_path, include_stack, &mut seen)
}

fn expand_file(
    root: &Path,
    rel_path: &Path,
    include_stack: &mut Vec<PathBuf>,
    seen: &mut HashSet<PathBuf>,
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

    let abs = root.join(&rel_path);
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
            let nested = expand_file(root, &include_rel, include_stack, seen).map_err(|e| {
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

    #[test]
    fn all_required_core_shaders_expand_and_parse_as_wgsl() {
        let core_pack = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core");
        let shader_root = core_pack.join("render").join("shaders");

        for shader in ShaderPath::REQUIRED {
            let mut include_stack = Vec::new();
            let source = expand_shader(
                &shader_root,
                Path::new(shader.relative()),
                &mut include_stack,
            )
            .unwrap_or_else(|error| {
                panic!(
                    "WGSL include expansion failed for {}: {}",
                    shader.relative(),
                    error
                )
            });

            naga::front::wgsl::parse_str(&source).unwrap_or_else(|error| {
                panic!("WGSL parse failed for {}: {:?}", shader.relative(), error)
            });
        }
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

        let mut include_stack = Vec::new();
        let source = expand_shader(
            &shader_root,
            Path::new("passes/debug/shader_contract_smoke.frag.wgsl"),
            &mut include_stack,
        )
        .expect("shader contract smoke shader expands");

        naga::front::wgsl::parse_str(&source)
            .expect("shader contract smoke shader parses");
    }
}
