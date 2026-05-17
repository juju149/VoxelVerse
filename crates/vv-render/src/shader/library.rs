use std::collections::HashMap;
use std::path::Path;

use vv_pack_compiler::shader::{PackShaderRoot, ShaderOverrideReport, ShaderResolver};

use crate::pipeline::graph::ShaderPath;

pub(crate) struct ShaderLibrary {
    sources: HashMap<ShaderPath, String>,
}

impl ShaderLibrary {
    /// Load every engine-required shader from an ordered pack stack via the
    /// shared [`ShaderResolver`].
    pub fn load_stack(
        packs: &[PackShaderRoot],
    ) -> Result<(Self, ShaderOverrideReport), String> {
        let mut resolver = ShaderResolver::new(packs)?;
        let mut sources = HashMap::with_capacity(ShaderPath::REQUIRED.len());
        for shader in ShaderPath::REQUIRED {
            let source = resolver.expand(Path::new(shader.relative()))?;
            sources.insert(*shader, source);
        }
        let report = resolver.into_override_report();
        Ok((Self { sources }, report))
    }

    pub fn source(&self, shader: ShaderPath) -> Result<&str, String> {
        self.sources
            .get(&shader)
            .map(String::as_str)
            .ok_or_else(|| format!("shader '{}' was not loaded", shader.relative()))
    }
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
        let bogus = vec![PackShaderRoot::new("ghost", Path::new("c:/does/not/exist"))];
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
        let stack = vec![PackShaderRoot::new("core", core_pack)];
        let mut resolver = ShaderResolver::new(&stack).expect("resolver ready");
        let source = resolver
            .expand(Path::new("passes/debug/shader_contract_smoke.frag.wgsl"))
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
