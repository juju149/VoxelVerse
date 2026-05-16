use crate::{
    BlockRegistry, CaveSurface, CompiledBiomeColorTint, CompiledBiomeSelector, CompiledBiomeSet,
    CompiledBiomeSurface, CompiledBiomeTerrain, CompiledCave, CompiledCaveCarver, CompiledClimate,
    CompiledClimateAxis, CompiledCurve, CompiledFauna, CompiledFeaturePlacement,
    CompiledHeightCurve, CompiledNoiseField, CompiledNoiseKind, CompiledNoiseRemap, CompiledOre,
    CompiledPlanet, CompiledPlanetStreaming, CompiledProceduralBiome, CompiledProceduralPlanet,
    CompiledStructure, CompiledTerrainLayer, CompiledTerrainLayerSet, CompiledTreeShapeKind,
    CompiledVegetation, CompiledVoxPropDrop, CompiledVoxPropScatter, CompiledVoxPropVariant,
    ContentCompiler, ProceduralRegistry,
};
use std::collections::HashMap;
use vv_content_schema::*;
use vv_pack_loader::RawProceduralPack;
use vv_voxel::VoxelId;

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
        let biome_map = key_map("biome", &raw.biomes, &mut errors);
        let climate_map = key_map("climate", &raw.climates, &mut errors);
        let biome_set_map = key_map("biome set", &raw.biome_sets, &mut errors);
        let layer_map = key_map("terrain layer set", &raw.terrain_layers, &mut errors);
        let ore_map = key_map("ore", &raw.ores, &mut errors);
        let cave_map = key_map("cave", &raw.caves, &mut errors);
        let vegetation_map = key_map("vegetation", &raw.vegetation, &mut errors);
        let structure_map = key_map("structure", &raw.structures, &mut errors);
        let fauna_map = key_map("spawn", &raw.fauna, &mut errors);
        let scatter_map = key_map("prop scatter", &raw.vox_prop_scatters, &mut errors);

        let fields = raw
            .fields
            .iter()
            .map(|(key, def)| compile_field(key, def, &field_map, &mut errors))
            .collect();
        let biomes = raw
            .biomes
            .iter()
            .enumerate()
            .map(|(idx, (key, def))| {
                compile_biome(idx, key, def, &field_map, &biome_map, blocks, &mut errors)
            })
            .collect();
        let climates = raw
            .climates
            .iter()
            .map(|(key, def)| compile_climate(key, def, &field_map, &mut errors))
            .collect();
        let biome_sets = raw
            .biome_sets
            .iter()
            .map(|(key, def)| compile_biome_set(key, def, &biome_map, &mut errors))
            .collect();
        let terrain_layers = raw
            .terrain_layers
            .iter()
            .map(|(key, def)| compile_terrain_layers(key, def, &field_map, blocks, &mut errors))
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
            .map(|(key, def)| compile_structure(key, def))
            .collect();
        let fauna = raw
            .fauna
            .iter()
            .map(|(key, def)| compile_fauna(key, def))
            .collect();
        let vox_prop_scatters = raw
            .vox_prop_scatters
            .iter()
            .filter_map(|(key, def)| {
                compile_vox_prop_scatter(key, def, &field_map, blocks, &mut errors)
            })
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
                    &scatter_map,
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
                vox_prop_scatters,
            })
        } else {
            Err(errors)
        }
    }
}

fn compile_field(
    key: &str,
    def: &RawNoiseFieldDef,
    field_map: &HashMap<String, usize>,
    errors: &mut Vec<String>,
) -> CompiledNoiseField {
    let domain_warp = def
        .domain_warp
        .as_ref()
        .and_then(|warp| resolve(field_map, "field", key, &warp.field, errors))
        .map(|idx| (idx, def.domain_warp.as_ref().unwrap().strength.max(0.0)));

    CompiledNoiseField {
        key: key.to_string(),
        kind: CompiledNoiseKind::from(&def.kind),
        frequency: finite_positive(def.frequency, 0.01),
        amplitude: finite(def.amplitude, 1.0),
        octaves: def.octaves.clamp(1, 12),
        persistence: finite_positive(def.persistence, 0.5),
        lacunarity: finite_positive(def.lacunarity, 2.0),
        seed_salt: stable_hash(&def.seed_salt),
        domain_warp,
        remap: def.remap.as_ref().map(|r| CompiledNoiseRemap {
            in_min: r.in_min,
            in_max: r.in_max,
            out_min: r.out_min,
            out_max: r.out_max,
            curve: CompiledCurve::from(&r.curve),
        }),
    }
}

fn compile_climate(
    key: &str,
    def: &RawClimateDef,
    field_map: &HashMap<String, usize>,
    errors: &mut Vec<String>,
) -> CompiledClimate {
    CompiledClimate {
        key: key.to_string(),
        temperature: axis_from_field(key, &def.fields.temperature, field_map, errors),
        humidity: axis_from_field(key, &def.fields.humidity, field_map, errors),
        continentality: axis_from_field(key, &def.fields.continentality, field_map, errors),
        erosion: axis_from_field(key, &def.fields.erosion, field_map, errors),
        weirdness: axis_from_field(key, &def.fields.weirdness, field_map, errors),
    }
}

fn axis_from_field(
    owner: &str,
    field: &ContentRef,
    field_map: &HashMap<String, usize>,
    errors: &mut Vec<String>,
) -> CompiledClimateAxis {
    CompiledClimateAxis {
        latitude_bias: 0.0,
        fields: resolve(field_map, "field", owner, field, errors)
            .map(|idx| vec![(idx, 1.0)])
            .unwrap_or_default(),
        ocean_bias: 0.0,
    }
}

fn compile_biome(
    idx: usize,
    key: &str,
    def: &RawBiomeProceduralDef,
    field_map: &HashMap<String, usize>,
    biome_map: &HashMap<String, usize>,
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
        .and_then(|field| resolve(field_map, "field", key, field, errors));

    CompiledProceduralBiome {
        id: idx.min(u8::MAX as usize) as u8,
        key: key.to_string(),
        display_name: def.display_name.clone(),
        surface: CompiledBiomeSurface {
            top,
            under,
            depth: normalized_u32_range(def.surface.depth_voxels),
        },
        terrain: CompiledBiomeTerrain {
            base_height: def.terrain.base_height,
            amplitude: def.terrain.amplitude.clamp(0.0, 2.0),
            flatness: def.terrain.flatness.clamp(0.0, 1.0),
            hill_field,
            ridge_field,
            terrace_strength: def.terrain.terrace_strength.clamp(0.0, 1.0),
            height_curve: def
                .terrain
                .height_curve
                .as_ref()
                .map(CompiledHeightCurve::from)
                .unwrap_or_default(),
            mountain_intensity: finite(def.terrain.mountain_intensity, 1.0).clamp(0.0, 2.0),
            slope_smoothing: finite(def.terrain.slope_smoothing, 0.0).clamp(0.0, 1.0),
        },
        color_tint: CompiledBiomeColorTint {
            grass: tuple_color(def.palette.grass),
            foliage: tuple_color(def.palette.foliage),
        },
        vegetation_tags: refs_to_strings(&def.vegetation_tags),
        fauna_tags: refs_to_strings(&def.fauna_tags),
        edge_of: def
            .edge_of
            .as_ref()
            .and_then(|biome| resolve(biome_map, "biome", key, biome, errors)),
    }
}

fn compile_biome_set(
    key: &str,
    def: &RawBiomeSetDef,
    biome_map: &HashMap<String, usize>,
    errors: &mut Vec<String>,
) -> CompiledBiomeSet {
    let RawBiomeSelectionDef::ClimateMap(map) = &def.selection;
    CompiledBiomeSet {
        key: key.to_string(),
        blend_radius: def.blend_radius.clamp(0.02, 0.40),
        selectors: map
            .entries
            .iter()
            .filter_map(|s| {
                resolve(biome_map, "biome", key, &s.biome, errors).map(|biome| {
                    CompiledBiomeSelector {
                        biome,
                        temperature: normalized_range(s.temperature),
                        humidity: normalized_range(s.humidity),
                        roughness: s.elevation.map(normalized_range).unwrap_or((0.0, 1.0)),
                        weight: finite_positive(s.weight, 1.0),
                        continentality: s.continentality.map(normalized_range),
                        erosion: s.erosion.map(normalized_range),
                        weirdness: s.weirdness.map(normalized_range),
                    }
                })
            })
            .collect(),
    }
}

fn compile_terrain_layers(
    key: &str,
    def: &RawTerrainLayerSetDef,
    field_map: &HashMap<String, usize>,
    blocks: &BlockRegistry,
    errors: &mut Vec<String>,
) -> CompiledTerrainLayerSet {
    CompiledTerrainLayerSet {
        key: key.to_string(),
        layers: def
            .layers
            .iter()
            .filter_map(|layer| {
                Some(CompiledTerrainLayer {
                    name: terrain_range_name(layer.range),
                    block: resolve_block(blocks, key, &layer.block, errors)?,
                    depth: layer.thickness.map(normalized_u32_range),
                    depth_from_center: None,
                    all_biomes: true,
                    biomes: Vec::new(),
                    noise_variation: layer
                        .noise_variation
                        .as_ref()
                        .and_then(|field| resolve(field_map, "field", key, field, errors)),
                })
            })
            .collect(),
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
            .filter_map(|block| resolve_block(blocks, key, block, errors))
            .collect(),
        depth: normalized_u32_range(def.depth_voxels),
        density: def.density.clamp(0.0, 1.0),
        vein_size: normalized_u32_range(def.vein_size),
        field: resolve(field_map, "field", key, &def.field, errors)?,
        biome_tags: refs_to_strings(&def.biome_tags),
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
            .fields
            .iter()
            .filter_map(|field| {
                Some(CompiledCaveCarver {
                    kind: "noise".to_string(),
                    field: resolve(field_map, "field", key, field, errors)?,
                    threshold: 0.58,
                    radius: (
                        def.carve.tunnel_radius.0.max(1.0) as u32,
                        def.carve.tunnel_radius.1.max(1.0) as u32,
                    ),
                    depth: (def.carve.min_depth_voxels, def.carve.max_depth_voxels),
                })
            })
            .collect(),
        surface_break_chance: 0.03,
        fill_below_sea: resolve_block(blocks, key, &def.carve.air_block, errors),
    }
}

fn compile_vegetation(
    key: &str,
    def: &RawVegetationDef,
    field_map: &HashMap<String, usize>,
    blocks: &BlockRegistry,
    errors: &mut Vec<String>,
) -> Option<CompiledVegetation> {
    let clump_field = def
        .clump_field
        .as_ref()
        .and_then(|f| resolve(field_map, "field", key, f, errors));
    let scale_variance = match def.scale_variance {
        Some((lo, hi)) => {
            let lo = finite_positive(lo, 1.0);
            let hi = finite_positive(hi, lo);
            if lo <= hi {
                (lo, hi)
            } else {
                (hi, lo)
            }
        }
        None => (1.0, 1.0),
    };
    let placement = CompiledFeaturePlacement {
        surface_blocks: def
            .allowed_surface_blocks
            .iter()
            .filter_map(|b| resolve_block(blocks, key, b, errors))
            .collect(),
        slope_max: (def.slope_max_degrees as f32 / 90.0).clamp(0.0, 1.0),
        density: def.density.clamp(0.0, 1.0),
        field: resolve(field_map, "field", key, &def.scatter_field, errors)?,
        biome_tags: refs_to_strings(&def.biome_tags),
        min_spacing: finite(def.min_spacing_voxels, 0.0).max(0.0),
        jitter_strength: finite(def.jitter_strength, 0.75).clamp(0.0, 1.0),
        clump_field,
        clump_strength: finite(def.clump_strength, 0.0).clamp(0.0, 1.0),
        altitude_range: def.altitude_range.map(|(a, b)| ordered_pair(a, b)),
        humidity_range: def.humidity_range.map(normalized_range),
        temperature_range: def.temperature_range.map(normalized_range),
        slope_min: def
            .slope_min_degrees
            .map(|d| (d as f32 / 90.0).clamp(0.0, 1.0))
            .unwrap_or(0.0),
        scale_variance,
        rotation_variance: finite(def.rotation_variance, 1.0).clamp(0.0, 1.0),
        cave_surface: CaveSurface::TopSurface,
    };
    Some(CompiledVegetation {
        key: key.to_string(),
        placement,
        trunk: resolve_block(blocks, key, &def.trunk, errors)?,
        leaves: resolve_block(blocks, key, &def.leaves, errors)?,
        height: normalized_u32_range(def.height),
        canopy_radius: normalized_u32_range(def.canopy_radius),
        trunk_thickness: normalized_u32_range(def.trunk_thickness),
        branch_count: normalized_u32_range(def.branch_count),
        branch_length: normalized_u32_range(def.branch_length),
        canopy_vertical_squash: finite_positive(def.canopy_vertical_squash, 0.85),
        branch_slope: def.branch_slope,
        canopy_lobe_count: normalized_u32_range(def.canopy_lobe_count),
        trunk_lean_max: finite(def.trunk_lean_max, 0.12),
        shape_kind: CompiledTreeShapeKind::from(&def.shape_kind),
        canopy_density: finite(def.canopy_density, 1.0).clamp(0.05, 1.0),
        root_radius: finite(def.root_radius, 0.0).max(0.0),
        fallen_chance: finite(def.fallen_chance, 0.0).clamp(0.0, 1.0),
        trunk_curve_strength: finite(def.trunk_curve_strength, 0.0).clamp(0.0, 1.0),
    })
}

fn compile_structure(key: &str, def: &RawStructureDef) -> CompiledStructure {
    CompiledStructure {
        key: key.to_string(),
        density: match def.rarity {
            RawStructureRarity::Common => 0.12,
            RawStructureRarity::Uncommon => 0.05,
            RawStructureRarity::Rare => 0.015,
        },
        min_spacing: def.spacing_voxels.0.max(1),
        biomes: Vec::new(),
        slope_max: (def.slope_max_degrees as f32 / 90.0).clamp(0.0, 1.0),
        footprint_radius: (def.spacing_voxels.0 / 32).max(4),
        priority: 0,
        stamp: def.structure.0.clone(),
    }
}

fn compile_fauna(key: &str, def: &RawFaunaDef) -> CompiledFauna {
    CompiledFauna {
        key: key.to_string(),
        entity: def.entity.0.clone(),
        biome_tags: refs_to_strings(&def.biome_tags),
        density: def.density.clamp(0.0, 1.0),
        group_size: normalized_u32_range(def.group_size),
        light: match def.light {
            RawSpawnLightDef::Daylight => (0.45, 1.0),
            RawSpawnLightDef::Night => (0.0, 0.45),
            RawSpawnLightDef::Any => (0.0, 1.0),
        },
        despawn_distance: 128,
        sim_distance: 96,
    }
}

fn compile_vox_prop_scatter(
    key: &str,
    def: &RawVoxPropScatterDef,
    field_map: &HashMap<String, usize>,
    blocks: &BlockRegistry,
    errors: &mut Vec<String>,
) -> Option<CompiledVoxPropScatter> {
    let placement = compile_placement(key, &def.placement, field_map, blocks, errors)?;
    let variants: Vec<CompiledVoxPropVariant> = def
        .variants
        .iter()
        .map(|v| CompiledVoxPropVariant {
            model_key: v.model.0.clone(),
            weight: v.weight.max(1),
            drops: v
                .drops
                .iter()
                .map(|d| CompiledVoxPropDrop {
                    item: d.item.0.clone(),
                    count: d.count,
                    chance: d.chance.clamp(0.0, 1.0),
                })
                .collect(),
            y_offset: v.y_offset,
        })
        .collect();
    let total_weight = variants.iter().map(|v| v.weight).sum();
    Some(CompiledVoxPropScatter {
        key: key.to_string(),
        placement,
        variants,
        total_weight,
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
    scatter_map: &HashMap<String, usize>,
    errors: &mut Vec<String>,
) -> Option<CompiledProceduralPlanet> {
    let RawPlanetShapeDef::SphericalVoxelPlanet(shape) = &def.shape;
    if shape.surface_layer < 4 || shape.surface_layer >= shape.resolution {
        errors.push(format!(
            "Planet '{}': surface_layer must be in 4..resolution",
            key
        ));
    }
    if shape.core_layers < 1 || shape.core_layers >= shape.surface_layer {
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
            resolution: shape.resolution.max(8),
            surface_layer: shape.surface_layer,
            voxel_size_meters: finite_positive(shape.voxel_size_meters, 1.0),
            edge_rounding_radius_voxels: finite(shape.edge_rounding_radius_voxels, 0.16)
                .clamp(0.0, 0.35),
            core_layers: shape.core_layers,
            inner_radius_fraction: 0.35,
            max_terrain_offset: shape.max_terrain_offset.max(0),
            spawn_clearance_layers: 8.0,
        },
        sea_level_offset: shape.sea_level_offset,
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
        ore_sets: resolve_many(ore_map, "ore", key, &def.ores, errors),
        vegetation_sets: resolve_many(vegetation_map, "vegetation", key, &def.vegetation, errors),
        structure_sets: resolve_many(structure_map, "structure", key, &def.structures, errors),
        fauna_sets: resolve_many(fauna_map, "spawn", key, &def.spawns, errors),
        vox_prop_scatters: resolve_many(
            scatter_map,
            "prop scatter",
            key,
            &def.visual_details,
            errors,
        ),
        streaming: CompiledPlanetStreaming {
            near_voxel_lod_radius: def.streaming.near_voxel_lod_radius,
            far_surface_lod_radius: def.streaming.far_surface_lod_radius,
            upload_budget_chunks_per_frame: def.streaming.upload_budget_chunks_per_frame,
            region_cell_voxels: def.streaming.region_cell_voxels.clamp(16, 256),
            feature_budget_per_chunk: def.streaming.feature_budget_per_chunk,
        },
    })
}

fn compile_placement(
    owner: &str,
    def: &RawFeaturePlacementDef,
    field_map: &HashMap<String, usize>,
    blocks: &BlockRegistry,
    errors: &mut Vec<String>,
) -> Option<CompiledFeaturePlacement> {
    let clump_field = def
        .clump_field
        .as_ref()
        .and_then(|f| resolve(field_map, "field", owner, f, errors));
    let scale_variance = match def.scale_variance {
        Some((lo, hi)) => {
            let lo = finite_positive(lo, 1.0);
            let hi = finite_positive(hi, lo);
            if lo <= hi {
                (lo, hi)
            } else {
                (hi, lo)
            }
        }
        None => (1.0, 1.0),
    };
    Some(CompiledFeaturePlacement {
        surface_blocks: def
            .allowed_surface_blocks
            .iter()
            .filter_map(|block| resolve_block(blocks, owner, block, errors))
            .collect(),
        slope_max: (def.slope_max_degrees as f32 / 90.0).clamp(0.0, 1.0),
        density: def.density.clamp(0.0, 1.0),
        field: resolve(field_map, "field", owner, &def.scatter_field, errors)?,
        biome_tags: refs_to_strings(&def.biome_tags),
        min_spacing: finite(def.min_spacing_voxels, 0.0).max(0.0),
        jitter_strength: finite(def.jitter_strength, 0.75).clamp(0.0, 1.0),
        clump_field,
        clump_strength: finite(def.clump_strength, 0.0).clamp(0.0, 1.0),
        altitude_range: def.altitude_range.map(|(a, b)| ordered_pair(a, b)),
        humidity_range: def.humidity_range.map(normalized_range),
        temperature_range: def.temperature_range.map(normalized_range),
        slope_min: def
            .slope_min_degrees
            .map(|d| (d as f32 / 90.0).clamp(0.0, 1.0))
            .unwrap_or(0.0),
        scale_variance,
        rotation_variance: finite(def.rotation_variance, 1.0).clamp(0.0, 1.0),
        cave_surface: match def.cave_surface {
            RawCaveSurface::TopSurface => CaveSurface::TopSurface,
            RawCaveSurface::CaveFloor => CaveSurface::CaveFloor,
            RawCaveSurface::CaveCeiling => CaveSurface::CaveCeiling,
        },
    })
}

fn ordered_pair(a: f32, b: f32) -> (f32, f32) {
    let a = finite(a, 0.0);
    let b = finite(b, 0.0);
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn resolve_many(
    map: &HashMap<String, usize>,
    label: &str,
    owner: &str,
    keys: &[ContentRef],
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
    key: &ContentRef,
    errors: &mut Vec<String>,
) -> Option<usize> {
    // Direct exact match.
    if let Some(idx) = map.get(&key.0).copied() {
        return Some(idx);
    }
    // Short-name stem fallback: "rolling_hills" matches "core:field/rolling_hills".
    if !key.0.contains(':') {
        let suffix = format!("/{}", &key.0);
        if let Some(idx) = map
            .iter()
            .find_map(|(k, &v)| k.ends_with(&suffix).then_some(v))
        {
            return Some(idx);
        }
    }
    errors.push(format!(
        "{} '{}': unknown {} '{}'",
        label, owner, label, key.0
    ));
    None
}

fn resolve_block(
    blocks: &BlockRegistry,
    owner: &str,
    key: &ContentRef,
    errors: &mut Vec<String>,
) -> Option<VoxelId> {
    if let Some(id) = blocks.lookup(&key.0) {
        return Some(id);
    }
    if let Some(id) = blocks.lookup_stem(&key.0) {
        return Some(id);
    }
    errors.push(format!("Procedural '{}': unknown block '{}'", owner, key.0));
    None
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

fn refs_to_strings(values: &[ContentRef]) -> Vec<String> {
    values.iter().map(|value| value.0.clone()).collect()
}

fn tuple_color(value: (f32, f32, f32)) -> [f32; 3] {
    [value.0, value.1, value.2]
}

fn terrain_range_name(value: RawTerrainLayerRange) -> String {
    match value {
        RawTerrainLayerRange::Surface => "surface",
        RawTerrainLayerRange::Subsurface => "subsurface",
        RawTerrainLayerRange::Crust => "crust",
        RawTerrainLayerRange::DeepCrust => "deep_crust",
        RawTerrainLayerRange::Core => "core",
    }
    .to_string()
}

fn stable_hash(value: &str) -> u32 {
    let mut hash = 0x811c9dc5u32;
    for byte in value.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

fn finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

fn finite_positive(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
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

#[cfg(test)]
mod tests {
    use super::ContentCompiler;
    use std::path::Path;
    use vv_pack_loader::PackLoader;

    #[test]
    #[ignore = "core pack is mid-migration; re-enable once object files parse cleanly"]
    fn core_pack_procedural_compiles_from_current_schema() {
        let core_pack_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core");
        let pack = PackLoader::load_from_dir(&core_pack_dir).expect("core pack");
        let objects =
            crate::object_compiler::compile_objects(pack.objects).expect("unified objects compile");
        let procedural_pack =
            PackLoader::load_procedural_from_dir(&core_pack_dir).expect("procedural pack");
        let procedural = ContentCompiler::compile_procedural(procedural_pack, &objects.blocks)
            .expect("procedural");

        assert!(!procedural.planets.is_empty());
        assert!(!procedural.fields.is_empty());
        assert!(!procedural.biomes.is_empty());
        assert!(!procedural.terrain_layers.is_empty());
    }
}
