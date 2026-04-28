use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Default for RgbColor {
    fn default() -> Self {
        RgbColor {
            r: 0.5,
            g: 0.5,
            b: 0.5,
        }
    }
}
