//! Tolerant RON parsing with location-preserving diagnostics.
//!
//! Every `.ron` file in a pack is parsed individually. A failure produces a
//! `ParseError` carrying the offending file's pack-relative path, the line and
//! column extracted from the underlying error, the raw message and a short
//! suggestion when the pattern is recognisable.

use std::path::Path;

use serde::de::DeserializeOwned;

/// A single parse failure attached to a pack-relative path.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub rel_path: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Read a file and try to deserialize it. The outer type-name (e.g. `Object(`)
/// is stripped before retrying so authors can keep the friendly wrapper form
/// the rest of the engine accepts.
pub fn read_typed<T: DeserializeOwned>(
    pack_root: &Path,
    abs_path: &Path,
) -> Result<T, ParseError> {
    let rel = pack_relative(pack_root, abs_path);
    let text = std::fs::read_to_string(abs_path).map_err(|e| ParseError {
        rel_path: rel.clone(),
        line: 0,
        column: 0,
        message: format!("cannot read file: {e}"),
        suggestion: None,
    })?;
    parse_string::<T>(rel, &text)
}

/// Parse pre-loaded text. Used by the scanner when it already had to read the
/// file (for example to compute its classification).
pub fn parse_string<T: DeserializeOwned>(rel_path: String, text: &str) -> Result<T, ParseError> {
    let opts =
        ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
    match opts.from_str::<T>(text) {
        Ok(v) => Ok(v),
        Err(first) => {
            let stripped = strip_outer_type_name(text);
            match opts.from_str::<T>(stripped) {
                Ok(v) => Ok(v),
                Err(second) if stripped.len() != text.len() => {
                    // The wrapper was stripped but a deeper error remains.
                    // Report the deeper one — it's the actionable issue.
                    Err(translate(rel_path, second))
                }
                Err(_) => Err(translate(rel_path, first)),
            }
        }
    }
}

/// Parse to an untyped `ron::Value` to confirm the file is at least valid RON.
pub fn parse_value(rel_path: String, text: &str) -> Result<ron::Value, ParseError> {
    parse_string::<ron::Value>(rel_path, text)
}

fn translate(rel_path: String, err: ron::error::SpannedError) -> ParseError {
    let pos = err.position;
    let message = err.code.to_string();
    let suggestion = suggest(&message);
    ParseError {
        rel_path,
        line: pos.line as u32,
        column: pos.col as u32,
        message,
        suggestion,
    }
}

fn suggest(message: &str) -> Option<String> {
    let lower = message.to_ascii_lowercase();
    if lower.contains("expected string") {
        return Some(
            "wrap bare identifiers in double quotes (e.g. `\"plains\"` instead of `plains`)"
                .to_string(),
        );
    }
    if lower.contains("expected identifier") {
        return Some(
            "this position expects a struct field, enum variant or type name — not a quoted string"
                .to_string(),
        );
    }
    if lower.contains("unknown field") {
        return Some(
            "remove or rename the unknown field; check the matching schema in vv-content-schema"
                .to_string(),
        );
    }
    if lower.contains("missing field") {
        return Some(
            "the schema requires this field — add it or remove the surrounding section"
                .to_string(),
        );
    }
    if lower.contains("trailing characters") {
        return Some(
            "remove text after the closing parenthesis of the top-level record".to_string(),
        );
    }
    None
}

/// `Object(...)` → `(...)`. Mirrors the behaviour of `vv-pack-loader` so the
/// doctor accepts whatever the loader accepts. Leading line comments and
/// whitespace are skipped before the outer identifier is detected.
pub fn strip_outer_type_name(text: &str) -> &str {
    let mut start = 0;
    let bytes = text.as_bytes();
    while start < bytes.len() {
        let c = bytes[start];
        if c == b' ' || c == b'\t' || c == b'\r' || c == b'\n' {
            start += 1;
            continue;
        }
        if c == b'/' && start + 1 < bytes.len() && bytes[start + 1] == b'/' {
            // line comment
            while start < bytes.len() && bytes[start] != b'\n' {
                start += 1;
            }
            continue;
        }
        if c == 0xEF && start + 2 < bytes.len() && bytes[start + 1] == 0xBB && bytes[start + 2] == 0xBF {
            start += 3;
            continue;
        }
        break;
    }
    // Now `start` points at the first significant character. Look for an
    // identifier followed by `(`.
    let mut end = start;
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }
    if end > start && end < bytes.len() && bytes[end] == b'(' {
        &text[end..]
    } else {
        text
    }
}

pub fn pack_relative(pack_root: &Path, abs_path: &Path) -> String {
    abs_path
        .strip_prefix(pack_root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| abs_path.to_string_lossy().replace('\\', "/"))
}
