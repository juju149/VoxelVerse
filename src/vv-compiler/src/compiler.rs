use std::{collections::HashMap, path::Path, str::FromStr};

use vv_pack::{load_packs_from_assets, PackLoadOrder, RawDocument};
use vv_registry::{
    BiomeId, BlockId, CompiledBiome, CompiledBiomeRelief, CompiledBlock, CompiledBlockMining,
    CompiledBlockPhysics, CompiledBlockRender, CompiledClimateCurves, CompiledClimateRange,
    CompiledClimateSampleRanges, CompiledClimateTags, CompiledContent, CompiledDerivedTagRule,
    CompiledDrops, CompiledEntity, CompiledFauna, CompiledFloatRange, CompiledFlora,
    CompiledFloraFeature, CompiledFloraPlacement, CompiledIdealRange, CompiledIngredient,
    CompiledItem, CompiledItemKind, CompiledLootEntry, CompiledLootPool, CompiledLootTable,
    CompiledMaterialPhase, CompiledOre, CompiledOreVein, CompiledPlaceable, CompiledPlanetType,
    CompiledRecipe, CompiledRecipePattern, CompiledStructure, CompiledStylizedMaterial,
    CompiledSurfaceLayer, CompiledTag, CompiledTextureLayout, CompiledTextureResource,
    CompiledTintMode, CompiledToolKind, CompiledVisualMaterialType, CompiledWeather,
    CompiledWorldSettings, ContentKey, EntityId, FaunaId, FloraId, ItemId, LootTableId, OreId,
    PlaceableId, PlanetTypeId, RecipeId, StructureId, TagDomain, TagId, TaggedContent, TextureId,
    WeatherId,
};
use vv_schema::{
    block::{
        BlockDef, BlockTextureRefs, MaterialPhase, TextureLayout, TintMode, VisualMaterialType,
    },
    common::tool::ToolKind,
    common::{BlockRef, EntityRef, ItemRef, LootTableRef, PlaceableRef, ResourceRef, TagRef},
    item::ItemKind,
    loot::{DropSpec, LootTableDef},
    recipe::{RecipeDef, RecipeIngredient, RecipePattern},
    tag::{TagContentKind, TagDef},
    worldgen::{
        biome::BiomeDef, fauna::FaunaDef, flora::FloraDef, flora::FloraFeature, ore::OreDef,
        planet::PlanetTypeDef, structure::StructureDef, weather::WeatherDef,
    },
};

use crate::{
    diagnostics::{CompileDiagnostic, CompileError, CompileResult, ReferenceKind},
    identity::derive_key,
    reference_index::ReferenceIndex,
};

pub fn compile_packs(load_order: &PackLoadOrder) -> CompileResult<CompiledContent> {
    let mut compiler = ContentCompiler::default();
    compiler.compile(load_order)
}

pub fn compile_assets_root(assets_root: &Path) -> CompileResult<CompiledContent> {
    let load_order = load_packs_from_assets(assets_root).map_err(|err| {
        CompileError::new(vec![CompileDiagnostic::InvalidReference {
            owner: "pack_loader".to_owned(),
            path: assets_root.to_path_buf(),
            reference: assets_root.display().to_string(),
            expected: ReferenceKind::Pack,
            reason: err.to_string(),
        }])
    })?;
    compile_packs(&load_order)
}

#[derive(Default)]
struct ContentCompiler {
    diagnostics: Vec<CompileDiagnostic>,
}

impl ContentCompiler {
    fn compile(&mut self, load_order: &PackLoadOrder) -> CompileResult<CompiledContent> {
        let mut index = ReferenceIndex::default();

        let block_docs = self.collect_domain::<BlockDef, BlockId>(
            load_order,
            |pack| &pack.blocks,
            "defs/blocks",
            &mut index.blocks,
        );
        let item_docs = self.collect_domain::<_, ItemId>(
            load_order,
            |pack| &pack.items,
            "defs/items",
            &mut index.items,
        );
        let entity_docs = self.collect_domain::<_, EntityId>(
            load_order,
            |pack| &pack.entities,
            "defs/entities",
            &mut index.entities,
        );
        let placeable_docs = self.collect_domain::<_, PlaceableId>(
            load_order,
            |pack| &pack.placeables,
            "defs/placeables",
            &mut index.placeables,
        );
        let recipe_docs = self.collect_domain::<_, RecipeId>(
            load_order,
            |pack| &pack.recipes,
            "defs/recipes",
            &mut index.recipes,
        );
        let loot_docs = self.collect_domain::<_, LootTableId>(
            load_order,
            |pack| &pack.loot_tables,
            "defs/loot_tables",
            &mut index.loot_tables,
        );
        let tag_docs = self.collect_domain::<_, TagId>(
            load_order,
            |pack| &pack.tags,
            "defs/tags",
            &mut index.tags,
        );
        let planet_docs = self.collect_domain::<_, PlanetTypeId>(
            load_order,
            |pack| &pack.planet_types,
            "defs/worldgen/planet_types",
            &mut index.planet_types,
        );
        let biome_docs = self.collect_domain::<_, BiomeId>(
            load_order,
            |pack| &pack.biomes,
            "defs/worldgen/biomes",
            &mut index.biomes,
        );
        let flora_docs = self.collect_domain::<_, FloraId>(
            load_order,
            |pack| &pack.flora,
            "defs/worldgen/flora",
            &mut index.flora,
        );
        let fauna_docs = self.collect_domain::<_, FaunaId>(
            load_order,
            |pack| &pack.fauna,
            "defs/worldgen/fauna",
            &mut index.fauna,
        );
        let ore_docs = self.collect_domain::<_, OreId>(
            load_order,
            |pack| &pack.ores,
            "defs/worldgen/ores",
            &mut index.ores,
        );
        let structure_docs = self.collect_domain::<_, StructureId>(
            load_order,
            |pack| &pack.structures,
            "defs/worldgen/structures",
            &mut index.structures,
        );
        let weather_docs = self.collect_domain::<_, WeatherId>(
            load_order,
            |pack| &pack.weather,
            "defs/worldgen/weather",
            &mut index.weather,
        );

        self.validate_universes(load_order, &index);
        self.validate_climate(load_order, &index);

        if !self.diagnostics.is_empty() {
            return Err(CompileError::new(std::mem::take(&mut self.diagnostics)));
        }

        let mut content = CompiledContent::default();
        content.world = self.compile_world_settings(load_order);
        content.default_planet_type = self.default_planet_type(load_order, &index);
        content.climate_tags = self.compile_climate_tags(load_order, &index);
        content.climate_curves = self.compile_climate_curves(load_order);
        let texture_ids = self.compile_texture_registry(&block_docs, &mut content);

        for (key, doc) in &tag_docs {
            let tag = self.compile_tag(doc, &index);
            content.tags.push(key.clone(), tag);
        }
        for (key, doc) in &block_docs {
            let block = self.compile_block(doc, &index, &texture_ids);
            content.blocks.push(key.clone(), block);
        }
        for (key, doc) in &placeable_docs {
            let placeable = self.compile_placeable(doc, &index);
            content.placeables.push(key.clone(), placeable);
        }
        for (key, doc) in &entity_docs {
            let entity = self.compile_entity(doc, &index);
            content.entities.push(key.clone(), entity);
        }
        for (key, doc) in &item_docs {
            let item = self.compile_item(doc, &index);
            content.items.push(key.clone(), item);
        }
        for (key, doc) in &loot_docs {
            let loot = self.compile_loot_table(doc, &index);
            content.loot_tables.push(key.clone(), loot);
        }
        for (key, doc) in &recipe_docs {
            let recipe = self.compile_recipe(doc, &index);
            content.recipes.push(key.clone(), recipe);
        }
        for (key, doc) in &planet_docs {
            let planet = self.compile_planet_type(doc, &index);
            content.planet_types.push(key.clone(), planet);
        }
        for (key, doc) in &biome_docs {
            let biome = self.compile_biome(doc, &index);
            content.biomes.push(key.clone(), biome);
        }
        for (key, doc) in &flora_docs {
            let flora = self.compile_flora(doc, &index);
            content.flora.push(key.clone(), flora);
        }
        for (key, doc) in &fauna_docs {
            let fauna = self.compile_fauna(doc, &index);
            content.fauna.push(key.clone(), fauna);
        }
        for (key, doc) in &ore_docs {
            let ore = self.compile_ore(doc, &index);
            content.ores.push(key.clone(), ore);
        }
        for (key, doc) in &structure_docs {
            let structure = self.compile_structure(doc, &index);
            content.structures.push(key.clone(), structure);
        }
        for (key, doc) in &weather_docs {
            let weather = self.compile_weather(doc, &index);
            content.weather.push(key.clone(), weather);
        }

        if self.diagnostics.is_empty() {
            Ok(content)
        } else {
            Err(CompileError::new(std::mem::take(&mut self.diagnostics)))
        }
    }

    fn collect_domain<'a, T, I>(
        &mut self,
        load_order: &'a PackLoadOrder,
        selector: fn(&'a vv_pack::RawContentSet) -> &'a Vec<RawDocument<T>>,
        family_root: &str,
        index: &mut HashMap<ContentKey, I>,
    ) -> Vec<(ContentKey, &'a RawDocument<T>)>
    where
        I: From<u32> + Copy,
    {
        let mut docs = Vec::new();
        let mut first_paths = HashMap::<ContentKey, std::path::PathBuf>::new();

        for pack in load_order.packs() {
            for doc in selector(&pack.content) {
                match derive_key(&doc.pack_namespace, &doc.relative_path, family_root) {
                    Ok(key) => {
                        if let Some(first_path) = first_paths.get(&key) {
                            self.diagnostics.push(CompileDiagnostic::DuplicateResource {
                                key: key.to_string(),
                                first_path: first_path.clone(),
                                second_path: doc.source_path.clone(),
                            });
                        } else {
                            let id = I::from(docs.len() as u32);
                            first_paths.insert(key.clone(), doc.source_path.clone());
                            index.insert(key.clone(), id);
                            docs.push((key, doc));
                        }
                    }
                    Err(diagnostic) => self.diagnostics.push(diagnostic),
                }
            }
        }

        docs
    }

    fn compile_block(
        &mut self,
        doc: &RawDocument<BlockDef>,
        index: &ReferenceIndex,
        texture_ids: &HashMap<ContentKey, TextureId>,
    ) -> CompiledBlock {
        self.validate_drop_spec("block", doc, &doc.value.drops, index);
        CompiledBlock {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            stack_max: doc.value.stack_max,
            tags: self.resolve_tags("block", doc, &doc.value.tags, index),
            mining: CompiledBlockMining {
                hardness: doc.value.mining.hardness,
                tool: compiled_tool_kind(doc.value.mining.tool),
                tool_tier_min: doc.value.mining.tool_tier_min,
                drop_xp: doc.value.mining.drop_xp,
            },
            physics: CompiledBlockPhysics {
                phase: match doc.value.physics.phase {
                    MaterialPhase::Solid => CompiledMaterialPhase::Solid,
                    MaterialPhase::Liquid => CompiledMaterialPhase::Liquid,
                    MaterialPhase::Passable => CompiledMaterialPhase::Passable,
                },
                density: doc.value.physics.density,
                friction: doc.value.physics.friction,
                drag: doc.value.physics.drag,
            },
            render: CompiledBlockRender {
                color: [
                    doc.value.render.color.r,
                    doc.value.render.color.g,
                    doc.value.render.color.b,
                ],
                roughness: doc.value.render.roughness,
                translucent: doc.value.render.translucent,
                emits_light: doc.value.render.emits_light,
                tint: compiled_tint_mode(&doc.value.render.tint),
                material: compiled_stylized_material(doc),
                texture_layout: match doc.value.render.texture {
                    TextureLayout::Single => CompiledTextureLayout::Single,
                    TextureLayout::Sides => CompiledTextureLayout::Sides,
                    TextureLayout::Custom => CompiledTextureLayout::Custom,
                },
                textures: self.compile_block_textures(doc, texture_ids),
                model: doc.value.render.model.as_ref().map(|model| model.0.clone()),
            },
            drops: self.compile_drop_spec("block", doc, &doc.value.drops, index),
        }
    }

    fn compile_texture_registry(
        &mut self,
        block_docs: &[(ContentKey, &RawDocument<BlockDef>)],
        content: &mut CompiledContent,
    ) -> HashMap<ContentKey, TextureId> {
        let mut ids = HashMap::new();
        for (_, doc) in block_docs {
            for resource in block_texture_refs(&doc.value.render.textures) {
                let Some(key) = self.parse_texture_ref("block", doc, resource) else {
                    continue;
                };
                ids.entry(key.clone())
                    .or_insert_with(|| content.textures.push(key, CompiledTextureResource));
            }
        }
        ids
    }

    fn compile_block_textures(
        &mut self,
        doc: &RawDocument<BlockDef>,
        texture_ids: &HashMap<ContentKey, TextureId>,
    ) -> vv_registry::CompiledBlockTextures {
        let refs = &doc.value.render.textures;
        vv_registry::CompiledBlockTextures {
            single: self.resolve_texture_ref(doc, refs.single.as_ref(), texture_ids),
            side: self.resolve_texture_ref(doc, refs.side.as_ref(), texture_ids),
            top: self.resolve_texture_ref(doc, refs.top.as_ref(), texture_ids),
            bottom: self.resolve_texture_ref(doc, refs.bottom.as_ref(), texture_ids),
            north: self.resolve_texture_ref(doc, refs.north.as_ref(), texture_ids),
            south: self.resolve_texture_ref(doc, refs.south.as_ref(), texture_ids),
            east: self.resolve_texture_ref(doc, refs.east.as_ref(), texture_ids),
            west: self.resolve_texture_ref(doc, refs.west.as_ref(), texture_ids),
        }
    }

    fn compile_item(
        &mut self,
        doc: &RawDocument<vv_schema::item::ItemDef>,
        index: &ReferenceIndex,
    ) -> CompiledItem {
        let kind = match &doc.value.kind {
            ItemKind::Block { block } => CompiledItemKind::Block {
                block: self
                    .resolve_block("item", doc, block, index)
                    .unwrap_or(BlockId::new(0)),
            },
            ItemKind::Resource => CompiledItemKind::Resource,
            ItemKind::Tool {
                tool_type,
                tool_tier,
                durability,
                mining_speed,
                attack_damage,
                ..
            } => CompiledItemKind::Tool {
                tool_type: compiled_tool_kind(*tool_type),
                tool_tier: *tool_tier,
                durability: *durability,
                mining_speed: *mining_speed,
                attack_damage: *attack_damage,
            },
            ItemKind::Armor { .. } => CompiledItemKind::Armor,
            ItemKind::Food { .. } => CompiledItemKind::Food,
            ItemKind::Placeable { placeable } => CompiledItemKind::Placeable {
                placeable: self
                    .resolve_placeable("item", doc, placeable, index)
                    .unwrap_or(PlaceableId::new(0)),
            },
        };

        CompiledItem {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            stack_max: doc.value.stack_max,
            tags: self.resolve_tags("item", doc, &doc.value.tags, index),
            kind,
        }
    }

    fn compile_entity(
        &mut self,
        doc: &RawDocument<vv_schema::entity::EntityDef>,
        index: &ReferenceIndex,
    ) -> CompiledEntity {
        self.validate_drop_spec("entity", doc, &doc.value.drops, index);
        CompiledEntity {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            tags: self.resolve_tags("entity", doc, &doc.value.tags, index),
            health: doc.value.health,
            light_level: doc.value.light_level,
        }
    }

    fn compile_placeable(
        &mut self,
        doc: &RawDocument<vv_schema::placeable::PlaceableDef>,
        index: &ReferenceIndex,
    ) -> CompiledPlaceable {
        self.validate_drop_spec("placeable", doc, &doc.value.drops, index);
        CompiledPlaceable {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            tags: self.resolve_tags("placeable", doc, &doc.value.tags, index),
            light_level: doc.value.light_level,
        }
    }

    fn compile_loot_table(
        &mut self,
        doc: &RawDocument<LootTableDef>,
        index: &ReferenceIndex,
    ) -> CompiledLootTable {
        let pools = doc
            .value
            .pools
            .iter()
            .map(|pool| CompiledLootPool {
                rolls: pool.rolls,
                bonus_rolls: pool.bonus_rolls,
                entries: pool
                    .entries
                    .iter()
                    .filter_map(|entry| {
                        self.resolve_item("loot_table", doc, &entry.item, index)
                            .map(|item| CompiledLootEntry {
                                item,
                                weight: entry.weight,
                                count_min: entry.count.min,
                                count_max: entry.count.max,
                            })
                    })
                    .collect(),
            })
            .collect();
        CompiledLootTable { pools }
    }

    fn compile_recipe(
        &mut self,
        doc: &RawDocument<RecipeDef>,
        index: &ReferenceIndex,
    ) -> CompiledRecipe {
        let ingredients = doc
            .value
            .ingredients
            .iter()
            .filter_map(|ingredient| match ingredient {
                RecipeIngredient::Item { item, count } => self
                    .resolve_item("recipe", doc, item, index)
                    .map(|item| CompiledIngredient::Item {
                        item,
                        count: *count,
                    }),
                RecipeIngredient::Tag { tag, count } => self
                    .resolve_tag("recipe", doc, tag, index)
                    .map(|tag| CompiledIngredient::Tag { tag, count: *count }),
            })
            .collect();

        CompiledRecipe {
            pattern: match doc.value.pattern {
                RecipePattern::Shapeless => CompiledRecipePattern::Shapeless,
                RecipePattern::Shaped => CompiledRecipePattern::Shaped,
                RecipePattern::Processing => CompiledRecipePattern::Processing,
            },
            result_item: self
                .resolve_item("recipe", doc, &doc.value.result.item, index)
                .unwrap_or(ItemId::new(0)),
            result_count: doc.value.result.count,
            ingredients,
            station: doc
                .value
                .station
                .as_ref()
                .and_then(|station| self.resolve_block("recipe", doc, station, index)),
            time_seconds: doc.value.time_seconds,
            tags: self.resolve_tags("recipe", doc, &doc.value.tags, index),
        }
    }

    fn compile_tag(&mut self, doc: &RawDocument<TagDef>, index: &ReferenceIndex) -> CompiledTag {
        let domain = match doc.value.kind {
            TagContentKind::Block => TagDomain::Block,
            TagContentKind::Item => TagDomain::Item,
            TagContentKind::Entity => TagDomain::Entity,
            TagContentKind::Placeable => TagDomain::Placeable,
            TagContentKind::Any => TagDomain::Any,
        };
        let values = doc
            .value
            .values
            .iter()
            .filter_map(|value| self.resolve_tagged_content("tag", doc, &value.0, domain, index))
            .collect();
        let extends = self.resolve_tags("tag", doc, &doc.value.extends, index);
        CompiledTag {
            domain,
            values,
            extends,
        }
    }

    fn compile_planet_type(
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

    fn compile_biome(
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

    fn default_planet_type(
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

    fn compile_world_settings(&self, load_order: &PackLoadOrder) -> CompiledWorldSettings {
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

    fn compile_climate_curves(&self, load_order: &PackLoadOrder) -> CompiledClimateCurves {
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

    fn compile_climate_tags(
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

    fn compile_climate_ranges(
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

    fn compile_flora(
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
            FloraFeature::Tree {
                log_block,
                leaf_block,
                trunk_height_min_m,
                trunk_height_max_m,
                canopy_radius_m,
                canopy_height_m,
            } => {
                let log = self
                    .resolve_block("flora", doc, log_block, index)
                    .unwrap_or(BlockId::new(0));
                let leaf = self
                    .resolve_block("flora", doc, leaf_block, index)
                    .unwrap_or(BlockId::new(0));
                CompiledFloraFeature::Tree {
                    log_block: log,
                    leaf_block: leaf,
                    trunk_height_min_m: *trunk_height_min_m,
                    trunk_height_max_m: *trunk_height_max_m,
                    canopy_radius_m: *canopy_radius_m,
                    canopy_height_m: *canopy_height_m,
                }
            }
            FloraFeature::Cluster {
                block,
                radius_min_m,
                radius_max_m,
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

    fn compile_fauna(
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

    fn compile_ore(&mut self, doc: &RawDocument<OreDef>, index: &ReferenceIndex) -> CompiledOre {
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

    fn compile_structure(
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

    fn compile_weather(
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

    fn validate_universes(&mut self, load_order: &PackLoadOrder, index: &ReferenceIndex) {
        for pack in load_order.packs() {
            for doc in &pack.content.universes {
                let reference = doc.value.default_planet_type.0.clone();
                self.resolve_key(
                    "universe",
                    &doc.source_path,
                    &reference,
                    ReferenceKind::PlanetType,
                    &index.planet_types,
                );
            }
        }
    }

    fn validate_climate(&mut self, load_order: &PackLoadOrder, index: &ReferenceIndex) {
        for pack in load_order.packs() {
            for doc in &pack.content.climate_tags {
                let tags = doc
                    .value
                    .temperature
                    .iter()
                    .map(|range| &range.tag)
                    .chain(doc.value.humidity.iter().map(|range| &range.tag))
                    .chain(doc.value.altitude.iter().map(|range| &range.tag))
                    .chain(doc.value.slope.iter().map(|range| &range.tag))
                    .chain(doc.value.latitude.iter().map(|range| &range.tag))
                    .chain(doc.value.depth.iter().map(|range| &range.tag))
                    .chain(
                        doc.value
                            .derived
                            .iter()
                            .flat_map(|rule| rule.requires.iter()),
                    )
                    .chain(
                        doc.value
                            .derived
                            .iter()
                            .flat_map(|rule| rule.produces.iter()),
                    );
                for tag in tags {
                    self.resolve_tag("climate", doc, tag, index);
                }
            }
            for doc in &pack.content.climate_transitions {
                for transition in &doc.value.transitions {
                    self.resolve_tag("climate_transition", doc, &transition.from, index);
                    self.resolve_tag("climate_transition", doc, &transition.to, index);
                }
            }
        }
    }

    fn validate_drop_spec<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        drops: &DropSpec,
        index: &ReferenceIndex,
    ) {
        match drops {
            DropSpec::None => {}
            DropSpec::Inline(pools) => {
                for pool in pools {
                    for entry in &pool.entries {
                        self.resolve_item(owner, doc, &entry.item, index);
                    }
                }
            }
            DropSpec::Table(table) => {
                self.resolve_loot_table(owner, doc, table, index);
            }
        }
    }

    fn compile_drop_spec<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        drops: &DropSpec,
        index: &ReferenceIndex,
    ) -> CompiledDrops {
        match drops {
            DropSpec::None => CompiledDrops::None,
            DropSpec::Table(table) => self
                .resolve_loot_table(owner, doc, table, index)
                .map(CompiledDrops::Table)
                .unwrap_or(CompiledDrops::None),
            DropSpec::Inline(pools) => CompiledDrops::Inline(
                pools
                    .iter()
                    .map(|pool| CompiledLootPool {
                        rolls: pool.rolls,
                        bonus_rolls: pool.bonus_rolls,
                        entries: pool
                            .entries
                            .iter()
                            .filter_map(|entry| {
                                self.resolve_item(owner, doc, &entry.item, index)
                                    .map(|item| CompiledLootEntry {
                                        item,
                                        weight: entry.weight,
                                        count_min: entry.count.min,
                                        count_max: entry.count.max,
                                    })
                            })
                            .collect(),
                    })
                    .collect(),
            ),
        }
    }

    fn resolve_tags<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        tags: &[TagRef],
        index: &ReferenceIndex,
    ) -> Vec<TagId> {
        tags.iter()
            .filter_map(|tag| self.resolve_tag(owner, doc, tag, index))
            .collect()
    }

    fn resolve_block<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &BlockRef,
        index: &ReferenceIndex,
    ) -> Option<BlockId> {
        self.resolve_key(
            owner,
            &doc.source_path,
            &reference.0,
            ReferenceKind::Block,
            &index.blocks,
        )
    }

    fn resolve_item<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &ItemRef,
        index: &ReferenceIndex,
    ) -> Option<ItemId> {
        self.resolve_key(
            owner,
            &doc.source_path,
            &reference.0,
            ReferenceKind::Item,
            &index.items,
        )
    }

    fn resolve_entity<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &EntityRef,
        index: &ReferenceIndex,
    ) -> Option<EntityId> {
        self.resolve_key(
            owner,
            &doc.source_path,
            &reference.0,
            ReferenceKind::Entity,
            &index.entities,
        )
    }

    fn resolve_placeable<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &PlaceableRef,
        index: &ReferenceIndex,
    ) -> Option<PlaceableId> {
        self.resolve_key(
            owner,
            &doc.source_path,
            &reference.0,
            ReferenceKind::Placeable,
            &index.placeables,
        )
    }

    fn resolve_loot_table<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &LootTableRef,
        index: &ReferenceIndex,
    ) -> Option<LootTableId> {
        self.resolve_key(
            owner,
            &doc.source_path,
            &reference.0,
            ReferenceKind::LootTable,
            &index.loot_tables,
        )
    }

    fn resolve_planet_type<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &vv_schema::worldgen::planet::PlanetTypeRef,
        index: &ReferenceIndex,
    ) -> Option<PlanetTypeId> {
        self.resolve_key(
            owner,
            &doc.source_path,
            &reference.0,
            ReferenceKind::PlanetType,
            &index.planet_types,
        )
    }

    fn resolve_tag<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &TagRef,
        index: &ReferenceIndex,
    ) -> Option<TagId> {
        self.resolve_key(
            owner,
            &doc.source_path,
            &reference.0,
            ReferenceKind::Tag,
            &index.tags,
        )
    }

    fn resolve_texture_ref<T>(
        &mut self,
        doc: &RawDocument<T>,
        reference: Option<&ResourceRef>,
        texture_ids: &HashMap<ContentKey, TextureId>,
    ) -> Option<TextureId> {
        let reference = reference?;
        let key = self.parse_texture_ref("block", doc, reference)?;
        texture_ids.get(&key).copied().or_else(|| {
            self.diagnostics.push(CompileDiagnostic::MissingReference {
                owner: "block".to_owned(),
                path: doc.source_path.clone(),
                reference: reference.0.clone(),
                expected: ReferenceKind::Texture,
            });
            None
        })
    }

    fn parse_texture_ref<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &ResourceRef,
    ) -> Option<ContentKey> {
        match ContentKey::from_str(&reference.0) {
            Ok(key) => Some(key),
            Err(err) => {
                self.diagnostics.push(CompileDiagnostic::InvalidReference {
                    owner: owner.to_owned(),
                    path: doc.source_path.clone(),
                    reference: reference.0.clone(),
                    expected: ReferenceKind::Texture,
                    reason: err.to_string(),
                });
                None
            }
        }
    }

    fn resolve_key<I>(
        &mut self,
        owner: &str,
        path: &std::path::Path,
        reference: &str,
        kind: ReferenceKind,
        index: &HashMap<ContentKey, I>,
    ) -> Option<I>
    where
        I: Copy,
    {
        match ContentKey::from_str(reference) {
            Ok(key) => index.get(&key).copied().or_else(|| {
                self.diagnostics.push(CompileDiagnostic::MissingReference {
                    owner: owner.to_owned(),
                    path: path.to_path_buf(),
                    reference: reference.to_owned(),
                    expected: kind,
                });
                None
            }),
            Err(err) => {
                self.diagnostics.push(CompileDiagnostic::InvalidReference {
                    owner: owner.to_owned(),
                    path: path.to_path_buf(),
                    reference: reference.to_owned(),
                    expected: kind,
                    reason: err.to_string(),
                });
                None
            }
        }
    }

    fn resolve_tagged_content<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &str,
        domain: TagDomain,
        index: &ReferenceIndex,
    ) -> Option<TaggedContent> {
        match domain {
            TagDomain::Block => self
                .resolve_key(
                    owner,
                    &doc.source_path,
                    reference,
                    ReferenceKind::Block,
                    &index.blocks,
                )
                .map(TaggedContent::Block),
            TagDomain::Item => self
                .resolve_key(
                    owner,
                    &doc.source_path,
                    reference,
                    ReferenceKind::Item,
                    &index.items,
                )
                .map(TaggedContent::Item),
            TagDomain::Entity => self
                .resolve_key(
                    owner,
                    &doc.source_path,
                    reference,
                    ReferenceKind::Entity,
                    &index.entities,
                )
                .map(TaggedContent::Entity),
            TagDomain::Placeable => self
                .resolve_key(
                    owner,
                    &doc.source_path,
                    reference,
                    ReferenceKind::Placeable,
                    &index.placeables,
                )
                .map(TaggedContent::Placeable),
            TagDomain::Any => {
                let key = match ContentKey::from_str(reference) {
                    Ok(key) => key,
                    Err(err) => {
                        self.diagnostics.push(CompileDiagnostic::InvalidReference {
                            owner: owner.to_owned(),
                            path: doc.source_path.clone(),
                            reference: reference.to_owned(),
                            expected: ReferenceKind::Tag,
                            reason: err.to_string(),
                        });
                        return None;
                    }
                };
                index
                    .blocks
                    .get(&key)
                    .copied()
                    .map(TaggedContent::Block)
                    .or_else(|| index.items.get(&key).copied().map(TaggedContent::Item))
                    .or_else(|| index.entities.get(&key).copied().map(TaggedContent::Entity))
                    .or_else(|| {
                        index
                            .placeables
                            .get(&key)
                            .copied()
                            .map(TaggedContent::Placeable)
                    })
                    .or_else(|| {
                        self.diagnostics.push(CompileDiagnostic::MissingReference {
                            owner: owner.to_owned(),
                            path: doc.source_path.clone(),
                            reference: reference.to_owned(),
                            expected: ReferenceKind::Tag,
                        });
                        None
                    })
            }
        }
    }
}

fn compiled_ideal_range(range: vv_schema::common::IdealRange) -> CompiledIdealRange {
    CompiledIdealRange {
        min: range.min,
        ideal_min: range.ideal_min,
        ideal_max: range.ideal_max,
        max: range.max,
    }
}

fn compiled_tool_kind(kind: ToolKind) -> CompiledToolKind {
    match kind {
        ToolKind::Hand => CompiledToolKind::Hand,
        ToolKind::Pickaxe => CompiledToolKind::Pickaxe,
        ToolKind::Axe => CompiledToolKind::Axe,
        ToolKind::Shovel => CompiledToolKind::Shovel,
        ToolKind::Sword => CompiledToolKind::Sword,
        ToolKind::Shears => CompiledToolKind::Shears,
        ToolKind::Hoe => CompiledToolKind::Hoe,
    }
}

fn compiled_tint_mode(mode: &TintMode) -> CompiledTintMode {
    match mode {
        TintMode::None => CompiledTintMode::None,
        TintMode::GrassColor => CompiledTintMode::GrassColor,
        TintMode::FoliageColor => CompiledTintMode::FoliageColor,
        TintMode::WaterColor => CompiledTintMode::WaterColor,
    }
}

fn compiled_stylized_material(doc: &RawDocument<BlockDef>) -> CompiledStylizedMaterial {
    let material = &doc.value.render.material;
    let secondary = material
        .secondary_color
        .as_ref()
        .map(|color| [color.r, color.g, color.b])
        .unwrap_or_else(|| {
            default_secondary_color(
                doc.value.render.color.r,
                doc.value.render.color.g,
                doc.value.render.color.b,
            )
        });
    CompiledStylizedMaterial {
        visual_type: compiled_visual_material_type(material.visual_type),
        secondary_color: secondary,
        texture_influence: material.texture_influence.clamp(0.0, 1.0),
        block_variation: material.block_variation.clamp(0.0, 0.5),
        face_variation: material.face_variation.clamp(0.0, 0.35),
        macro_variation: material.macro_variation.clamp(0.0, 0.4),
        detail_strength: material.detail_strength.clamp(0.0, 0.25),
    }
}

fn default_secondary_color(r: f32, g: f32, b: f32) -> [f32; 3] {
    [
        (r * 1.12 + 0.025).min(1.0),
        (g * 1.08 + 0.020).min(1.0),
        (b * 0.96 + 0.015).min(1.0),
    ]
}

fn compiled_visual_material_type(kind: VisualMaterialType) -> CompiledVisualMaterialType {
    match kind {
        VisualMaterialType::Generic => CompiledVisualMaterialType::Generic,
        VisualMaterialType::Grass => CompiledVisualMaterialType::Grass,
        VisualMaterialType::Dirt => CompiledVisualMaterialType::Dirt,
        VisualMaterialType::Snow => CompiledVisualMaterialType::Snow,
        VisualMaterialType::Stone => CompiledVisualMaterialType::Stone,
        VisualMaterialType::Sand => CompiledVisualMaterialType::Sand,
        VisualMaterialType::Wood => CompiledVisualMaterialType::Wood,
        VisualMaterialType::Leaves => CompiledVisualMaterialType::Leaves,
        VisualMaterialType::Ice => CompiledVisualMaterialType::Ice,
        VisualMaterialType::CutStone => CompiledVisualMaterialType::CutStone,
        VisualMaterialType::Planks => CompiledVisualMaterialType::Planks,
        VisualMaterialType::Ore => CompiledVisualMaterialType::Ore,
        VisualMaterialType::Water => CompiledVisualMaterialType::Water,
    }
}

fn block_texture_refs(textures: &BlockTextureRefs) -> impl Iterator<Item = &ResourceRef> {
    [
        textures.single.as_ref(),
        textures.side.as_ref(),
        textures.top.as_ref(),
        textures.bottom.as_ref(),
        textures.north.as_ref(),
        textures.south.as_ref(),
        textures.east.as_ref(),
        textures.west.as_ref(),
    ]
    .into_iter()
    .flatten()
}
