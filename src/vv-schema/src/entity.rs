use crate::common::{LangKey, ResourceRef, ScriptRef, TagRef};
use crate::loot::DropSpec;
use serde::{Deserialize, Serialize};

/// Raw entity definition (living creature, mob, NPC, fauna).
/// Non-living placed objects use PlaceableDef.
/// Deserialized from defs/entities/<name>.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EntityDef {
    /// Lang key override. If absent: auto-derived as "entity.<ns>.<name>".
    pub display_key: Option<LangKey>,
    pub model: Option<ResourceRef>,
    pub health: f32,
    pub tags: Vec<TagRef>,
    pub drops: DropSpec,
    pub light_level: u8,
    #[serde(default)]
    pub ai: Option<AiSpec>,
    pub movement: MovementSpec,
    pub interactions: Vec<EntityInteraction>,
    pub body: EntityBody,
}

impl Default for EntityDef {
    fn default() -> Self {
        EntityDef {
            display_key: None,
            model: None,
            health: 20.0,
            tags: vec![],
            drops: DropSpec::default(),
            light_level: 0,
            ai: None,
            movement: MovementSpec::default(),
            interactions: vec![],
            body: EntityBody::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EntityBody {
    pub radius_m: f32,
    pub height_m: f32,
    pub eye_height_m: f32,
}

impl Default for EntityBody {
    fn default() -> Self {
        EntityBody {
            radius_m: 0.3,
            height_m: 1.7,
            eye_height_m: 1.55,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiSpec {
    pub behavior: AiBehavior,
    pub sight_range: f32,
    #[serde(default)]
    pub flee_range: f32,
    #[serde(default)]
    pub prey_tags: Vec<TagRef>,
    #[serde(default)]
    pub fear_tags: Vec<TagRef>,
    pub wander_radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiBehavior {
    Passive,
    Neutral,
    Hostile,
    /// Flees from entities with matching `fear_tags`.
    Skittish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MovementSpec {
    pub walk_speed: f32,
    pub swim_speed: f32,
    #[serde(default)]
    pub fly_speed: Option<f32>,
}

impl Default for MovementSpec {
    fn default() -> Self {
        MovementSpec {
            walk_speed: 1.0,
            swim_speed: 0.5,
            fly_speed: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum EntityInteraction {
    Ride,
    Trade,
    Custom { script: ScriptRef },
}
