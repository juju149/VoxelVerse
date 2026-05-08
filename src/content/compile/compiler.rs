use crate::content::block_registry::{
    BlockMaterialLayers, BlockRegistry, BlockShape, CompiledBlock, CompiledBlockVisual,
    MaterialTextureSet,
};
use crate::content::schema::{BlockRole, RawBlockDef, RawBlockVisual, RawMaterialTextureSet};
use crate::voxel::VoxelId;
use std::collections::HashMap;

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
    ) -> Result<BlockRegistry, Vec<String>> {
        let mut errors = Vec::new();

        let air_pos = raw.iter().position(|(key, _)| key.ends_with(":air"));
        if air_pos.is_none() {
            errors.push("Pack must define a block with key ending in ':air'.".into());
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
            if def.role == Some(BlockRole::DefaultPlace) {
                default_place = id;
            }
            if def.role == Some(BlockRole::PlanetCore) && planet_core.replace(id).is_some() {
                errors.push("Only one block may declare role = \"planet_core\".".into());
            }

            let visual = if let Some(raw_visual) = def.visual {
                match Self::compile_block_visual(&key, def.color, raw_visual, &mut material_sets) {
                    Ok(visual) => visual,
                    Err(err) => {
                        errors.extend(err);
                        CompiledBlockVisual {
                            layers: BlockMaterialLayers::default(),
                            tint: [1.0, 1.0, 1.0],
                            flat_color: def.color,
                            shape: BlockShape::Cube,
                        }
                    }
                }
            } else {
                CompiledBlockVisual {
                    layers: BlockMaterialLayers::default(),
                    tint: [1.0, 1.0, 1.0],
                    flat_color: def.color,
                    shape: BlockShape::Cube,
                }
            };

            key_to_id.insert(key.clone(), id);
            blocks.push(CompiledBlock {
                id,
                key,
                display_name: def.display_name,
                solid: def.solid,
                color: def.color,
                hardness: def.hardness,
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

        Ok(BlockRegistry::new(
            blocks,
            key_to_id,
            material_sets,
            default_place,
            planet_core,
        ))
    }

    fn compile_block_visual(
        key: &str,
        color: [f32; 3],
        raw: RawBlockVisual,
        material_sets: &mut Vec<MaterialTextureSet>,
    ) -> Result<CompiledBlockVisual, Vec<String>> {
        let mut errors = Vec::new();
        let shape = BlockShape::from(raw.shape);
        let mut layer_for = |face: &str, material: Option<&RawMaterialTextureSet>| -> u32 {
            let Some(material) = material else {
                // Cross-planes only need the `all` layer; missing per-face materials are
                // expected and must not error out.
                if shape == BlockShape::CrossPlane {
                    return 0;
                }
                errors.push(format!(
                    "Block '{}': visual must define material for {} face",
                    key, face
                ));
                return 0;
            };
            Self::material_layer(material, material_sets)
        };

        let layers = BlockMaterialLayers {
            top: layer_for("top", raw.top.as_ref().or(raw.all.as_ref())),
            bottom: layer_for("bottom", raw.bottom.as_ref().or(raw.all.as_ref())),
            front: layer_for(
                "front",
                raw.front
                    .as_ref()
                    .or(raw.side.as_ref())
                    .or(raw.all.as_ref()),
            ),
            back: layer_for(
                "back",
                raw.back.as_ref().or(raw.side.as_ref()).or(raw.all.as_ref()),
            ),
            left: layer_for(
                "left",
                raw.left.as_ref().or(raw.side.as_ref()).or(raw.all.as_ref()),
            ),
            right: layer_for(
                "right",
                raw.right
                    .as_ref()
                    .or(raw.side.as_ref())
                    .or(raw.all.as_ref()),
            ),
        };

        if errors.is_empty() {
            Ok(CompiledBlockVisual {
                layers,
                tint: raw.tint,
                flat_color: color,
                shape,
            })
        } else {
            Err(errors)
        }
    }

    fn material_layer(
        raw: &RawMaterialTextureSet,
        material_sets: &mut Vec<MaterialTextureSet>,
    ) -> u32 {
        let material = MaterialTextureSet {
            albedo: raw.albedo.0.clone(),
            normal: raw.normal.0.clone(),
            roughness: raw.roughness.0.clone(),
        };
        if let Some(index) = material_sets.iter().position(|m| m == &material) {
            (index + 1) as u32
        } else {
            material_sets.push(material);
            material_sets.len() as u32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ContentCompiler;
    use crate::content::schema::{BlockRole, RawBlockDef};

    fn block(role: Option<BlockRole>) -> RawBlockDef {
        RawBlockDef {
            display_name: "Block".to_string(),
            solid: true,
            color: [1.0, 1.0, 1.0],
            hardness: 1.0,
            role,
            visual: None,
        }
    }

    #[test]
    fn block_compilation_requires_planet_core_role() {
        let err = match ContentCompiler::compile_blocks(vec![
            ("core:air".to_string(), block(None)),
            ("core:stone".to_string(), block(None)),
        ]) {
            Ok(_) => panic!("missing planet core should be rejected"),
            Err(err) => err,
        };

        assert!(err.iter().any(|e| e.contains("planet_core")));
    }

    #[test]
    fn core_pack_solid_blocks_have_all_faces_materialized() {
        use crate::content::pack::PackLoader;
        use std::path::Path;

        let pack = PackLoader::load_from_dir(Path::new("packs/core")).expect("core pack");
        let blocks = ContentCompiler::compile_blocks(pack.blocks).expect("blocks");

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
