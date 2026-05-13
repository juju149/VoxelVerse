//! Compiler pass for the unified `.object.ron` format.
//!
//! `compile_objects()` takes the raw `Vec<(String, RawObjectDef)>` from the
//! loader and produces all compiled registries that the runtime needs.
//!
//! Separation contract:
//! - Raw:     `RawObjectDef`           — lives in `vv-content-schema`
//! - Loaded:  `LoadedPack.objects`     — lives in `vv-pack-loader`
//! - Compiled:`CompiledObjects`        — this module
//! - Runtime: `BlockRegistry`, `ItemRegistry`, … — unchanged downstream types

use std::collections::{HashMap, HashSet};

use vv_content_schema::{
    RawObjectCount, RawObjectDef, RawObjectRecipeKind, RawObjectRecipeSection,
    RawObjectRenderMode, RawObjectShape, RawObjectTexture, RawObjectTint, RawObjectToolKind,
    RawObjectWeaponKind,
};
use vv_voxel::VoxelId;

use crate::block_family::{BlockStateValue, CompiledBlockFamily};
use crate::block_registry::{
    BlockMaterialLayers, BlockModelId, BlockModelRegistry, BlockRegistry, CompiledBlock,
    CompiledBlockModel, CompiledBlockVisual, CompiledCollision, CompiledMesh, MaterialTextureSet,
};
use crate::item_registry::{
    CompiledConsumableData, CompiledFoodData, CompiledIngredientData, CompiledItem,
    CompiledItemGameplay, CompiledItemVisual, CompiledItemWorldModel, CompiledToolData,
    CompiledWeaponClass, CompiledWeaponData, ItemId, ItemRegistry, StackSize,
};
use crate::loot_registry::{CompiledLootEntry, CompiledLootTable, LootRegistry, LootTableId};
use crate::recipe_registry::{
    CompiledIngredient, CompiledRecipe, CompiledRecipeKind, CompiledShapedRecipe,
    CompiledShapelessRecipe, CompiledSmeltingRecipe, RecipeId, RecipeRegistry,
};
use crate::tag_registry::{CompiledTag, TagId, TagRegistry};

/// All compiled registries produced from a set of `RawObjectDef` files.
pub struct CompiledObjects {
    pub blocks: BlockRegistry,
    pub items: ItemRegistry,
    pub loot: LootRegistry,
    pub tags: TagRegistry,
    pub recipes: RecipeRegistry,
}

pub fn compile_objects(
    mut raw: Vec<(String, RawObjectDef)>,
) -> Result<CompiledObjects, Vec<String>> {
    // Sort all objects alphabetically for deterministic IDs — air exception
    // handled below.
    raw.sort_by(|(a, _), (b, _)| a.cmp(b));

    // ── 1. Tag registry ──────────────────────────────────────────────────────
    let tags = compile_tags_from_objects(&raw);

    // ── 2. Item registry ─────────────────────────────────────────────────────
    let items = compile_items_from_objects(&raw)?;

    // ── 3. Loot registry — needs ItemRegistry ────────────────────────────────
    let loot = compile_loot_from_objects(&raw, &items);

    // ── 4. Block registry ────────────────────────────────────────────────────
    let blocks = compile_blocks(&raw)?;

    // ── 5. Recipe registry ───────────────────────────────────────────────────
    let recipes = compile_recipes_from_objects(&raw, &items, &tags)?;

    Ok(CompiledObjects {
        blocks,
        items,
        loot,
        tags,
        recipes,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tag compilation
// ─────────────────────────────────────────────────────────────────────────────

fn compile_tags_from_objects(raw: &[(String, RawObjectDef)]) -> TagRegistry {
    // Collect all unique tag names (sorted for deterministic IDs).
    let all_tags: Vec<String> = raw
        .iter()
        .flat_map(|(_, def)| def.tags.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    // Accumulate member object-keys per tag name.
    let mut tag_members: HashMap<String, HashSet<String>> = HashMap::new();
    for (obj_key, def) in raw {
        for tag in &def.tags {
            tag_members
                .entry(tag.clone())
                .or_default()
                .insert(obj_key.clone());
        }
    }

    let compiled: Vec<CompiledTag> = all_tags
        .iter()
        .enumerate()
        .map(|(idx, tag_name)| {
            let values = tag_members
                .remove(tag_name)
                .unwrap_or_default();
            CompiledTag {
                id: TagId::from_raw(idx as u32),
                key: format!("core:tag/{}", tag_name),
                values,
            }
        })
        .collect();

    TagRegistry::new(compiled)
}

// ─────────────────────────────────────────────────────────────────────────────
// ─────────────────────────────────────────────────────────────────────────────
// Block compilation
// ─────────────────────────────────────────────────────────────────────────────

fn compile_blocks(raw: &[(String, RawObjectDef)]) -> Result<BlockRegistry, Vec<String>> {
    let mut errors = Vec::new();

    // Find the air object: must have tag "air" AND a block section.
    let air_key = raw
        .iter()
        .find(|(_, def)| def.tags.contains(&"air".to_string()) && def.block.is_some())
        .map(|(k, _)| k.clone());

    if air_key.is_none() {
        errors.push("Object pack must have exactly one block object tagged 'air'.".into());
        return Err(errors);
    }
    let air_key = air_key.unwrap();

    let model_registry = build_stub_model_registry();
    let cube_id = BlockModelId::from_raw(0);
    let column_id = BlockModelId::from_raw(1);

    let mut blocks: Vec<CompiledBlock> = Vec::new();
    let mut families: Vec<CompiledBlockFamily> = Vec::new();
    let mut material_sets: Vec<MaterialTextureSet> = Vec::new();
    let mut mat_dedup: HashMap<MaterialTextureSet, u32> = HashMap::new();
    let mut next_voxel_id: u16 = 0;
    let mut default_place = VoxelId::AIR;
    let mut planet_core_opt: Option<VoxelId> = None;

    // Insertion order: air first, then rest sorted.
    let mut ordered: Vec<&(String, RawObjectDef)> = raw
        .iter()
        .filter(|(k, def)| k.as_str() == air_key.as_str() && def.block.is_some())
        .collect();
    ordered.extend(
        raw.iter()
            .filter(|(k, def)| k.as_str() != air_key.as_str() && def.block.is_some()),
    );

    for (key, def) in &ordered {
        let block_sec = def.block.as_ref().expect("filtered above");

        let solid = block_sec.solid && block_sec.render != RawObjectRenderMode::Invisible;
        let hardness = block_sec.hardness;
        let is_air = key.as_str() == air_key.as_str();
        let id = VoxelId::new(next_voxel_id);
        next_voxel_id = next_voxel_id.saturating_add(1);

        let model_id = match block_sec.shape {
            RawObjectShape::Column => column_id,
            _ => cube_id,
        };

        let tint = block_sec.tint.map(tint_color).unwrap_or([1.0, 1.0, 1.0]);
        let namespace = key.split_once(':').map(|(ns, _)| ns).unwrap_or("core");
        let layers = texture_to_layers(
            &block_sec.texture,
            namespace,
            &mut material_sets,
            &mut mat_dedup,
        );
        let visual = CompiledBlockVisual {
            layers,
            tint,
            model_id,
        };

        let category = def.tags.first().cloned().unwrap_or_else(|| "terrain".into());
        let color = category_color(&category);
        let max_stack = def.item.as_ref().map(|i| i.stack).unwrap_or(99);

        // Loot key string — resolved at runtime; `compile_loot_from_objects`
        // creates the actual table under this same key.
        let drops_key = if def.mining.is_some() {
            loot_key_for_object(key)
        } else {
            String::new()
        };

        let preferred_tool_tag = def.mining.as_ref().map(|m| tool_tag_key(m.tool));

        if !is_air && default_place == VoxelId::AIR && solid {
            default_place = id;
        }
        if planet_core_opt.is_none()
            && (def.tags.contains(&"core".to_string())
                || def.tags.contains(&"unbreakable".to_string()))
        {
            planet_core_opt = Some(id);
        }

        let state = BlockStateValue::default();
        blocks.push(CompiledBlock {
            id,
            family_key: key.to_string(),
            state: state.clone(),
            display_name: def.name.clone(),
            solid,
            color,
            hardness,
            visual,
            category,
            max_stack,
            drops_key,
            preferred_tool_tag,
        });
        families.push(CompiledBlockFamily {
            family_key: key.to_string(),
            state_schema: Default::default(),
            variants: vec![id],
            default_variant: id,
            state_to_id: {
                let mut m = HashMap::new();
                m.insert(state.clone(), id);
                m
            },
            id_to_state: {
                let mut m = HashMap::new();
                m.insert(id, state);
                m
            },
        });
    }

    if blocks.is_empty() {
        errors.push("No block objects found in pack.".into());
        return Err(errors);
    }

    let planet_core = planet_core_opt
        .or_else(|| blocks.iter().find(|b| b.hardness < 0.0).map(|b| b.id))
        .unwrap_or(VoxelId::AIR);

    // One white entry for the fallback (index 0) plus one per actual material set.
    let material_colors: Vec<[f32; 4]> = std::iter::once([1.0_f32, 1.0, 1.0, 1.0])
        .chain(material_sets.iter().map(|_| [1.0_f32, 1.0, 1.0, 1.0]))
        .collect();

    Ok(BlockRegistry::new(
        blocks,
        families,
        material_sets,
        material_colors,
        default_place,
        planet_core,
        model_registry,
    ))
}

fn build_stub_model_registry() -> BlockModelRegistry {
    let cube = CompiledBlockModel {
        id: BlockModelId::from_raw(0),
        key: "object:model/cube".into(),
        mesh: CompiledMesh::Cube {
            ambient_occlusion: true,
        },
        collision: CompiledCollision::FullCube,
        face_layers: vec![
            "py".into(), "ny".into(), "pz".into(),
            "nz".into(), "px".into(), "nx".into(),
        ],
    };
    let column = CompiledBlockModel {
        id: BlockModelId::from_raw(1),
        key: "object:model/column".into(),
        mesh: CompiledMesh::CubeColumn {
            ambient_occlusion: true,
        },
        collision: CompiledCollision::FullCube,
        face_layers: vec!["end".into(), "side".into()],
    };
    BlockModelRegistry::new(vec![cube, column])
}

// ─────────────────────────────────────────────────────────────────────────────
// Loot compilation (needs ItemRegistry to resolve item_id)
// ─────────────────────────────────────────────────────────────────────────────

fn compile_loot_from_objects(
    raw: &[(String, RawObjectDef)],
    items: &ItemRegistry,
) -> LootRegistry {
    let mut tables: Vec<CompiledLootTable> = Vec::new();
    let mut idx = 0u32;

    for (key, def) in raw {
        let Some(mining) = &def.mining else {
            continue;
        };

        let entries: Vec<CompiledLootEntry> = match &mining.drops {
            None => {
                // Self-drop: use the item with the same key as this object.
                if let Some(item_id) = items.lookup(key) {
                    vec![CompiledLootEntry {
                        item_id,
                        count_min: 1,
                        count_max: 1,
                        chance: 1.0,
                    }]
                } else {
                    Vec::new()
                }
            }
            Some(list) => list
                .iter()
                .filter_map(|e| {
                    let item_id = resolve_item_id(&e.item, items)?;
                    let (count_min, count_max) = match e.count {
                        RawObjectCount::Fixed(n) => (n, n),
                        RawObjectCount::Range(a, b) => (a, b),
                    };
                    Some(CompiledLootEntry {
                        item_id,
                        count_min,
                        count_max,
                        chance: e.chance,
                    })
                })
                .collect(),
        };

        tables.push(CompiledLootTable {
            id: LootTableId::from_raw(idx),
            key: loot_key_for_object(key),
            rolls: 1,
            entries,
        });
        idx += 1;
    }

    LootRegistry::new(tables)
}

fn loot_key_for_object(obj_key: &str) -> String {
    if let Some((ns, path)) = obj_key.split_once(':') {
        format!("{}:loot/{}", ns, path)
    } else {
        format!("loot/{}", obj_key)
    }
}

fn resolve_item_id(short_name: &str, items: &ItemRegistry) -> Option<ItemId> {
    if let Some(id) = items.lookup(short_name) {
        return Some(id);
    }
    items.items().iter().find_map(|item| {
        let stem = item.key.rsplit('/').next().unwrap_or(&item.key);
        if stem == short_name {
            Some(item.id)
        } else {
            None
        }
    })
}

fn tool_tag_key(kind: RawObjectToolKind) -> String {
    let short = match kind {
        RawObjectToolKind::Pickaxe => "pickaxe",
        RawObjectToolKind::Shovel => "shovel",
        RawObjectToolKind::Axe => "axe",
        RawObjectToolKind::Shears => "shears",
        RawObjectToolKind::Hoe => "hoe",
        RawObjectToolKind::Any => return String::new(),
    };
    format!("core:tag/tool/{}", short)
}

fn tint_color(tint: RawObjectTint) -> [f32; 3] {
    match tint {
        RawObjectTint::Grass => [0.5, 0.8, 0.3],
        RawObjectTint::Foliage => [0.4, 0.7, 0.2],
        RawObjectTint::Water => [0.2, 0.4, 0.9],
    }
}

fn category_color(cat: &str) -> [f32; 3] {
    match cat {
        "terrain" | "stone" | "sand" | "snow" => [0.6, 0.6, 0.6],
        "soil" | "grass" => [0.4, 0.7, 0.3],
        "ore" => [0.5, 0.5, 0.8],
        "log" | "wood" | "leaves" => [0.5, 0.7, 0.4],
        _ => [0.8, 0.8, 0.8],
    }
}

/// Intern a texture path into the material set list, returning its 1-based
/// atlas index. Index 0 is reserved for the fallback white material added by
/// `TextureRegistry::load`.
fn intern_texture(
    path: &str,
    namespace: &str,
    material_sets: &mut Vec<MaterialTextureSet>,
    mat_dedup: &mut HashMap<MaterialTextureSet, u32>,
) -> u32 {
    let mat = MaterialTextureSet {
        albedo:    format!("{}:{}.albedo.png",    namespace, path),
        normal:    format!("{}:{}.normal.png",    namespace, path),
        roughness: format!("{}:{}.roughness.png", namespace, path),
    };
    if let Some(&idx) = mat_dedup.get(&mat) {
        return idx;
    }
    let idx = material_sets.len() as u32 + 1; // 0 = fallback
    mat_dedup.insert(mat.clone(), idx);
    material_sets.push(mat);
    idx
}

/// Map a `RawObjectTexture` to per-face atlas indices.
fn texture_to_layers(
    tex: &RawObjectTexture,
    namespace: &str,
    material_sets: &mut Vec<MaterialTextureSet>,
    mat_dedup: &mut HashMap<MaterialTextureSet, u32>,
) -> BlockMaterialLayers {
    match tex {
        RawObjectTexture::None => BlockMaterialLayers::default(),
        RawObjectTexture::All(path) => {
            let idx = intern_texture(path, namespace, material_sets, mat_dedup);
            BlockMaterialLayers {
                top: idx, bottom: idx,
                front: idx, back: idx, left: idx, right: idx,
            }
        }
        RawObjectTexture::Cube { top, side, bottom } => {
            let top_idx  = intern_texture(top,    namespace, material_sets, mat_dedup);
            let side_idx = intern_texture(side,   namespace, material_sets, mat_dedup);
            let bot_idx  = intern_texture(bottom, namespace, material_sets, mat_dedup);
            BlockMaterialLayers {
                top: top_idx, bottom: bot_idx,
                front: side_idx, back: side_idx, left: side_idx, right: side_idx,
            }
        }
        RawObjectTexture::Column { top, side } => {
            let top_idx  = intern_texture(top,  namespace, material_sets, mat_dedup);
            let side_idx = intern_texture(side, namespace, material_sets, mat_dedup);
            BlockMaterialLayers {
                top: top_idx, bottom: top_idx,
                front: side_idx, back: side_idx, left: side_idx, right: side_idx,
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Item compilation
// ─────────────────────────────────────────────────────────────────────────────

fn compile_items_from_objects(
    raw: &[(String, RawObjectDef)],
) -> Result<ItemRegistry, Vec<String>> {
    let mut items: Vec<CompiledItem> = Vec::new();
    let mut idx = 0u32;

    // Alphabetical order guaranteed by pre-sort.
    for (key, def) in raw {
        // An object produces an item if it has an item section, OR if it has
        // a block section (block items are automatically grantable).
        let has_item_section = def.item.is_some();
        let has_block = def.block.is_some();

        if !has_item_section && !has_block {
            continue;
        }

        // Determine gameplay category.
        let category = infer_item_category(def);

        let stack_size = def
            .item
            .as_ref()
            .map(|i| i.stack)
            .unwrap_or(if has_block { 99 } else { 1 });

        let icon_key = def
            .item
            .as_ref()
            .and_then(|i| i.icon.clone())
            .unwrap_or_else(|| format!("items/{}", stem_from_key(key)));

        let world_model = def
            .item
            .as_ref()
            .and_then(|i| i.model.clone())
            .map(CompiledItemWorldModel::Voxel)
            .unwrap_or(if has_block {
                CompiledItemWorldModel::BlockItem(key.clone())
            } else {
                CompiledItemWorldModel::None
            });

        let visual = CompiledItemVisual {
            icon_key,
            world_model,
            hand_model_key: None,
        };

        let gameplay = compile_item_gameplay(def, key);
        let tag_keys = def
            .tags
            .iter()
            .map(|t| format!("core:tag/{}", t))
            .collect();

        items.push(CompiledItem {
            id: ItemId::from_raw(idx),
            key: key.clone(),
            display_name: def.name.clone(),
            category,
            stack_size: StackSize(stack_size),
            visual,
            gameplay,
            tag_keys,
        });
        idx += 1;
    }

    Ok(ItemRegistry::new(items))
}

fn compile_item_gameplay(def: &RawObjectDef, key: &str) -> CompiledItemGameplay {
    // Higher-priority sections first.
    if let Some(tool) = &def.tool {
        return CompiledItemGameplay::Tool(CompiledToolData {
            tool_tag_keys: vec![format!(
                "core:tag/tool/{}",
                format!("{:?}", tool.tool_type).to_lowercase()
            )],
            tier: tool.tier,
            mining_speed: tool.speed,
            durability: tool.durability,
        });
    }

    if let Some(weapon) = &def.weapon {
        return CompiledItemGameplay::Weapon(CompiledWeaponData {
            class: match weapon.weapon_type {
                RawObjectWeaponKind::Bow => CompiledWeaponClass::Bow,
                _ => CompiledWeaponClass::Sword,
            },
            damage: weapon.damage,
            attack_speed: 1.0 / weapon.draw_time.max(0.01),
            durability: weapon.durability,
            projectile_key: None,
        });
    }

    if let Some(food) = &def.food {
        return CompiledItemGameplay::Food(CompiledFoodData {
            nutrition: food.hunger,
            saturation: food.saturation,
            eat_seconds: food.eat_time,
        });
    }

    if let Some(effect) = &def.effect {
        return CompiledItemGameplay::Consumable(CompiledConsumableData {
            effect_key: format!("{:?}", effect.on_use).to_lowercase(),
            magnitude: effect.heal,
            use_seconds: effect.use_time,
        });
    }

    if def.block.is_some() {
        return CompiledItemGameplay::PlaceBlock {
            block_key: key.to_string(),
        };
    }

    // Default: crafting ingredient with optional fuel.
    let fuel = def.fuel.as_ref().map(|f| f.duration);
    CompiledItemGameplay::CraftingIngredient(CompiledIngredientData {
        fuel_value: fuel,
        smelts_to_key: None,
    })
}

fn infer_item_category(def: &RawObjectDef) -> String {
    if def.tool.is_some() {
        return "tool".into();
    }
    if def.weapon.is_some() {
        return "weapon".into();
    }
    if def.food.is_some() {
        return "food".into();
    }
    if def.effect.is_some() {
        return "consumable".into();
    }
    if def.station.is_some() || def.storage.is_some() {
        return "furniture".into();
    }
    if def.block.is_some() {
        return def.tags.first().cloned().unwrap_or_else(|| "block".into());
    }
    "resource".into()
}

fn stem_from_key(key: &str) -> String {
    key.rsplit('/').next().unwrap_or(key).to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Recipe compilation
// ─────────────────────────────────────────────────────────────────────────────

fn compile_recipes_from_objects(
    raw: &[(String, RawObjectDef)],
    items: &ItemRegistry,
    _tags: &TagRegistry,
) -> Result<RecipeRegistry, Vec<String>> {
    let mut recipes: Vec<CompiledRecipe> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut idx = 0u32;

    for (key, def) in raw {
        for (recipe_idx, recipe) in def.recipes.iter().enumerate() {
            let Some((compiled, next_idx)) =
                compile_one_recipe(key, recipe_idx, recipe, items, idx, &mut errors)
            else {
                continue;
            };
            recipes.push(compiled);
            idx = next_idx;
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(RecipeRegistry::new(recipes))
}

fn compile_one_recipe(
    object_key: &str,
    recipe_idx: usize,
    recipe: &RawObjectRecipeSection,
    items: &ItemRegistry,
    next_id: u32,
    errors: &mut Vec<String>,
) -> Option<(CompiledRecipe, u32)> {
    let Some(output_id) = resolve_item_id(&recipe.output.item, items) else {
        return None;
    };
    let recipe_key = if recipe_idx == 0 {
        format!("core:recipe/{}", object_key.trim_start_matches("core:"))
    } else {
        format!(
            "core:recipe/{}#{}",
            object_key.trim_start_matches("core:"),
            recipe_idx
        )
    };

    let kind = match &recipe.kind {
        RawObjectRecipeKind::Shaped(shaped) => {
            let mut grid: [Option<CompiledIngredient>; 9] = Default::default();
            let mut slot = 0usize;
            'outer: for row in &shaped.pattern {
                for ch in row.chars() {
                    if slot >= 9 {
                        break 'outer;
                    }
                    grid[slot] = if ch == ' ' {
                        None
                    } else {
                        let sym = ch.to_string();
                        match shaped.legend.get(&sym) {
                            Some(item_name) => {
                                resolve_item_id(item_name, items).map(CompiledIngredient::Item)
                            }
                            None => {
                                errors.push(format!(
                                    "recipe '{}'#{}: legend symbol '{}' has no mapping",
                                    object_key, recipe_idx, sym
                                ));
                                None
                            }
                        }
                    };
                    slot += 1;
                }
            }
            CompiledRecipeKind::Shaped(CompiledShapedRecipe {
                grid,
                mirrored: true,
            })
        }
        RawObjectRecipeKind::Shapeless(shapeless) => {
            let ingredients = shapeless
                .ingredients
                .iter()
                .filter_map(|name| resolve_item_id(name, items).map(CompiledIngredient::Item))
                .collect();
            CompiledRecipeKind::Shapeless(CompiledShapelessRecipe { ingredients })
        }
        RawObjectRecipeKind::Processing(processing) => {
            if processing.inputs.is_empty() {
                errors.push(format!(
                    "recipe '{}'#{}: processing inputs list is empty",
                    object_key, recipe_idx
                ));
                return None;
            }
            let Some(ing_id) = resolve_item_id(&processing.inputs[0].item, items) else {
                return None;
            };
            CompiledRecipeKind::Smelting(CompiledSmeltingRecipe {
                ingredient: CompiledIngredient::Item(ing_id),
                fuel: 1,
                smelt_seconds: processing.duration_seconds,
            })
        }
    };

    Some((
        CompiledRecipe {
            id: RecipeId::from_raw(next_id),
            key: recipe_key,
            output_item: output_id,
            output_count: recipe.output.count,
            station_tag: recipe.station.clone(),
            group: recipe.group.clone(),
            kind,
        },
        next_id + 1,
    ))
}
