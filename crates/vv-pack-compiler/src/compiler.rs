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

        if raw.len() > MAX_BLOCK_COUNT {
            return Err(vec![format!(
                "Block count {} exceeds u16 VoxelId capacity ({}).",
                raw.len(),
                MAX_BLOCK_COUNT
            )]);
        }
        if raw.len() > BLOCK_COUNT_WARNING {
            eprintln!(
                "[content warning] block count {} approaching u16 VoxelId limit ({}); plan a chunk-palette migration before reaching {}.",
                raw.len(),
                MAX_BLOCK_COUNT,
                MAX_BLOCK_COUNT
            );
        }

        let mut blocks: Vec<CompiledBlock> = Vec::with_capacity(raw.len());
        let mut key_to_id: HashMap<String, VoxelId> = HashMap::with_capacity(raw.len());
        let mut material_sets: Vec<MaterialTextureSet> = Vec::new();
        let mut default_place = VoxelId::AIR;
        let mut planet_core = None;

        for (idx, (key, def)) in raw.into_iter().enumerate() {
            let id = VoxelId::new(idx as u16);

            if let Err(e) =
                check_format_version(def.format_version, BLOCK_FORMAT_VERSION, "block", &key)
            {
                errors.push(e);
                continue;
            }

            if def.runtime.role == Some(BlockRole::DefaultPlace) {
                default_place = id;
            }
            if def.runtime.role == Some(BlockRole::PlanetCore) && planet_core.replace(id).is_some() {
                errors.push("Only one block may declare role = \"planet_core\".".into());
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

            let Some(model_id) = models.lookup(&def.model.0) else {
                // The strict ref check above already produced an error;
                // emit a stub visual so the rest of the pipeline keeps going.
                blocks.push(CompiledBlock {
                    id,
                    key: key.clone(),
                    display_name: def.display_name,
                    solid: def.physical.solid,
                    color: category_color(&def.category),
                    hardness: def.physical.hardness,
                    visual: CompiledBlockVisual::default(),
                });
                key_to_id.insert(key, id);
                continue;
            };
            let model = models.get(model_id).expect("model id from same registry");

            let color = category_color(&def.category);
            let visual = match Self::compile_block_visual(
                &key,
                color,
                def.visual,
                model,
                model_id,
                &material_map,
                &mut material_sets,
            ) {
                Ok(visual) => visual,
                Err(err) => {
                    errors.extend(err);
                    CompiledBlockVisual {
                        layers: BlockMaterialLayers::default(),
                        tint: [1.0, 1.0, 1.0],
                        flat_color: color,
                        model_id,
                    }
                }
            };

            key_to_id.insert(key.clone(), id);
            blocks.push(CompiledBlock {
                id,
                key,
                display_name: def.display_name,
                solid: def.physical.solid,
                color,
                hardness: def.physical.hardness,
                visual,
            });
        }

        if default_place == VoxelId::AIR {
            if let Some(solid) = blocks.iter().find(|b| b.solid) {
                default_place = solid.id;
            }
        }

        let Some(planet_core) = planet_core else {
            return Err(vec![
                "Pack must define one block with role = \"planet_core\".".into(),
            ]);
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
            key_to_id,
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
            "core:block_model/cube",
        ])
    }

    fn synthetic_models() -> BlockModelRegistry {
        ContentCompiler::compile_block_models(vec![(
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
        )])
        .expect("models compile")
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
                render: RawRenderMode::Invisible,
                materials: HashMap::new(),
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
        }
    }

    #[test]
    fn block_compilation_requires_planet_core_role() {
        let mut air = block(None);
        air.runtime.reserved_id = Some(0);
        air.visual.render = RawRenderMode::Invisible;
        let err = match ContentCompiler::compile_blocks(
            vec![
                ("core:block/air/air".to_string(), air),
                ("core:block/terrain/stone".to_string(), block(None)),
            ],
            Vec::new(),
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
        let mut air = block(None);
        air.runtime.reserved_id = Some(0);
        let mut stone = block(Some(BlockRole::PlanetCore));
        stone.gameplay.drops = ContentRef("core:loot/blocks/does_not_exist".to_string());
        let err = match ContentCompiler::compile_blocks(
            vec![
                ("core:block/air/air".to_string(), air),
                ("core:block/terrain/stone".to_string(), stone),
            ],
            Vec::new(),
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
            assert!(layers.top > 0, "{} missing top material", block.key);
            assert!(layers.bottom > 0, "{} missing bottom material", block.key);
            assert!(layers.front > 0, "{} missing front material", block.key);
            assert!(layers.back > 0, "{} missing back material", block.key);
            assert!(layers.left > 0, "{} missing left material", block.key);
            assert!(layers.right > 0, "{} missing right material", block.key);
        }
    }
}
