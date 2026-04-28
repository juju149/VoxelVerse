use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Noise node graph, deserialized from a .ron noise file.
/// Canonical type. The version in vv-world-gen/src/assets.rs is to be migrated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseGraph {
    /// Identifier of the output node in `nodes`.
    pub output: String,
    pub nodes: HashMap<String, NoiseNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NoiseNode {
    Perlin {
        scale: f32,
        octaves: u32,
        persistence: f32,
        lacunarity: f32,
    },
    Simplex {
        scale: f32,
        octaves: u32,
        persistence: f32,
        lacunarity: f32,
    },
    Cellular {
        scale: f32,
    },
    Add {
        a: String,
        b: String,
    },
    Multiply {
        a: String,
        b: String,
    },
    Clamp {
        input: String,
        min: f32,
        max: f32,
    },
    Terrace {
        input: String,
        points: Vec<[f32; 2]>,
    },
    Abs {
        input: String,
    },
    Invert {
        input: String,
    },
    Constant {
        value: f32,
    },
}
