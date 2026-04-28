use glam::Vec3;
use vv_core::BlockId;

/// Cube-sphere coordinate system for round planets.
///
/// The planet is modelled as a cube mapped onto a sphere using the
/// Nowell/Tarini cube-sphere projection. Each cube face is subdivided
/// into a `resolution × resolution` grid; layers extend radially outward
/// from the centre using an exponential radial mapping with constant `K`.
pub struct CoordSystem;

impl CoordSystem {
    /// Radial scale factor for the exponential layer mapping.
    /// Increasing K compresses inner layers; decreasing it spreads them.
    const K: f64 = 0.85;

    // --- Cube ↔ sphere projection -------------------------------------------

    fn cube_to_sphere(x: f64, y: f64, z: f64) -> Vec3 {
        let x2 = x * x;
        let y2 = y * y;
        let z2 = z * z;
        let sx = x * (1.0 - y2 * 0.5 - z2 * 0.5 + y2 * z2 / 3.0).sqrt();
        let sy = y * (1.0 - z2 * 0.5 - x2 * 0.5 + z2 * x2 / 3.0).sqrt();
        let sz = z * (1.0 - x2 * 0.5 - y2 * 0.5 + x2 * y2 / 3.0).sqrt();
        Vec3::new(sx as f32, sy as f32, sz as f32)
    }

    fn cubize_point(pos: Vec3) -> Vec3 {
        let mut x = pos.x as f64;
        let mut y = pos.y as f64;
        let mut z = pos.z as f64;

        let fx = x.abs();
        let fy = y.abs();
        let fz = z.abs();

        const INVERSE_SQRT_2: f64 = 0.70710676908493042;

        if fy >= fx && fy >= fz {
            let a2 = x * x * 2.0;
            let b2 = z * z * 2.0;
            let inner = -a2 + b2 - 3.0;
            let inner_sqrt = -((inner * inner) - 12.0 * a2).sqrt();
            if x == 0.0 { x = 0.0; } else { x = (inner_sqrt + a2 - b2 + 3.0).sqrt() * INVERSE_SQRT_2; }
            if z == 0.0 { z = 0.0; } else { z = (inner_sqrt - a2 + b2 + 3.0).sqrt() * INVERSE_SQRT_2; }
            if x > 1.0 { x = 1.0; }
            if z > 1.0 { z = 1.0; }
            if pos.x < 0.0 { x = -x; }
            if pos.z < 0.0 { z = -z; }
            y = if pos.y > 0.0 { 1.0 } else { -1.0 };
        } else if fx >= fy && fx >= fz {
            let a2 = y * y * 2.0;
            let b2 = z * z * 2.0;
            let inner = -a2 + b2 - 3.0;
            let inner_sqrt = -((inner * inner) - 12.0 * a2).sqrt();
            if y == 0.0 { y = 0.0; } else { y = (inner_sqrt + a2 - b2 + 3.0).sqrt() * INVERSE_SQRT_2; }
            if z == 0.0 { z = 0.0; } else { z = (inner_sqrt - a2 + b2 + 3.0).sqrt() * INVERSE_SQRT_2; }
            if y > 1.0 { y = 1.0; }
            if z > 1.0 { z = 1.0; }
            if pos.y < 0.0 { y = -y; }
            if pos.z < 0.0 { z = -z; }
            x = if pos.x > 0.0 { 1.0 } else { -1.0 };
        } else {
            let a2 = x * x * 2.0;
            let b2 = y * y * 2.0;
            let inner = -a2 + b2 - 3.0;
            let inner_sqrt = -((inner * inner) - 12.0 * a2).sqrt();
            if x == 0.0 { x = 0.0; } else { x = (inner_sqrt + a2 - b2 + 3.0).sqrt() * INVERSE_SQRT_2; }
            if y == 0.0 { y = 0.0; } else { y = (inner_sqrt - a2 + b2 + 3.0).sqrt() * INVERSE_SQRT_2; }
            if x > 1.0 { x = 1.0; }
            if y > 1.0 { y = 1.0; }
            if pos.x < 0.0 { x = -x; }
            if pos.y < 0.0 { y = -y; }
            z = if pos.z > 0.0 { 1.0 } else { -1.0 };
        }
        Vec3::new(x as f32, y as f32, z as f32)
    }

    // --- Public API ---------------------------------------------------------

    /// Convert a 3-D world position to a `BlockId` plus fractional sub-cell
    /// coordinates `(f_u, f_v, f_layer)` each in `[0, 1)`.
    ///
    /// Returns `None` when `pos` is inside the planet core.
    pub fn get_local_coords(pos: Vec3, res: u32) -> Option<(BlockId, Vec3)> {
        let dist = pos.length() as f64;
        let s = res as f64 / 2.0;
        let min_r = s * (-Self::K).exp();
        if dist < min_r { return None; }

        let layer_f = s * (1.0 + (dist / s).ln() / Self::K);
        let layer = layer_f.floor() as i32;
        if layer < 0 || layer >= res as i32 { return None; }

        let f_layer = (layer_f - layer as f64) as f32;

        let cube_pos = Self::cubize_point(pos.normalize());
        let abs = cube_pos.abs();

        let (face, u_local, v_local) = if abs.y >= abs.x && abs.y >= abs.z {
            if cube_pos.y > 0.0 { (0, cube_pos.x, cube_pos.z) } else { (1, cube_pos.x, cube_pos.z) }
        } else if abs.x >= abs.y && abs.x >= abs.z {
            if cube_pos.x > 0.0 { (2, cube_pos.y, cube_pos.z) } else { (3, cube_pos.y, cube_pos.z) }
        } else {
            if cube_pos.z > 0.0 { (4, cube_pos.x, cube_pos.y) } else { (5, cube_pos.x, cube_pos.y) }
        };

        let rf = res as f64;
        let u_raw = (u_local as f64 * rf + rf) / 2.0;
        let v_raw = (v_local as f64 * rf + rf) / 2.0;
        let u = (u_raw.floor() as i32).clamp(0, res as i32 - 1) as u32;
        let v = (v_raw.floor() as i32).clamp(0, res as i32 - 1) as u32;
        let f_u = (u_raw - u as f64) as f32;
        let f_v = (v_raw - v as f64) as f32;

        Some((BlockId { face: face as u8, layer: layer as u32, u, v }, Vec3::new(f_u, f_v, f_layer)))
    }

    /// World-space radius of a given radial layer index.
    pub fn get_layer_radius(layer: u32, res: u32) -> f32 {
        let s = res as f64 / 2.0;
        (s * (Self::K * ((layer as f64 / s) - 1.0)).exp()) as f32
    }

    /// Outward unit direction for a face grid cell.
    pub fn get_direction(face: u8, u: u32, v: u32, res: u32) -> Vec3 {
        let rf = res as f64;
        let x_local = if u == 0 { -1.0 } else if u == res { 1.0 } else { (u as f64 * 2.0 - rf) / rf };
        let y_local = if v == 0 { -1.0 } else if v == res { 1.0 } else { (v as f64 * 2.0 - rf) / rf };
        let (cx, cy, cz) = match face {
            0 => (x_local, 1.0, y_local),
            1 => (x_local, -1.0, y_local),
            2 => (1.0, x_local, y_local),
            3 => (-1.0, x_local, y_local),
            4 => (x_local, y_local, 1.0),
            _ => (x_local, y_local, -1.0),
        };
        Self::cube_to_sphere(cx, cy, cz).normalize()
    }

    /// World-space position of a voxel corner vertex.
    pub fn get_vertex_pos(face: u8, u: u32, v: u32, layer: u32, res: u32) -> Vec3 {
        Self::get_direction(face, u, v, res) * Self::get_layer_radius(layer, res)
    }

    /// World-space centre of a voxel.
    pub fn get_block_center(face: u8, u: u32, v: u32, layer: u32, res: u32) -> Vec3 {
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
        let dir = Self::cube_to_sphere(cx, cy, cz).normalize();
        let s = rf / 2.0;
        let radius = s * (Self::K * (((layer as f64 + 0.5) / s) - 1.0)).exp();
        dir * radius as f32
    }

    /// Map a world-space position to the `BlockId` it occupies, or `None`
    /// when inside the core.
    pub fn pos_to_id(pos: Vec3, res: u32) -> Option<BlockId> {
        let dist = pos.length() as f64;
        let s = res as f64 / 2.0;
        let min_r = s * (-Self::K).exp();
        if dist < min_r { return None; }

        let layer_f = s * (1.0 + (dist / s).ln() / Self::K);
        let layer = layer_f.floor() as i32;
        if layer < 0 { return None; }
        let layer = layer as u32;
        if layer >= res { return None; }

        let cube_pos = Self::cubize_point(pos.normalize());
        let abs = cube_pos.abs();
        let (face, u_local, v_local) = if abs.y >= abs.x && abs.y >= abs.z {
            if cube_pos.y > 0.0 { (0, cube_pos.x, cube_pos.z) } else { (1, cube_pos.x, cube_pos.z) }
        } else if abs.x >= abs.y && abs.x >= abs.z {
            if cube_pos.x > 0.0 { (2, cube_pos.y, cube_pos.z) } else { (3, cube_pos.y, cube_pos.z) }
        } else {
            if cube_pos.z > 0.0 { (4, cube_pos.x, cube_pos.y) } else { (5, cube_pos.x, cube_pos.y) }
        };

        let rf = res as f64;
        let u = ((u_local as f64 * rf + rf) / 2.0).floor() as i32;
        let v = ((v_local as f64 * rf + rf) / 2.0).floor() as i32;
        Some(BlockId {
            face: face as u8,
            layer,
            u: u.clamp(0, res as i32 - 1) as u32,
            v: v.clamp(0, res as i32 - 1) as u32,
        })
    }
}
