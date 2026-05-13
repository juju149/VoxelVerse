use super::ContentCompiler;
use std::path::Path;
use vv_content_schema::ContentRef;
use vv_pack_loader::PackLoader;

fn load_core_pack() -> vv_pack_loader::LoadedPack {
    let core_pack_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core");
    PackLoader::load_from_dir(&core_pack_dir).expect("core pack")
}

#[test]
#[ignore = "core pack is mid-migration; re-enable once object files parse cleanly"]
fn core_render_content_compiles() {
    let pack = load_core_pack();
    let render = ContentCompiler::compile_render_content(&pack).expect("render content");
    assert!(render.registry.shader_module_count() >= 12);
    assert!(render.registry.technique_count() >= 6);
    assert!(render
        .registry
        .technique_by_key("core:render/techniques/terrain/terrain_opaque")
        .is_some());
    assert!(render
        .registry
        .profile_by_key("core:render/profiles/balanced")
        .is_some());
    assert!(render
        .registry
        .material_family_by_key("core:render/material_families/voxel_surface")
        .is_some());
}

#[test]
#[ignore = "core pack is mid-migration; re-enable once object files parse cleanly"]
fn render_compilation_rejects_unknown_shader_module() {
    let mut pack = load_core_pack();
    let (_, technique) = pack
        .render
        .techniques
        .iter_mut()
        .find(|(key, _)| key == "core:render/techniques/terrain/terrain_opaque")
        .expect("terrain technique");
    technique.stages.vertex = "core:render/shader_modules/missing".to_string();

    let errors = ContentCompiler::compile_render_content(&pack)
        .expect_err("unknown shader module should fail");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("stages.vertex") && e.contains("missing")),
        "got: {errors:?}"
    );
}

#[test]
#[ignore = "core pack is mid-migration; re-enable once object files parse cleanly"]
fn render_compilation_rejects_unknown_material_family() {
    let mut pack = load_core_pack();
    let (_, technique) = pack
        .render
        .techniques
        .iter_mut()
        .find(|(key, _)| key == "core:render/techniques/terrain/terrain_opaque")
        .expect("terrain technique");
    technique.material_family = "core:render/material_families/missing".to_string();

    let errors = ContentCompiler::compile_render_content(&pack)
        .expect_err("unknown material family should fail");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("material_family") && e.contains("missing")),
        "got: {errors:?}"
    );
}

#[test]
#[ignore = "core pack is mid-migration; re-enable once object files parse cleanly"]
fn render_compilation_rejects_shader_import_cycle() {
    let mut pack = load_core_pack();
    let constants = "core:render/shader_modules/math/constants";
    let tonemap = "core:render/shader_modules/color/tonemap";
    let module = pack
        .render
        .shader_modules
        .iter_mut()
        .find(|module| module.key == constants)
        .expect("constants module");
    module.metadata.imports.push(tonemap.to_string());

    let errors = ContentCompiler::compile_render_content(&pack)
        .expect_err("shader import cycle should fail");
    assert!(
        errors.iter().any(|e| e.contains("cycle")),
        "got: {errors:?}"
    );
}
