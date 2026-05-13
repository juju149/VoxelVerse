//! RON round-trip parsing tests for every top-level schema type.
//!
//! These tests are pure schema-level — they don't touch the actual content
//! pack on disk. Each test renders a small, hand-written RON sample and
//! confirms the schema can ingest it without error. When you add a new
//! top-level field, add a sample here so the doctor isn't the only line of
//! defense.

use vv_content_schema::*;

fn parse<T: serde::de::DeserializeOwned>(name: &str, src: &str) -> T {
    ron::from_str::<T>(src).unwrap_or_else(|e| panic!("{name} sample failed to parse: {e}"))
}

// ── pack.ron ─────────────────────────────────────────────────────────────────

#[test]
fn pack_manifest_parses() {
    let src = r#"PackManifest(
        format_version: 1,
        namespace: "core",
        display_name: "VoxelVerse Core",
        version: "0.1.0",
        kind: builtin,
        description: "Built-in foundation content.",
        authors: ["VoxelVerse Team"],
        license: "project-internal",
        load_priority: 0,
        dependencies: [],
        features: ["objects", "world"],
        content_roots: (definitions: "defs", media: "media", generated: "generated"),
        rules: (identity: path_derived, id_style: "namespace:category/name",
                runtime_loads_raw_files: false),
    )"#;
    let m: RawPackManifest = parse("pack manifest", strip_wrapper(src));
    assert_eq!(m.namespace, "core");
    assert_eq!(m.features.len(), 2);
}

// ── object.ron — minimal block ───────────────────────────────────────────────

#[test]
fn object_block_minimum_parses() {
    let src = r#"Object(
        format_version: 1,
        name: "Dirt",
        tags: ["terrain", "soil"],
        block: (
            texture: All("blocks/dirt/all"),
            hardness: 0.5,
        ),
        item: (stack: 99),
    )"#;
    let o: RawObjectDef = parse("object block", strip_wrapper(src));
    assert_eq!(o.format_version, 1);
    assert!(o.block.is_some());
    assert!(o.item.is_some());
    assert!(o.recipes.is_empty());
}

#[test]
fn object_uses_default_format_version_when_omitted() {
    let src = r#"Object(
        name: "Dirt",
        block: (texture: None),
    )"#;
    let o: RawObjectDef = parse("object default version", strip_wrapper(src));
    assert_eq!(o.format_version, OBJECT_FORMAT_VERSION);
}

// ── object.ron — recipes Vec + RawObjectRecipeKind enum ──────────────────────

#[test]
fn object_with_two_recipes_parses() {
    let src = r#"Object(
        name: "Torch",
        item: (stack: 64),
        recipes: [
            (
                station: Some("#station.construction"),
                kind: shaped((
                    pattern: ["C", "S"],
                    legend: { "C": "coal", "S": "stick" },
                )),
                output: (item: "torch", count: 4),
            ),
            (
                station: None,
                kind: shapeless((ingredients: ["resin", "stick"])),
                output: (item: "torch", count: 2),
            ),
        ],
    )"#;
    let o: RawObjectDef = parse("two recipes", strip_wrapper(src));
    assert_eq!(o.recipes.len(), 2);
    match &o.recipes[0].kind {
        RawObjectRecipeKind::Shaped(s) => {
            assert_eq!(s.pattern.len(), 2);
            assert!(s.legend.contains_key("C"));
        }
        _ => panic!("expected shaped recipe"),
    }
    match &o.recipes[1].kind {
        RawObjectRecipeKind::Shapeless(_) => {}
        _ => panic!("expected shapeless recipe"),
    }
    assert!(o.recipes[1].station.is_none());
}

#[test]
fn object_processing_recipe_parses() {
    let src = r#"Object(
        name: "Iron Ingot",
        item: (stack: 64),
        recipes: [(
            station: Some("#station.smelting"),
            kind: processing((
                inputs: [(item: "iron_ore_chunk", count: Fixed(1), chance: 1.0)],
                duration_seconds: 8.0,
            )),
            output: (item: "iron_ingot", count: 1),
        )],
    )"#;
    let o: RawObjectDef = parse("processing recipe", strip_wrapper(src));
    match &o.recipes[0].kind {
        RawObjectRecipeKind::Processing(p) => {
            assert_eq!(p.inputs.len(), 1);
            assert!((p.duration_seconds - 8.0).abs() < 1e-6);
        }
        _ => panic!("expected processing recipe"),
    }
}

// ── object.ron — station with station_tags ───────────────────────────────────

#[test]
fn object_station_with_tags_parses() {
    let src = r#"Object(
        name: "Construction Workbench",
        block: (texture: All("blocks/workbench/all")),
        item: (stack: 1),
        station: (
            type: workbench,
            station_tags: ["station.construction", "station.tool"],
            slots: Some(9),
        ),
    )"#;
    let o: RawObjectDef = parse("station tags", strip_wrapper(src));
    let s = o.station.expect("station");
    assert_eq!(s.station_tags.len(), 2);
    assert!(matches!(s.station_type, RawObjectStationType::Workbench));
}

// ── object.ron — entity with skeleton ─────────────────────────────────────────

#[test]
fn object_entity_with_skeleton_parses() {
    let src = r#"Object(
        name: "Rabbit",
        entity: (
            model: "voxel/creatures/rabbit/male",
            skeleton: "skeleton/quadruped/small",
            ai: passive_wanderer,
            health: 6,
            move_speed: 1.8,
        ),
    )"#;
    let o: RawObjectDef = parse("entity skeleton", strip_wrapper(src));
    let e = o.entity.expect("entity");
    assert_eq!(e.skeleton.as_deref(), Some("skeleton/quadruped/small"));
}

// ── world.ron — biome (just enough to confirm the schema chains) ─────────────

#[test]
fn world_biome_parses() {
    let src = r#"Biome(
        display_name: "Plains",
        surface: (top: "core:object/terrain/grass", under: "core:object/terrain/dirt",
                  depth_voxels: (2, 5)),
        terrain: (
            base_height: -0.01, amplitude: 0.15, flatness: 0.72,
            hill_field: "core:field/soft_hills", terrace_strength: 0.0,
        ),
        palette: (grass: (0.5, 0.7, 0.3), foliage: (0.3, 0.6, 0.3), fog_bias: (0.0, 0.0, 0.0)),
        placement: (vegetation_tags: [], fauna_tags: [], structure_tags: []),
    )"#;
    let _: RawBiomeProceduralDef = parse("biome", strip_wrapper(src));
}

// ── render — shader module + technique ───────────────────────────────────────

#[test]
fn render_shader_module_parses() {
    let src = r#"ShaderModule(
        language: wgsl,
        imports: [],
        feature_class: "surface",
        contracts: [],
        allow_override: true,
    )"#;
    let m: RawShaderModule = parse("shader module", strip_wrapper(src));
    assert!(m.allow_override);
}

#[test]
fn render_technique_parses() {
    let src = r#"RenderTechnique(
        label: "Terrain Opaque",
        pass: "terrain_opaque",
        stages: (
            vertex: "core:render/shader_modules/voxel/terrain_vertex",
            fragment: Some("core:render/shader_modules/surface/stylized_pbr_lite"),
        ),
        vertex_layout: "terrain_chunk_mesh",
        material_family: "core:render/material_families/voxel_surface",
        contracts: [],
        depth: (write: true, compare: less_equal),
        culling: back,
        blend: opaque,
        outputs: ["main_color"],
        features: [],
        profile_overrides: [],
    )"#;
    let _: RawRenderTechnique = parse("technique", strip_wrapper(src));
}

// ── skeleton.ron ─────────────────────────────────────────────────────────────

#[test]
fn skeleton_parses() {
    let src = r#"Skeleton(
        format_version: 1,
        display_name: "Quadruped Small",
        bones: [
            (name: "root", rest: (translation: (0.0, 0.0, 0.0),
                                  rotation: (0.0, 0.0, 0.0, 1.0))),
            (name: "spine", parent: Some("root"),
             rest: (translation: (0.0, 0.2, 0.0), rotation: (0.0, 0.0, 0.0, 1.0))),
        ],
        slots: [(name: "saddle", bone: "spine")],
    )"#;
    let s: RawSkeletonDef = parse("skeleton", strip_wrapper(src));
    assert_eq!(s.bones.len(), 2);
    assert_eq!(s.slots.len(), 1);
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn strip_wrapper(text: &str) -> &str {
    let trimmed = text.trim_start();
    if let Some(open) = trimmed.find('(') {
        if trimmed[..open]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
            && open > 0
        {
            return &trimmed[open..];
        }
    }
    text
}
