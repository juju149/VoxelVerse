//! Raw schema for the unified `.object.ron` format.
//!
//! One file = one gameplay concept. Each section is optional; the compiler
//! decides which registries to populate based on what sections are present.
//!
//! Block-state property declarations live here too — they belong to a block
//! and have no other home. When `object.rs` grows past ~800 lines we'll split
//! it into `object/{block,item,recipe,station,entity}.rs`.

use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

// ── Top-level ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectDef {
    #[serde(default = "default_format_version")]
    pub format_version: u32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Short tag names carried by this object (e.g. `["terrain", "soil"]`).
    #[serde(default)]
    pub tags: Vec<String>,

    // Gameplay sections — all optional.
    #[serde(default)]
    pub block: Option<RawObjectBlock>,
    #[serde(default)]
    pub item: Option<RawObjectItem>,
    #[serde(default)]
    pub mining: Option<RawObjectMining>,
    #[serde(default)]
    pub tool: Option<RawObjectToolSection>,
    #[serde(default)]
    pub weapon: Option<RawObjectWeaponSection>,
    #[serde(default)]
    pub food: Option<RawObjectFoodSection>,
    #[serde(default)]
    pub effect: Option<RawObjectEffectSection>,
    #[serde(default)]
    pub station: Option<RawObjectStationSection>,
    #[serde(default)]
    pub storage: Option<RawObjectStorageSection>,
    #[serde(default)]
    pub light: Option<RawObjectLightSection>,
    #[serde(default)]
    pub fuel: Option<RawObjectFuelSection>,
    #[serde(default)]
    pub entity: Option<RawObjectEntitySection>,
    #[serde(default)]
    pub loot: Option<RawObjectLootSection>,

    /// Zero or more recipes that yield this object. Multiple entries make
    /// alternate-ingredient recipes representable (e.g. torch from coal *or*
    /// resin *or* animal fat) without forcing authors to invent fake objects.
    #[serde(default)]
    pub recipes: Vec<RawObjectRecipeSection>,
}

fn default_format_version() -> u32 {
    crate::OBJECT_FORMAT_VERSION
}

// ── Block section ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectBlock {
    pub texture: RawObjectTexture,
    #[serde(default)]
    pub render: RawObjectRenderMode,
    #[serde(default = "default_true")]
    pub solid: bool,
    #[serde(default)]
    pub replaceable: bool,
    /// Negative value means unbreakable.
    #[serde(default)]
    pub hardness: f32,
    #[serde(default)]
    pub sound: RawObjectSoundKind,
    #[serde(default)]
    pub tint: Option<RawObjectTint>,
    #[serde(default)]
    pub shape: RawObjectShape,
    #[serde(default)]
    pub mesh_class: Option<RawObjectMeshClass>,
    #[serde(default)]
    pub states: Option<RawBlockStates>,
}

fn default_true() -> bool {
    true
}

/// Block texture descriptor. Variant name is lowercase in RON
/// (e.g. `texture: all("blocks/stone/all")`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectTexture {
    None,
    All(String),
    Cube {
        top: String,
        side: String,
        bottom: String,
    },
    Column {
        top: String,
        side: String,
    },
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectRenderMode {
    #[default]
    Opaque,
    Invisible,
    Translucent,
    Cutout,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectMeshClass {
    OpaqueCube,
    Cutout,
    Prop,
    Water,
    Foliage,
    Emissive,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectSoundKind {
    #[default]
    None,
    Grass,
    Stone,
    Wood,
    Sand,
    Snow,
    Dirt,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectTint {
    Grass,
    Foliage,
    Water,
}

/// Block shape hint used by the mesher. Default = full cube.
#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectShape {
    #[default]
    Cube,
    Column,
    Cross,
}

// ── Block-state declarations ─────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RawBlockStates {
    #[serde(default)]
    pub properties: BTreeMap<String, RawBlockStateProperty>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawBlockStateProperty {
    Axis {
        default: String,
    },
    FacingHorizontal {
        default: String,
    },
    Facing {
        default: String,
    },
    Half {
        default: String,
    },
    StairShape {
        default: String,
    },
    Bool {
        default: bool,
    },
    Enum {
        values: Vec<String>,
        default: String,
    },
}

impl RawBlockStateProperty {
    pub fn allowed_values(&self) -> Vec<&str> {
        match self {
            Self::Axis { .. } => vec!["x", "y", "z"],
            Self::FacingHorizontal { .. } => vec!["north", "south", "east", "west"],
            Self::Facing { .. } => vec!["north", "south", "east", "west", "up", "down"],
            Self::Half { .. } => vec!["top", "bottom"],
            Self::StairShape { .. } => vec![
                "straight",
                "inner_left",
                "inner_right",
                "outer_left",
                "outer_right",
            ],
            Self::Bool { .. } => vec!["false", "true"],
            Self::Enum { values, .. } => values.iter().map(String::as_str).collect(),
        }
    }

    pub fn kind_tag(&self) -> &'static str {
        match self {
            Self::Axis { .. } => "axis",
            Self::FacingHorizontal { .. } => "facing_horizontal",
            Self::Facing { .. } => "facing",
            Self::Half { .. } => "half",
            Self::StairShape { .. } => "stair_shape",
            Self::Bool { .. } => "bool",
            Self::Enum { .. } => "enum",
        }
    }

    pub fn validate_into(&self, name: &str, ctx: &str, errors: &mut Vec<String>) {
        match self {
            Self::Axis { default }
            | Self::FacingHorizontal { default }
            | Self::Facing { default }
            | Self::Half { default }
            | Self::StairShape { default } => {
                let allowed = self.allowed_values();
                if !allowed.contains(&default.as_str()) {
                    errors.push(format!(
                        "{ctx}: state '{name}' ({}) default '{}' not in [{}]",
                        self.kind_tag(),
                        default,
                        allowed.join(", ")
                    ));
                }
            }
            Self::Bool { .. } => {}
            Self::Enum { values, default } => {
                if values.is_empty() {
                    errors.push(format!("{ctx}: state '{name}' (enum) has empty `values`"));
                    return;
                }
                let mut seen = std::collections::HashSet::new();
                for v in values {
                    if !seen.insert(v.as_str()) {
                        errors.push(format!(
                            "{ctx}: state '{name}' (enum) duplicate value '{}'",
                            v
                        ));
                    }
                }
                if !values.iter().any(|v| v == default) {
                    errors.push(format!(
                        "{ctx}: state '{name}' (enum) default '{}' not in declared values [{}]",
                        default,
                        values.join(", ")
                    ));
                }
            }
        }
    }
}

impl RawBlockStates {
    pub fn validate_into(&self, ctx: &str, errors: &mut Vec<String>) {
        for (name, prop) in &self.properties {
            prop.validate_into(name, ctx, errors);
        }
    }
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }
    pub fn len(&self) -> usize {
        self.properties.len()
    }
}

// ── Item section ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectItem {
    #[serde(default = "default_stack_99")]
    pub stack: u32,
    pub category: RawObjectItemCategory,
    pub visible_in_inventory: bool,
    #[serde(default)]
    pub inventory_icon: Option<RawObjectInventoryIcon>,
    #[serde(default)]
    pub world_model: Option<String>,
}

fn default_stack_99() -> u32 {
    99
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectItemCategory {
    Block,
    Resource,
    Food,
    Tool,
    Weapon,
    Armor,
    Station,
    Utility,
    Material,
    Quest,
    Debug,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectInventoryIcon {
    Texture(String),
    Block,
    VoxelModel(String),
    AutoGenerated,
}

// ── Mining section ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectMining {
    pub tool: RawObjectToolKind,
    #[serde(default)]
    pub tier: u32,
    #[serde(default)]
    pub drops: Option<Vec<RawObjectDropEntry>>,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectToolKind {
    #[default]
    Any,
    Pickaxe,
    Shovel,
    Axe,
    Shears,
    Hoe,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectDropEntry {
    pub item: String,
    #[serde(default = "default_count_one")]
    pub count: RawObjectCount,
    #[serde(default = "default_f_one")]
    pub chance: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RawObjectCount {
    Fixed(u32),
    Range(u32, u32),
}

impl Default for RawObjectCount {
    fn default() -> Self {
        Self::Fixed(1)
    }
}

fn default_count_one() -> RawObjectCount {
    RawObjectCount::Fixed(1)
}

fn default_f_one() -> f32 {
    1.0
}

// ── Tool / Weapon / Food / Effect sections ────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectToolSection {
    #[serde(rename = "type")]
    pub tool_type: RawObjectToolKind,
    pub tier: u32,
    pub speed: f32,
    pub durability: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectWeaponSection {
    #[serde(rename = "type")]
    pub weapon_type: RawObjectWeaponKind,
    pub damage: f32,
    #[serde(default = "default_f_one")]
    pub draw_time: f32,
    pub durability: u32,
    #[serde(default)]
    pub ammo: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectWeaponKind {
    Bow,
    Sword,
    Axe,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectFoodSection {
    pub hunger: u32,
    pub saturation: f32,
    #[serde(default = "default_eat_time")]
    pub eat_time: f32,
}

fn default_eat_time() -> f32 {
    1.6
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectEffectSection {
    pub on_use: RawObjectEffectKind,
    #[serde(default)]
    pub heal: f32,
    #[serde(default = "default_f_one")]
    pub use_time: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectEffectKind {
    InstantHeal,
    InstantDamage,
    Poison,
    Speed,
}

// ── Station section ───────────────────────────────────────────────────────────
//
// VoxelVerse stations are broader than a Minecraft-style enum: a station is
// either a *workbench* (you craft on it), a *processor* (it transforms inputs
// over time, like a furnace), or a *storage* (chest-likes). The actual recipe
// routing is data-driven via `station_tags` — recipes name the tags they
// require so authors can introduce new station kinds without growing the enum.

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectStationSection {
    #[serde(rename = "type")]
    pub station_type: RawObjectStationType,
    /// Tags that recipes can target via `#station.<name>`. At least one entry
    /// expected if the station hosts any recipe.
    #[serde(default)]
    pub station_tags: Vec<String>,
    /// Number of processing / storage slots in the UI.
    #[serde(default)]
    pub slots: Option<u32>,
    /// Whether this station has a dedicated fuel slot.
    #[serde(default)]
    pub fuel_slot: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectStationType {
    /// Player-driven crafting surface (construction bench, weapon bench, …).
    Workbench,
    /// Time-driven transformation (furnace, campfire, anvil, …).
    Processor,
    /// Chest-likes — no recipe, only inventory.
    Storage,
}

// ── Storage / Light / Fuel sections ───────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectStorageSection {
    pub slots: u32,
    #[serde(default)]
    pub persistent: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectLightSection {
    pub level: u32,
    #[serde(default)]
    pub flicker: bool,
    #[serde(default)]
    pub colour: Option<(f32, f32, f32)>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectFuelSection {
    pub duration: u32,
}

// ── Entity section ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectEntitySection {
    #[serde(default)]
    pub model: Option<String>,
    /// Skeleton reference (path-as-id under `defs/skeletons/`). Optional so
    /// static props can keep using a single model with no rig.
    #[serde(default)]
    pub skeleton: Option<String>,
    pub ai: RawObjectAiKind,
    pub health: u32,
    #[serde(default)]
    pub move_speed: f32,
    #[serde(default)]
    pub jump: f32,
    #[serde(default)]
    pub reach: f32,
    #[serde(default)]
    pub gravity: Option<RawObjectGravityKind>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectAiKind {
    PlayerControlled,
    PassiveWanderer,
    PassiveGrazer,
    AggressiveMelee,
    AggressiveRanged,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectGravityKind {
    PlanetSurface,
    Standard,
    None,
}

// ── Loot section ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RawObjectLootSection {
    #[serde(default)]
    pub when_killed: Vec<RawObjectDropEntry>,
}

// ── Recipe section ────────────────────────────────────────────────────────────

/// One recipe that yields the enclosing object.
///
/// A recipe is exactly one of (shaped, shapeless, processing) — never a mix.
/// The `kind` enum enforces that at parse time so an invalid `.ron` cannot
/// silently produce ambiguous content.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectRecipeSection {
    /// Station tag required (e.g. `"#station.construction"`).
    /// `None` = available in the personal 2×2 grid.
    #[serde(default)]
    pub station: Option<String>,
    /// Ingredients and how they're laid out.
    pub kind: RawObjectRecipeKind,
    /// Output of this recipe. Short item name resolved by the compiler.
    pub output: RawObjectRecipeOutput,
    /// Optional grouping tag for the recipe book UI.
    #[serde(default)]
    pub group: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectRecipeKind {
    Shaped(RawShapedRecipe),
    Shapeless(RawShapelessRecipe),
    /// Time-driven transformations (smelting, cooking, …).
    Processing(RawProcessingRecipe),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawShapedRecipe {
    /// Grid pattern rows, e.g. `["SSS", " F ", " F "]`.
    pub pattern: Vec<String>,
    /// Symbol → item-name mapping.
    pub legend: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawShapelessRecipe {
    pub ingredients: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProcessingRecipe {
    pub inputs: Vec<RawObjectDropEntry>,
    #[serde(default = "default_processing_time")]
    pub duration_seconds: f32,
}

fn default_processing_time() -> f32 {
    10.0
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawObjectRecipeOutput {
    pub item: String,
    #[serde(default = "default_one_u32")]
    pub count: u32,
}

fn default_one_u32() -> u32 {
    1
}
