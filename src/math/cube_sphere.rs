use glam::Vec3;

pub fn unit_cube_to_sphere(x: f64, y: f64, z: f64) -> Vec3 {
    let x2 = x * x;
    let y2 = y * y;
    let z2 = z * z;

    let sx = x * (1.0 - y2 * 0.5 - z2 * 0.5 + y2 * z2 / 3.0).sqrt();
    let sy = y * (1.0 - z2 * 0.5 - x2 * 0.5 + z2 * x2 / 3.0).sqrt();
    let sz = z * (1.0 - x2 * 0.5 - y2 * 0.5 + x2 * y2 / 3.0).sqrt();

    Vec3::new(sx as f32, sy as f32, sz as f32)
}

pub fn sphere_to_cube_surface(pos: Vec3) -> Vec3 {
    let mut x = pos.x as f64;
    let mut y = pos.y as f64;
    let mut z = pos.z as f64;

    let fx = x.abs();
    let fy = y.abs();
    let fz = z.abs();

    const INVERSE_SQRT_2: f64 = 0.707_106_769_084_930_4;

    let sqrt = |value: f64| value.max(0.0).sqrt();

    if fy >= fx && fy >= fz {
        let a2 = x * x * 2.0;
        let b2 = z * z * 2.0;
        let inner = -a2 + b2 - 3.0;
        let inner_sqrt = -sqrt((inner * inner) - 12.0 * a2);

        x = if x == 0.0 {
            0.0
        } else {
            sqrt(inner_sqrt + a2 - b2 + 3.0) * INVERSE_SQRT_2
        };
        z = if z == 0.0 {
            0.0
        } else {
            sqrt(inner_sqrt - a2 + b2 + 3.0) * INVERSE_SQRT_2
        };

        x = x.min(1.0).copysign(pos.x as f64);
        z = z.min(1.0).copysign(pos.z as f64);
        y = if pos.y >= 0.0 { 1.0 } else { -1.0 };
    } else if fx >= fy && fx >= fz {
        let a2 = y * y * 2.0;
        let b2 = z * z * 2.0;
        let inner = -a2 + b2 - 3.0;
        let inner_sqrt = -sqrt((inner * inner) - 12.0 * a2);

        y = if y == 0.0 {
            0.0
        } else {
            sqrt(inner_sqrt + a2 - b2 + 3.0) * INVERSE_SQRT_2
        };
        z = if z == 0.0 {
            0.0
        } else {
            sqrt(inner_sqrt - a2 + b2 + 3.0) * INVERSE_SQRT_2
        };

        y = y.min(1.0).copysign(pos.y as f64);
        z = z.min(1.0).copysign(pos.z as f64);
        x = if pos.x >= 0.0 { 1.0 } else { -1.0 };
    } else {
        let a2 = x * x * 2.0;
        let b2 = y * y * 2.0;
        let inner = -a2 + b2 - 3.0;
        let inner_sqrt = -sqrt((inner * inner) - 12.0 * a2);

        x = if x == 0.0 {
            0.0
        } else {
            sqrt(inner_sqrt + a2 - b2 + 3.0) * INVERSE_SQRT_2
        };
        y = if y == 0.0 {
            0.0
        } else {
            sqrt(inner_sqrt - a2 + b2 + 3.0) * INVERSE_SQRT_2
        };

        x = x.min(1.0).copysign(pos.x as f64);
        y = y.min(1.0).copysign(pos.y as f64);
        z = if pos.z >= 0.0 { 1.0 } else { -1.0 };
    }

    Vec3::new(x as f32, y as f32, z as f32)
}

#[cfg(test)]
mod tests {
    use super::{sphere_to_cube_surface, unit_cube_to_sphere};

    #[test]
    fn cube_sphere_inverse_keeps_surface_coordinates_stable() {
        let samples = [
            (-1.0, 0.0, 0.0),
            (1.0, 0.25, -0.5),
            (0.3, 1.0, -0.2),
            (-0.75, -1.0, 0.1),
            (0.5, -0.4, 1.0),
            (-0.1, 0.8, -1.0),
        ];

        for (x, y, z) in samples {
            let sphere = unit_cube_to_sphere(x, y, z);
            let cube = sphere_to_cube_surface(sphere);

            assert!((cube.x - x as f32).abs() < 0.001);
            assert!((cube.y - y as f32).abs() < 0.001);
            assert!((cube.z - z as f32).abs() < 0.001);
        }
    }
}
