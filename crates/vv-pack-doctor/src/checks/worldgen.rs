//! World generation validation and planet report assembly.
//!
//! This module owns high-level content sanity for procedural planets:
//! references between world definitions, biome selector ranges, placement
//! densities, streaming budgets and the summary used by JSON/HTML reports.

use crate::index::PackIndex;
use crate::report::{
    BiomeSetSummary, BiomeSummary, CaveSummary, Diagnostic, FeatureSummary, OreSummary,
    PlanetCounts, PlanetProfileSummary, Report,
};
use crate::scan::{ParsedWorldFile, WorldCategory};

const CHECK: &str = "worldgen";

pub fn run(index: &PackIndex<'_>, report: &mut Report) {
    report.planet.counts = PlanetCounts {
        planet_profiles: count(index, WorldCategory::Planets),
        biome_sets: count(index, WorldCategory::BiomeSet),
        biomes: count(index, WorldCategory::Biome),
        vegetation_rules: count(index, WorldCategory::Vegetation),
        prop_scatters: count(index, WorldCategory::PropScatter),
        ore_rules: count(index, WorldCategory::Ores),
        cave_rules: count(index, WorldCategory::Caves),
        render_profiles: index.scan.render.profiles.len(),
    };

    check_planets(index, report);
    check_biome_sets(index, report);
    check_biomes(index, report);
    check_noise_fields(index, report);
    check_features(index, report);
    check_ores(index, report);
    check_caves(index, report);
}

fn count(index: &PackIndex<'_>, category: WorldCategory) -> usize {
    index
        .scan
        .world_files
        .iter()
        .filter(|f| f.category == category)
        .count()
}

fn check_planets(index: &PackIndex<'_>, report: &mut Report) {
    let planets: Vec<_> = index
        .scan
        .world_files
        .iter()
        .filter(|f| f.category == WorldCategory::Planets)
        .collect();
    if planets.is_empty() {
        report.error(
            Diagnostic::new(CHECK, "pack contains no planet profiles")
                .with_path("defs/world/planets/".to_string())
                .with_suggestion(
                    "add at least one `defs/world/planets/*.profile.ron` so the world can spawn",
                ),
        );
        return;
    }

    for planet in planets {
        require_world_ref(index, report, planet, "climate", WorldCategory::Climate);
        require_world_ref(index, report, planet, "biome_set", WorldCategory::BiomeSet);
        require_world_ref(
            index,
            report,
            planet,
            "terrain_layers",
            WorldCategory::Terrain,
        );
        require_world_ref_list(index, report, planet, "caves", WorldCategory::Caves);
        require_world_ref_list(index, report, planet, "ores", WorldCategory::Ores);
        require_world_ref_list(
            index,
            report,
            planet,
            "vegetation",
            WorldCategory::Vegetation,
        );
        require_world_ref_list(
            index,
            report,
            planet,
            "structures",
            WorldCategory::Structures,
        );
        require_world_ref_list(
            index,
            report,
            planet,
            "visual_details",
            WorldCategory::PropScatter,
        );

        let streaming = value_field(&planet.value, "streaming");
        let near = streaming
            .and_then(|v| u32_field(v, "near_voxel_lod_radius"))
            .unwrap_or(0);
        let far = streaming
            .and_then(|v| u32_field(v, "far_surface_lod_radius"))
            .unwrap_or(0);
        let upload = streaming
            .and_then(|v| u32_field(v, "upload_budget_chunks_per_frame"))
            .unwrap_or(0);
        let region = streaming
            .and_then(|v| u32_field(v, "region_cell_voxels"))
            .unwrap_or(64);
        let features = streaming
            .and_then(|v| u32_field(v, "feature_budget_per_chunk"))
            .unwrap_or(384);

        if near == 0 || far == 0 || far <= near {
            report.error(
                Diagnostic::new(CHECK, "planet streaming LOD radii are impossible")
                    .with_path(planet.rel_path.clone())
                    .with_id(planet.id.clone())
                    .with_field("streaming.near_voxel_lod_radius/far_surface_lod_radius")
                    .with_suggestion("use far_surface_lod_radius > near_voxel_lod_radius > 0"),
            );
        }
        if upload == 0 || upload > 16 {
            report.warn(
                Diagnostic::new(
                    CHECK,
                    format!("upload budget {upload} chunks/frame can cause streaming spikes"),
                )
                .with_path(planet.rel_path.clone())
                .with_id(planet.id.clone())
                .with_field("streaming.upload_budget_chunks_per_frame")
                .with_suggestion("keep the core pack between 1 and 16 chunks/frame"),
            );
        }
        if !(16..=256).contains(&region) || !region.is_power_of_two() {
            report.error(
                Diagnostic::new(
                    CHECK,
                    "region_cell_voxels must be a power of two from 16 to 256",
                )
                .with_path(planet.rel_path.clone())
                .with_id(planet.id.clone())
                .with_field("streaming.region_cell_voxels"),
            );
        }
        if features == 0 || features > 512 {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!("feature budget {features} per chunk is outside the safe range"),
                )
                .with_path(planet.rel_path.clone())
                .with_id(planet.id.clone())
                .with_field("streaming.feature_budget_per_chunk")
                .with_suggestion("use 64..512; 384 is the recommended dense-but-safe core budget"),
            );
        } else if features > 384 {
            report.warn(
                Diagnostic::new(
                    CHECK,
                    format!(
                        "feature budget {features} per chunk is above the recommended core budget"
                    ),
                )
                .with_path(planet.rel_path.clone())
                .with_id(planet.id.clone())
                .with_field("streaming.feature_budget_per_chunk")
                .with_suggestion(
                    "prefer 384 unless a profile-specific performance test justifies more",
                ),
            );
        }

        report.planet.planet_profiles.push(PlanetProfileSummary {
            id: planet.id.clone(),
            display_name: display_name(planet),
            near_voxel_lod_radius: near,
            far_surface_lod_radius: far,
            upload_budget_chunks_per_frame: upload,
            region_cell_voxels: region,
            feature_budget_per_chunk: features,
            vegetation_refs: string_list_field(&planet.value, "vegetation").len(),
            prop_scatter_refs: string_list_field(&planet.value, "visual_details").len(),
            ore_refs: string_list_field(&planet.value, "ores").len(),
            cave_refs: string_list_field(&planet.value, "caves").len(),
        });
        report.planet.budget_notes.push(format!(
            "{}: near LOD {}, far LOD {}, upload {}/frame, feature budget {}",
            planet.id, near, far, upload, features
        ));
    }
}

fn check_biome_sets(index: &PackIndex<'_>, report: &mut Report) {
    for set in index
        .scan
        .world_files
        .iter()
        .filter(|f| f.category == WorldCategory::BiomeSet)
    {
        let entries: &[ron::Value] = find_field(&set.value, "entries")
            .and_then(as_seq)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let blend = f32_field(&set.value, "blend_radius").unwrap_or(0.08);
        if entries.is_empty() {
            report.error(
                Diagnostic::new(CHECK, "biome set has no climate-map entries")
                    .with_path(set.rel_path.clone())
                    .with_id(set.id.clone())
                    .with_field("selection.entries"),
            );
        }
        if !(0.0..=0.35).contains(&blend) {
            report.warn(
                Diagnostic::new(
                    CHECK,
                    format!("biome blend radius {blend:.3} is outside the stable visual range"),
                )
                .with_path(set.rel_path.clone())
                .with_id(set.id.clone())
                .with_field("blend_radius")
                .with_suggestion("use 0.04..0.30 for readable transitions without muddy blends"),
            );
        }
        for (i, entry) in entries.iter().enumerate() {
            let field = format!("selection.entries[{i}]");
            if let Some(biome) = str_field(entry, "biome") {
                if index.resolve_world(WorldCategory::Biome, biome).is_none() {
                    report.error(
                        Diagnostic::new(
                            CHECK,
                            format!("biome selector references unknown biome '{biome}'"),
                        )
                        .with_path(set.rel_path.clone())
                        .with_id(set.id.clone())
                        .with_field(format!("{field}.biome")),
                    );
                }
            } else {
                report.error(
                    Diagnostic::new(CHECK, "biome selector has no biome")
                        .with_path(set.rel_path.clone())
                        .with_id(set.id.clone())
                        .with_field(format!("{field}.biome")),
                );
            }
            for axis in [
                "temperature",
                "humidity",
                "elevation",
                "continentality",
                "erosion",
                "weirdness",
            ] {
                if let Some(range) = f32_pair_field(entry, axis) {
                    check_unit_range(report, set, &format!("{field}.{axis}"), range);
                }
            }
            let weight = f32_field(entry, "weight").unwrap_or(0.0);
            if weight <= 0.0 || weight > 4.0 {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!("biome selector weight {weight:.3} is not credible"),
                    )
                    .with_path(set.rel_path.clone())
                    .with_id(set.id.clone())
                    .with_field(format!("{field}.weight")),
                );
            }
        }
        report.planet.biome_sets.push(BiomeSetSummary {
            id: set.id.clone(),
            display_name: display_name(set),
            selectors: entries.len(),
            blend_radius: blend,
        });
    }
}

fn check_biomes(index: &PackIndex<'_>, report: &mut Report) {
    for biome in index
        .scan
        .world_files
        .iter()
        .filter(|f| f.category == WorldCategory::Biome)
    {
        let surface = value_field(&biome.value, "surface");
        let top = surface.and_then(|v| str_field(v, "top")).unwrap_or("");
        let under = surface.and_then(|v| str_field(v, "under")).unwrap_or("");
        if top.is_empty() {
            report.error(
                Diagnostic::new(CHECK, "biome has no `surface.top` block")
                    .with_path(biome.rel_path.clone())
                    .with_id(biome.id.clone())
                    .with_field("surface.top"),
            );
        }
        if under.is_empty() {
            report.error(
                Diagnostic::new(CHECK, "biome has no `surface.under` block")
                    .with_path(biome.rel_path.clone())
                    .with_id(biome.id.clone())
                    .with_field("surface.under"),
            );
        }
        if let Some(depth) = surface
            .and_then(|v| u32_pair_field(v, "depth_voxels").or_else(|| u32_pair_field(v, "depth")))
        {
            if depth.0 == 0 || depth.0 > depth.1 {
                report.error(
                    Diagnostic::new(CHECK, "biome surface depth range is impossible")
                        .with_path(biome.rel_path.clone())
                        .with_id(biome.id.clone())
                        .with_field("surface.depth_voxels"),
                );
            }
        }

        let terrain = value_field(&biome.value, "terrain");
        let amplitude = terrain
            .and_then(|v| f32_field(v, "amplitude"))
            .unwrap_or(0.0);
        let flatness = terrain
            .and_then(|v| f32_field(v, "flatness"))
            .unwrap_or(0.0);
        if !(0.0..=1.25).contains(&amplitude) {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!("biome terrain amplitude {amplitude:.3} is outside safe bounds"),
                )
                .with_path(biome.rel_path.clone())
                .with_id(biome.id.clone())
                .with_field("terrain.amplitude"),
            );
        }
        if !(0.0..=1.0).contains(&flatness) {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!("biome terrain flatness {flatness:.3} must be 0..1"),
                )
                .with_path(biome.rel_path.clone())
                .with_id(biome.id.clone())
                .with_field("terrain.flatness"),
            );
        }
        if let Some(terrain) = terrain {
            require_noise_ref(index, report, biome, terrain, "hill_field");
            require_noise_ref(index, report, biome, terrain, "ridge_field");
        }

        report.planet.biomes.push(BiomeSummary {
            id: biome.id.clone(),
            display_name: display_name(biome),
            surface_top: top.to_string(),
            surface_under: under.to_string(),
            amplitude,
            flatness,
            tags: string_list_field(&biome.value, "tags"),
        });
    }
}

fn check_noise_fields(index: &PackIndex<'_>, report: &mut Report) {
    for field in index
        .scan
        .world_files
        .iter()
        .filter(|f| f.category == WorldCategory::Noise)
    {
        let frequency = f32_field(&field.value, "frequency").unwrap_or(0.0);
        let amplitude = f32_field(&field.value, "amplitude").unwrap_or(0.0);
        let octaves = u32_field(&field.value, "octaves").unwrap_or(0);
        let persistence = f32_field(&field.value, "persistence").unwrap_or(-1.0);
        let lacunarity = f32_field(&field.value, "lacunarity").unwrap_or(0.0);
        if frequency <= 0.0 || frequency > 64.0 {
            report.error(diag(
                field,
                format!("noise frequency {frequency:.3} is invalid"),
                "frequency",
            ));
        }
        if amplitude < 0.0 || amplitude > 4.0 {
            report.error(diag(
                field,
                format!("noise amplitude {amplitude:.3} is invalid"),
                "amplitude",
            ));
        }
        if octaves == 0 || octaves > 8 {
            report.warn(diag(
                field,
                format!("noise octaves {octaves} are expensive or invalid"),
                "octaves",
            ));
        }
        if !(0.0..=1.0).contains(&persistence) {
            report.error(diag(
                field,
                format!("noise persistence {persistence:.3} must be 0..1"),
                "persistence",
            ));
        }
        if !(1.0..=4.0).contains(&lacunarity) {
            report.warn(diag(
                field,
                format!("noise lacunarity {lacunarity:.3} is unusual"),
                "lacunarity",
            ));
        }
        if let Some(warp) = value_field(&field.value, "domain_warp").and_then(unwrap_option) {
            require_noise_ref(index, report, field, warp, "field");
        }
        if let Some(remap) = value_field(&field.value, "remap").and_then(unwrap_option) {
            if let (Some(lo), Some(hi)) = (f32_field(remap, "in_min"), f32_field(remap, "in_max")) {
                if lo >= hi {
                    report.error(diag(field, "noise remap input range is inverted", "remap"));
                }
            }
        }
    }
}

fn check_features(index: &PackIndex<'_>, report: &mut Report) {
    for feature in index.scan.world_files.iter().filter(|f| {
        f.category == WorldCategory::Vegetation || f.category == WorldCategory::PropScatter
    }) {
        let placement = value_field(&feature.value, "placement").unwrap_or(&feature.value);
        let density = f32_field(placement, "density").unwrap_or(0.0);
        let spacing = f32_field(placement, "min_spacing_voxels").unwrap_or(0.0);
        let kind = if feature.category == WorldCategory::PropScatter {
            "prop_scatter"
        } else {
            "vegetation"
        };
        if !(0.0..=1.0).contains(&density) {
            report.error(diag(
                feature,
                format!("feature density {density:.3} must be 0..1"),
                "density",
            ));
        } else if density > 0.35 {
            report.warn(diag(
                feature,
                format!("feature density {density:.3} is visually heavy"),
                "density",
            ));
        }
        if spacing < 1.0 {
            report.error(diag(
                feature,
                "feature min_spacing_voxels must be at least 1",
                "min_spacing_voxels",
            ));
        }
        if string_list_field(placement, "allowed_surface_blocks").is_empty()
            && string_list_field(placement, "allowed_surface_tags").is_empty()
        {
            report.error(
                diag(
                    feature,
                    "feature has no allowed surface blocks or tags",
                    "allowed_surface_blocks",
                )
                .with_suggestion(
                    "gate every feature to explicit surfaces so mods cannot scatter everywhere",
                ),
            );
        }
        require_noise_ref(index, report, feature, placement, "scatter_field");
        require_noise_ref(index, report, feature, placement, "clump_field");
        check_optional_unit_pair(report, feature, placement, "humidity_range");
        check_optional_unit_pair(report, feature, placement, "temperature_range");
        if let Some((lo, hi)) = f32_pair_field(placement, "scale_variance") {
            if lo <= 0.0 || lo > hi || hi > 4.0 {
                report.error(diag(
                    feature,
                    "scale_variance must be positive and ordered",
                    "scale_variance",
                ));
            }
        }
        let variants = value_field(&feature.value, "variants")
            .and_then(as_seq)
            .map(|v| v.len())
            .unwrap_or(0);
        if feature.category == WorldCategory::PropScatter && variants == 0 {
            report.error(diag(feature, "prop scatter has no variants", "variants"));
        }
        report.planet.features.push(FeatureSummary {
            id: feature.id.clone(),
            kind: kind.to_string(),
            density,
            min_spacing_voxels: spacing,
            variant_count: variants,
        });
    }
}

fn check_ores(index: &PackIndex<'_>, report: &mut Report) {
    for ore in index
        .scan
        .world_files
        .iter()
        .filter(|f| f.category == WorldCategory::Ores)
    {
        require_noise_ref(index, report, ore, &ore.value, "field");
        let density = f32_field(&ore.value, "density").unwrap_or(0.0);
        if density <= 0.0 || density > 0.25 {
            report.error(diag(
                ore,
                format!("ore density {density:.3} is outside safe bounds"),
                "density",
            ));
        }
        let depth = u32_pair_field(&ore.value, "depth_voxels")
            .or_else(|| u32_pair_field(&ore.value, "depth"))
            .unwrap_or_default();
        if depth.0 > depth.1 {
            report.error(diag(ore, "ore depth range is inverted", "depth_voxels"));
        }
        report.planet.ores.push(OreSummary {
            id: ore.id.clone(),
            block: str_field(&ore.value, "block").unwrap_or("").to_string(),
            density,
            depth_voxels: depth,
        });
    }
}

fn check_caves(index: &PackIndex<'_>, report: &mut Report) {
    for cave in index
        .scan
        .world_files
        .iter()
        .filter(|f| f.category == WorldCategory::Caves)
    {
        for (i, field) in string_list_field(&cave.value, "fields").iter().enumerate() {
            if index.resolve_world(WorldCategory::Noise, field).is_none() {
                report.error(
                    Diagnostic::new(CHECK, format!("cave field '{field}' does not exist"))
                        .with_path(cave.rel_path.clone())
                        .with_id(cave.id.clone())
                        .with_field(format!("fields[{i}]")),
                );
            }
        }
        let carve = value_field(&cave.value, "carve");
        let depth = carve
            .map(|v| {
                (
                    u32_field(v, "min_depth_voxels").unwrap_or(0),
                    u32_field(v, "max_depth_voxels").unwrap_or(0),
                )
            })
            .unwrap_or_default();
        let tunnel = carve
            .and_then(|v| f32_pair_field(v, "tunnel_radius"))
            .unwrap_or_default();
        let chamber = carve
            .and_then(|v| f32_pair_field(v, "chamber_radius"))
            .unwrap_or_default();
        if depth.0 >= depth.1 {
            report.error(diag(cave, "cave depth range is impossible", "carve"));
        }
        if tunnel.0 <= 0.0 || tunnel.0 > tunnel.1 || chamber.0 < tunnel.0 || chamber.1 < chamber.0 {
            report.error(diag(cave, "cave radius ranges are impossible", "carve"));
        }
        report.planet.caves.push(CaveSummary {
            id: cave.id.clone(),
            fields: string_list_field(&cave.value, "fields").len(),
            depth_voxels: depth,
            tunnel_radius: tunnel,
            chamber_radius: chamber,
        });
    }
}

fn require_world_ref(
    index: &PackIndex<'_>,
    report: &mut Report,
    file: &ParsedWorldFile,
    field: &str,
    category: WorldCategory,
) {
    match str_field(&file.value, field) {
        Some(r) if index.resolve_world(category, r).is_some() => {}
        Some(r) => report.error(
            Diagnostic::new(
                CHECK,
                format!("'{field}' references unknown world definition '{r}'"),
            )
            .with_path(file.rel_path.clone())
            .with_id(file.id.clone())
            .with_field(field.to_string()),
        ),
        None => report.error(
            Diagnostic::new(CHECK, format!("missing required world reference '{field}'"))
                .with_path(file.rel_path.clone())
                .with_id(file.id.clone())
                .with_field(field.to_string()),
        ),
    }
}

fn require_world_ref_list(
    index: &PackIndex<'_>,
    report: &mut Report,
    file: &ParsedWorldFile,
    field: &str,
    category: WorldCategory,
) {
    for (i, r) in string_list_field(&file.value, field).iter().enumerate() {
        if index.resolve_world(category, r).is_none() {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!("'{field}' references unknown world definition '{r}'"),
                )
                .with_path(file.rel_path.clone())
                .with_id(file.id.clone())
                .with_field(format!("{field}[{i}]")),
            );
        }
    }
}

fn require_noise_ref(
    index: &PackIndex<'_>,
    report: &mut Report,
    file: &ParsedWorldFile,
    value: &ron::Value,
    field: &str,
) {
    let Some(r) = str_field(value, field) else {
        return;
    };
    if index.resolve_world(WorldCategory::Noise, r).is_none() {
        report.error(
            Diagnostic::new(CHECK, format!("noise field '{r}' does not exist"))
                .with_path(file.rel_path.clone())
                .with_id(file.id.clone())
                .with_field(field.to_string()),
        );
    }
}

fn check_unit_range(report: &mut Report, file: &ParsedWorldFile, field: &str, range: (f32, f32)) {
    if range.0 < 0.0 || range.1 > 1.0 || range.0 > range.1 {
        report.error(
            Diagnostic::new(
                CHECK,
                format!(
                    "range ({:.3}, {:.3}) must be ordered inside 0..1",
                    range.0, range.1
                ),
            )
            .with_path(file.rel_path.clone())
            .with_id(file.id.clone())
            .with_field(field.to_string()),
        );
    }
}

fn check_optional_unit_pair(
    report: &mut Report,
    file: &ParsedWorldFile,
    value: &ron::Value,
    field: &str,
) {
    if let Some(range) = f32_pair_field(value, field) {
        check_unit_range(report, file, field, range);
    }
}

fn diag(file: &ParsedWorldFile, message: impl Into<String>, field: &str) -> Diagnostic {
    Diagnostic::new(CHECK, message)
        .with_path(file.rel_path.clone())
        .with_id(file.id.clone())
        .with_field(field.to_string())
}

fn display_name(file: &ParsedWorldFile) -> String {
    str_field(&file.value, "display_name")
        .or_else(|| str_field(&file.value, "name"))
        .unwrap_or_else(|| file.id.rsplit('/').next().unwrap_or(&file.id))
        .to_string()
}

fn value_field<'a>(value: &'a ron::Value, key: &str) -> Option<&'a ron::Value> {
    let value = unwrap_option(value).unwrap_or(value);
    let ron::Value::Map(map) = value else {
        return None;
    };
    for (k, v) in map.iter() {
        if let ron::Value::String(s) = k {
            if s == key {
                return Some(unwrap_option(v).unwrap_or(v));
            }
        }
    }
    None
}

fn find_field<'a>(value: &'a ron::Value, key: &str) -> Option<&'a ron::Value> {
    if let Some(v) = value_field(value, key) {
        return Some(v);
    }
    match unwrap_option(value).unwrap_or(value) {
        ron::Value::Map(map) => map.iter().find_map(|(_, v)| find_field(v, key)),
        ron::Value::Seq(seq) => seq.iter().find_map(|v| find_field(v, key)),
        _ => None,
    }
}

fn str_field<'a>(value: &'a ron::Value, key: &str) -> Option<&'a str> {
    match value_field(value, key)? {
        ron::Value::String(s) => Some(s.as_str()),
        _ => None,
    }
}

fn f32_field(value: &ron::Value, key: &str) -> Option<f32> {
    number_as_f32(value_field(value, key)?)
}

fn u32_field(value: &ron::Value, key: &str) -> Option<u32> {
    number_as_f32(value_field(value, key)?).map(|v| v as u32)
}

fn f32_pair_field(value: &ron::Value, key: &str) -> Option<(f32, f32)> {
    pair(value_field(value, key)?).and_then(|(a, b)| Some((number_as_f32(a)?, number_as_f32(b)?)))
}

fn u32_pair_field(value: &ron::Value, key: &str) -> Option<(u32, u32)> {
    f32_pair_field(value, key).map(|(a, b)| (a as u32, b as u32))
}

fn string_list_field(value: &ron::Value, key: &str) -> Vec<String> {
    value_field(value, key)
        .and_then(as_seq)
        .map(|seq| {
            seq.iter()
                .filter_map(|v| match unwrap_option(v).unwrap_or(v) {
                    ron::Value::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn as_seq(value: &ron::Value) -> Option<&Vec<ron::Value>> {
    match unwrap_option(value).unwrap_or(value) {
        ron::Value::Seq(seq) => Some(seq),
        _ => None,
    }
}

fn pair(value: &ron::Value) -> Option<(&ron::Value, &ron::Value)> {
    let seq = as_seq(value)?;
    if seq.len() == 2 {
        Some((&seq[0], &seq[1]))
    } else {
        None
    }
}

fn unwrap_option(value: &ron::Value) -> Option<&ron::Value> {
    match value {
        ron::Value::Option(Some(inner)) => Some(inner.as_ref()),
        _ => None,
    }
}

fn number_as_f32(value: &ron::Value) -> Option<f32> {
    match unwrap_option(value).unwrap_or(value) {
        ron::Value::Number(n) => Some(n.into_f64() as f32),
        _ => None,
    }
}
