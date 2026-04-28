use crate::{BlockId, ItemId, RecipeId, RegistryTable, TagId};

#[derive(Debug, Clone)]
pub struct CompiledRecipe {
    pub pattern: CompiledRecipePattern,
    pub result_item: ItemId,
    pub result_count: u32,
    pub ingredients: Vec<CompiledIngredient>,
    pub station: Option<BlockId>,
    pub time_seconds: Option<f32>,
    pub tags: Vec<TagId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledRecipePattern {
    Shapeless,
    Shaped,
    Processing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledIngredient {
    Item { item: ItemId, count: u32 },
    Tag { tag: TagId, count: u32 },
}

#[derive(Debug, Clone, Default)]
pub struct RecipeRegistry {
    table: RegistryTable<RecipeId, CompiledRecipe>,
    by_station: Vec<(Option<BlockId>, RecipeId)>,
}

impl RecipeRegistry {
    pub fn push(&mut self, key: crate::ContentKey, recipe: CompiledRecipe) -> RecipeId {
        let station = recipe.station;
        let id = self.table.push(key, recipe);
        self.by_station.push((station, id));
        id
    }

    pub fn get(&self, id: RecipeId) -> Option<&CompiledRecipe> {
        self.table.get(id)
    }

    pub fn id(&self, key: &crate::ContentKey) -> Option<RecipeId> {
        self.table.id(key)
    }

    pub fn key(&self, id: RecipeId) -> Option<&crate::ContentKey> {
        self.table.key(id)
    }

    pub fn recipes_for_station(
        &self,
        station: Option<BlockId>,
    ) -> impl Iterator<Item = RecipeId> + '_ {
        self.by_station
            .iter()
            .filter(move |(candidate, _)| *candidate == station)
            .map(|(_, id)| *id)
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}
