use crate::{PlaceableId, RegistryTable, TagId};

#[derive(Debug, Clone)]
pub struct CompiledPlaceable {
    pub display_key: Option<String>,
    pub tags: Vec<TagId>,
    pub light_level: u8,
}

pub type PlaceableRegistry = RegistryTable<PlaceableId, CompiledPlaceable>;
