//! Render-feature compilation.
//!
//! Authoring side: `defs/render/features/*.render_feature.ron` (declared via
//! `vv_content_schema::RawRenderFeatureDef`).
//!
//! Compilation produces a [`RenderFeatureRegistry`] of typed, validated
//! features. Every `ShaderRef` is resolved through [`crate::shader::ShaderResolver`]
//! so the same pack-stack semantics that drive runtime shader loading also
//! drive feature validation. Each referenced shader is naga-parsed and the
//! declared entry points are confirmed to exist.
//!
//! The compiler does **not** touch GPU state — it produces data only. The
//! runtime (`vv-render`) is responsible for translating the registry into
//! pipelines.

use std::path::{Path, PathBuf};

use vv_content_schema::{
    check_format_version, RawAllowedBindGroup, RawBlendMode, RawDepthMode, RawRenderFeatureCost,
    RawRenderFeatureDef, RawRenderFeatureKind, RawRenderFeatureSlot, RawRenderProfileDef,
    RawRenderQualityClass, RawRenderTargetKind, RENDER_FEATURE_FORMAT_VERSION,
    RENDER_PROFILE_FORMAT_VERSION,
};

use crate::shader::{self, PackShaderRoot, ShaderResolver};

// ── Compiled types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CompiledRenderFeature {
    pub source_rel_path: String,
    pub name: String,
    pub kind: RawRenderFeatureKind,
    pub slot: RawRenderFeatureSlot,
    pub vertex_shader: CompiledShaderRef,
    pub fragment_shader: Option<CompiledShaderRef>,
    pub vertex_entry_point: String,
    pub fragment_entry_point: Option<String>,
    pub bind_groups: Vec<RawAllowedBindGroup>,
    pub target: RawRenderTargetKind,
    pub blend: RawBlendMode,
    pub depth: RawDepthMode,
    pub min_profile: RawRenderQualityClass,
    pub cost: RawRenderFeatureCost,
    pub can_disable_on_low: bool,
}

#[derive(Debug, Clone)]
pub struct CompiledShaderRef {
    /// Original authoring string (`"core:shader/passes/post/fullscreen.vert"`).
    pub raw: String,
    /// Pack the reference resolved to.
    pub pack_namespace: String,
    /// Path relative to `<pack>/render/shaders/`, with the `.wgsl` extension.
    pub relative_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CompiledRenderProfile {
    pub source_rel_path: String,
    pub name: String,
    pub quality_class: RawRenderQualityClass,
    pub enable_features: Vec<String>,
    pub disable_features: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RenderFeatureRegistry {
    pub features: Vec<CompiledRenderFeature>,
    pub profiles: Vec<CompiledRenderProfile>,
}

impl RenderFeatureRegistry {
    pub fn is_empty(&self) -> bool {
        self.features.is_empty() && self.profiles.is_empty()
    }

    pub fn feature_by_name(&self, name: &str) -> Option<&CompiledRenderFeature> {
        self.features.iter().find(|f| f.name == name)
    }
}

// ── Compilation API ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RawRenderFeatureInput<'a> {
    pub rel_path: String,
    pub def: &'a RawRenderFeatureDef,
}

#[derive(Debug, Clone)]
pub struct RawRenderProfileInput<'a> {
    pub rel_path: String,
    pub def: &'a RawRenderProfileDef,
}

#[derive(Debug, Clone)]
pub struct RenderFeatureCompileError {
    pub source_rel_path: String,
    pub feature_name: Option<String>,
    pub message: String,
}

pub fn compile_render_features(
    pack_stack: &[PackShaderRoot],
    features: &[RawRenderFeatureInput<'_>],
    profiles: &[RawRenderProfileInput<'_>],
) -> (RenderFeatureRegistry, Vec<RenderFeatureCompileError>) {
    let mut errors = Vec::new();
    let mut registry = RenderFeatureRegistry::default();

    let mut resolver = match ShaderResolver::new(pack_stack) {
        Ok(r) => r,
        Err(e) => {
            errors.push(RenderFeatureCompileError {
                source_rel_path: "<pack-stack>".to_string(),
                feature_name: None,
                message: format!("cannot build shader resolver: {e}"),
            });
            return (registry, errors);
        }
    };
    let allowed_packs: Vec<String> =
        resolver.pack_names().iter().map(|s| s.to_string()).collect();

    for input in features {
        match compile_one_feature(input, &allowed_packs, &mut resolver) {
            Ok(feat) => registry.features.push(feat),
            Err(e) => errors.push(e),
        }
    }

    for input in profiles {
        match compile_one_profile(input) {
            Ok(profile) => registry.profiles.push(profile),
            Err(e) => errors.push(e),
        }
    }

    validate_profile_feature_refs(&registry, &mut errors);

    (registry, errors)
}

fn compile_one_feature(
    input: &RawRenderFeatureInput<'_>,
    allowed_packs: &[String],
    resolver: &mut ShaderResolver,
) -> Result<CompiledRenderFeature, RenderFeatureCompileError> {
    let def = input.def;
    let make_err = |msg: String| RenderFeatureCompileError {
        source_rel_path: input.rel_path.clone(),
        feature_name: Some(def.name.clone()),
        message: msg,
    };

    check_format_version(
        def.format_version,
        RENDER_FEATURE_FORMAT_VERSION,
        "render_feature",
        &def.name,
    )
    .map_err(make_err)?;

    if def.bind_groups.is_empty() {
        return Err(make_err(
            "render feature must declare at least one bind group".to_string(),
        ));
    }
    let mut seen: Vec<RawAllowedBindGroup> = Vec::with_capacity(def.bind_groups.len());
    for bg in &def.bind_groups {
        if seen.contains(bg) {
            return Err(make_err(format!("duplicate bind group {:?}", bg)));
        }
        seen.push(*bg);
    }

    let vertex_ref = parse_shader_ref(&def.vertex, allowed_packs).map_err(make_err)?;
    validate_shader_exists_and_entry(
        resolver,
        &vertex_ref,
        &def.entry_points.vertex,
        naga::ShaderStage::Vertex,
        &input.rel_path,
        &def.name,
    )?;

    let fragment_ref = match (&def.fragment, &def.entry_points.fragment) {
        (Some(raw), Some(entry)) => {
            let r = parse_shader_ref(raw, allowed_packs).map_err(make_err)?;
            validate_shader_exists_and_entry(
                resolver,
                &r,
                entry,
                naga::ShaderStage::Fragment,
                &input.rel_path,
                &def.name,
            )?;
            Some(r)
        }
        (None, None) => None,
        (Some(_), None) | (None, Some(_)) => {
            return Err(make_err(
                "fragment shader and fragment entry-point must be set together".to_string(),
            ));
        }
    };

    validate_kind_target_blend_combo(def.kind, def.target, def.blend).map_err(make_err)?;

    Ok(CompiledRenderFeature {
        source_rel_path: input.rel_path.clone(),
        name: def.name.clone(),
        kind: def.kind,
        slot: def.slot,
        vertex_shader: vertex_ref,
        fragment_shader: fragment_ref,
        vertex_entry_point: def.entry_points.vertex.clone(),
        fragment_entry_point: def.entry_points.fragment.clone(),
        bind_groups: def.bind_groups.clone(),
        target: def.target,
        blend: def.blend,
        depth: def.depth,
        min_profile: def.quality.min_profile,
        cost: def.quality.cost,
        can_disable_on_low: def.quality.can_disable_on_low,
    })
}

fn compile_one_profile(
    input: &RawRenderProfileInput<'_>,
) -> Result<CompiledRenderProfile, RenderFeatureCompileError> {
    let def = input.def;
    let make_err = |msg: String| RenderFeatureCompileError {
        source_rel_path: input.rel_path.clone(),
        feature_name: Some(def.name.clone()),
        message: msg,
    };

    check_format_version(
        def.format_version,
        RENDER_PROFILE_FORMAT_VERSION,
        "render_profile",
        &def.name,
    )
    .map_err(make_err)?;

    // A feature can be enabled or disabled but not both.
    for name in &def.enable_features {
        if def.disable_features.contains(name) {
            return Err(make_err(format!(
                "feature '{name}' appears in both enable_features and disable_features"
            )));
        }
    }

    Ok(CompiledRenderProfile {
        source_rel_path: input.rel_path.clone(),
        name: def.name.clone(),
        quality_class: def.quality_class,
        enable_features: def.enable_features.clone(),
        disable_features: def.disable_features.clone(),
    })
}

fn validate_profile_feature_refs(
    registry: &RenderFeatureRegistry,
    errors: &mut Vec<RenderFeatureCompileError>,
) {
    for profile in &registry.profiles {
        for name in profile
            .enable_features
            .iter()
            .chain(profile.disable_features.iter())
        {
            if registry.feature_by_name(name).is_none() {
                errors.push(RenderFeatureCompileError {
                    source_rel_path: profile.source_rel_path.clone(),
                    feature_name: Some(profile.name.clone()),
                    message: format!(
                        "profile references unknown feature '{name}' \
                         (no manifest declares this name)"
                    ),
                });
            }
        }
    }
}

fn parse_shader_ref(raw: &str, allowed_packs: &[String]) -> Result<CompiledShaderRef, String> {
    let (ns, rest) = raw
        .split_once(':')
        .ok_or_else(|| format!("shader ref '{raw}' must be '<namespace>:shader/<path>'"))?;
    if !allowed_packs.iter().any(|p| p == ns) {
        return Err(format!(
            "shader ref '{raw}': namespace '{ns}' is not in the pack stack \
             ({})",
            allowed_packs.join(", ")
        ));
    }
    let rel = rest
        .strip_prefix("shader/")
        .ok_or_else(|| format!("shader ref '{raw}' must use the 'shader/' domain prefix"))?;
    if rel.is_empty() {
        return Err(format!("shader ref '{raw}' has empty path after 'shader/'"));
    }
    // Authors omit the `.wgsl` extension by convention; add it back.
    let mut path = PathBuf::from(rel);
    if path.extension().and_then(|e| e.to_str()) != Some("wgsl") {
        let new_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => format!("{name}.wgsl"),
            None => return Err(format!("shader ref '{raw}' has no file name")),
        };
        path.set_file_name(new_name);
    }
    Ok(CompiledShaderRef {
        raw: raw.to_string(),
        pack_namespace: ns.to_string(),
        relative_path: path,
    })
}

fn validate_shader_exists_and_entry(
    resolver: &mut ShaderResolver,
    shader_ref: &CompiledShaderRef,
    entry_point: &str,
    expected_stage: naga::ShaderStage,
    source_rel_path: &str,
    feature_name: &str,
) -> Result<(), RenderFeatureCompileError> {
    let make_err = |msg: String| RenderFeatureCompileError {
        source_rel_path: source_rel_path.to_string(),
        feature_name: Some(feature_name.to_string()),
        message: msg,
    };

    let source = resolver
        .expand(&shader_ref.relative_path)
        .map_err(|e| make_err(format!("shader ref '{}': {e}", shader_ref.raw)))?;
    shader::validate_wgsl(&source, &shader_ref.raw).map_err(make_err)?;

    let module = naga::front::wgsl::parse_str(&source).map_err(|e| {
        make_err(format!(
            "shader ref '{}' naga parse failed: {}",
            shader_ref.raw,
            e.emit_to_string(&source)
        ))
    })?;
    let found = module
        .entry_points
        .iter()
        .find(|ep| ep.name == entry_point);
    match found {
        None => {
            let available = module
                .entry_points
                .iter()
                .map(|ep| format!("{} ({:?})", ep.name, ep.stage))
                .collect::<Vec<_>>()
                .join(", ");
            Err(make_err(format!(
                "shader '{}' does not export entry point '{entry_point}' (available: [{available}])",
                shader_ref.raw,
            )))
        }
        Some(ep) if ep.stage != expected_stage => Err(make_err(format!(
            "shader '{}' entry point '{entry_point}' has stage {:?}, expected {:?}",
            shader_ref.raw, ep.stage, expected_stage
        ))),
        Some(_) => Ok(()),
    }
}

fn validate_kind_target_blend_combo(
    kind: RawRenderFeatureKind,
    target: RawRenderTargetKind,
    blend: RawBlendMode,
) -> Result<(), String> {
    use RawBlendMode::*;
    use RawRenderFeatureKind::*;
    use RawRenderTargetKind::*;

    // Engine-side topology rules. Anything not listed is forbidden.
    let ok = match (kind, target) {
        (PostProcess, SceneHdr | PostPingPong | SwapchainLdr) => true,
        (SkyLayer, SceneHdr) => true,
        (WeatherLayer, SceneHdr | PostPingPong) => true,
        (WaterSurface, SceneHdr) => true,
        (FoliageSurface, SceneHdr) => true,
        (TerrainMaterial, SceneHdr) => true,
        (DebugView, SceneHdr | SwapchainLdr) => true,
        (UiEffect, SwapchainLdr) => true,
        _ => false,
    };
    if !ok {
        return Err(format!(
            "feature kind {:?} cannot render into target {:?}",
            kind, target
        ));
    }

    // Depth-target writes must use additive/none blend (no alpha into depth).
    if target == Depth && !matches!(blend, None | Additive) {
        return Err(format!(
            "blend {:?} not allowed when writing into the depth target",
            blend
        ));
    }

    Ok(())
}

// ── File-system entrypoint ───────────────────────────────────────────────────

/// Walks `<pack>/defs/render/features/*.render_feature.ron` and
/// `<pack>/defs/render/profiles/*.render_profile.ron`, parses each file, and
/// hands them to [`compile_render_features`].
///
/// On parse errors a `RenderFeatureCompileError` is emitted for the offending
/// file and compilation continues — the registry then reflects everything
/// that *did* compile cleanly.
pub fn compile_pack_render_features(
    pack_stack: &[PackShaderRoot],
    primary_pack_root: &Path,
) -> (RenderFeatureRegistry, Vec<RenderFeatureCompileError>) {
    let mut errors = Vec::new();
    let mut raw_features: Vec<(String, RawRenderFeatureDef)> = Vec::new();
    let mut raw_profiles: Vec<(String, RawRenderProfileDef)> = Vec::new();

    let features_dir = primary_pack_root.join("defs/render/features");
    collect_typed_ron::<RawRenderFeatureDef>(
        &features_dir,
        "render_feature",
        primary_pack_root,
        &mut raw_features,
        &mut errors,
    );

    let profiles_dir = primary_pack_root.join("defs/render/profiles");
    collect_typed_ron::<RawRenderProfileDef>(
        &profiles_dir,
        "render_profile",
        primary_pack_root,
        &mut raw_profiles,
        &mut errors,
    );

    let feature_inputs: Vec<RawRenderFeatureInput<'_>> = raw_features
        .iter()
        .map(|(rel, def)| RawRenderFeatureInput {
            rel_path: rel.clone(),
            def,
        })
        .collect();
    let profile_inputs: Vec<RawRenderProfileInput<'_>> = raw_profiles
        .iter()
        .map(|(rel, def)| RawRenderProfileInput {
            rel_path: rel.clone(),
            def,
        })
        .collect();

    let (registry, compile_errs) =
        compile_render_features(pack_stack, &feature_inputs, &profile_inputs);
    errors.extend(compile_errs);
    (registry, errors)
}

fn collect_typed_ron<T: for<'de> serde::Deserialize<'de>>(
    dir: &Path,
    kind: &str,
    pack_root: &Path,
    out: &mut Vec<(String, T)>,
    errors: &mut Vec<RenderFeatureCompileError>,
) {
    if !dir.is_dir() {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            errors.push(RenderFeatureCompileError {
                source_rel_path: relative_to(dir, pack_root),
                feature_name: None,
                message: format!("cannot read {kind} dir: {e}"),
            });
            return;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ron") {
            continue;
        }
        let rel = relative_to(&path, pack_root);
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                errors.push(RenderFeatureCompileError {
                    source_rel_path: rel,
                    feature_name: None,
                    message: format!("cannot read {kind} file: {e}"),
                });
                continue;
            }
        };
        let opts = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
        match opts.from_str::<T>(&text) {
            Ok(def) => out.push((rel, def)),
            Err(e) => errors.push(RenderFeatureCompileError {
                source_rel_path: rel,
                feature_name: None,
                message: format!("{kind} parse failed: {e}"),
            }),
        }
    }
}

fn relative_to(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    struct TempPack {
        root: PathBuf,
    }
    impl Drop for TempPack {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn unique_dir(prefix: &str) -> PathBuf {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("vv-feature-test-{prefix}-{n}"))
    }

    fn write_file(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn make_pack_with_fullscreen_post() -> TempPack {
        let root = unique_dir("happy");
        fs::create_dir_all(&root).unwrap();

        // Minimal vertex shader that produces a fullscreen triangle.
        let vert = r#"
@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    return vec4<f32>(pos[idx], 0.0, 1.0);
}
"#;
        let frag = r#"
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.5, 0.25, 1.0);
}
"#;
        write_file(
            &root.join("render/shaders/passes/post/fullscreen.vert.wgsl"),
            vert,
        );
        write_file(
            &root.join("render/shaders/features/example/example.frag.wgsl"),
            frag,
        );

        write_file(
            &root.join("defs/render/features/example.render_feature.ron"),
            r#"(
                format_version: 1,
                name: "Example Post Process",
                kind: post_process,
                slot: after_scene_before_post,
                vertex: "tmp:shader/passes/post/fullscreen.vert",
                fragment: "tmp:shader/features/example/example.frag",
                entry_points: (
                    vertex: "vs_main",
                    fragment: "fs_main",
                ),
                bind_groups: [global],
                target: scene_hdr,
                blend: alpha,
                depth: none,
                quality: (
                    min_profile: high,
                    cost: medium,
                    can_disable_on_low: true,
                ),
            )"#,
        );

        write_file(
            &root.join("defs/render/profiles/cinematic.render_profile.ron"),
            r#"(
                format_version: 1,
                name: "cinematic",
                quality_class: ultra,
                enable_features: ["Example Post Process"],
            )"#,
        );

        TempPack { root }
    }

    fn pack_stack(root: &Path) -> Vec<PackShaderRoot> {
        vec![PackShaderRoot::new("tmp", root.to_path_buf())]
    }

    #[test]
    fn happy_path_compiles_feature_and_profile() {
        let pack = make_pack_with_fullscreen_post();
        let stack = pack_stack(&pack.root);
        let (registry, errors) = compile_pack_render_features(&stack, &pack.root);
        assert!(errors.is_empty(), "expected no errors, got {errors:#?}");
        assert_eq!(registry.features.len(), 1);
        assert_eq!(registry.profiles.len(), 1);
        let f = &registry.features[0];
        assert_eq!(f.name, "Example Post Process");
        assert_eq!(f.kind, RawRenderFeatureKind::PostProcess);
        assert_eq!(f.vertex_entry_point, "vs_main");
        assert_eq!(f.fragment_entry_point.as_deref(), Some("fs_main"));
        assert_eq!(f.vertex_shader.pack_namespace, "tmp");
    }

    #[test]
    fn missing_entry_point_is_a_hard_error() {
        let pack = make_pack_with_fullscreen_post();
        // Replace the manifest with one whose entry-point name is wrong.
        write_file(
            &pack.root
                .join("defs/render/features/example.render_feature.ron"),
            r#"(
                format_version: 1,
                name: "Bad Entry",
                kind: post_process,
                slot: after_scene_before_post,
                vertex: "tmp:shader/passes/post/fullscreen.vert",
                fragment: "tmp:shader/features/example/example.frag",
                entry_points: (vertex: "does_not_exist", fragment: "fs_main"),
                bind_groups: [global],
                target: scene_hdr,
                blend: alpha,
                depth: none,
                quality: (min_profile: high, cost: medium),
            )"#,
        );
        let stack = pack_stack(&pack.root);
        let (registry, errors) = compile_pack_render_features(&stack, &pack.root);
        assert!(registry.features.is_empty());
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("does not export entry point 'does_not_exist'")),
            "unexpected errors: {errors:#?}"
        );
    }

    #[test]
    fn unknown_namespace_in_shader_ref_is_rejected() {
        let pack = make_pack_with_fullscreen_post();
        write_file(
            &pack.root
                .join("defs/render/features/example.render_feature.ron"),
            r#"(
                format_version: 1,
                name: "Bad Namespace",
                kind: post_process,
                slot: after_scene_before_post,
                vertex: "ghost:shader/passes/post/fullscreen.vert",
                entry_points: (vertex: "vs_main"),
                bind_groups: [global],
                target: scene_hdr,
                blend: none,
                depth: none,
                quality: (min_profile: high, cost: cheap),
            )"#,
        );
        let stack = pack_stack(&pack.root);
        let (_registry, errors) = compile_pack_render_features(&stack, &pack.root);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("namespace 'ghost' is not in the pack stack")),
            "unexpected errors: {errors:#?}"
        );
    }

    #[test]
    fn profile_referencing_unknown_feature_is_rejected() {
        let pack = make_pack_with_fullscreen_post();
        write_file(
            &pack.root
                .join("defs/render/profiles/cinematic.render_profile.ron"),
            r#"(
                format_version: 1,
                name: "cinematic",
                quality_class: ultra,
                enable_features: ["Ghost Feature"],
            )"#,
        );
        let stack = pack_stack(&pack.root);
        let (_registry, errors) = compile_pack_render_features(&stack, &pack.root);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("references unknown feature 'Ghost Feature'")),
            "unexpected errors: {errors:#?}"
        );
    }

    #[test]
    fn invalid_kind_target_combo_is_rejected() {
        let pack = make_pack_with_fullscreen_post();
        write_file(
            &pack.root
                .join("defs/render/features/example.render_feature.ron"),
            r#"(
                format_version: 1,
                name: "Bad Combo",
                kind: ui_effect,
                slot: ui_overlay,
                vertex: "tmp:shader/passes/post/fullscreen.vert",
                fragment: "tmp:shader/features/example/example.frag",
                entry_points: (vertex: "vs_main", fragment: "fs_main"),
                bind_groups: [global],
                target: scene_hdr,
                blend: alpha,
                depth: none,
                quality: (min_profile: high, cost: cheap),
            )"#,
        );
        let stack = pack_stack(&pack.root);
        let (_registry, errors) = compile_pack_render_features(&stack, &pack.root);
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("cannot render into target")),
            "unexpected errors: {errors:#?}"
        );
    }
}
