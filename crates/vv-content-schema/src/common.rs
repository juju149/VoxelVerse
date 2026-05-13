//! Common types shared by every other schema module.
//!
//! Kept deliberately small: anything specific to a domain (textures, blocks,
//! shaders, etc.) lives next to that domain. This module only carries pieces
//! that have **no** sensible home elsewhere.

use serde::{de, Deserialize, Deserializer};

/// Logical pack reference in `namespace:domain/path` form.
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct ContentRef(pub String);

impl<'de> Deserialize<'de> for ContentRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        validate_content_ref(&value).map_err(de::Error::custom)?;
        Ok(Self(value))
    }
}

fn validate_content_ref(value: &str) -> Result<(), String> {
    let Some((namespace, path)) = value.split_once(':') else {
        return Err(format!("ref '{}' must use namespace:path form", value));
    };
    if namespace.is_empty() || path.is_empty() {
        return Err(format!("ref '{}' has empty namespace or path", value));
    }
    if path.contains("..") || path.starts_with('/') || path.starts_with('\\') {
        return Err(format!("ref '{}' must stay inside its pack", value));
    }
    Ok(())
}
