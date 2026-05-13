//! Mutable state for procedurally placed vox props.
//!
//! Props are generated from terrain data and are not stored in the voxel grid.
//! This layer only records columns where the procedural prop was destroyed by
//! the player so generation can stay deterministic while edits persist.

use std::collections::HashSet;

/// Compact key for one surface column that may support a procedural prop.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PropSupportKey {
    face: u8,
    u: u32,
    v: u32,
}

impl PropSupportKey {
    fn new(face: u8, u: u32, v: u32) -> Self {
        Self { face, u, v }
    }
}

/// Tracks prop columns explicitly destroyed by the player.
#[derive(Clone, Debug, Default)]
pub struct BrokenPropLayer {
    broken: HashSet<PropSupportKey>,
}

impl BrokenPropLayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn break_prop(&mut self, face: u8, u: u32, v: u32) {
        self.broken.insert(PropSupportKey::new(face, u, v));
    }

    pub fn is_alive(&self, face: u8, u: u32, v: u32) -> bool {
        !self.broken.contains(&PropSupportKey::new(face, u, v))
    }
}

