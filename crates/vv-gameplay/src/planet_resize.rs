use crate::Player;
use glam::Vec3;
use vv_world::{PlanetData, PlanetGeometry};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanetResizeIntent {
    Grow,
    Shrink,
}

pub struct PlanetResize;

impl PlanetResize {
    pub fn apply(intent: PlanetResizeIntent, planet: &mut PlanetData, player: &mut Player) {
        planet.resize(intent == PlanetResizeIntent::Grow);
        let current_dir = if player.position.length() > 0.1 {
            player.position.normalize()
        } else {
            Vec3::Y
        };

        let profile = planet.profile();
        let probe_pos = current_dir * profile.surface_radius;
        let spawn_radius = if let Some(id) = PlanetGeometry::pos_to_id(probe_pos, profile) {
            let height = planet.surface_height(id.face, id.u, id.v);
            profile.layer_radius(height + 1) + profile.spawn_clearance()
        } else {
            profile.surface_radius + profile.spawn_clearance()
        };

        player.position = current_dir * spawn_radius;
        player.velocity = Vec3::ZERO;
    }
}
