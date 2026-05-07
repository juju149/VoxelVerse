use crate::content::biome_registry::{BiomeRegistry, CompiledBiome};
use crate::content::block_registry::{
    BlockMaterialLayers, BlockRegistry, CompiledBlock, CompiledBlockVisual, MaterialTextureSet,
};
use crate::content::schema::{
    BlockRole, RawBiomeDef, RawBlockDef, RawBlockVisual, RawMaterialTextureSet, RawPlanetDef,
};
use crate::content::CompiledPlanet;
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
                        }
                    }
                }
            } else {
                CompiledBlockVisual {
                    layers: BlockMaterialLayers::default(),
                    tint: [1.0, 1.0, 1.0],
                    flat_color: def.color,
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
        let mut layer_for = |face: &str, material: Option<&RawMaterialTextureSet>| -> u32 {
            let Some(material) = material else {
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

    /// Compile raw biome definitions into a `BiomeRegistry`.
    ///
    /// Rules:
    /// - At least one biome must be defined.
    /// - Biomes are sorted by temperature_center descending (tropical first) for
    ///   stable ID assignment.
    /// - All block references are resolved against the provided `BlockRegistry`.
    /// - Returns human-readable errors on failure.
    pub fn compile_biomes(
        mut raw: Vec<(String, RawBiomeDef)>,
        block_registry: &BlockRegistry,
    ) -> Result<BiomeRegistry, Vec<String>> {
        let mut errors = Vec::new();

        if raw.is_empty() {
            errors.push("Pack must define at least one biome.".into());
            return Err(errors);
        }

        // Sort by temperature_center descending for deterministic IDs.
        raw.sort_by(|(_, a), (_, b)| {
            b.temperature_center
                .partial_cmp(&a.temperature_center)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut biomes: Vec<CompiledBiome> = Vec::with_capacity(raw.len());

        for (idx, (key, def)) in raw.into_iter().enumerate() {
            let surface = block_registry.lookup(&def.surface_block);
            let subsurface = block_registry.lookup(&def.subsurface_block);

            if surface.is_none() {
                errors.push(format!(
                    "Biome '{}': unknown surface_block '{}'",
                    key, def.surface_block
                ));
            }
            if subsurface.is_none() {
                errors.push(format!(
                    "Biome '{}': unknown subsurface_block '{}'",
                    key, def.subsurface_block
                ));
            }

            if errors.is_empty() {
                biomes.push(CompiledBiome {
                    id: idx as u8,
                    key,
                    display_name: def.display_name,
                    surface_block: surface.unwrap(),
                    subsurface_block: subsurface.unwrap(),
                    temperature_center: def.temperature_center.clamp(0.0, 1.0),
                    roughness_center: def.roughness_center.clamp(0.0, 1.0),
                    terrain_amplitude: def.terrain_amplitude.clamp(0.0, 1.0),
                    terrain_flatness: def.terrain_flatness.clamp(0.0, 1.0),
                });
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(BiomeRegistry::new(biomes))
    }

    pub fn compile_planets(
        raw: Vec<(String, RawPlanetDef)>,
    ) -> Result<Vec<CompiledPlanet>, Vec<String>> {
        let mut errors = Vec::new();

        if raw.is_empty() {
            return Err(vec!["Pack must define at least one planet.".into()]);
        }

        let mut planets = Vec::with_capacity(raw.len());
        for (key, def) in raw {
            let resolution = def.resolution.max(8);
            let surface_layer = def.surface_layer.unwrap_or(resolution / 2);

            if surface_layer < 4 || surface_layer >= resolution {
                errors.push(format!(
                    "Planet '{}': surface_layer {} must be in 4..{}",
                    key, surface_layer, resolution
                ));
            }
            if def.core_layers < 1 || def.core_layers >= surface_layer {
                errors.push(format!(
                    "Planet '{}': core_layers {} must be at least 1 and below surface_layer {}",
                    key, def.core_layers, surface_layer
                ));
            }
            if !def.voxel_size_meters.is_finite() || def.voxel_size_meters <= 0.0 {
                errors.push(format!(
                    "Planet '{}': voxel_size_meters must be a finite positive value",
                    key
                ));
            }
            if !(0.02..0.95).contains(&def.inner_radius_fraction) {
                errors.push(format!(
                    "Planet '{}': inner_radius_fraction {} must be in 0.02..0.95",
                    key, def.inner_radius_fraction
                ));
            }
            if def.max_terrain_offset < 0 {
                errors.push(format!("Planet '{}': max_terrain_offset must be >= 0", key));
            }
            if def.spawn_clearance_layers <= 0.0 {
                errors.push(format!(
                    "Planet '{}': spawn_clearance_layers must be positive",
                    key
                ));
            }

            planets.push(CompiledPlanet {
                key,
                display_name: def.display_name,
                seed: def.seed,
                resolution,
                surface_layer,
                voxel_size_meters: def.voxel_size_meters,
                core_layers: def.core_layers,
                inner_radius_fraction: def.inner_radius_fraction,
                max_terrain_offset: def.max_terrain_offset,
                spawn_clearance_layers: def.spawn_clearance_layers,
            });
        }

        if errors.is_empty() {
            Ok(planets)
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ContentCompiler;
    use crate::content::schema::{BlockRole, RawBiomeDef, RawBlockDef};

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
    fn biome_unknown_block_reference_is_reported() {
        let blocks = ContentCompiler::compile_blocks(vec![
            (
                "core:air".to_string(),
                RawBlockDef {
                    solid: false,
                    ..block(None)
                },
            ),
            ("core:core".to_string(), block(Some(BlockRole::PlanetCore))),
        ])
        .expect("valid blocks");

        let err = match ContentCompiler::compile_biomes(
            vec![(
                "core:bad".to_string(),
                RawBiomeDef {
                    display_name: "Bad".to_string(),
                    surface_block: "core:missing".to_string(),
                    subsurface_block: "core:core".to_string(),
                    temperature_center: 0.5,
                    roughness_center: 0.5,
                    terrain_amplitude: 0.5,
                    terrain_flatness: 0.5,
                },
            )],
            &blocks,
        ) {
            Ok(_) => panic!("unknown biome block should be rejected"),
            Err(err) => err,
        };

        assert!(err.iter().any(|e| e.contains("unknown surface_block")));
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
