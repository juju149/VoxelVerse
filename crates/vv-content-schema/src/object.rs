/// Raw schema for the unified `.object.ron` format.
///
/// One file = one gameplay concept.  Each section is optional; the compiler
/// decides which registries to populate based on what sections are present.
///
/// All content references are plain `String` — the compiler resolves short
/// names (`"stone"`, `"oak_log"`) to their namespaced runtime keys.
use serde::Deserialize;
use std::collections::HashMap;

// ── Top-level ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RawObjectDef {
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
    #[serde(default)]
    pub spawn: Option<RawObjectSpawnSection>,
    #[serde(default)]
    pub recipe: Option<RawObjectRecipeSection>,
}

// ── Block section ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
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
}

fn default_true() -> bool {
    true
}

/// Block texture descriptor. Variant name is lowercase in RON
/// (e.g. `texture: all("blocks/stone/all")`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectTexture {
    /// No texture (air-like blocks).
    None,
    /// Same texture on all six faces.
    All(String),
    /// Independent top, four sides, bottom.
    Cube {
        top: String,
        side: String,
        bottom: String,
    },
    /// Axial column: distinct end-caps and lateral band.
    Column { top: String, side: String },
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
    /// Vertical column (like logs). Distinct end-cap and side textures.
    Column,
    /// Cross-shaped (grass tuft, flowers).
    Cross,
}

// ── Item section ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawObjectItem {
    #[serde(default = "default_stack_99")]
    pub stack: u32,
    /// Pack-relative path to the inventory icon (no namespace prefix).
    #[serde(default)]
    pub icon: Option<String>,
    /// Pack-relative path to the world / hand model.
    #[serde(default)]
    pub model: Option<String>,
}

fn default_stack_99() -> u32 {
    99
}

// ── Mining section ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RawObjectMining {
    pub tool: RawObjectToolKind,
    #[serde(default)]
    pub tier: u32,
    /// `None`  → drops the item form of this block (implicit self-drop).
    /// `Some([])` → drops nothing.
    /// `Some([...])` → explicit drop entries.
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
pub struct RawObjectDropEntry {
    /// Short item name resolved by the compiler.
    pub item: String,
    #[serde(default = "default_count_one")]
    pub count: RawObjectCount,
    #[serde(default = "default_f_one")]
    pub chance: f32,
}

/// Item count: either a fixed number or a `(min, max)` range.
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

// ── Tool section ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RawObjectToolSection {
    #[serde(rename = "type")]
    pub tool_type: RawObjectToolKind,
    pub tier: u32,
    pub speed: f32,
    pub durability: u32,
}

// ── Weapon section ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
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

// ── Food section ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RawObjectFoodSection {
    pub hunger: u32,
    pub saturation: f32,
    #[serde(default = "default_eat_time")]
    pub eat_time: f32,
}

fn default_eat_time() -> f32 {
    1.6
}

// ── Effect section ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct RawObjectStationSection {
    #[serde(rename = "type")]
    pub station_type: RawObjectStationType,
    /// Number of processing / storage slots in the UI.
    #[serde(default)]
    pub slots: Option<u32>,
    /// Whether this station has a dedicated fuel slot.
    #[serde(default)]
    pub fuel_slot: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectStationType {
    Crafting,
    Smelting,
    CampfireCooking,
}

// ── Storage section ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RawObjectStorageSection {
    pub slots: u32,
    #[serde(default)]
    pub persistent: bool,
}

// ── Light section ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RawObjectLightSection {
    pub level: u32,
    #[serde(default)]
    pub flicker: bool,
    /// RGB colour components, each 0.0–1.0.
    #[serde(default)]
    pub colour: Option<(f32, f32, f32)>,
}

// ── Fuel section ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RawObjectFuelSection {
    /// Fuel ticks provided when burned (800 ≈ one coal in Minecraft).
    pub duration: u32,
}

// ── Entity section ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RawObjectEntitySection {
    #[serde(default)]
    pub model: Option<String>,
    pub ai: RawObjectAiKind,
    pub health: u32,
    #[serde(default)]
    pub move_speed: f32,
    #[serde(default)]
    pub jump: f32,
    /// Attack / interaction reach in voxels (player only).
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
pub struct RawObjectLootSection {
    #[serde(default)]
    pub when_killed: Vec<RawObjectDropEntry>,
}

// ── Spawn section ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RawObjectSpawnSection {
    pub density: f32,
    pub group: RawObjectSpawnGroup,
    #[serde(default)]
    pub biomes: Vec<String>,
    #[serde(default)]
    pub surface: Option<String>,
    #[serde(default)]
    pub light: RawObjectSpawnLight,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawObjectSpawnGroup {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RawObjectSpawnLight {
    #[default]
    Any,
    Daylight,
    Night,
}

// ── Recipe section ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RawObjectRecipeSection {
    /// Station tag required (e.g. `"#station.construction"`).
    /// `None` = available in the personal 2×2 grid.
    pub station: String,
    /// Grid pattern rows for shaped recipes.
    #[serde(default)]
    pub shaped: Option<Vec<String>>,
    /// Symbol → item-name mapping for shaped recipes.
    #[serde(default)]
    pub legend: Option<HashMap<String, String>>,
    /// Unordered ingredient list for shapeless recipes.
    #[serde(default)]
    pub shapeless: Option<Vec<String>>,
    /// Single-ingredient smelting recipes.
    #[serde(default)]
    pub inputs: Option<Vec<RawObjectDropEntry>>,
    /// Output of this recipe. Short item name resolved by the compiler.
    pub output: RawObjectRecipeOutput,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawObjectRecipeOutput {
    pub item: String,
    #[serde(default = "default_one_u32")]
    pub count: u32,
}

fn default_one_u32() -> u32 {
    1
}
