use std::f32::consts::TAU;

use glam::{Mat3, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct CelestialState {
    pub sun_direction_world: Vec3,
    pub moon_direction_world: Vec3,
    pub sun_elevation: f32,
    pub moon_elevation: f32,
}

impl CelestialState {
    pub fn from_time01(time01: f32, axial_tilt_deg: f32) -> Self {
        let time01 = time01.rem_euclid(1.0);
        let phase = time01 * TAU;

        let elevation = -phase.cos();
        let azimuth = phase.sin();
        let raw_sun = Vec3::new(azimuth * 0.92, elevation, 0.28);
        let tilted_sun = Mat3::from_rotation_z(axial_tilt_deg.to_radians()) * raw_sun;
        let sun_direction_world = normalized_or(tilted_sun, Vec3::Y);

        let moon_direction_world = -sun_direction_world;

        Self {
            sun_direction_world,
            moon_direction_world,
            sun_elevation: sun_direction_world.y,
            moon_elevation: moon_direction_world.y,
        }
    }
}

fn normalized_or(value: Vec3, fallback: Vec3) -> Vec3 {
    value.try_normalize().unwrap_or(fallback)
}
