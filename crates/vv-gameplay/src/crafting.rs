use crate::{Hotbar, HotbarSlot, Inventory};
use vv_pack_compiler::{
    CompiledIngredient, CompiledItem, CompiledRecipe, CompiledRecipeKind, ItemId, ItemRegistry,
    RecipeRegistry, TagRegistry,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CraftError {
    NotCraftableHere,
    MissingIngredients,
    OutputFull,
}

pub fn quick_craft_recipe_indices(recipes: &RecipeRegistry) -> Vec<usize> {
    recipes
        .recipes()
        .iter()
        .enumerate()
        .filter_map(|(index, recipe)| match recipe.kind {
            CompiledRecipeKind::Shaped(_) | CompiledRecipeKind::Shapeless(_) => Some(index),
            CompiledRecipeKind::Smelting(_) => None,
        })
        .collect()
}

pub fn can_craft_recipe(
    recipe: &CompiledRecipe,
    items: &ItemRegistry,
    tags: &TagRegistry,
    hotbar: &Hotbar,
    inventory: &Inventory,
    quantity: u32,
) -> bool {
    craft_recipe_inner(recipe, items, tags, hotbar, inventory, quantity, false).is_ok()
}

pub fn craft_recipe(
    recipe: &CompiledRecipe,
    items: &ItemRegistry,
    tags: &TagRegistry,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    quantity: u32,
) -> Result<(), CraftError> {
    let (crafted_hotbar, crafted_inventory) =
        craft_recipe_inner(recipe, items, tags, hotbar, inventory, quantity, true)?;
    *hotbar = crafted_hotbar;
    *inventory = crafted_inventory;
    Ok(())
}

fn craft_recipe_inner(
    recipe: &CompiledRecipe,
    items: &ItemRegistry,
    tags: &TagRegistry,
    hotbar: &Hotbar,
    inventory: &Inventory,
    quantity: u32,
    _commit: bool,
) -> Result<(Hotbar, Inventory), CraftError> {
    let ingredients = quick_craft_ingredients(recipe)?;
    let quantity = quantity.max(1);
    let mut next_hotbar = hotbar.clone();
    let mut next_inventory = inventory.clone();

    for _ in 0..quantity {
        for ingredient in &ingredients {
            if !consume_one_matching(
                ingredient,
                items,
                tags,
                &mut next_hotbar,
                &mut next_inventory,
            ) {
                return Err(CraftError::MissingIngredients);
            }
        }
    }

    let output_item = items
        .get(recipe.output_item)
        .ok_or(CraftError::NotCraftableHere)?;
    let output_count = recipe.output_count.saturating_mul(quantity);
    if !add_output(
        recipe.output_item,
        output_count,
        output_item,
        &mut next_hotbar,
        &mut next_inventory,
    ) {
        return Err(CraftError::OutputFull);
    }

    Ok((next_hotbar, next_inventory))
}

fn quick_craft_ingredients(recipe: &CompiledRecipe) -> Result<Vec<CompiledIngredient>, CraftError> {
    match &recipe.kind {
        CompiledRecipeKind::Shaped(shaped) => {
            Ok(shaped.grid.iter().filter_map(Clone::clone).collect())
        }
        CompiledRecipeKind::Shapeless(shapeless) => Ok(shapeless.ingredients.clone()),
        CompiledRecipeKind::Smelting(_) => Err(CraftError::NotCraftableHere),
    }
}

fn consume_one_matching(
    ingredient: &CompiledIngredient,
    items: &ItemRegistry,
    tags: &TagRegistry,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
) -> bool {
    let mut hotbar_slots = *hotbar.slots();
    for slot in &mut hotbar_slots {
        if slot_matches(*slot, ingredient, items, tags) {
            decrement_slot(slot);
            hotbar.set_slots(hotbar_slots);
            return true;
        }
    }

    for index in 0..inventory.slots().len() {
        let slot = inventory.slot(index);
        if slot_matches(slot, ingredient, items, tags) {
            let mut updated = slot;
            decrement_slot(&mut updated);
            inventory.set(index, updated);
            return true;
        }
    }

    false
}

fn slot_matches(
    slot: Option<HotbarSlot>,
    ingredient: &CompiledIngredient,
    items: &ItemRegistry,
    tags: &TagRegistry,
) -> bool {
    let Some(stack) = slot else { return false };
    let Some(item) = items.get(stack.item_id) else {
        return false;
    };
    ingredient.matches_id(stack.item_id, &item.key, tags)
}

fn decrement_slot(slot: &mut Option<HotbarSlot>) {
    if let Some(stack) = slot.as_mut() {
        stack.quantity = stack.quantity.saturating_sub(1);
        if stack.quantity == 0 {
            *slot = None;
        }
    }
}

fn add_output(
    item_id: ItemId,
    count: u32,
    item: &CompiledItem,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
) -> bool {
    let max_stack = item.stack_size.0;
    let original_hotbar = hotbar.clone();
    if hotbar.add(item_id, count, max_stack) {
        return true;
    }
    *hotbar = original_hotbar;
    inventory.add(item_id, count, max_stack)
}
