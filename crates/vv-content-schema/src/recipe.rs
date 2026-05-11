//! Raw (pre-compilation) recipe definitions.
//!
//! Three recipe kinds are supported:
//!   - `Shaped`    — ingredients arranged in a ≤3×3 grid pattern.
//!   - `Shapeless` — an unordered bag of ingredients; any grid arrangement works.
//!   - `Smelting`  — one ingredient converted to one output with a fuel cost.
//!
//! Ingredients may be a direct item reference or a tag reference (matches any
//! item carrying that tag).  Tags are resolved by the compiler against the
//! compiled `ItemRegistry` and `TagRegistry`.

use crate::ContentRef;
use serde::Deserialize;
use std::collections::HashMap;

/// Top-level recipe file type.
///
/// File suffix: `.recipe.ron`
/// Discovered under: `defs/recipes/**/*.recipe.ron`
/// Content key:  `<namespace>:recipe/<sub-path>` (path-as-identity).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "RecipeDef")]
pub struct RawRecipeDef {
    pub format_version: u32,
    /// The item produced and how many copies per craft.
    pub result: RawRecipeOutput,
    /// Which kind of crafting process produces this item.
    pub recipe: RawRecipeKind,
    /// Arbitrary station tag required to craft (e.g. `"core:tag/station/crafting_table"`).
    /// `None` means the recipe is available in the 2×2 personal grid.
    #[serde(default)]
    pub station: Option<ContentRef>,
    /// Optional human-readable group name for UI clustering (e.g. `"tools"`).
    #[serde(default)]
    pub group: Option<String>,
}

/// The item output of a recipe.
#[derive(Debug, Clone, Deserialize)]
pub struct RawRecipeOutput {
    /// Namespaced item key (`namespace:item/path`).
    pub item: ContentRef,
    /// Stack size produced per successful craft. Defaults to 1.
    #[serde(default = "one")]
    pub count: u32,
}

fn one() -> u32 {
    1
}

/// Discriminated union over the three recipe kinds.
#[derive(Debug, Clone, Deserialize)]
pub enum RawRecipeKind {
    /// Positional 3×3 grid recipe.
    Shaped(RawShapedRecipe),
    /// Unordered ingredient set; grid size is irrelevant.
    Shapeless(RawShapelessRecipe),
    /// Single-ingredient furnace / smelter recipe.
    Smelting(RawSmeltingRecipe),
}

/// A shaped recipe encodes its layout as a `pattern` (up to 3 rows, each a
/// `String` of ≤3 characters) plus a `keys` map from single-character symbols
/// to `RawIngredient`s.  Spaces in the pattern are treated as empty slots.
///
/// Example pattern for a pickaxe:
/// ```ron
/// pattern: ["MMM", " S ", " S "],
/// keys: { "M": Item("core:item/resource/stone"), "S": Item("core:item/resource/stick") },
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct RawShapedRecipe {
    /// Up to 3 rows.  Each row contains up to 3 characters.
    /// `' '` (space) means an empty slot.
    pub pattern: Vec<String>,
    /// One entry per unique symbol used in `pattern`.
    pub keys: HashMap<String, RawIngredient>,
}

/// A shapeless recipe requires exactly the listed ingredients (with
/// repetition), in any arrangement in the crafting grid.
#[derive(Debug, Clone, Deserialize)]
pub struct RawShapelessRecipe {
    /// All ingredients required.  Repetition encodes quantity (e.g. two sticks
    /// → list `"core:item/resource/stick"` twice).
    pub ingredients: Vec<RawIngredient>,
}

/// A smelting recipe converts one stack of `ingredient` into `result` after
/// spending `fuel` units of fuel and waiting `smelt_seconds`.
#[derive(Debug, Clone, Deserialize)]
pub struct RawSmeltingRecipe {
    pub ingredient: RawIngredient,
    /// Fuel units consumed from the fuel slot per smelt operation.
    pub fuel: u32,
    /// How many in-game seconds the smelting takes.
    pub smelt_seconds: f32,
}

/// An ingredient slot: either a direct item key or a tag that any
/// matching item satisfies.
#[derive(Debug, Clone, Deserialize)]
pub enum RawIngredient {
    /// Exact item key match.
    Item(ContentRef),
    /// Any item that carries this tag qualifies.
    Tag(ContentRef),
}
