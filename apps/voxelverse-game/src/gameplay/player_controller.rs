use crate::gameplay::{Player, PlayerInput};
use crate::world::PlanetData;

pub struct PlayerController;

impl PlayerController {
    pub fn update(player: &mut Player, planet: &PlanetData, input: PlayerInput, dt: f32) {
        player.update(dt, planet, input);
    }
}
