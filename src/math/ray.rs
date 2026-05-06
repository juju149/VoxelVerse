use glam::{Mat4, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn from_clip_space(inverse_view_projection: Mat4, ndc_x: f32, ndc_y: f32) -> Self {
        let origin = inverse_view_projection.project_point3(Vec3::new(ndc_x, ndc_y, 0.0));
        let end = inverse_view_projection.project_point3(Vec3::new(ndc_x, ndc_y, 1.0));
        let direction = (end - origin).normalize();

        Self { origin, direction }
    }

    pub fn point_at(self, distance: f32) -> Vec3 {
        self.origin + self.direction * distance
    }
}

#[cfg(test)]
mod tests {
    use super::Ray;
    use glam::{Mat4, Vec3};

    #[test]
    fn clip_space_ray_points_forward_from_camera() {
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(60_f32.to_radians(), 1.0, 0.1, 100.0);
        let ray = Ray::from_clip_space((proj * view).inverse(), 0.0, 0.0);

        assert!(ray.direction.z < -0.99);
        assert!((ray.direction.length() - 1.0).abs() < 0.001);
    }
}
