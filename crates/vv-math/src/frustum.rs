use glam::{Mat4, Vec3, Vec4};

pub struct Frustum {
    planes: [Vec4; 6],
}

impl Frustum {
    pub fn from_matrix(matrix: Mat4) -> Self {
        let r0 = matrix.row(0);
        let r1 = matrix.row(1);
        let r2 = matrix.row(2);
        let r3 = matrix.row(3);

        let mut planes = [r3 + r0, r3 - r0, r3 + r1, r3 - r1, r3 + r2, r3 - r2];

        for plane in &mut planes {
            let len = Vec3::new(plane.x, plane.y, plane.z).length();
            *plane /= len;
        }

        Self { planes }
    }

    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            let dist = plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w;

            if dist < -radius {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::Frustum;
    use glam::{Mat4, Vec3};

    #[test]
    fn sphere_intersection_rejects_far_outside_points() {
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(60_f32.to_radians(), 1.0, 0.1, 100.0);
        let frustum = Frustum::from_matrix(proj * view);

        assert!(frustum.intersects_sphere(Vec3::ZERO, 1.0));
        assert!(!frustum.intersects_sphere(Vec3::new(1000.0, 0.0, 0.0), 1.0));
    }
}
