use super::helpers::*;
use super::prelude::*;

use super::*;

impl ContentCompiler {
    pub(super) fn compile(&mut self, load_order: &PackLoadOrder) -> CompileResult<CompiledContent> {
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
        let mut material_ids = HashMap::<ContentKey, MaterialId>::new();

        for (key, doc) in &tag_docs {
            let tag = self.compile_tag(doc, &index);
            content.tags.push(key.clone(), tag);
        }
        for (key, doc) in &block_docs {
            let visual_id = self.compile_block_visual(key, doc, &mut content, &mut material_ids);
            let block = self.compile_block(doc, &index, &texture_ids, visual_id);
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

    pub(super) fn collect_domain<'a, T, I>(
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
}
