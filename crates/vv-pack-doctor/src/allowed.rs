//! Allowed-unused exception loader.
//!
//! Reads `source/production/allowed_unused.ron` if it exists. Content listed
//! here is intentionally orphaned and Pack Doctor skips "unused" warnings for
//! it. The file is the single legitimate way to silence such warnings.

use std::collections::HashSet;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(rename = "AllowedUnused")]
pub struct RawAllowedUnused {
    #[serde(default)]
    pub blocks: Vec<String>,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub materials: Vec<String>,
    #[serde(default)]
    pub textures: Vec<String>,
    #[serde(default)]
    pub loot_tables: Vec<String>,
    #[serde(default)]
    pub notes: Vec<AllowedNote>,
}

#[derive(Debug, Deserialize)]
pub struct AllowedNote {
    pub id: String,
    #[serde(default)]
    pub why: String,
}

#[derive(Debug, Default)]
pub struct AllowedUnused {
    pub blocks: HashSet<String>,
    pub items: HashSet<String>,
    pub materials: HashSet<String>,
    pub textures: HashSet<String>,
    pub loot_tables: HashSet<String>,
}

impl AllowedUnused {
    pub fn load(pack_root: &Path) -> Result<Self, String> {
        let path = pack_root
            .join("source")
            .join("production")
            .join("allowed_unused.ron");
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
        let raw: RawAllowedUnused = ron::from_str(&text)
            .or_else(|_| ron::from_str(strip_outer_type_name(&text)))
            .map_err(|e| format!("Parse error in {}:\n  {}", path.display(), e))?;
        let mut blocks: HashSet<String> = raw.blocks.into_iter().collect();
        let mut items: HashSet<String> = raw.items.into_iter().collect();
        let mut materials: HashSet<String> = raw.materials.into_iter().collect();
        let mut textures: HashSet<String> = raw.textures.into_iter().collect();
        let mut loot_tables: HashSet<String> = raw.loot_tables.into_iter().collect();
        for note in raw.notes {
            let id = note.id;
            if id.starts_with("core:block/") {
                blocks.insert(id);
            } else if id.starts_with("core:item/") {
                items.insert(id);
            } else if id.starts_with("core:material/") {
                materials.insert(id);
            } else if id.starts_with("core:texture/") {
                textures.insert(id);
            } else if id.starts_with("core:loot/") {
                loot_tables.insert(id);
            } else {
                // Treat unknown ID as a texture path (for raw filesystem refs)
                textures.insert(id);
            }
        }
        Ok(Self {
            blocks,
            items,
            materials,
            textures,
            loot_tables,
        })
    }
}

fn strip_outer_type_name(text: &str) -> &str {
    let trimmed = text.trim_start_matches('\u{feff}').trim_start();
    let Some(open) = trimmed.find('(') else {
        return text;
    };
    if trimmed[..open]
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        &trimmed[open..]
    } else {
        text
    }
}
