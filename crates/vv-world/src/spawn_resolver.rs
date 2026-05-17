//! Initial-spawn position derivation from terrain + planet geometry.

use crate::{PlanetData, PlanetGeometry};

impl PlanetData {
    pub fn spawn_position(&self) -> glam::Vec3 {
        // Face 4 = equatorial +Z face.  At center: dir.y ≈ 0 → latitude ≈ 0
        // → temperature ≈ 1.0 → tropical biome.  Face 0 is the +Y pole.
        let u = self.resolution / 2;
        let v = self.resolution / 2;
        let dir = PlanetGeometry::get_direction(4, u, v, self.resolution);
        dir * (self.surface_radius(4, u, v) + self.profile.spawn_clearance())
    }
}
