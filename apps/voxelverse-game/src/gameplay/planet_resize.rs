use crate::gameplay::Player;
use crate::generation::CoordSystem;
use crate::world::PlanetData;
use glam::Vec3;

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

        let probe_pos = current_dir * planet.profile.surface_radius;
        let spawn_radius = if let Some(id) = CoordSystem::pos_to_id(probe_pos, planet.profile) {
            let height = planet.terrain.get_height(id.face, id.u, id.v);
            planet.profile.layer_radius(height + 1) + planet.profile.spawn_clearance()
        } else {
            planet.profile.surface_radius + planet.profile.spawn_clearance()
        };

        player.position = current_dir * spawn_radius;
        player.velocity = Vec3::ZERO;
    }
}
