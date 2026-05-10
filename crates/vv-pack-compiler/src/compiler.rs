use crate::block_family::{
    default_state, enumerate_variants, BlockStateValue, CompiledBlockFamily,
    MAX_VARIANTS_PER_FAMILY,
};
use crate::block_registry::{
    BlockMaterialLayers, BlockModelId, BlockModelRegistry, BlockRegistry, CompiledBlock,
    CompiledBlockModel, CompiledBlockVisual, CompiledCollision, CompiledMesh, MaterialTextureSet,
};
use crate::content_index::ContentIndex;
use std::collections::{HashMap, HashSet};
use vv_content_schema::{
    BLOCK_FORMAT_VERSION, BLOCK_MODEL_FORMAT_VERSION, BlockRole, RawBlockCollisionShape,
    RawBlockDef, RawBlockMesh, RawBlockModelDef, RawBlockVisual, RawMaterialDef, RawRenderMode,
    check_format_version,
};
use vv_voxel::VoxelId;

/// Hard limit on the total number of compiled blocks (must fit in `VoxelId(u16)`
/// minus the `UNSET` sentinel).
const MAX_BLOCK_COUNT: usize = u16::MAX as usize;
const BLOCK_COUNT_WARNING: usize = 50_000;

pub struct ContentCompiler;

impl ContentCompiler {
    /// Compile raw block-model definitions into a runtime `BlockModelRegistry`.
    /// Models are sorted alphabetically for deterministic IDs.
    pub fn compile_block_models(
        mut raw: Vec<(String, RawBlockModelDef)>,
    ) -> Result<BlockModelRegistry, Vec<String>> {
        let mut errors = Vec::new();
        raw.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut models = Vec::with_capacity(raw.len());
        for (idx, (key, def)) in raw.into_iter().enumerate() {
            if let Err(e) = check_format_version(
                def.format_version,
                BLOCK_MODEL_FORMAT_VERSION,
                "block_model",
                &key,
            ) {
                errors.push(e);
                continue;
            }

            // Validate face_layers: count matches mesh kind, no duplicates,
            // non-empty names.
            let face_layers = def.mesh.face_layers().to_vec();
            let required = def.mesh.required_face_layer_count();
            if face_layers.len() != required {
                errors.push(format!(
                    "block_model '{}': mesh '{}' requires {} face_layer(s) but has {}",
                    key,
                    def.mesh.kind_tag(),
                    required,
                    face_layers.len()
                ));
            }
            let mut seen = HashSet::new();
            for layer in &face_layers {
                if layer.is_empty() {
                    errors.push(format!(
                        "block_model '{}': face_layer name is empty",
                        key
                    ));
                }
                if !seen.insert(layer.as_str()) {
                    errors.push(format!(
                        "block_model '{}': duplicate face_layer '{}'",
                        key, layer
                    ));
                }
            }

            let mesh = match def.mesh {
                RawBlockMesh::None => CompiledMesh::None,
                RawBlockMesh::Cube { ambient_occlusion, .. } => {
                    CompiledMesh::Cube { ambient_occlusion }
                }
                RawBlockMesh::CubeColumn { ambient_occlusion, .. } => {
                    CompiledMesh::CubeColumn { ambient_occlusion }
                }
                RawBlockMesh::CrossPlane { .. } => CompiledMesh::CrossPlane,
            };
            let collision = match def.collision {
                RawBlockCollisionShape::None => CompiledCollision::None,
                RawBlockCollisionShape::FullCube => CompiledCollision::FullCube,
                RawBlockCollisionShape::SoftCube => CompiledCollision::SoftCube,
                RawBlockCollisionShape::LeafVolume => CompiledCollision::LeafVolume,
            };

            models.push(CompiledBlockModel {
                id: model_id_from_index(idx),
                key,
                mesh,
                collision,
                face_layers,
            });
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(BlockModelRegistry::new(models))
    }

    /// Compile raw block definitions into a runtime `BlockRegistry`.
    pub fn compile_blocks(
        mut raw: Vec<(String, RawBlockDef)>,
        materials: Vec<(String, RawMaterialDef)>,
        models: BlockModelRegistry,
        index: &ContentIndex,
    ) -> Result<BlockRegistry, Vec<String>> {
        let mut errors = Vec::new();
        let material_map: HashMap<String, RawMaterialDef> = materials.into_iter().collect();

        let air_pos = raw
            .iter()
            .position(|(key, def)| key == "core:block/air/air" || def.runtime.reserved_id == Some(0));
        if air_pos.is_none() {
            errors.push("Pack must define one air block with runtime.reserved_id = 0.".into());
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let air_pos = air_pos.unwrap();
        let air_entry = raw.remove(air_pos);
        raw.sort_by(|(a, _), (b, _)| a.cmp(b));
        raw.insert(0, air_entry);

        let mut blocks: Vec<CompiledBlock> = Vec::with_capacity(raw.len());
        let mut families: Vec<CompiledBlockFamily> = Vec::with_capacity(raw.len());
        let mut material_sets: Vec<MaterialTextureSet> = Vec::new();
        let mut default_place = VoxelId::AIR;
        let mut planet_core = None;
        let mut next_id: u32 = 0;

        for (key, def) in raw.into_iter() {
            if let Err(e) =
                check_format_version(def.format_version, BLOCK_FORMAT_VERSION, "block", &key)
            {
                errors.push(e);
                continue;
            }

            // Strict reference validation. New checks land here as their
            // target domains gain real defs (tags/tools: later).
            index.require(
                &def.gameplay.drops,
                &format!("block '{}' gameplay.drops", key),
                &mut errors,
            );
            index.require(
                &def.audio.footstep,
                &format!("block '{}' audio.footstep", key),
                &mut errors,
            );
            index.require(
                &def.audio.break_sound,
                &format!("block '{}' audio.break", key),
                &mut errors,
            );
            index.require(
                &def.audio.place,
                &format!("block '{}' audio.place", key),
                &mut errors,
            );
            index.require(
                &def.model,
                &format!("block '{}' model", key),
                &mut errors,
            );

            // Validate state property defaults & enum sanity.
            def.states
                .validate_into(&format!("block '{}' states", key), &mut errors);

            // Build the variant list. Stateless blocks return one empty value.
            let variant_states = enumerate_variants(&def.states);
            if variant_states.len() > MAX_VARIANTS_PER_FAMILY {
                errors.push(format!(
                    "block '{}': {} variants exceeds the per-family cap of {}",
                    key,
                    variant_states.len(),
                    MAX_VARIANTS_PER_FAMILY
                ));
                continue;
            }

            // Resolve the model. If unknown, the strict ref check above
            // already produced an error — emit stub variants and skip the
            // visual compilation to keep the pipeline going.
            let model_id_opt = models.lookup(&def.model.0);
            let visual_template: CompiledBlockVisual = if let Some(model_id) = model_id_opt {
                let model = models.get(model_id).expect("model id from same registry");
                let color = category_color(&def.category);
                match Self::compile_block_visual(
                    &key,
                    color,
                    def.visual,
                    model,
                    model_id,
                    &material_map,
                    &mut material_sets,
                ) {
                    Ok(v) => v,
                    Err(err) => {
                        errors.extend(err);
                        CompiledBlockVisual {
                            layers: BlockMaterialLayers::default(),
                            tint: [1.0, 1.0, 1.0],
                            flat_color: color,
                            model_id,
                        }
                    }
                }
            } else {
                CompiledBlockVisual::default()
            };

            // Pre-compute the per-family bookkeeping. Each variant gets the
            // next sequential `VoxelId`.
            let next_id_after =
                next_id as usize + variant_states.len();
            if next_id_after > MAX_BLOCK_COUNT {
                errors.push(format!(
                    "Block count would exceed u16 VoxelId capacity ({}) when compiling '{}' ({} variants).",
                    MAX_BLOCK_COUNT,
                    key,
                    variant_states.len()
                ));
                continue;
            }

            let mut family_variants: Vec<VoxelId> = Vec::with_capacity(variant_states.len());
            let mut state_to_id: HashMap<BlockStateValue, VoxelId> =
                HashMap::with_capacity(variant_states.len());
            let mut id_to_state: HashMap<VoxelId, BlockStateValue> =
                HashMap::with_capacity(variant_states.len());

            let color = category_color(&def.category);
            for state in variant_states {
                let id = VoxelId::new(next_id as u16);
                next_id += 1;

                blocks.push(CompiledBlock {
                    id,
                    family_key: key.clone(),
                    state: state.clone(),
                    display_name: def.display_name.clone(),
                    solid: def.physical.solid,
                    color,
                    hardness: def.physical.hardness,
                    visual: visual_template.clone(),
                });
                family_variants.push(id);
                state_to_id.insert(state.clone(), id);
                id_to_state.insert(id, state);
            }

            let default_state_value = default_state(&def.states);
            let default_id = match state_to_id.get(&default_state_value) {
                Some(id) => *id,
                None => {
                    errors.push(format!(
                        "block '{}': default state '{}' not found among generated variants — likely an invalid declared default",
                        key, default_state_value
                    ));
                    *family_variants.first().expect("at least one variant generated")
                }
            };

            // Default-place / planet-core point at the *default variant*.
            if def.runtime.role == Some(BlockRole::DefaultPlace) {
                default_place = default_id;
            }
            if def.runtime.role == Some(BlockRole::PlanetCore)
                && planet_core.replace(default_id).is_some()
            {
                errors.push("Only one block may declare role = \"planet_core\".".into());
            }

            families.push(CompiledBlockFamily {
                family_key: key,
                state_schema: def.states.properties.clone(),
                variants: family_variants,
                default_variant: default_id,
                state_to_id,
                id_to_state,
            });
        }

        if next_id as usize > BLOCK_COUNT_WARNING {
            eprintln!(
                "[content warning] block count {} approaching u16 VoxelId limit ({}); plan a chunk-palette migration before reaching {}.",
                next_id,
                MAX_BLOCK_COUNT,
                MAX_BLOCK_COUNT
            );
        }

        if default_place == VoxelId::AIR {
            if let Some(solid) = blocks.iter().find(|b| b.solid) {
                default_place = solid.id;
            }
        }

        let planet_core = match planet_core {
            Some(id) => id,
            None => {
                errors.push("Pack must define one block with role = \"planet_core\".".into());
                VoxelId::AIR
            }
        };

        if !errors.is_empty() {
            return Err(errors);
        }

        let mut material_colors: Vec<[f32; 4]> =
            vec![[1.0, 1.0, 1.0, 1.0]; material_sets.len() + 1];
        for block in &blocks {
            let layers = [
                block.visual.layers.top,
                block.visual.layers.bottom,
                block.visual.layers.front,
                block.visual.layers.back,
                block.visual.layers.left,
                block.visual.layers.right,
            ];
            for layer in layers {
                let idx = layer as usize;
                if idx == 0 || idx >= material_colors.len() {
                    continue;
                }
                if material_colors[idx] == [1.0, 1.0, 1.0, 1.0] {
                    material_colors[idx] = [block.color[0], block.color[1], block.color[2], 1.0];
                }
            }
        }

        Ok(BlockRegistry::new(
            blocks,
            families,
            material_sets,
            material_colors,
            default_place,
            planet_core,
            models,
        ))
    }

    fn compile_block_visual(
        key: &str,
        color: [f32; 3],
        raw: RawBlockVisual,
        model: &CompiledBlockModel,
        model_id: BlockModelId,
        material_map: &HashMap<String, RawMaterialDef>,
        material_sets: &mut Vec<MaterialTextureSet>,
    ) -> Result<CompiledBlockVisual, Vec<String>> {
        let mut errors = Vec::new();

        // Strict slot-keys validation. Block.materials must have exactly the
        // same set of keys as the model.face_layers — no missing, no extra.
        let model_slots: HashSet<&str> =
            model.face_layers.iter().map(String::as_str).collect();
        let block_slots: HashSet<&str> =
            raw.materials.keys().map(String::as_str).collect();

        let missing: Vec<&str> = model_slots.difference(&block_slots).copied().collect();
        let extra: Vec<&str> = block_slots.difference(&model_slots).copied().collect();
        for slot in &missing {
            errors.push(format!(
                "block '{}': missing material slot '{}' required by model '{}'",
                key, slot, model.key
            ));
        }
        for slot in &extra {
            errors.push(format!(
                "block '{}': extra material slot '{}' not declared by model '{}'",
                key, slot, model.key
            ));
        }

        // Air-like blocks (model with no face_layers and Invisible render) are
        // allowed to have an empty materials map; visual layers stay at 0.
        if model.face_layers.is_empty() {
            if !raw.materials.is_empty() {
                errors.push(format!(
                    "block '{}': model '{}' declares no face_layers but block provides {} material(s)",
                    key,
                    model.key,
                    raw.materials.len()
                ));
            }
            if errors.is_empty() {
                return Ok(CompiledBlockVisual {
                    layers: BlockMaterialLayers::default(),
                    tint: [1.0, 1.0, 1.0],
                    flat_color: color,
                    model_id,
                });
            } else {
                return Err(errors);
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Resolve each declared slot to an atlas layer index.
        let mut slot_layers: HashMap<&str, u32> = HashMap::with_capacity(model.face_layers.len());
        for slot in &model.face_layers {
            let material_ref = raw
                .materials
                .get(slot)
                .expect("slot existence checked above");
            let layer = match material_map.get(&material_ref.0) {
                Some(def) => Self::material_layer(def, material_sets),
                None => {
                    errors.push(format!(
                        "block '{}': unknown material '{}' for slot '{}'",
                        key, material_ref.0, slot
                    ));
                    0
                }
            };
            slot_layers.insert(slot.as_str(), layer);
        }

        let layers = match &model.mesh {
            CompiledMesh::Cube { .. } => {
                // Convention: face_layers slot names map to renderer face fields.
                // py→top, ny→bottom, pz→front, nz→back, px→right, nx→left.
                BlockMaterialLayers {
                    top: lookup_slot(&slot_layers, "py", key, &mut errors),
                    bottom: lookup_slot(&slot_layers, "ny", key, &mut errors),
                    front: lookup_slot(&slot_layers, "pz", key, &mut errors),
                    back: lookup_slot(&slot_layers, "nz", key, &mut errors),
                    right: lookup_slot(&slot_layers, "px", key, &mut errors),
                    left: lookup_slot(&slot_layers, "nx", key, &mut errors),
                }
            }
            CompiledMesh::CubeColumn { .. } => {
                // Default Y-axis: end on top/bottom, side on the four laterals.
                let end = lookup_slot(&slot_layers, "end", key, &mut errors);
                let side = lookup_slot(&slot_layers, "side", key, &mut errors);
                BlockMaterialLayers {
                    top: end,
                    bottom: end,
                    front: side,
                    back: side,
                    left: side,
                    right: side,
                }
            }
            CompiledMesh::CrossPlane => {
                // The mesher reads only one slot for cross-planes, but we
                // populate all six for layout uniformity.
                let plane = lookup_slot(&slot_layers, "plane", key, &mut errors);
                BlockMaterialLayers {
                    top: plane,
                    bottom: plane,
                    front: plane,
                    back: plane,
                    left: plane,
                    right: plane,
                }
            }
            CompiledMesh::None => BlockMaterialLayers::default(),
        };

        if !errors.is_empty() {
            return Err(errors);
        }

        // An invisible render mode is allowed and means the block is never
        // drawn even if it has materials (covered by air/none model branch
        // above; here we keep a consistency check).
        if raw.render == RawRenderMode::Invisible && !model.face_layers.is_empty() {
            errors.push(format!(
                "block '{}': render is 'invisible' but model '{}' declares {} face_layer(s)",
                key,
                model.key,
                model.face_layers.len()
            ));
        }
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(CompiledBlockVisual {
            layers,
            tint: [1.0, 1.0, 1.0],
            flat_color: color,
            model_id,
        })
    }

    fn material_layer(
        raw: &RawMaterialDef,
        material_sets: &mut Vec<MaterialTextureSet>,
    ) -> u32 {
        let material = MaterialTextureSet {
            albedo: raw.albedo.0.clone(),
            normal: raw
                .normal
                .as_ref()
                .unwrap_or(&raw.albedo)
                .0
                .clone(),
            roughness: raw
                .roughness
                .as_ref()
                .unwrap_or(&raw.albedo)
                .0
                .clone(),
        };
        if let Some(index) = material_sets.iter().position(|m| m == &material) {
            (index + 1) as u32
        } else {
            material_sets.push(material);
            material_sets.len() as u32
        }
    }
}

fn model_id_from_index(idx: usize) -> BlockModelId {
    // Safe: pack compilers don't ship millions of models.
    BlockModelId::from_raw(idx as u32)
}

fn lookup_slot(
    slot_layers: &HashMap<&str, u32>,
    name: &str,
    key: &str,
    errors: &mut Vec<String>,
) -> u32 {
    match slot_layers.get(name) {
        Some(&layer) => layer,
        None => {
            errors.push(format!(
                "block '{}': internal slot '{}' missing after validation (compiler bug)",
                key, name
            ));
            0
        }
    }
}

fn category_color(category: &str) -> [f32; 3] {
    match category {
        "air" => [0.0, 0.0, 0.0],
        "terrain" => [0.46, 0.42, 0.34],
        "natural/log" => [0.43, 0.27, 0.13],
        "natural/leaves" => [0.23, 0.50, 0.18],
        "flora" => [0.38, 0.72, 0.25],
        "ore" => [0.45, 0.45, 0.45],
        _ => [1.0, 1.0, 1.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use vv_content_schema::{
        ContentRef, RawBlockAudioDef, RawBlockGameplayDef, RawBlockPhysicalDef,
        RawBlockPlacement, RawBlockRuntimeDef, RawBlockSimulationDef,
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
            m.insert(slot.into(), ContentRef("core:material/test/all".to_string()));
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

        assert!(err.iter().any(|e| e.contains("planet_core")), "got: {err:?}");
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
        let models =
            ContentCompiler::compile_block_models(pack.block_models).expect("block_models");
        let blocks =
            ContentCompiler::compile_blocks(pack.blocks, pack.materials, models, &index)
                .expect("blocks");

        for block in blocks.blocks().iter().filter(|b| b.solid) {
            let layers = block.visual.layers;
            assert!(layers.top > 0, "{} missing top material", block.family_key);
            assert!(layers.bottom > 0, "{} missing bottom material", block.family_key);
            assert!(layers.front > 0, "{} missing front material", block.family_key);
            assert!(layers.back > 0, "{} missing back material", block.family_key);
            assert!(layers.left > 0, "{} missing left material", block.family_key);
            assert!(layers.right > 0, "{} missing right material", block.family_key);
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
        let fam = reg
            .family_of(stone_default)
            .expect("stone family");
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
            .lookup_variant(
                "core:block/terrain/stone",
                &axis_water_value("y", "false"),
            )
            .expect("default present");
        assert_eq!(fam.default_variant, id);
        // All combinations resolvable.
        for axis in ["x", "y", "z"] {
            for water in ["false", "true"] {
                assert!(reg
                    .lookup_variant(
                        "core:block/terrain/stone",
                        &axis_water_value(axis, water),
                    )
                    .is_some(), "missing {axis}/{water}");
            }
        }
    }

    #[test]
    fn state_of_round_trips() {
        let reg = compile_pair(axis_and_bool_state("y", true));
        let id = reg
            .lookup_variant(
                "core:block/terrain/stone",
                &axis_water_value("z", "true"),
            )
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
}
