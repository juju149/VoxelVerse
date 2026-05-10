use crate::{
    BlockRegistry, CompiledBiomeColorTint, CompiledBiomeSelector, CompiledBiomeSet,
    CompiledBiomeSurface, CompiledBiomeTerrain, CompiledCave, CompiledCaveCarver, CompiledClimate,
    CompiledClimateAxis, CompiledCurve, CompiledFauna, CompiledFeaturePlacement,
    CompiledNoiseField, CompiledNoiseKind, CompiledNoiseRemap, CompiledOre, CompiledPlanet,
    CompiledProceduralBiome, CompiledProceduralPlanet, CompiledStructure, CompiledTerrainLayer,
    CompiledTerrainLayerSet, CompiledTreeShapeKind, CompiledVegetation, CompiledVisualDetail,
    CompiledVisualDetailItem, ContentCompiler, ProceduralRegistry,
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
                "Procedural pack must define at least one planet.".to_string(),
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
        let detail_map = key_map("visual detail", &raw.visual_details, &mut errors);

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
        },
        color_tint: CompiledBiomeColorTint {
            grass: tuple_color(def.palette.grass),
            foliage: tuple_color(def.palette.foliage),
        },
        vegetation_tags: refs_to_strings(&def.placement.vegetation_tags),
        fauna_tags: refs_to_strings(&def.placement.fauna_tags),
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
        blend_radius: 0.06,
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
    Some(CompiledVegetation {
        key: key.to_string(),
        placement: compile_placement(key, &def.placement, field_map, blocks, errors)?,
        trunk: resolve_block(blocks, key, &def.stamp.trunk, errors)?,
        leaves: resolve_block(blocks, key, &def.stamp.leaves, errors)?,
        height: normalized_u32_range(def.stamp.height),
        canopy_radius: normalized_u32_range(def.stamp.canopy_radius),
        trunk_thickness: normalized_u32_range(def.stamp.trunk_thickness),
        branch_count: normalized_u32_range(def.stamp.branch_count),
        branch_length: normalized_u32_range(def.stamp.branch_length),
        canopy_vertical_squash: finite_positive(def.stamp.canopy_vertical_squash, 0.85),
        branch_slope: def.stamp.branch_slope,
        canopy_lobe_count: normalized_u32_range(def.stamp.canopy_lobe_count),
        trunk_lean_max: finite(def.stamp.trunk_lean_max, 0.12),
        shape_kind: CompiledTreeShapeKind::from(&def.stamp.shape_kind),
        canopy_density: finite(def.stamp.canopy_density, 1.0).clamp(0.05, 1.0),
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

fn compile_detail(
    key: &str,
    def: &RawVisualDetailDef,
    field_map: &HashMap<String, usize>,
    blocks: &BlockRegistry,
    errors: &mut Vec<String>,
) -> Option<CompiledVisualDetail> {
    Some(CompiledVisualDetail {
        key: key.to_string(),
        placement: CompiledFeaturePlacement {
            surface_blocks: def
                .surface_blocks
                .iter()
                .filter_map(|block| resolve_block(blocks, key, block, errors))
                .collect(),
            slope_max: 0.65,
            density: def.density.clamp(0.0, 1.0),
            field: resolve(
                field_map,
                "field",
                key,
                &ContentRef("core:field/flower_noise".to_string()),
                errors,
            )?,
            biome_tags: refs_to_strings(&def.biome_tags),
        },
        details: def
            .details
            .iter()
            .filter_map(|detail| {
                Some(CompiledVisualDetailItem {
                    block: resolve_block(blocks, key, &detail.block, errors)?,
                    weight: detail.weight,
                })
            })
            .collect(),
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
        visual_detail_sets: resolve_many(
            detail_map,
            "visual detail",
            key,
            &def.visual_details,
            errors,
        ),
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
            .allowed_surface_blocks
            .iter()
            .filter_map(|block| resolve_block(blocks, owner, block, errors))
            .collect(),
        slope_max: (def.slope_max_degrees as f32 / 90.0).clamp(0.0, 1.0),
        density: def.density.clamp(0.0, 1.0),
        field: resolve(field_map, "field", owner, &def.scatter_field, errors)?,
        biome_tags: refs_to_strings(&def.biome_tags),
    })
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
    match map.get(&key.0).copied() {
        Some(idx) => Some(idx),
        None => {
            errors.push(format!(
                "{} '{}': unknown {} '{}'",
                label, owner, label, key.0
            ));
            None
        }
    }
}

fn resolve_block(
    blocks: &BlockRegistry,
    owner: &str,
    key: &ContentRef,
    errors: &mut Vec<String>,
) -> Option<VoxelId> {
    match blocks.lookup(&key.0) {
        Some(id) => Some(id),
        None => {
            errors.push(format!("Procedural '{}': unknown block '{}'", owner, key.0));
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
    if a <= b { (a, b) } else { (b, a) }
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
    fn core_pack_procedural_compiles_from_current_schema() {
        let core_pack_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core");
        let pack = PackLoader::load_from_dir(&core_pack_dir).expect("core pack");
        let blocks = ContentCompiler::compile_blocks(pack.blocks, pack.materials).expect("blocks");
        let procedural_pack =
            PackLoader::load_procedural_from_dir(&core_pack_dir).expect("procedural pack");
        let procedural =
            ContentCompiler::compile_procedural(procedural_pack, &blocks).expect("procedural");

        assert!(!procedural.planets.is_empty());
        assert!(!procedural.fields.is_empty());
        assert!(!procedural.biomes.is_empty());
        assert!(!procedural.terrain_layers.is_empty());
    }
}
