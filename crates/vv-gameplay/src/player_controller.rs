use crate::{Player, PlayerInput};
use vv_world::PlanetData;

pub struct PlayerController;

impl PlayerController {
    pub fn update(player: &mut Player, planet: &PlanetData, input: PlayerInput, dt: f32) {
        player.update(dt, planet, input);
    }
}
