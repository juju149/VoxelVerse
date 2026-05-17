//! Format version constants for content packs.
//!
//! Each kind of definition embeds a `format_version: u32` so the compiler can
//! reject content authored against an older or future schema. There is no
//! migration layer yet — content must match the current version exactly.

pub const PACK_FORMAT_VERSION: u32 = 1;
pub const OBJECT_FORMAT_VERSION: u32 = 1;
pub const BLOCK_FORMAT_VERSION: u32 = 1;
pub const BLOCK_MODEL_FORMAT_VERSION: u32 = 1;
pub const MATERIAL_FORMAT_VERSION: u32 = 1;
pub const ITEM_FORMAT_VERSION: u32 = 1;
pub const ENTITY_FORMAT_VERSION: u32 = 1;
pub const LOOT_FORMAT_VERSION: u32 = 1;
pub const SKELETON_FORMAT_VERSION: u32 = 1;
pub const SOUND_EVENT_FORMAT_VERSION: u32 = 1;
pub const TAG_FORMAT_VERSION: u32 = 1;
pub const RECIPE_FORMAT_VERSION: u32 = 1;
pub const WORLDGEN_FORMAT_VERSION: u32 = 1;
pub const WEATHER_FORMAT_VERSION: u32 = 1;
pub const BIOME_AMBIENCE_FORMAT_VERSION: u32 = 1;
pub const CELESTIAL_FORMAT_VERSION: u32 = 1;
pub const STAR_CATALOG_FORMAT_VERSION: u32 = 1;
pub const RENDER_FEATURE_FORMAT_VERSION: u32 = 1;
pub const RENDER_PROFILE_FORMAT_VERSION: u32 = 1;

/// Checks a `format_version` field against the expected constant.
/// Returns a structured error string if the version mismatches.
pub fn check_format_version(
    actual: u32,
    expected: u32,
    kind: &str,
    key: &str,
) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{kind} '{key}': format_version {actual} unsupported (expected {expected})"
        ))
    }
}
