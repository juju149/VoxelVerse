use vv_registry::{CompiledContent, CompiledIngredient, CompiledRecipe, RecipeId};

use crate::{has_recipe_ingredients, Inventory, ItemStack};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CraftError {
    MissingRecipe,
    StationRecipe,
    MissingIngredients,
    InventoryFull,
}

pub fn can_craft_hand_recipe(
    inventory: &Inventory,
    recipe_id: RecipeId,
    content: &CompiledContent,
) -> bool {
    let Some(recipe) = content.recipes.get(recipe_id) else {
        return false;
    };
    recipe.station.is_none() && has_recipe_ingredients(inventory, recipe, content)
}

pub fn craft_hand_recipe(
    inventory: &mut Inventory,
    recipe_id: RecipeId,
    content: &CompiledContent,
) -> Result<(), CraftError> {
    let recipe = content
        .recipes
        .get(recipe_id)
        .ok_or(CraftError::MissingRecipe)?;
    if recipe.station.is_some() {
        return Err(CraftError::StationRecipe);
    }
    if !has_recipe_ingredients(inventory, recipe, content) {
        return Err(CraftError::MissingIngredients);
    }
    let result = ItemStack::new(recipe.result_item, recipe.result_count);
    if !inventory.can_insert_stack(result, content) {
        return Err(CraftError::InventoryFull);
    }

    for ingredient in &recipe.ingredients {
        match ingredient {
            CompiledIngredient::Item { item, count } => {
                inventory.remove_item(*item, *count);
            }
            CompiledIngredient::Tag { .. } => {
                return Err(CraftError::MissingIngredients);
            }
        }
    }

    inventory.insert_stack(result, content);

    Ok(())
}

fn has_ingredients(inventory: &Inventory, recipe: &CompiledRecipe) -> bool {
    recipe
        .ingredients
        .iter()
        .all(|ingredient| match ingredient {
            CompiledIngredient::Item { item, count } => inventory.item_count(*item) >= *count,
            CompiledIngredient::Tag { .. } => false,
        })
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use vv_compiler::compile_assets_root;
    use vv_registry::{ContentKey, ItemId};

    use super::*;

    #[test]
    fn alpha_crafting_progression_is_recipe_driven() {
        let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let content = compile_assets_root(&assets).expect("core content should compile");
        let mut inventory = Inventory::player_default();

        let wood_log = item(&content, "wood_log");
        let planks = item(&content, "planks_oak");
        let sticks = item(&content, "stick");
        let cobblestone = item(&content, "cobblestone");
        let pickaxe_wood = item(&content, "pickaxe_wood");
        let pickaxe_stone = item(&content, "pickaxe_stone");

        inventory.insert_stack(ItemStack::new(wood_log, 2), &content);
        craft(&mut inventory, &content, "logs_to_planks");
        craft(&mut inventory, &content, "logs_to_planks");
        craft(&mut inventory, &content, "planks_to_sticks");
        craft(&mut inventory, &content, "pickaxe_wood");

        assert_eq!(inventory.item_count(pickaxe_wood), 1);
        assert!(inventory.item_count(planks) >= 3);
        assert!(inventory.item_count(sticks) >= 2);

        inventory.insert_stack(ItemStack::new(cobblestone, 3), &content);
        craft(&mut inventory, &content, "pickaxe_stone");

        assert_eq!(inventory.item_count(pickaxe_stone), 1);
    }

    fn item(content: &CompiledContent, name: &str) -> ItemId {
        content
            .items
            .id(&ContentKey::from_str(&format!("voxelverse:{name}")).unwrap())
            .unwrap_or_else(|| panic!("missing item {name}"))
    }

    fn craft(inventory: &mut Inventory, content: &CompiledContent, name: &str) {
        let recipe = content
            .recipes
            .id(&ContentKey::from_str(&format!("voxelverse:{name}")).unwrap())
            .unwrap_or_else(|| panic!("missing recipe {name}"));
        craft_hand_recipe(inventory, recipe, content).unwrap_or_else(|err| {
            panic!("recipe {name} should craft from current inventory: {err:?}")
        });
    }
}
