use crate::block_registry::{
    BlockMaterialLayers, BlockRegistry, BlockShape, CompiledBlock, CompiledBlockVisual,
    MaterialTextureSet,
};
use crate::content_index::ContentIndex;
use std::collections::HashMap;
use vv_content_schema::{
    BlockRole, ContentRef, RawBlockDef, RawBlockMaterials, RawBlockVisual, RawMaterialDef,
    RawRenderMode,
};
use vv_voxel::VoxelId;

pub struct ContentCompiler;

impl ContentCompiler {
    /// Compile raw block definitions into a runtime `BlockRegistry`.
    ///
    /// Rules:
    /// - A block with key ending in `:air` must be present — it is assigned `VoxelId(0)`.
    /// - All other blocks are sorted alphabetically for deterministic ID assignment.
    /// - Returns a list of human-readable errors if validation fails.
    pub fn compile_blocks(
        mut raw: Vec<(String, RawBlockDef)>,
        materials: Vec<(String, RawMaterialDef)>,
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

        // Pull air out first — it must become VoxelId(0).
        let air_pos = air_pos.unwrap();
        let air_entry = raw.remove(air_pos);

        // Sort the rest alphabetically for stable IDs across reloads.
        raw.sort_by(|(a, _), (b, _)| a.cmp(b));

        // Reinsert air at the front.
        raw.insert(0, air_entry);

        let mut blocks: Vec<CompiledBlock> = Vec::with_capacity(raw.len());
        let mut key_to_id: HashMap<String, VoxelId> = HashMap::with_capacity(raw.len());
        let mut material_sets: Vec<MaterialTextureSet> = Vec::new();
        let mut default_place = VoxelId::AIR;
        let mut planet_core = None;

        for (idx, (key, def)) in raw.into_iter().enumerate() {
            let id = VoxelId::new(idx as u16);

            // A block that declares `role = "default_place"` is used as the
            // initial placement block.  No name heuristics needed.
            if def.runtime.role == Some(BlockRole::DefaultPlace) {
                default_place = id;
            }
            if def.runtime.role == Some(BlockRole::PlanetCore) && planet_core.replace(id).is_some() {
                errors.push("Only one block may declare role = \"planet_core\".".into());
            }

            // Strict reference validation. New checks land here as their
            // target domains gain real defs (audio: step 3, tags/tools: later).
            index.require(
                &def.gameplay.drops,
                &format!("block '{}' gameplay.drops", key),
                &mut errors,
            );

            let color = category_color(&def.category);
            let visual = match Self::compile_block_visual(
                &key,
                color,
                def.visual,
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
                        shape: BlockShape::Cube,
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

        // Fallback: if no block declared `role = "default_place"`, use first solid block.
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

        // One representative color per atlas layer. Layer 0 is the neutral
        // fallback tile; layers 1..=material_sets.len() correspond to each
        // distinct material set. We fill the array by walking the blocks: the
        // first block that references a layer wins (deterministic since blocks
        // are sorted alphabetically). Any layer left unclaimed keeps white.
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
        ))
    }

    fn compile_block_visual(
        key: &str,
        color: [f32; 3],
        raw: RawBlockVisual,
        material_map: &HashMap<String, RawMaterialDef>,
        material_sets: &mut Vec<MaterialTextureSet>,
    ) -> Result<CompiledBlockVisual, Vec<String>> {
        let mut errors = Vec::new();
        let shape = BlockShape::from(raw.shape);
        let mut layer_for = |face: &str, material: &ContentRef| -> u32 {
            match material_map.get(&material.0) {
                Some(def) => Self::material_layer(def, material_sets),
                None => {
                    errors.push(format!(
                        "Block '{}': unknown material '{}' for {} face",
                        key, material.0, face
                    ));
                    0
                }
            }
        };

        let layers = match raw.materials {
            RawBlockMaterials::None => {
                if raw.render != RawRenderMode::Invisible {
                    errors.push(format!(
                        "Block '{}': non-invisible visual must define materials",
                        key
                    ));
                }
                BlockMaterialLayers::default()
            }
            RawBlockMaterials::All(material) => {
                let layer = layer_for("all", &material);
                BlockMaterialLayers {
                    top: layer,
                    bottom: layer,
                    front: layer,
                    back: layer,
                    left: layer,
                    right: layer,
                }
            }
            RawBlockMaterials::Faces(faces) => BlockMaterialLayers {
                top: layer_for("top", &faces.top),
                bottom: layer_for("bottom", &faces.bottom),
                front: layer_for("front", faces.front.as_ref().unwrap_or(&faces.sides)),
                back: layer_for("back", faces.back.as_ref().unwrap_or(&faces.sides)),
                left: layer_for("left", faces.left.as_ref().unwrap_or(&faces.sides)),
                right: layer_for("right", faces.right.as_ref().unwrap_or(&faces.sides)),
            },
        };

        if errors.is_empty() {
            Ok(CompiledBlockVisual {
                layers,
                tint: [1.0, 1.0, 1.0],
                flat_color: color,
                shape,
            })
        } else {
            Err(errors)
        }
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
    use super::ContentCompiler;
    use vv_content_schema::*;

    fn block(role: Option<BlockRole>) -> RawBlockDef {
        RawBlockDef {
            display_name: "Block".to_string(),
            category: "terrain".to_string(),
            physical: RawBlockPhysicalDef {
                solid: true,
                opaque: true,
                collision: RawBlockCollision::FullCube,
                hardness: 1.0,
                blast_resistance: 1.0,
                friction: 0.8,
                restitution: 0.0,
            },
            visual: RawBlockVisual {
                shape: RawBlockShape::Cube,
                render: RawRenderMode::Invisible,
                materials: RawBlockMaterials::None,
                ambient_occlusion: true,
                casts_shadow: true,
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

    fn synthetic_index() -> crate::ContentIndex {
        crate::ContentIndex::from_keys(["core:loot/blocks/empty"])
    }

    #[test]
    fn block_compilation_requires_planet_core_role() {
        let mut air = block(None);
        air.runtime.reserved_id = Some(0);
        let err = match ContentCompiler::compile_blocks(
            vec![
                ("core:block/air/air".to_string(), air),
                ("core:block/terrain/stone".to_string(), block(None)),
            ],
            Vec::new(),
            &synthetic_index(),
        ) {
            Ok(_) => panic!("missing planet core should be rejected"),
            Err(err) => err,
        };

        assert!(err.iter().any(|e| e.contains("planet_core")));
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
            &synthetic_index(),
        ) {
            Ok(_) => panic!("dangling drops ref should be rejected"),
            Err(err) => err,
        };
        assert!(
            err.iter().any(|e| e.contains("dangling reference") && e.contains("does_not_exist")),
            "expected dangling-reference error, got: {err:?}"
        );
    }

    #[test]
    fn core_pack_solid_blocks_have_all_faces_materialized() {
        use std::path::Path;
        use vv_pack_loader::PackLoader;

        let core_pack_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core");
        let pack = PackLoader::load_from_dir(&core_pack_dir).expect("core pack");
        let index = crate::ContentIndex::build(&pack);
        let blocks =
            ContentCompiler::compile_blocks(pack.blocks, pack.materials, &index).expect("blocks");

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
