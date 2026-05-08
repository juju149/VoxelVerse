use crate::content::compile::ContentCompiler;
use crate::content::pack::RawProceduralPack;
use crate::content::schema::*;
use crate::content::{
    BlockRegistry, CompiledBiomeColorTint, CompiledBiomeSelector, CompiledBiomeSet,
    CompiledBiomeSurface, CompiledBiomeTerrain, CompiledCave, CompiledCaveCarver, CompiledClimate,
    CompiledClimateAxis, CompiledCurve, CompiledFauna, CompiledFeaturePlacement,
    CompiledNoiseField, CompiledNoiseKind, CompiledNoiseRemap, CompiledOre, CompiledPlanet,
    CompiledProceduralBiome, CompiledProceduralPlanet, CompiledStructure, CompiledTerrainLayer,
    CompiledTerrainLayerSet, CompiledVegetation, CompiledVisualDetail, CompiledVisualDetailItem,
    ProceduralRegistry,
};
use crate::voxel::VoxelId;
use std::collections::HashMap;

impl ContentCompiler {
    pub fn compile_procedural(
        raw: RawProceduralPack,
        blocks: &BlockRegistry,
    ) -> Result<ProceduralRegistry, Vec<String>> {
        let mut errors = Vec::new();

        if raw.planets.is_empty() {
            return Err(vec![
                "Procedural pack must define at least one planet.".to_string()
            ]);
        }

        let field_map = key_map("field", &raw.fields, &mut errors);
        let biome_map = key_map("procedural biome", &raw.biomes, &mut errors);
        let climate_map = key_map("climate", &raw.climates, &mut errors);
        let biome_set_map = key_map("biome set", &raw.biome_sets, &mut errors);
        let layer_map = key_map("terrain layer set", &raw.terrain_layers, &mut errors);
        let ore_map = key_map("ore", &raw.ores, &mut errors);
        let cave_map = key_map("cave", &raw.caves, &mut errors);
        let vegetation_map = key_map("vegetation", &raw.vegetation, &mut errors);
        let structure_map = key_map("structure", &raw.structures, &mut errors);
        let fauna_map = key_map("fauna", &raw.fauna, &mut errors);
        let detail_map = key_map("visual detail", &raw.visual_details, &mut errors);

        let fields = raw
            .fields
            .iter()
            .map(|(key, def)| {
                let domain_warp = match &def.domain_warp {
                    Some(warp) => resolve(&field_map, "field", key, &warp.field, &mut errors)
                        .map(|idx| (idx, warp.strength.max(0.0))),
                    None => None,
                };
                CompiledNoiseField {
                    key: key.to_string(),
                    kind: CompiledNoiseKind::from(&def.kind),
                    frequency: finite_positive(def.frequency, 0.01),
                    amplitude: finite(def.amplitude, 1.0),
                    octaves: def.octaves.clamp(1, 12),
                    persistence: finite_positive(def.persistence, 0.5),
                    lacunarity: finite_positive(def.lacunarity, 2.0),
                    seed_salt: def.seed_salt,
                    domain_warp,
                    remap: def.remap.as_ref().map(|r| CompiledNoiseRemap {
                        in_min: r.in_min,
                        in_max: r.in_max,
                        out_min: r.out_min,
                        out_max: r.out_max,
                        curve: CompiledCurve::from(&r.curve),
                    }),
                }
            })
            .collect();

        let biomes: Vec<_> = raw
            .biomes
            .iter()
            .enumerate()
            .map(|(idx, (key, def))| compile_biome(idx, key, def, &field_map, blocks, &mut errors))
            .collect();

        let climates = raw
            .climates
            .iter()
            .map(|(key, def)| CompiledClimate {
                key: key.clone(),
                temperature: compile_axis(
                    key,
                    "temperature",
                    &def.temperature,
                    &field_map,
                    &mut errors,
                ),
                humidity: compile_axis(key, "humidity", &def.humidity, &field_map, &mut errors),
                continentality: compile_axis(
                    key,
                    "continentality",
                    &def.continentality,
                    &field_map,
                    &mut errors,
                ),
                erosion: compile_axis(key, "erosion", &def.erosion, &field_map, &mut errors),
                weirdness: compile_axis(key, "weirdness", &def.weirdness, &field_map, &mut errors),
            })
            .collect();

        let biome_sets = raw
            .biome_sets
            .iter()
            .map(|(key, def)| CompiledBiomeSet {
                key: key.clone(),
                blend_radius: def.blend_radius.clamp(0.001, 1.0),
                selectors: def
                    .selectors
                    .iter()
                    .filter_map(|s| {
                        resolve(&biome_map, "procedural biome", key, &s.biome, &mut errors).map(
                            |biome| CompiledBiomeSelector {
                                biome,
                                temperature: normalized_range(s.temperature),
                                humidity: normalized_range(s.humidity),
                                roughness: normalized_range(s.roughness),
                                weight: finite_positive(s.weight, 1.0),
                            },
                        )
                    })
                    .collect(),
            })
            .collect();

        let terrain_layers = raw
            .terrain_layers
            .iter()
            .map(|(key, def)| CompiledTerrainLayerSet {
                key: key.clone(),
                layers: def
                    .layers
                    .iter()
                    .filter_map(|layer| {
                        let block = resolve_block(blocks, key, &layer.block, &mut errors)?;
                        let all_biomes =
                            layer.biomes.iter().any(|b| b == "*") || layer.biomes.is_empty();
                        let biomes = if all_biomes {
                            Vec::new()
                        } else {
                            layer
                                .biomes
                                .iter()
                                .filter_map(|b| {
                                    resolve(&biome_map, "procedural biome", key, b, &mut errors)
                                })
                                .collect()
                        };
                        Some(CompiledTerrainLayer {
                            name: layer.name.clone(),
                            block,
                            depth: layer.depth.map(normalized_u32_range),
                            depth_from_center: layer.depth_from_center.map(normalized_u32_range),
                            all_biomes,
                            biomes,
                            noise_variation: layer
                                .noise_variation
                                .as_ref()
                                .and_then(|f| resolve(&field_map, "field", key, f, &mut errors)),
                        })
                    })
                    .collect(),
            })
            .collect();

        let ores = raw
            .ores
            .iter()
            .filter_map(|(key, def)| compile_ore(key, def, &field_map, blocks, &mut errors))
            .collect();
        let caves = raw
            .caves
            .iter()
            .map(|(key, def)| compile_cave(key, def, &field_map, blocks, &mut errors))
            .collect();
        let vegetation = raw
            .vegetation
            .iter()
            .filter_map(|(key, def)| compile_vegetation(key, def, &field_map, blocks, &mut errors))
            .collect();
        let structures = raw
            .structures
            .iter()
            .map(|(key, def)| compile_structure(key, def, &biome_map, &mut errors))
            .collect();
        let fauna = raw
            .fauna
            .iter()
            .map(|(key, def)| compile_fauna(key, def, &mut errors))
            .collect();
        let visual_details = raw
            .visual_details
            .iter()
            .filter_map(|(key, def)| compile_detail(key, def, &field_map, blocks, &mut errors))
            .collect();

        let planets = raw
            .planets
            .iter()
            .filter_map(|(key, def)| {
                compile_planet(
                    key,
                    def,
                    &climate_map,
                    &biome_set_map,
                    &layer_map,
                    &ore_map,
                    &cave_map,
                    &vegetation_map,
                    &structure_map,
                    &fauna_map,
                    &detail_map,
                    &mut errors,
                )
            })
            .collect();

        if errors.is_empty() {
            Ok(ProceduralRegistry {
                planets,
                fields,
                climates,
                biome_sets,
                biomes,
                terrain_layers,
                ores,
                caves,
                vegetation,
                structures,
                fauna,
                visual_details,
            })
        } else {
            Err(errors)
        }
    }
}

fn compile_biome(
    idx: usize,
    key: &str,
    def: &RawBiomeProceduralDef,
    field_map: &HashMap<String, usize>,
    blocks: &BlockRegistry,
    errors: &mut Vec<String>,
) -> CompiledProceduralBiome {
    let top = resolve_block(blocks, key, &def.surface.top, errors).unwrap_or(VoxelId::AIR);
    let under = resolve_block(blocks, key, &def.surface.under, errors).unwrap_or(VoxelId::AIR);
    let hill_field = resolve(field_map, "field", key, &def.terrain.hill_field, errors).unwrap_or(0);
    let ridge_field = def
        .terrain
        .ridge_field
        .as_ref()
        .and_then(|f| resolve(field_map, "field", key, f, errors));

    CompiledProceduralBiome {
        id: idx.min(u8::MAX as usize) as u8,
        key: key.to_string(),
        display_name: def.display_name.clone(),
        surface: CompiledBiomeSurface {
            top,
            under,
            depth: normalized_u32_range(def.surface.depth),
        },
        terrain: CompiledBiomeTerrain {
            base_height: def.terrain.base_height,
            amplitude: def.terrain.amplitude.clamp(0.0, 2.0),
            flatness: def.terrain.flatness.clamp(0.0, 1.0),
            hill_field,
            ridge_field,
            terrace_strength: def.terrain.terrace_strength.clamp(0.0, 1.0),
        },
        color_tint: CompiledBiomeColorTint {
            grass: def.color_tint.grass,
            foliage: def.color_tint.foliage,
        },
        vegetation_tags: def.vegetation_tags.clone(),
        fauna_tags: def.fauna_tags.clone(),
    }
}

fn compile_axis(
    owner: &str,
    _axis_name: &str,
    axis: &RawClimateAxisDef,
    field_map: &HashMap<String, usize>,
    errors: &mut Vec<String>,
) -> CompiledClimateAxis {
    CompiledClimateAxis {
        latitude_bias: axis.latitude_bias,
        fields: axis
            .fields
            .iter()
            .filter_map(|(field, weight)| {
                resolve(field_map, "field", owner, field, errors).map(|idx| (idx, *weight))
            })
            .collect(),
        ocean_bias: axis.ocean_bias,
    }
}

fn compile_ore(
    key: &str,
    def: &RawOreDef,
    field_map: &HashMap<String, usize>,
    blocks: &BlockRegistry,
    errors: &mut Vec<String>,
) -> Option<CompiledOre> {
    Some(CompiledOre {
        key: key.to_string(),
        block: resolve_block(blocks, key, &def.block, errors)?,
        replace: def
            .replace
            .iter()
            .filter_map(|b| resolve_block(blocks, key, b, errors))
            .collect(),
        depth: normalized_u32_range(def.depth),
        density: def.density.clamp(0.0, 1.0),
        vein_size: normalized_u32_range(def.vein_size),
        field: resolve(field_map, "field", key, &def.field, errors)?,
        biome_tags: def.biome_tags.clone(),
    })
}

fn compile_cave(
    key: &str,
    def: &RawCaveDef,
    field_map: &HashMap<String, usize>,
    blocks: &BlockRegistry,
    errors: &mut Vec<String>,
) -> CompiledCave {
    CompiledCave {
        key: key.to_string(),
        carvers: def
            .carvers
            .iter()
            .filter_map(|c| {
                Some(CompiledCaveCarver {
                    kind: c.kind.clone(),
                    field: resolve(field_map, "field", key, &c.field, errors)?,
                    threshold: c.threshold.clamp(0.0, 1.0),
                    radius: normalized_u32_range(c.radius),
                    depth: normalized_u32_range(c.depth),
                })
            })
            .collect(),
        surface_break_chance: def.surface_break_chance.clamp(0.0, 1.0),
        fill_below_sea: def
            .fill_below_sea
            .as_ref()
            .and_then(|b| resolve_block(blocks, key, b, errors)),
    }
}

fn compile_vegetation(
    key: &str,
    def: &RawVegetationDef,
    field_map: &HashMap<String, usize>,
    blocks: &BlockRegistry,
    errors: &mut Vec<String>,
) -> Option<CompiledVegetation> {
    Some(CompiledVegetation {
        key: key.to_string(),
        placement: compile_placement(key, &def.placement, field_map, blocks, errors)?,
        trunk: resolve_block(blocks, key, &def.stamp.trunk, errors)?,
        leaves: resolve_block(blocks, key, &def.stamp.leaves, errors)?,
        height: normalized_u32_range(def.stamp.height),
        canopy_radius: normalized_u32_range(def.stamp.canopy_radius),
        trunk_thickness: clamp_thickness(def.stamp.trunk_thickness),
        branch_count: normalized_u32_range(def.stamp.branch_count),
        branch_length: normalized_u32_range(def.stamp.branch_length),
        canopy_vertical_squash: finite_positive(def.stamp.canopy_vertical_squash, 0.85)
            .clamp(0.2, 3.0),
    })
}

fn clamp_thickness(range: (u32, u32)) -> (u32, u32) {
    let (a, b) = normalized_u32_range(range);
    (a.max(1).min(4), b.max(1).min(4))
}

fn compile_structure(
    key: &str,
    def: &RawStructureDef,
    biome_map: &HashMap<String, usize>,
    errors: &mut Vec<String>,
) -> CompiledStructure {
    if !def.stamp.contains(':') {
        errors.push(format!(
            "Structure '{}': stamp '{}' must be a namespaced key",
            key, def.stamp
        ));
    }
    CompiledStructure {
        key: key.to_string(),
        density: def.placement.density.clamp(0.0, 1.0),
        min_spacing: def.placement.min_spacing.max(1),
        biomes: def
            .placement
            .biomes
            .iter()
            .filter_map(|b| resolve(biome_map, "procedural biome", key, b, errors))
            .collect(),
        slope_max: def.placement.slope_max.clamp(0.0, 1.0),
        footprint_radius: def.footprint_radius,
        priority: def.priority,
        stamp: def.stamp.clone(),
    }
}

fn compile_fauna(key: &str, def: &RawFaunaDef, errors: &mut Vec<String>) -> CompiledFauna {
    if !def.entity.contains(':') {
        errors.push(format!(
            "Fauna '{}': entity '{}' must be a namespaced key",
            key, def.entity
        ));
    }
    CompiledFauna {
        key: key.to_string(),
        entity: def.entity.clone(),
        biome_tags: def.spawn.biome_tags.clone(),
        density: def.spawn.density.clamp(0.0, 1.0),
        group_size: normalized_u32_range(def.spawn.group_size),
        light: normalized_range(def.spawn.light),
        despawn_distance: def.runtime.despawn_distance,
        sim_distance: def.runtime.sim_distance,
    }
}

fn compile_detail(
    key: &str,
    def: &RawVisualDetailDef,
    field_map: &HashMap<String, usize>,
    blocks: &BlockRegistry,
    errors: &mut Vec<String>,
) -> Option<CompiledVisualDetail> {
    Some(CompiledVisualDetail {
        key: key.to_string(),
        placement: compile_placement(key, &def.placement, field_map, blocks, errors)?,
        details: def
            .details
            .iter()
            .filter_map(|d| {
                Some(CompiledVisualDetailItem {
                    block: resolve_block(blocks, key, &d.block, errors)?,
                    weight: d.weight,
                })
            })
            .collect(),
    })
}

fn compile_placement(
    owner: &str,
    def: &RawFeaturePlacementDef,
    field_map: &HashMap<String, usize>,
    blocks: &BlockRegistry,
    errors: &mut Vec<String>,
) -> Option<CompiledFeaturePlacement> {
    Some(CompiledFeaturePlacement {
        surface_blocks: def
            .surface_blocks
            .iter()
            .filter_map(|b| resolve_block(blocks, owner, b, errors))
            .collect(),
        slope_max: def.slope_max.clamp(0.0, 1.0),
        density: def.density.clamp(0.0, 1.0),
        field: resolve(field_map, "field", owner, &def.field, errors)?,
    })
}

#[allow(clippy::too_many_arguments)]
fn compile_planet(
    key: &str,
    def: &RawPlanetProceduralDef,
    climate_map: &HashMap<String, usize>,
    biome_set_map: &HashMap<String, usize>,
    layer_map: &HashMap<String, usize>,
    ore_map: &HashMap<String, usize>,
    cave_map: &HashMap<String, usize>,
    vegetation_map: &HashMap<String, usize>,
    structure_map: &HashMap<String, usize>,
    fauna_map: &HashMap<String, usize>,
    detail_map: &HashMap<String, usize>,
    errors: &mut Vec<String>,
) -> Option<CompiledProceduralPlanet> {
    if def.surface_layer < 4 || def.surface_layer >= def.resolution {
        errors.push(format!(
            "Planet '{}': surface_layer must be in 4..resolution",
            key
        ));
    }
    if def.core_layers < 1 || def.core_layers >= def.surface_layer {
        errors.push(format!(
            "Planet '{}': core_layers must be at least 1 and below surface_layer",
            key
        ));
    }

    Some(CompiledProceduralPlanet {
        key: key.to_string(),
        base: CompiledPlanet {
            key: key.to_string(),
            display_name: def.display_name.clone(),
            seed: def.seed,
            resolution: def.resolution.max(8),
            surface_layer: def.surface_layer,
            voxel_size_meters: finite_positive(def.voxel_size_meters, 1.0),
            edge_rounding_radius_voxels: 0.42,
            core_layers: def.core_layers,
            inner_radius_fraction: def.inner_radius_fraction.clamp(0.02, 0.95),
            max_terrain_offset: def.max_terrain_offset.max(0),
            spawn_clearance_layers: 8.0,
        },
        sea_level_offset: def.sea_level_offset,
        climate: resolve(climate_map, "climate", key, &def.climate, errors)?,
        biome_set: resolve(biome_set_map, "biome set", key, &def.biome_set, errors)?,
        terrain_layers: resolve(
            layer_map,
            "terrain layer set",
            key,
            &def.terrain_layers,
            errors,
        )?,
        caves: resolve_many(cave_map, "cave", key, &def.caves, errors),
        ore_sets: resolve_many(ore_map, "ore", key, &def.ore_sets, errors),
        vegetation_sets: resolve_many(
            vegetation_map,
            "vegetation",
            key,
            &def.vegetation_sets,
            errors,
        ),
        structure_sets: resolve_many(structure_map, "structure", key, &def.structure_sets, errors),
        fauna_sets: resolve_many(fauna_map, "fauna", key, &def.fauna_sets, errors),
        visual_detail_sets: resolve_many(
            detail_map,
            "visual detail",
            key,
            &def.visual_detail_sets,
            errors,
        ),
    })
}

fn resolve_many(
    map: &HashMap<String, usize>,
    label: &str,
    owner: &str,
    keys: &[String],
    errors: &mut Vec<String>,
) -> Vec<usize> {
    keys.iter()
        .filter_map(|key| resolve(map, label, owner, key, errors))
        .collect()
}

fn resolve(
    map: &HashMap<String, usize>,
    label: &str,
    owner: &str,
    key: &str,
    errors: &mut Vec<String>,
) -> Option<usize> {
    match map.get(key).copied() {
        Some(idx) => Some(idx),
        None => {
            errors.push(format!(
                "{} '{}': unknown {} '{}'",
                label, owner, label, key
            ));
            None
        }
    }
}

fn resolve_block(
    blocks: &BlockRegistry,
    owner: &str,
    key: &str,
    errors: &mut Vec<String>,
) -> Option<crate::voxel::VoxelId> {
    match blocks.lookup(key) {
        Some(id) => Some(id),
        None => {
            errors.push(format!("Procedural '{}': unknown block '{}'", owner, key));
            None
        }
    }
}

fn key_map<T>(
    label: &str,
    values: &[(String, T)],
    errors: &mut Vec<String>,
) -> HashMap<String, usize> {
    let mut map = HashMap::with_capacity(values.len());
    for (idx, (key, _)) in values.iter().enumerate() {
        if map.insert(key.clone(), idx).is_some() {
            errors.push(format!("Duplicate {} key '{}'", label, key));
        }
    }
    map
}

fn finite(value: f32, fallback: f32) -> f32 {
    value.is_finite().then_some(value).unwrap_or(fallback)
}

fn finite_positive(value: f32, fallback: f32) -> f32 {
    (value.is_finite() && value > 0.0)
        .then_some(value)
        .unwrap_or(fallback)
}

fn normalized_range(range: (f32, f32)) -> (f32, f32) {
    let a = finite(range.0, 0.0).clamp(0.0, 1.0);
    let b = finite(range.1, 1.0).clamp(0.0, 1.0);
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn normalized_u32_range(range: (u32, u32)) -> (u32, u32) {
    if range.0 <= range.1 {
        range
    } else {
        (range.1, range.0)
    }
}
