use crate::{EntityId, RegistryTable, TagId};

#[derive(Debug, Clone)]
pub struct CompiledEntity {
    pub display_key: Option<String>,
    pub tags: Vec<TagId>,
    pub health: f32,
    pub light_level: u8,
}

pub type EntityRegistry = RegistryTable<EntityId, CompiledEntity>;
