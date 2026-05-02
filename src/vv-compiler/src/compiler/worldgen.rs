use super::helpers::*;
use super::prelude::*;

use super::*;

impl ContentCompiler {
    pub(super) fn compile_planet_type(
        &mut self,
        doc: &RawDocument<PlanetTypeDef>,
        index: &ReferenceIndex,
    ) -> CompiledPlanetType {
        CompiledPlanetType {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            tags: self.resolve_tags("planet_type", doc, &doc.value.global_tags, index),
            forbidden_tags: self.resolve_tags(
                "planet_type",
                doc,
                &doc.value.global_forbidden_tags,
                index,
            ),
            temperature_bias: doc.value.climate_bias.temperature,
            humidity_bias: doc.value.climate_bias.humidity,
            altitude_variance_multiplier: doc.value.altitude_variance_multiplier,
            climate_transition_speed: doc.value.climate_transition_speed,
            min_radius_km: doc.value.size.min_km,
            max_radius_km: doc.value.size.max_km,
            ocean_coverage: doc.value.ocean_coverage,
        }
    }

    pub(super) fn compile_biome(
        &mut self,
        doc: &RawDocument<BiomeDef>,
        index: &ReferenceIndex,
    ) -> CompiledBiome {
        CompiledBiome {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            weight: doc.value.weight,
            required_tags: self.resolve_tags("biome", doc, &doc.value.required_tags, index),
            forbidden_tags: self.resolve_tags("biome", doc, &doc.value.forbidden_tags, index),
            preferred_tags: self.resolve_tags("biome", doc, &doc.value.preferred_tags, index),
            provided_tags: self.resolve_tags("biome", doc, &doc.value.provided_tags, index),
            climate: CompiledClimateSampleRanges {
                temperature: compiled_ideal_range(doc.value.climate.temperature),
                humidity: compiled_ideal_range(doc.value.climate.humidity),
                altitude: compiled_ideal_range(doc.value.climate.altitude),
            },
            relief: CompiledBiomeRelief {
                base_height_m: doc.value.relief.base_height_m,
                height_variance_m: doc.value.relief.height_variance_m,
                roughness: doc.value.relief.roughness,
            },
            surface_layers: doc
                .value
                .surface
                .iter()
                .filter_map(|layer| {
                    self.resolve_block("biome", doc, &layer.block, index)
                        .map(|block| CompiledSurfaceLayer {
                            block,
                            depth_m: layer.depth_m,
                        })
                })
                .collect(),
        }
    }

    pub(super) fn default_planet_type(
        &mut self,
        load_order: &PackLoadOrder,
        index: &ReferenceIndex,
    ) -> Option<PlanetTypeId> {
        load_order
            .packs()
            .iter()
            .rev()
            .flat_map(|pack| pack.content.universes.iter().rev())
            .find_map(|doc| {
                self.resolve_planet_type("universe", doc, &doc.value.default_planet_type, index)
            })
    }

    pub(super) fn compile_world_settings(
        &self,
        load_order: &PackLoadOrder,
    ) -> CompiledWorldSettings {
        load_order
            .packs()
            .iter()
            .rev()
            .flat_map(|pack| pack.content.world_settings.iter().rev())
            .next()
            .map(|doc| CompiledWorldSettings {
                chunk_size: doc.value.chunk_size,
                render_distance_chunks: doc.value.render_distance_chunks,
                max_planet_radius_km: doc.value.max_planet_radius_km,
                voxel_size_m: doc.value.voxel_size_m,
            })
            .unwrap_or_default()
    }

    pub(super) fn compile_climate_curves(
        &self,
        load_order: &PackLoadOrder,
    ) -> CompiledClimateCurves {
        load_order
            .packs()
            .iter()
            .rev()
            .flat_map(|pack| pack.content.climate_curves.iter().rev())
            .next()
            .map(|doc| CompiledClimateCurves {
                temperature_noise_scale: doc.value.temperature_noise_scale,
                humidity_noise_scale: doc.value.humidity_noise_scale,
                minimum_biome_transition_m: doc.value.minimum_biome_transition_m,
            })
            .unwrap_or_default()
    }

    pub(super) fn compile_climate_tags(
        &mut self,
        load_order: &PackLoadOrder,
        index: &ReferenceIndex,
    ) -> CompiledClimateTags {
        let Some(doc) = load_order
            .packs()
            .iter()
            .rev()
            .flat_map(|pack| pack.content.climate_tags.iter().rev())
            .next()
        else {
            return CompiledClimateTags::default();
        };

        CompiledClimateTags {
            temperature: self.compile_climate_ranges(doc, &doc.value.temperature, index),
            humidity: self.compile_climate_ranges(doc, &doc.value.humidity, index),
            altitude: self.compile_climate_ranges(doc, &doc.value.altitude, index),
            slope: self.compile_climate_ranges(doc, &doc.value.slope, index),
            latitude: self.compile_climate_ranges(doc, &doc.value.latitude, index),
            depth: self.compile_climate_ranges(doc, &doc.value.depth, index),
            derived: doc
                .value
                .derived
                .iter()
                .map(|rule| CompiledDerivedTagRule {
                    requires: self.resolve_tags("climate", doc, &rule.requires, index),
                    produces: self.resolve_tags("climate", doc, &rule.produces, index),
                })
                .collect(),
        }
    }

    pub(super) fn compile_climate_ranges(
        &mut self,
        doc: &RawDocument<vv_schema::worldgen::climate::ClimateTagsDef>,
        ranges: &[vv_schema::worldgen::climate::ClimateRange],
        index: &ReferenceIndex,
    ) -> Vec<CompiledClimateRange> {
        ranges
            .iter()
            .filter_map(|range| {
                self.resolve_tag("climate", doc, &range.tag, index)
                    .map(|tag| CompiledClimateRange {
                        tag,
                        range: CompiledFloatRange {
                            min: range.range.min,
                            max: range.range.max,
                        },
                    })
            })
            .collect()
    }

    pub(super) fn compile_flora(
        &mut self,
        doc: &RawDocument<FloraDef>,
        index: &ReferenceIndex,
    ) -> CompiledFlora {
        let feature = match &doc.value.feature {
            FloraFeature::Plant {
                block,
                height_min_m,
                height_max_m,
            } => {
                let block_id = self
                    .resolve_block("flora", doc, block, index)
                    .unwrap_or(BlockId::new(0));
                CompiledFloraFeature::Plant {
                    block: block_id,
                    height_min_m: *height_min_m,
                    height_max_m: *height_max_m,
                }
            }
            FloraFeature::Tree(tree) => {
                let log_block = self
                    .resolve_block("flora", doc, &tree.blocks.log, index)
                    .expect("flora tree log block should resolve");

                let leaf_block = self
                    .resolve_block("flora", doc, &tree.blocks.leaves, index)
                    .expect("flora tree leaf block should resolve");

                let trunk_height_min_m = tree.size.height_min_m;
                let trunk_height_max_m = tree.size.height_max_m;
                let canopy_radius_m = tree.size.radius_max_m;
                let canopy_height_m = tree.crown.height_max_m;
                let canopy_start_t = tree.crown.start_t;
                let trunk_girth = (tree.trunk.base_radius_m / 0.60).clamp(0.0, 1.0);
                let crown_bias = tree
                    .variation
                    .archetypes
                    .iter()
                    .fold(0.0, |acc, archetype| {
                        let bias = match archetype.kind {
                            vv_schema::worldgen::flora::TreeArchetypeKind::Spreading => 1.0,
                            vv_schema::worldgen::flora::TreeArchetypeKind::Columnar => -1.0,
                            _ => 0.0,
                        };
                        acc + bias * archetype.weight.max(0.0)
                    })
                    .clamp(-1.0, 1.0);

                CompiledFloraFeature::Tree {
                    log_block,
                    leaf_block,
                    trunk_height_min_m,
                    trunk_height_max_m,
                    canopy_radius_m,
                    canopy_height_m,
                    canopy_start_t,
                    trunk_girth,
                    crown_bias,
                }
            }
            FloraFeature::Cluster {
                block,
                radius_min_m,
                radius_max_m,
                ..
            } => {
                let block_id = self
                    .resolve_block("flora", doc, block, index)
                    .unwrap_or(BlockId::new(0));
                CompiledFloraFeature::Cluster {
                    block: block_id,
                    radius_min_m: *radius_min_m,
                    radius_max_m: *radius_max_m,
                }
            }
        };

        CompiledFlora {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            weight: doc.value.weight,
            required_tags: self.resolve_tags("flora", doc, &doc.value.required_tags, index),
            forbidden_tags: self.resolve_tags("flora", doc, &doc.value.forbidden_tags, index),
            provided_tags: self.resolve_tags("flora", doc, &doc.value.provided_tags, index),
            placement: CompiledFloraPlacement {
                density_base: doc.value.placement.density_base,
                altitude_max_m: doc.value.placement.altitude_max_m,
                slope_max: doc.value.placement.slope_max,
                cluster_radius_m: doc.value.placement.cluster_radius_m,
                cluster_min: doc.value.placement.cluster_min,
                cluster_max: doc.value.placement.cluster_max,
            },
            feature,
        }
    }

    pub(super) fn compile_fauna(
        &mut self,
        doc: &RawDocument<FaunaDef>,
        index: &ReferenceIndex,
    ) -> CompiledFauna {
        CompiledFauna {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            entity: self
                .resolve_entity("fauna", doc, &doc.value.entity, index)
                .unwrap_or(EntityId::new(0)),
            required_tags: self.resolve_tags("fauna", doc, &doc.value.required_tags, index),
            provided_tags: self.resolve_tags("fauna", doc, &doc.value.provided_tags, index),
        }
    }

    pub(super) fn compile_ore(
        &mut self,
        doc: &RawDocument<OreDef>,
        index: &ReferenceIndex,
    ) -> CompiledOre {
        CompiledOre {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            weight: doc.value.weight,
            block: self
                .resolve_block("ore", doc, &doc.value.block, index)
                .unwrap_or(BlockId::new(0)),
            required_tags: self.resolve_tags("ore", doc, &doc.value.required_tags, index),
            forbidden_tags: self.resolve_tags("ore", doc, &doc.value.forbidden_tags, index),
            vein: CompiledOreVein {
                size_min: doc.value.vein.size.min.max(0) as u32,
                size_max: doc.value.vein.size.max.max(0) as u32,
                depth_min_m: doc.value.vein.depth_m.min,
                depth_max_m: doc.value.vein.depth_m.max,
                frequency: doc.value.vein.frequency,
            },
        }
    }

    pub(super) fn compile_structure(
        &mut self,
        doc: &RawDocument<StructureDef>,
        index: &ReferenceIndex,
    ) -> CompiledStructure {
        CompiledStructure {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            required_tags: self.resolve_tags("structure", doc, &doc.value.required_tags, index),
            provided_tags: self.resolve_tags("structure", doc, &doc.value.provided_tags, index),
            loot_table: doc
                .value
                .loot_table
                .as_ref()
                .and_then(|loot| self.resolve_loot_table("structure", doc, loot, index)),
        }
    }

    pub(super) fn compile_weather(
        &mut self,
        doc: &RawDocument<WeatherDef>,
        index: &ReferenceIndex,
    ) -> CompiledWeather {
        CompiledWeather {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            required_tags: self.resolve_tags("weather", doc, &doc.value.required_tags, index),
            provided_tags: self.resolve_tags("weather", doc, &doc.value.provided_tags, index),
        }
    }
}
