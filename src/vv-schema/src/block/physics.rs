use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockPhysicsDef {
    pub phase: MaterialPhase,
    pub density: f32,
    pub collider: ColliderShape,

    #[serde(default)]
    pub selection: Option<SelectionBox>,

    pub friction: f32,
    pub drag: f32,
}

impl Default for BlockPhysicsDef {
    fn default() -> Self {
        Self {
            phase: MaterialPhase::Solid,
            density: 1.5,
            collider: ColliderShape::Full,
            selection: None,
            friction: 1.0,
            drag: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialPhase {
    Solid,
    Liquid,
    Passable,
}

impl Default for MaterialPhase {
    fn default() -> Self {
        Self::Solid
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColliderShape {
    Full,
    None,
    Aabb { min: [f32; 3], max: [f32; 3] },
}

impl Default for ColliderShape {
    fn default() -> Self {
        Self::Full
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SelectionBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}
