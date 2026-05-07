// Visual schema types are ready for the texture-atlas pipeline but not yet
// consumed by the content compiler.  Dead-code lint suppressed intentionally.
#![allow(dead_code)]

use serde::Deserialize;

/// A logical reference to a texture resource.
/// At this stage the atlas can render it as a debug color tile.
/// Future: resolved against the pack's texture atlas.
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceRef {
    /// Namespaced path, e.g. `"core:blocks/grass_top"`.
    pub path: String,
}

/// Raw visual data for a block as written in pack files.
/// Kept separate from gameplay data so visual systems never depend on hardcoded identifiers.
#[derive(Debug, Clone, Deserialize)]
pub struct RawBlockVisual {
    pub kind: RawBlockVisualKind,
    /// Optional RGB tint multiplied over the texture or color. `[1,1,1]` = no tint.
    #[serde(default = "default_tint")]
    pub tint: [f32; 3],
}

fn default_tint() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

/// How the block looks.
///
/// Variants are ordered from simplest to most complex.  The renderer can
/// handle each case explicitly and fall back gracefully to `FlatColor` if
/// a texture is not yet loaded.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RawBlockVisualKind {
    /// Solid color — usable without any texture atlas. Debug-friendly.
    FlatColor { color: [f32; 3] },
    /// Same texture on all six faces.
    TextureCube { all: ResourceRef },
    /// Different texture per group of faces (top / bottom / sides).
    TextureFaces {
        top: ResourceRef,
        bottom: ResourceRef,
        side: ResourceRef,
    },
}

impl Default for RawBlockVisual {
    /// Default visual is a bright magenta flat color so missing visuals are obvious.
    fn default() -> Self {
        Self {
            kind: RawBlockVisualKind::FlatColor {
                color: [1.0, 0.0, 1.0],
            },
            tint: [1.0, 1.0, 1.0],
        }
    }
}
