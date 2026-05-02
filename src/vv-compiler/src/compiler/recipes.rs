use super::helpers::*;
use super::prelude::*;

use super::*;

impl ContentCompiler {
    pub(super) fn compile_recipe(
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
}
