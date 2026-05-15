use glam::Vec3;

pub(super) struct TerrainRenderer;

impl TerrainRenderer {
    pub fn behind_planet_horizon(
        surface_radius: f32,
        camera_position: Vec3,
        center: Vec3,
        radius: f32,
    ) -> bool {
        let cam_dist = camera_position.length();
        if cam_dist <= surface_radius * 1.001 {
            return false;
        }
        let dist = center.length();
        if dist < 1e-3 {
            return false;
        }

        let cam_dir = camera_position / cam_dist;
        let cos_horizon = surface_radius / cam_dist;
        let cos_angle = cam_dir.dot(center) / dist;
        let angular_radius = radius / dist;
        cos_angle < cos_horizon - 1.5 * angular_radius
    }
}
