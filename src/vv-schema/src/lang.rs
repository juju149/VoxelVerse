use crate::common::LangKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Localization file. Locale is derived from the filename (en_us.ron → "en_us").
/// Deserialized from lang/<locale>.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LangFileDef {
    /// Locale override if different from the filename.
    #[serde(default)]
    pub locale: Option<String>,
    pub entries: HashMap<LangKey, String>,
}
