use crate::ContentRef;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RawItemDef {
    pub display_name: String,
    pub category: String,
    pub stack_size: u32,
    pub visual: RawItemVisualDef,
    pub gameplay: RawItemGameplayDef,
    #[serde(default)]
    pub tags: Vec<ContentRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawItemVisualDef {
    pub inventory_icon: ContentRef,
    pub world_model: RawItemWorldModel,
    #[serde(default)]
    pub hand_model: Option<ContentRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum RawItemWorldModel {
    None,
    BlockItem(ContentRef),
    Voxel(ContentRef),
}

#[derive(Debug, Clone, Deserialize)]
pub enum RawItemGameplayDef {
    PlaceBlock(ContentRef),
    CraftingIngredient(RawCraftingIngredientDef),
    Tool(RawToolDef),
    Weapon(RawWeaponDef),
    Food(RawFoodDef),
    Consumable(RawConsumableDef),
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawCraftingIngredientDef {
    #[serde(default)]
    pub fuel_value: Option<u32>,
    #[serde(default)]
    pub smelts_to: Option<ContentRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawToolDef {
    pub tool_tags: Vec<ContentRef>,
    pub tier: u32,
    pub mining_speed: f32,
    pub durability: u32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawWeaponClass {
    Bow,
    Sword,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawWeaponDef {
    pub class: RawWeaponClass,
    pub damage: f32,
    pub attack_speed: f32,
    pub durability: u32,
    #[serde(default)]
    pub projectile: Option<ContentRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawFoodDef {
    pub nutrition: u32,
    pub saturation: f32,
    pub eat_seconds: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawConsumableDef {
    pub effect: ContentRef,
    pub magnitude: f32,
    pub use_seconds: f32,
}
