use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FloatRange {
    pub min: f32,
    pub max: f32,
}

impl Default for FloatRange {
    fn default() -> Self {
        FloatRange { min: 0.0, max: 1.0 }
    }
}

impl FloatRange {
    pub fn exact(v: f32) -> Self {
        FloatRange { min: v, max: v }
    }
    pub fn full() -> Self {
        FloatRange { min: 0.0, max: 1.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntRange {
    pub min: i32,
    pub max: i32,
}

impl Default for IntRange {
    fn default() -> Self {
        IntRange { min: 1, max: 1 }
    }
}

impl IntRange {
    pub fn exact(v: i32) -> Self {
        IntRange { min: v, max: v }
    }
}

/// Range with an inner ideal zone (trapezoid fitness curve).
/// Value 1.0 within [ideal_min..ideal_max], declines to 0.0 at the edges.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IdealRange {
    pub min: f32,
    pub ideal_min: f32,
    pub ideal_max: f32,
    pub max: f32,
}

impl Default for IdealRange {
    fn default() -> Self {
        IdealRange {
            min: 0.0,
            ideal_min: 0.25,
            ideal_max: 0.75,
            max: 1.0,
        }
    }
}
