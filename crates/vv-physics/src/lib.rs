use glam::{Quat, Vec3};
use vv_math::CoordSystem;
use vv_voxel::VoxelCoord;
use vv_world::PlanetData;

pub struct Physics;
impl Physics {
    pub const GRAVITY: f32 = 12.0;
    pub const PLAYER_HEIGHT: f32 = 1.8;
    pub const EYE_HEIGHT: f32 = 1.6;
    pub const PLAYER_RADIUS: f32 = 0.3;
    pub const STEP_HEIGHT: f32 = 0.6;

    pub fn get_up_vector(pos: Vec3) -> Vec3 {
        pos.normalize_or_zero()
    }

    pub fn align_to_planet(rotation: Quat, up: Vec3) -> Quat {
        let current_up = rotation * Vec3::Y;
        let rotation_diff = Quat::from_rotation_arc(current_up, up);
        (rotation_diff * rotation).normalize()
    }

    pub fn is_solid(pos: Vec3, planet: &PlanetData) -> bool {
        let res = planet.resolution;

        let (id, local) = match CoordSystem::get_local_coords(pos, planet.profile) {
            Some(val) => val,
            None => {
                return pos.length() < planet.profile.inner_radius;
            }
        };

        if !planet.exists(id) {
            return false;
        }

        let margin = 0.05;

        if local.x < margin && id.u > 0 {
            let neighbor = VoxelCoord { u: id.u - 1, ..id };
            if !planet.exists(neighbor) {
                return false;
            }
        } else if local.x > (1.0 - margin) && id.u < res - 1 {
            let neighbor = VoxelCoord { u: id.u + 1, ..id };
            if !planet.exists(neighbor) {
                return false;
            }
        }

        if local.y < margin && id.v > 0 {
            let neighbor = VoxelCoord { v: id.v - 1, ..id };
            if !planet.exists(neighbor) {
                return false;
            }
        } else if local.y > (1.0 - margin) && id.v < res - 1 {
            let neighbor = VoxelCoord { v: id.v + 1, ..id };
            if !planet.exists(neighbor) {
                return false;
            }
        }

        if local.z < margin && id.layer > 0 {
            let neighbor = VoxelCoord {
                layer: id.layer - 1,
                ..id
            };
            if !planet.exists(neighbor) {
                return false;
            }
        } else if local.z > (1.0 - margin) && id.layer < res - 1 {
            let neighbor = VoxelCoord {
                layer: id.layer + 1,
                ..id
            };
            if !planet.exists(neighbor) {
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

    pub fn check_collision(pos: Vec3, planet: &PlanetData) -> bool {
        let up = pos.normalize();

        let checks = [
            pos,
            pos + up * 0.9,
            pos + up * Self::EYE_HEIGHT,
            pos + up * Self::PLAYER_HEIGHT,
        ];
        let (right_dir, fwd_dir) = Self::get_grid_axes(up, pos);
        let right = right_dir * Self::PLAYER_RADIUS;
        let fwd = fwd_dir * Self::PLAYER_RADIUS;

        for center_p in checks {
            if Self::is_solid(center_p, planet) {
                return true;
            }
            if Self::is_solid(center_p + right, planet) {
                return true;
            }
            if Self::is_solid(center_p - right, planet) {
                return true;
            }
            if Self::is_solid(center_p + fwd, planet) {
                return true;
            }
            if Self::is_solid(center_p - fwd, planet) {
                return true;
            }
        }
        false
    }

    pub fn solve_movement(
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
            let desired_pos = curr_pos + horz_vel * dt;

            if !Self::check_collision(desired_pos, planet) {
                curr_pos = desired_pos;
            } else {
                let (grid_right, grid_fwd) = Self::get_grid_axes(up, curr_pos);

                let v_right = grid_right * horz_vel.dot(grid_right);
                let v_fwd = grid_fwd * horz_vel.dot(grid_fwd);

                let mut moved = false;

                let try_right = curr_pos + v_right * dt;
                if !Self::check_collision(try_right, planet) {
                    curr_pos = try_right;
                    moved = true;
                } else {
                    final_horz_vel -= v_right;
                }

                let try_fwd = curr_pos + v_fwd * dt;
                if !Self::check_collision(try_fwd, planet) {
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

        let ground_check_pos = curr_pos - up * 0.1;
        let on_ground = Self::is_solid(ground_check_pos, planet);

        if on_ground && vert_speed <= 0.0 {
            grounded = true;
            final_vel -= vert_vel;
        } else {
            let new_vert_pos = curr_pos + vert_vel * dt;
            if !Self::check_collision(new_vert_pos, planet) {
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

        if grounded
            && final_horz_vel.length() < horz_vel.length() * 0.5
            && horz_vel.length() > 0.001
        {
            for step_height in [Self::STEP_HEIGHT * 0.5, Self::STEP_HEIGHT] {
                let step_test = curr_pos + up * step_height;
                let step_forward = step_test + horz_vel.normalize() * Self::PLAYER_RADIUS * 1.5;

                if !Self::check_collision(step_test, planet)
                    && !Self::check_collision(step_forward, planet)
                {
                    curr_pos = step_test;
                    final_vel = horz_vel;
                    break;
                }
            }
        }

        if Self::check_collision(curr_pos, planet) {
            curr_pos += up * 4.0 * dt;
        }

        (curr_pos, final_vel, grounded)
    }
}
