use glam::{Quat, Vec3};
use vv_config::PhysicsConfig;
use vv_planet::CoordSystem;
use vv_voxel::BlockId;
use vv_world_runtime::PlanetData;

/// Physics solver for round-planet movement and collision.
///
/// Constructed from `PhysicsConfig`; all configurable parameters are stored
/// as fields so callers never hardcode them.
pub struct Physics {
    pub gravity: f32,
    pub player_height: f32,
    pub eye_height: f32,
    pub player_radius: f32,
    pub step_height: f32,
}

impl Physics {
    pub fn new(cfg: PhysicsConfig) -> Self {
        Self {
            gravity: cfg.gravity,
            player_height: cfg.player_height,
            eye_height: cfg.eye_height,
            player_radius: cfg.player_radius,
            step_height: cfg.step_height,
        }
    }

    // --- Pure helpers (no config needed) ------------------------------------

    /// Returns the "up" direction at `pos` (radial outward from planet centre).
    pub fn get_up_vector(pos: Vec3) -> Vec3 {
        pos.normalize_or_zero()
    }

    /// Rotate `rotation` so that its local Y axis aligns with `up`.
    pub fn align_to_planet(rotation: Quat, up: Vec3) -> Quat {
        let current_up = rotation * Vec3::Y;
        let diff = Quat::from_rotation_arc(current_up, up);
        (diff * rotation).normalize()
    }

    // --- Collision ----------------------------------------------------------

    pub fn is_solid(&self, pos: Vec3, planet: &PlanetData) -> bool {
        let res = planet.resolution;
        let (id, local) = match CoordSystem::get_local_coords(pos, planet.geometry) {
            Some(v) => v,
            None => {
                return pos.length() <= planet.geometry.voxel_size_m;
            }
        };

        if !planet.exists(id) {
            return false;
        }

        let margin = 0.05;
        if local.x < margin && id.u > 0 {
            if !planet.exists(BlockId { u: id.u - 1, ..id }) {
                return false;
            }
        } else if local.x > 1.0 - margin && id.u < res - 1 {
            if !planet.exists(BlockId { u: id.u + 1, ..id }) {
                return false;
            }
        }
        if local.y < margin && id.v > 0 {
            if !planet.exists(BlockId { v: id.v - 1, ..id }) {
                return false;
            }
        } else if local.y > 1.0 - margin && id.v < res - 1 {
            if !planet.exists(BlockId { v: id.v + 1, ..id }) {
                return false;
            }
        }
        if local.z < margin && id.layer > 0 {
            if !planet.exists(BlockId {
                layer: id.layer - 1,
                ..id
            }) {
                return false;
            }
        } else if local.z > 1.0 - margin && id.layer < res - 1 {
            if !planet.exists(BlockId {
                layer: id.layer + 1,
                ..id
            }) {
                return false;
            }
        }
        true
    }

    fn get_grid_axes(up: Vec3, pos: Vec3) -> (Vec3, Vec3) {
        let abs_p = pos.abs();
        let rigid_axis = if abs_p.y >= abs_p.x && abs_p.y >= abs_p.z {
            Vec3::X
        } else {
            Vec3::Y
        };
        let right = up.cross(rigid_axis).normalize_or_zero();
        let fwd = up.cross(right).normalize_or_zero();
        if right.length_squared() < 0.001 {
            let r = up.any_orthogonal_vector().normalize();
            (r, up.cross(r).normalize())
        } else {
            (right, fwd)
        }
    }

    pub fn check_collision(&self, pos: Vec3, planet: &PlanetData) -> bool {
        let up = pos.normalize();
        let checks = [
            pos,
            pos + up * 0.9,
            pos + up * self.eye_height,
            pos + up * self.player_height,
        ];
        let (rd, fd) = Self::get_grid_axes(up, pos);
        let right = rd * self.player_radius;
        let fwd = fd * self.player_radius;
        for cp in checks {
            if self.is_solid(cp, planet) {
                return true;
            }
            if self.is_solid(cp + right, planet) {
                return true;
            }
            if self.is_solid(cp - right, planet) {
                return true;
            }
            if self.is_solid(cp + fwd, planet) {
                return true;
            }
            if self.is_solid(cp - fwd, planet) {
                return true;
            }
        }
        false
    }

    // --- Movement solver ----------------------------------------------------

    /// Advance `start_pos` by `velocity * dt`, sliding along surfaces.
    ///
    /// Returns `(new_pos, new_velocity, grounded)`.
    pub fn solve_movement(
        &self,
        start_pos: Vec3,
        velocity: Vec3,
        dt: f32,
        planet: &PlanetData,
        flying: bool,
    ) -> (Vec3, Vec3, bool) {
        if flying {
            return (start_pos + velocity * dt, velocity, false);
        }

        let up = Self::get_up_vector(start_pos);
        let vert_speed = velocity.dot(up);
        let vert_vel = up * vert_speed;
        let horz_vel = velocity - vert_vel;

        let mut curr_pos = start_pos;
        let mut final_horz_vel = horz_vel;

        if horz_vel.length() > 0.001 {
            let desired = curr_pos + horz_vel * dt;
            if !self.check_collision(desired, planet) {
                curr_pos = desired;
            } else {
                let (gright, gfwd) = Self::get_grid_axes(up, curr_pos);
                let v_right = gright * horz_vel.dot(gright);
                let v_fwd = gfwd * horz_vel.dot(gfwd);
                let mut moved = false;
                let try_right = curr_pos + v_right * dt;
                if !self.check_collision(try_right, planet) {
                    curr_pos = try_right;
                    moved = true;
                } else {
                    final_horz_vel -= v_right;
                }
                let try_fwd = curr_pos + v_fwd * dt;
                if !self.check_collision(try_fwd, planet) {
                    curr_pos = try_fwd;
                    moved = true;
                } else {
                    final_horz_vel -= v_fwd;
                }
                if !moved {
                    final_horz_vel = Vec3::ZERO;
                }
            }
        }

        let mut final_vel = final_horz_vel + vert_vel;
        let mut grounded = false;

        let on_ground = self.is_solid(curr_pos - up * 0.1, planet);
        if on_ground && vert_speed <= 0.0 {
            grounded = true;
            final_vel -= vert_vel;
        } else {
            let new_vert_pos = curr_pos + vert_vel * dt;
            if !self.check_collision(new_vert_pos, planet) {
                curr_pos = new_vert_pos;
            } else {
                if vert_speed > 0.0 {
                    final_vel -= vert_vel;
                } else {
                    grounded = true;
                    final_vel -= vert_vel;
                }
            }
        }

        // Auto step-up
        if grounded
            && final_horz_vel.length() < horz_vel.length() * 0.5
            && horz_vel.length() > 0.001
        {
            for sh in [0.3f32, 0.6f32] {
                if sh > self.step_height {
                    break;
                }
                let step_test = curr_pos + up * sh;
                let step_fwd = step_test + horz_vel.normalize() * self.player_radius * 1.5;
                if !self.check_collision(step_fwd, planet) {
                    curr_pos = step_test;
                    break;
                }
            }
        }

        (curr_pos, final_vel, grounded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vv_config::{PhysicsConfig, PlayerConfig};
    use vv_planet::PlanetGeometry;

    #[test]
    fn player_physical_config_does_not_depend_on_voxel_density() {
        let coarse_geometry = PlanetGeometry::new(128.0, 0.5);
        let fine_geometry = PlanetGeometry::new(128.0, 0.05);
        let physics = Physics::new(PhysicsConfig::default());
        let player = PlayerConfig::default();

        assert_eq!(coarse_geometry.radius_m, fine_geometry.radius_m);
        assert!(fine_geometry.resolution > coarse_geometry.resolution);
        assert_eq!(physics.player_height, 1.8);
        assert_eq!(physics.player_radius, 0.3);
        assert_eq!(physics.step_height, 0.6);
        assert_eq!(physics.gravity, 12.0);
        assert_eq!(player.reach_distance, 8.0);
        assert_eq!(player.move_speed, 5.0);
    }
}
