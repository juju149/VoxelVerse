use super::*;
use std::collections::HashMap;
use vv_content_schema::{
    ContentRef, RawBlockAudioDef, RawBlockGameplayDef, RawBlockPhysicalDef, RawBlockPlacement,
    RawBlockRuntimeDef, RawBlockSimulationDef,
};

fn synthetic_index() -> ContentIndex {
    ContentIndex::from_keys([
        "core:loot/blocks/empty",
        "core:sound/step/stone",
        "core:sound/break/stone",
        "core:sound/place/stone",
        "core:block_model/air",
        "core:block_model/cube",
        "core:material/test/all",
    ])
}

fn synthetic_models() -> BlockModelRegistry {
    ContentCompiler::compile_block_models(vec![
        (
            "core:block_model/air".to_string(),
            RawBlockModelDef {
                format_version: 1,
                display_name: "Air".into(),
                mesh: RawBlockMesh::None,
                collision: RawBlockCollisionShape::None,
            },
        ),
        (
            "core:block_model/cube".to_string(),
            RawBlockModelDef {
                format_version: 1,
                display_name: "Cube".to_string(),
                mesh: RawBlockMesh::Cube {
                    face_layers: vec![
                        "py".into(),
                        "ny".into(),
                        "pz".into(),
                        "nz".into(),
                        "px".into(),
                        "nx".into(),
                    ],
                    ambient_occlusion: true,
                },
                collision: RawBlockCollisionShape::FullCube,
            },
        ),
    ])
    .expect("models compile")
}

fn synthetic_materials() -> Vec<(String, RawMaterialDef)> {
    use vv_content_schema::{RawAuthoringDef, RawMaterialCategory, RawTextureSampling};
    vec![(
        "core:material/test/all".to_string(),
        RawMaterialDef {
            display_name: "Test material".into(),
            category: RawMaterialCategory::BlockSurface,
            albedo: ContentRef("core:texture/test/albedo".to_string()),
            normal: None,
            roughness: None,
            tint: None,
            render: RawRenderMode::Opaque,
            sampling: RawTextureSampling::PixelArtNearest,
            atlas: ContentRef("core:atlas/test".to_string()),
            authoring: RawAuthoringDef {
                source: String::new(),
                generated_by: String::new(),
            },
        },
    )]
}

fn cube_materials_map() -> HashMap<String, ContentRef> {
    let mut m = HashMap::new();
    for slot in ["py", "ny", "pz", "nz", "px", "nx"] {
        m.insert(
            slot.into(),
            ContentRef("core:material/test/all".to_string()),
        );
    }
    m
}

fn block(role: Option<BlockRole>) -> RawBlockDef {
    RawBlockDef {
        format_version: 1,
        display_name: "Block".to_string(),
        category: "terrain".to_string(),
        model: ContentRef("core:block_model/cube".to_string()),
        physical: RawBlockPhysicalDef {
            solid: true,
            opaque: true,
            hardness: 1.0,
            blast_resistance: 1.0,
            friction: 0.8,
            restitution: 0.0,
        },
        visual: RawBlockVisual {
            render: RawRenderMode::Opaque,
            materials: cube_materials_map(),
        },
        gameplay: RawBlockGameplayDef {
            preferred_tool: None,
            drops: ContentRef("core:loot/blocks/empty".to_string()),
            placement: RawBlockPlacement::GridAligned,
            replaceable: false,
        },
        audio: RawBlockAudioDef {
            footstep: ContentRef("core:sound/step/stone".to_string()),
            break_sound: ContentRef("core:sound/break/stone".to_string()),
            place: ContentRef("core:sound/place/stone".to_string()),
        },
        tags: Vec::new(),
        runtime: RawBlockRuntimeDef {
            role,
            reserved_id: None,
            can_target: true,
            blocks_light: true,
        },
        simulation: RawBlockSimulationDef::default(),
        states: vv_content_schema::RawBlockStates::default(),
    }
}

fn air_block() -> RawBlockDef {
    let mut a = block(None);
    a.runtime.reserved_id = Some(0);
    a.physical.solid = false;
    a.physical.opaque = false;
    a.model = ContentRef("core:block_model/air".to_string());
    a.visual.render = RawRenderMode::Invisible;
    a.visual.materials = HashMap::new();
    a
}

#[test]
fn block_compilation_requires_planet_core_role() {
    let air = air_block();
    let err = match ContentCompiler::compile_blocks(
        vec![
            ("core:block/air/air".to_string(), air),
            ("core:block/terrain/stone".to_string(), block(None)),
        ],
        synthetic_materials(),
        synthetic_models(),
        &synthetic_index(),
    ) {
        Ok(_) => panic!("missing planet core should be rejected"),
        Err(err) => err,
    };

    assert!(
        err.iter().any(|e| e.contains("planet_core")),
        "got: {err:?}"
    );
}

#[test]
fn block_compilation_rejects_dangling_drops() {
    let air = air_block();
    let mut stone = block(Some(BlockRole::PlanetCore));
    stone.gameplay.drops = ContentRef("core:loot/blocks/does_not_exist".to_string());
    let err = match ContentCompiler::compile_blocks(
        vec![
            ("core:block/air/air".to_string(), air),
            ("core:block/terrain/stone".to_string(), stone),
        ],
        synthetic_materials(),
        synthetic_models(),
        &synthetic_index(),
    ) {
        Ok(_) => panic!("dangling drops ref should be rejected"),
        Err(err) => err,
    };
    assert!(
        err.iter()
            .any(|e| e.contains("dangling reference") && e.contains("does_not_exist")),
        "expected dangling-reference error, got: {err:?}"
    );
}

#[test]
fn core_pack_solid_blocks_have_all_faces_materialized() {
    use std::path::Path;
    use vv_pack_loader::PackLoader;

    let core_pack_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core");
    let pack = PackLoader::load_from_dir(&core_pack_dir).expect("core pack");
    let index = ContentIndex::build(&pack);
    let models = ContentCompiler::compile_block_models(pack.block_models).expect("block_models");
    let blocks = ContentCompiler::compile_blocks(pack.blocks, pack.materials, models, &index)
        .expect("blocks");

    for block in blocks.blocks().iter().filter(|b| b.solid) {
        let layers = block.visual.layers;
        assert!(layers.top > 0, "{} missing top material", block.family_key);
        assert!(
            layers.bottom > 0,
            "{} missing bottom material",
            block.family_key
        );
        assert!(
            layers.front > 0,
            "{} missing front material",
            block.family_key
        );
        assert!(
            layers.back > 0,
            "{} missing back material",
            block.family_key
        );
        assert!(
            layers.left > 0,
            "{} missing left material",
            block.family_key
        );
        assert!(
            layers.right > 0,
            "{} missing right material",
            block.family_key
        );
    }
}

// -----------------------------------------------------------------
//  Variant pipeline (Jalon 5C)
// -----------------------------------------------------------------

use vv_content_schema::{RawBlockStateProperty, RawBlockStates};

fn axis_state(default: &str) -> RawBlockStates {
    let mut s = RawBlockStates::default();
    s.properties.insert(
        "axis".into(),
        RawBlockStateProperty::Axis {
            default: default.into(),
        },
    );
    s
}

fn axis_and_bool_state(axis_default: &str, bool_default: bool) -> RawBlockStates {
    let mut s = RawBlockStates::default();
    s.properties.insert(
        "axis".into(),
        RawBlockStateProperty::Axis {
            default: axis_default.into(),
        },
    );
    s.properties.insert(
        "waterlogged".into(),
        RawBlockStateProperty::Bool {
            default: bool_default,
        },
    );
    s
}

fn compile_pair(stone_states: RawBlockStates) -> BlockRegistry {
    let air = air_block();
    let mut stone = block(Some(BlockRole::PlanetCore));
    stone.states = stone_states;
    ContentCompiler::compile_blocks(
        vec![
            ("core:block/air/air".to_string(), air),
            ("core:block/terrain/stone".to_string(), stone),
        ],
        synthetic_materials(),
        synthetic_models(),
        &synthetic_index(),
    )
    .expect("compile ok")
}

fn axis_value(axis: &str) -> BlockStateValue {
    let mut m = std::collections::BTreeMap::new();
    m.insert("axis".into(), axis.into());
    BlockStateValue::from_btree(m)
}

fn axis_water_value(axis: &str, water: &str) -> BlockStateValue {
    let mut m = std::collections::BTreeMap::new();
    m.insert("axis".into(), axis.into());
    m.insert("waterlogged".into(), water.into());
    BlockStateValue::from_btree(m)
}

#[test]
fn block_with_no_states_yields_one_variant() {
    let reg = compile_pair(RawBlockStates::default());
    let stone_default = reg
        .lookup_default("core:block/terrain/stone")
        .expect("stone default");
    let fam = reg.family_of(stone_default).expect("stone family");
    assert_eq!(fam.variants.len(), 1);
    assert_eq!(fam.variants[0], stone_default);
    assert!(reg.state_of(stone_default).unwrap().is_empty());
}

#[test]
fn axis_state_yields_three_variants() {
    let reg = compile_pair(axis_state("y"));
    let fam = reg
        .family_of(reg.lookup_default("core:block/terrain/stone").unwrap())
        .unwrap();
    assert_eq!(fam.variants.len(), 3);
    let x = reg
        .lookup_variant("core:block/terrain/stone", &axis_value("x"))
        .expect("axis=x");
    let y = reg
        .lookup_variant("core:block/terrain/stone", &axis_value("y"))
        .expect("axis=y");
    let z = reg
        .lookup_variant("core:block/terrain/stone", &axis_value("z"))
        .expect("axis=z");
    assert_ne!(x, y);
    assert_ne!(y, z);
    // Default is axis=y by declaration.
    assert_eq!(fam.default_variant, y);
}

#[test]
fn bool_state_yields_two_variants_false_then_true() {
    let mut s = RawBlockStates::default();
    s.properties.insert(
        "powered".into(),
        RawBlockStateProperty::Bool { default: false },
    );
    let reg = compile_pair(s);
    let fam = reg
        .family_of(reg.lookup_default("core:block/terrain/stone").unwrap())
        .unwrap();
    assert_eq!(fam.variants.len(), 2);
    // false comes first canonically.
    let mut false_state = std::collections::BTreeMap::new();
    false_state.insert("powered".into(), "false".into());
    let id_false = reg
        .lookup_variant(
            "core:block/terrain/stone",
            &BlockStateValue::from_btree(false_state),
        )
        .unwrap();
    assert_eq!(fam.variants[0], id_false);
    assert_eq!(fam.default_variant, id_false);
}

#[test]
fn axis_and_bool_state_yields_six_variants() {
    let reg = compile_pair(axis_and_bool_state("y", false));
    let fam = reg
        .family_of(reg.lookup_default("core:block/terrain/stone").unwrap())
        .unwrap();
    assert_eq!(fam.variants.len(), 6);
    // Default = axis=y, waterlogged=false.
    let id = reg
        .lookup_variant("core:block/terrain/stone", &axis_water_value("y", "false"))
        .expect("default present");
    assert_eq!(fam.default_variant, id);
    // All combinations resolvable.
    for axis in ["x", "y", "z"] {
        for water in ["false", "true"] {
            assert!(
                reg.lookup_variant("core:block/terrain/stone", &axis_water_value(axis, water),)
                    .is_some(),
                "missing {axis}/{water}"
            );
        }
    }
}

#[test]
fn state_of_round_trips() {
    let reg = compile_pair(axis_and_bool_state("y", true));
    let id = reg
        .lookup_variant("core:block/terrain/stone", &axis_water_value("z", "true"))
        .unwrap();
    let state = reg.state_of(id).unwrap();
    assert_eq!(state.get("axis"), Some("z"));
    assert_eq!(state.get("waterlogged"), Some("true"));
}

#[test]
fn variant_compilation_is_deterministic_across_runs() {
    let a = compile_pair(axis_and_bool_state("y", false));
    let b = compile_pair(axis_and_bool_state("y", false));
    let fam_a = a
        .family_of(a.lookup_default("core:block/terrain/stone").unwrap())
        .unwrap();
    let fam_b = b
        .family_of(b.lookup_default("core:block/terrain/stone").unwrap())
        .unwrap();
    assert_eq!(fam_a.variants, fam_b.variants);
    assert_eq!(fam_a.default_variant, fam_b.default_variant);
}

#[test]
fn invalid_axis_default_is_rejected() {
    let air = air_block();
    let mut stone = block(Some(BlockRole::PlanetCore));
    stone.states = axis_state("w"); // invalid axis value
    let err = ContentCompiler::compile_blocks(
        vec![
            ("core:block/air/air".to_string(), air),
            ("core:block/terrain/stone".to_string(), stone),
        ],
        synthetic_materials(),
        synthetic_models(),
        &synthetic_index(),
    )
    .expect_err("invalid default should be rejected");
    assert!(
        err.iter().any(|e| e.contains("axis") && e.contains("'w'")),
        "expected axis-default error, got: {err:?}"
    );
}

#[test]
fn variant_count_over_cap_is_rejected() {
    // Build a 257-value Enum; cartesian product = 257 > 256.
    let values: Vec<String> = (0..257).map(|i| format!("v{i}")).collect();
    let mut s = RawBlockStates::default();
    s.properties.insert(
        "k".into(),
        RawBlockStateProperty::Enum {
            values: values.clone(),
            default: "v0".into(),
        },
    );
    let air = air_block();
    let mut stone = block(Some(BlockRole::PlanetCore));
    stone.states = s;
    let err = ContentCompiler::compile_blocks(
        vec![
            ("core:block/air/air".to_string(), air),
            ("core:block/terrain/stone".to_string(), stone),
        ],
        synthetic_materials(),
        synthetic_models(),
        &synthetic_index(),
    )
    .expect_err("over-cap variant count should be rejected");
    assert!(
        err.iter()
            .any(|e| e.contains("exceeds the per-family cap of 256")),
        "expected cap error, got: {err:?}"
    );
}
