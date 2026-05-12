//! Runtime recipe registry — shaped crafting, shapeless crafting, and smelting.
//!
//! Recipes are compiled after both `ItemRegistry` and `TagRegistry` so that
//! every ingredient reference (item or tag) can be resolved to stable IDs.
//! The registry supports:
//!   - Shaped recipes (≤3×3 grid, positional).
//!   - Shapeless recipes (unordered bag of ingredients).
//!   - Smelting recipes (one ingredient → one output with fuel cost).

use crate::{ItemId, ItemRegistry, TagRegistry};
use std::collections::HashMap;
use vv_content_schema::{RawIngredient, RawRecipeDef, RawRecipeKind, RECIPE_FORMAT_VERSION};
use vv_content_schema::check_format_version;

// ─── RecipeId ────────────────────────────────────────────────────────────────

/// Compact, stable identifier for a compiled recipe.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct RecipeId(u32);

impl RecipeId {
    pub fn raw(self) -> u32 {
        self.0
    }
    pub(crate) fn from_raw(id: u32) -> Self {
        Self(id)
    }
}

// ─── Compiled ingredient ─────────────────────────────────────────────────────

/// A resolved ingredient: either a direct item or a tag-based match.
#[derive(Clone, Debug)]
pub enum CompiledIngredient {
    /// Matches exactly one item.
    Item(ItemId),
    /// Matches any item that carries this tag key.
    Tag(String),
}

impl CompiledIngredient {
    /// Returns `true` if `item_key` satisfies this ingredient requirement.
    pub fn matches(&self, item_key: &str, tag_registry: &TagRegistry) -> bool {
        match self {
            CompiledIngredient::Item(id) => {
                // Compare by key — we need the item registry for this.
                // Callers that have `ItemRegistry` available use `matches_id`.
                // This variant is a fallback when only the key is known.
                let _ = id;
                false
            }
            CompiledIngredient::Tag(tag_key) => tag_registry.has_tag(item_key, tag_key),
        }
    }

    /// Returns `true` if `candidate_id` satisfies this ingredient requirement.
    pub fn matches_id(
        &self,
        candidate_id: ItemId,
        candidate_key: &str,
        tag_registry: &TagRegistry,
    ) -> bool {
        match self {
            CompiledIngredient::Item(required_id) => candidate_id == *required_id,
            CompiledIngredient::Tag(tag_key) => tag_registry.has_tag(candidate_key, tag_key),
        }
    }
}

// ─── Compiled recipe kinds ───────────────────────────────────────────────────

/// A positional 3×3 grid recipe.
///
/// `grid` is a flat 9-element array (row-major, left-to-right, top-to-bottom).
/// `None` slots are empty.
#[derive(Clone, Debug)]
pub struct CompiledShapedRecipe {
    /// 3×3 grid stored row-major. `None` = empty slot.
    pub grid: [Option<CompiledIngredient>; 9],
    /// True if the pattern can be mirrored horizontally and still match.
    pub mirrored: bool,
}

impl CompiledShapedRecipe {
    /// Returns `true` if `slots` (9-element, `None` = empty) satisfies this recipe.
    /// `slots` is indexed the same way as `grid`.
    pub fn matches(
        &self,
        slots: &[Option<(ItemId, &str)>; 9],
        tags: &TagRegistry,
    ) -> bool {
        self.matches_grid(slots, tags, false)
            || (self.mirrored && self.matches_grid(slots, tags, true))
    }

    fn matches_grid(
        &self,
        slots: &[Option<(ItemId, &str)>; 9],
        tags: &TagRegistry,
        mirror: bool,
    ) -> bool {
        for row in 0..3usize {
            for col in 0..3usize {
                let grid_col = if mirror { 2 - col } else { col };
                let recipe_slot = &self.grid[row * 3 + grid_col];
                let item_slot = &slots[row * 3 + col];
                match (recipe_slot, item_slot) {
                    (None, None) => {}
                    (None, Some(_)) => return false,
                    (Some(_), None) => return false,
                    (Some(ingredient), Some((id, key))) => {
                        if !ingredient.matches_id(*id, key, tags) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
}

/// An unordered set of ingredients.
#[derive(Clone, Debug)]
pub struct CompiledShapelessRecipe {
    /// Every ingredient required; repetition encodes quantity.
    pub ingredients: Vec<CompiledIngredient>,
}

/// A furnace/smelter recipe.
#[derive(Clone, Debug)]
pub struct CompiledSmeltingRecipe {
    pub ingredient: CompiledIngredient,
    pub fuel: u32,
    pub smelt_seconds: f32,
}

/// Discriminated union over the three recipe kinds.
#[derive(Clone, Debug)]
pub enum CompiledRecipeKind {
    Shaped(CompiledShapedRecipe),
    Shapeless(CompiledShapelessRecipe),
    Smelting(CompiledSmeltingRecipe),
}

/// Fully compiled recipe.
#[derive(Clone, Debug)]
pub struct CompiledRecipe {
    pub id: RecipeId,
    pub key: String,
    pub output_item: ItemId,
    pub output_count: u32,
    /// Optional crafting station tag key.
    pub station_tag: Option<String>,
    /// UI grouping hint.
    pub group: Option<String>,
    pub kind: CompiledRecipeKind,
}

// ─── RecipeRegistry ──────────────────────────────────────────────────────────

/// Runtime registry of all compiled recipes.
#[derive(Debug, Default)]
pub struct RecipeRegistry {
    recipes: Vec<CompiledRecipe>,
    key_to_id: HashMap<String, RecipeId>,
    /// Shaped recipes indexed separately for fast crafting-grid lookup.
    shaped: Vec<RecipeId>,
    shapeless: Vec<RecipeId>,
    smelting: Vec<RecipeId>,
    /// Maps output ItemId → all recipes that produce it.
    by_output: HashMap<ItemId, Vec<RecipeId>>,
}

impl RecipeRegistry {
    pub(crate) fn new(recipes: Vec<CompiledRecipe>) -> Self {
        let key_to_id = recipes
            .iter()
            .map(|r| (r.key.clone(), r.id))
            .collect::<HashMap<_, _>>();

        let mut shaped = Vec::new();
        let mut shapeless = Vec::new();
        let mut smelting = Vec::new();
        let mut by_output: HashMap<ItemId, Vec<RecipeId>> = HashMap::new();

        for recipe in &recipes {
            by_output.entry(recipe.output_item).or_default().push(recipe.id);
            match &recipe.kind {
                CompiledRecipeKind::Shaped(_) => shaped.push(recipe.id),
                CompiledRecipeKind::Shapeless(_) => shapeless.push(recipe.id),
                CompiledRecipeKind::Smelting(_) => smelting.push(recipe.id),
            }
        }

        Self {
            recipes,
            key_to_id,
            shaped,
            shapeless,
            smelting,
            by_output,
        }
    }

    pub fn lookup(&self, key: &str) -> Option<RecipeId> {
        self.key_to_id.get(key).copied()
    }

    pub fn get(&self, id: RecipeId) -> Option<&CompiledRecipe> {
        self.recipes.get(id.raw() as usize)
    }

    pub fn get_by_key(&self, key: &str) -> Option<&CompiledRecipe> {
        self.get(self.lookup(key)?)
    }

    pub fn recipes(&self) -> &[CompiledRecipe] {
        &self.recipes
    }

    /// All shaped recipe IDs in registration order.
    pub fn shaped_ids(&self) -> &[RecipeId] {
        &self.shaped
    }

    /// All shapeless recipe IDs in registration order.
    pub fn shapeless_ids(&self) -> &[RecipeId] {
        &self.shapeless
    }

    /// All smelting recipe IDs in registration order.
    pub fn smelting_ids(&self) -> &[RecipeId] {
        &self.smelting
    }

    /// Returns all recipe IDs that produce `item_id` as output.
    pub fn recipes_for_output(&self, item_id: ItemId) -> &[RecipeId] {
        self.by_output.get(&item_id).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Find the first smelting recipe whose ingredient matches `item_key`.
    pub fn find_smelting(
        &self,
        item_id: ItemId,
        item_key: &str,
        tags: &TagRegistry,
    ) -> Option<&CompiledRecipe> {
        self.smelting.iter().find_map(|id| {
            let recipe = self.get(*id)?;
            match &recipe.kind {
                CompiledRecipeKind::Smelting(s) if s.ingredient.matches_id(item_id, item_key, tags) => {
                    Some(recipe)
                }
                _ => None,
            }
        })
    }

    pub fn len(&self) -> usize {
        self.recipes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }
}

// ─── Compilation ─────────────────────────────────────────────────────────────

/// Compile raw recipe definitions into a `RecipeRegistry`.
///
/// Requires `ItemRegistry` (for resolving output and direct-item ingredients)
/// and `TagRegistry` (for tag-based ingredients).
pub fn compile_recipes(
    mut raw: Vec<(String, RawRecipeDef)>,
    items: &ItemRegistry,
    tags: &TagRegistry,
) -> Result<RecipeRegistry, Vec<String>> {
    raw.sort_by(|(a, _), (b, _)| a.cmp(b));

    let mut errors = Vec::new();
    let mut recipes: Vec<CompiledRecipe> = Vec::with_capacity(raw.len());

    for (idx, (key, def)) in raw.into_iter().enumerate() {
        if let Err(e) = check_format_version(
            def.format_version,
            RECIPE_FORMAT_VERSION,
            "recipe",
            &key,
        ) {
            errors.push(e);
            continue;
        }

        // Resolve output item.
        let output_key = &def.result.item.0;
        let Some(output_item) = items.lookup(output_key) else {
            errors.push(format!(
                "recipe '{}': unknown output item '{}'",
                key, output_key
            ));
            continue;
        };

        let station_tag = def.station.map(|r| r.0);
        let group = def.group;
        let output_count = def.result.count;

        let kind = match def.recipe {
            RawRecipeKind::Shaped(raw_shaped) => {
                match compile_shaped(&key, raw_shaped, items, tags, &mut errors) {
                    Some(s) => CompiledRecipeKind::Shaped(s),
                    None => continue,
                }
            }
            RawRecipeKind::Shapeless(raw_sl) => {
                let ingredients = raw_sl
                    .ingredients
                    .into_iter()
                    .map(|ing| resolve_ingredient(&key, ing, items, &mut errors))
                    .flatten()
                    .collect();
                CompiledRecipeKind::Shapeless(CompiledShapelessRecipe { ingredients })
            }
            RawRecipeKind::Smelting(raw_sm) => {
                let Some(ingredient) =
                    resolve_ingredient(&key, raw_sm.ingredient, items, &mut errors)
                else {
                    continue;
                };
                if raw_sm.fuel == 0 {
                    errors.push(format!("recipe '{}': smelting fuel must be ≥ 1", key));
                    continue;
                }
                CompiledRecipeKind::Smelting(CompiledSmeltingRecipe {
                    ingredient,
                    fuel: raw_sm.fuel,
                    smelt_seconds: raw_sm.smelt_seconds.max(0.1),
                })
            }
        };

        recipes.push(CompiledRecipe {
            id: RecipeId(idx as u32),
            key,
            output_item,
            output_count,
            station_tag,
            group,
            kind,
        });
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(RecipeRegistry::new(recipes))
}

fn resolve_ingredient(
    recipe_key: &str,
    raw: RawIngredient,
    items: &ItemRegistry,
    errors: &mut Vec<String>,
) -> Option<CompiledIngredient> {
    match raw {
        RawIngredient::Item(item_ref) => {
            match items.lookup(&item_ref.0) {
                Some(id) => Some(CompiledIngredient::Item(id)),
                None => {
                    errors.push(format!(
                        "recipe '{}': unknown item ingredient '{}'",
                        recipe_key, item_ref.0
                    ));
                    None
                }
            }
        }
        RawIngredient::Tag(tag_ref) => {
            // Tag references are valid as long as the tag key is non-empty.
            // We don't error on unknown tags — tags may be defined by other packs.
            Some(CompiledIngredient::Tag(tag_ref.0))
        }
    }
}

fn compile_shaped(
    recipe_key: &str,
    raw: vv_content_schema::RawShapedRecipe,
    items: &ItemRegistry,
    tags: &TagRegistry,
    errors: &mut Vec<String>,
) -> Option<CompiledShapedRecipe> {
    if raw.pattern.is_empty() || raw.pattern.len() > 3 {
        errors.push(format!(
            "recipe '{}': shaped pattern must have 1–3 rows (has {})",
            recipe_key,
            raw.pattern.len()
        ));
        return None;
    }

    let mut grid: [Option<CompiledIngredient>; 9] = Default::default();
    let mut had_error = false;

    for (row_idx, row_str) in raw.pattern.iter().enumerate() {
        let chars: Vec<char> = row_str.chars().collect();
        if chars.len() > 3 {
            errors.push(format!(
                "recipe '{}': shaped pattern row {} has {} chars (max 3)",
                recipe_key,
                row_idx,
                chars.len()
            ));
            had_error = true;
            continue;
        }
        for (col_idx, ch) in chars.iter().enumerate() {
            if *ch == ' ' {
                continue; // empty slot
            }
            let sym = ch.to_string();
            match raw.keys.get(&sym) {
                Some(raw_ing) => {
                    match resolve_ingredient(recipe_key, raw_ing.clone(), items, errors) {
                        Some(ing) => grid[row_idx * 3 + col_idx] = Some(ing),
                        None => had_error = true,
                    }
                }
                None => {
                    errors.push(format!(
                        "recipe '{}': pattern symbol '{}' has no entry in keys",
                        recipe_key, ch
                    ));
                    had_error = true;
                }
            }
        }
    }

    if had_error {
        return None;
    }

    // Recipes are mirrored by default — disable only if explicitly asymmetric.
    Some(CompiledShapedRecipe { grid, mirrored: true })
}
