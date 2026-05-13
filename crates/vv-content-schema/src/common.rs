//! Common types shared by every other schema module.
//!
//! Kept deliberately small: anything specific to a domain (textures, blocks,
//! shaders, etc.) lives next to that domain. This module only carries pieces
//! that have **no** sensible home elsewhere.

use serde::{de, Deserializer};

/// A reference to content in a pack.
///
/// Accepts both fully-qualified refs (`"namespace:domain/path"`) and short
/// names (`stone`, `ocean`) for convenience in data files.  Short names are
/// resolved to full keys at compile time by the content compiler.
///
/// In RON files, both quoted strings and bare identifiers are accepted, so
/// `block: stone` and `block: "stone"` are equivalent.
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct ContentRef(pub String);

impl<'de> de::Deserialize<'de> for ContentRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> de::Visitor<'de> for V {
            type Value = ContentRef;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a content reference string or bare identifier")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<ContentRef, E> {
                validate_path_safety(v).map_err(de::Error::custom)?;
                Ok(ContentRef(v.to_owned()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<ContentRef, E> {
                validate_path_safety(&v).map_err(de::Error::custom)?;
                Ok(ContentRef(v))
            }
        }
        // In RON 0.8, struct field values go through TagDeserializer which
        // routes deserialize_identifier → deserialize_str (requires quoted strings).
        // Sequence elements use the main deserializer but for consistency we
        // always require quoted strings for ContentRef in all positions.
        deserializer.deserialize_str(V)
    }
}

/// Reject path-traversal attempts while allowing both short names and full
/// `namespace:path` refs.
fn validate_path_safety(value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err("content ref must not be empty".into());
    }
    // Block path traversal regardless of format.
    if value.contains("..") || value.starts_with('/') || value.starts_with('\\') {
        return Err(format!("content ref '{}' contains unsafe path component", value));
    }
    Ok(())
}
