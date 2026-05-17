use crate::{sphere_to_cube_surface, unit_cube_to_sphere};
use glam::Vec3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GridCoord {
    pub face: u8,
    pub layer: u32,
    pub u: u32,
    pub v: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct SphericalGrid {
    pub resolution: u32,
    pub inner_radius: f32,
    pub layer_height: f32,
}

impl SphericalGrid {
    pub fn new(resolution: u32, inner_radius: f32, layer_height: f32) -> Self {
        Self {
            resolution: resolution.max(1),
            inner_radius,
            layer_height: layer_height.max(f32::EPSILON),
        }
    }

    pub fn layer_radius(self, layer: u32) -> f32 {
        self.inner_radius + self.layer_height * layer as f32
    }

    pub fn layer_center_radius(self, layer: u32) -> f32 {
        self.layer_radius(layer) + self.layer_height * 0.5
    }

    pub fn radius_to_layer(self, radius: f32) -> Option<(u32, f32)> {
        if radius < self.inner_radius || radius.is_nan() {
            return None;
        }

        let layer_f = (radius - self.inner_radius) / self.layer_height;
        let layer = layer_f.floor() as i32;
        if layer < 0 || layer >= self.resolution as i32 {
            return None;
        }

        Some((layer as u32, layer_f.fract()))
    }
}

pub struct CoordSystem;

impl CoordSystem {
    fn direction_to_face_uv(dir: Vec3) -> (u8, f32, f32) {
        let cube_pos = sphere_to_cube_surface(dir.normalize_or_zero());
        let abs = cube_pos.abs();

        if abs.y >= abs.x && abs.y >= abs.z {
            let face = if cube_pos.y >= 0.0 { 0 } else { 1 };
            (face, cube_pos.x, cube_pos.z)
        } else if abs.x >= abs.y && abs.x >= abs.z {
            let face = if cube_pos.x >= 0.0 { 2 } else { 3 };
            (face, cube_pos.y, cube_pos.z)
        } else {
            let face = if cube_pos.z >= 0.0 { 4 } else { 5 };
            (face, cube_pos.x, cube_pos.y)
        }
    }

    pub fn get_local_coords(pos: Vec3, grid: SphericalGrid) -> Option<(GridCoord, Vec3)> {
        let res = grid.resolution;
        let dist = pos.length();
        let (layer, f_layer) = grid.radius_to_layer(dist)?;

        let (face, u_local, v_local) = Self::direction_to_face_uv(pos);

        let rf = res as f64;
        let u_raw = (u_local as f64 * rf + rf) / 2.0;
        let v_raw = (v_local as f64 * rf + rf) / 2.0;

        let u = u_raw.floor() as i32;
        let v = v_raw.floor() as i32;

        let f_u = (u_raw - u as f64) as f32;
        let f_v = (v_raw - v as f64) as f32;

        let u = u.clamp(0, res as i32 - 1) as u32;
        let v = v.clamp(0, res as i32 - 1) as u32;

        Some((
            GridCoord { face, layer, u, v },
            Vec3::new(f_u, f_v, f_layer),
        ))
    }

    pub fn get_layer_radius(layer: u32, grid: SphericalGrid) -> f32 {
        grid.layer_radius(layer)
    }

    pub fn get_direction(face: u8, u: u32, v: u32, res: u32) -> Vec3 {
        let rf = res as f64;

        let x_local = if u == 0 {
            -1.0
        } else if u == res {
            1.0
        } else {
            (u as f64 * 2.0 - rf) / rf
        };

        let y_local = if v == 0 {
            -1.0
        } else if v == res {
            1.0
        } else {
            (v as f64 * 2.0 - rf) / rf
        };

        let (cx, cy, cz) = match face {
            0 => (x_local, 1.0, y_local),
            1 => (x_local, -1.0, y_local),
            2 => (1.0, x_local, y_local),
            3 => (-1.0, x_local, y_local),
            4 => (x_local, y_local, 1.0),
            _ => (x_local, y_local, -1.0),
        };

        unit_cube_to_sphere(cx, cy, cz).normalize()
    }

    pub fn get_vertex_pos(face: u8, u: u32, v: u32, layer: u32, grid: SphericalGrid) -> Vec3 {
        let res = grid.resolution;
        let dir = Self::get_direction(face, u, v, res);
        let radius = Self::get_layer_radius(layer, grid);
        dir * radius
    }

    pub fn get_vertex_pos_f32(face: u8, u: f32, v: f32, layer: f32, grid: SphericalGrid) -> Vec3 {
        let res = grid.resolution as f64;
        let uf = u as f64;
        let vf = v as f64;

        let x_local = (uf * 2.0 - res) / res;
        let y_local = (vf * 2.0 - res) / res;

        let (cx, cy, cz) = match face {
            0 => (x_local, 1.0, y_local),
            1 => (x_local, -1.0, y_local),
            2 => (1.0, x_local, y_local),
            3 => (-1.0, x_local, y_local),
            4 => (x_local, y_local, 1.0),
            _ => (x_local, y_local, -1.0),
        };

        let dir = unit_cube_to_sphere(cx, cy, cz).normalize();
        let layer_i = layer.floor() as u32;
        let frac = layer - layer.floor();
        let r0 = Self::get_layer_radius(layer_i, grid) as f64;
        let r1 = Self::get_layer_radius(layer_i + 1, grid) as f64;
        let radius = (r0 + frac as f64 * (r1 - r0)) as f32;
        dir * radius
    }

    pub fn get_block_center(face: u8, u: u32, v: u32, layer: u32, grid: SphericalGrid) -> Vec3 {
        let res = grid.resolution;
        let rf = res as f64;
        let uf = u as f64 + 0.5;
        let vf = v as f64 + 0.5;

        let x_local = (uf * 2.0 - rf) / rf;
        let y_local = (vf * 2.0 - rf) / rf;

        let (cx, cy, cz) = match face {
            0 => (x_local, 1.0, y_local),
            1 => (x_local, -1.0, y_local),
            2 => (1.0, x_local, y_local),
            3 => (-1.0, x_local, y_local),
            4 => (x_local, y_local, 1.0),
            _ => (x_local, y_local, -1.0),
        };

        let dir = unit_cube_to_sphere(cx, cy, cz).normalize();
        let radius = grid.layer_center_radius(layer);
        dir * radius
    }

    pub fn pos_to_id(pos: Vec3, grid: SphericalGrid) -> Option<GridCoord> {
        let res = grid.resolution;
        let (layer, _) = grid.radius_to_layer(pos.length())?;
        let (face, u_local, v_local) = Self::direction_to_face_uv(pos);

        let rf = res as f64;
        let u_raw = ((u_local as f64 * rf + rf) / 2.0).floor() as i32;
        let v_raw = ((v_local as f64 * rf + rf) / 2.0).floor() as i32;

        let u = u_raw.clamp(0, res as i32 - 1) as u32;
        let v = v_raw.clamp(0, res as i32 - 1) as u32;

        Some(GridCoord { face, layer, u, v })
    }
}

#[cfg(test)]
mod tests {
    use super::{CoordSystem, SphericalGrid};

    #[test]
    fn cube_sphere_coords_roundtrip_on_all_faces() {
        let grid = SphericalGrid::new(49, 4.0, 1.0);
        let surface_layer = 20;
        let samples = [
            (0, 3, 5),
            (1, 9, 17),
            (2, 12, 7),
            (3, 18, 23),
            (4, 27, 11),
            (5, 33, 31),
        ];

        for (face, u, v) in samples {
            let pos = CoordSystem::get_block_center(face, u, v, surface_layer, grid);
            let coord = CoordSystem::pos_to_id(pos, grid)
                .expect("surface sample should map back to a voxel coordinate");

            assert_eq!(coord.face, face);
            assert!((coord.u as i32 - u as i32).abs() <= 1);
            assert!((coord.v as i32 - v as i32).abs() <= 1);
            assert_eq!(coord.layer, surface_layer);
        }
    }

    #[test]
    fn profile_voxel_size_changes_world_scale() {
        let one_meter = SphericalGrid::new(64, 4.0, 1.0);
        let half_meter = SphericalGrid::new(64, 2.0, 0.5);
        let surface_layer = 32;

        let p1 = CoordSystem::get_vertex_pos(4, 32, 32, surface_layer, one_meter);
        let p2 = CoordSystem::get_vertex_pos(4, 32, 32, surface_layer, half_meter);

        assert!((p2.length() - p1.length() * 0.5).abs() < 0.001);
    }
}
