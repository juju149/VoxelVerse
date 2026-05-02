use glam::Vec3;

use super::{grid, SoftCubeFace, SoftCubeParams, SoftCubePoint};

/// Samples a real rounded cube surface in local block space.
///
/// Local space:
/// - block center = (0, 0, 0)
/// - hard cube bounds = [-0.5, 0.5]
/// - y axis = voxel layer/outward direction
/// - x axis = voxel u direction
/// - z axis = voxel v direction
///
/// This is not a bevel strip.
/// This is a rounded-box projection with pillowed faces.
pub(crate) fn sample_soft_cube(
    face: SoftCubeFace,
    u_index: u8,
    v_index: u8,
    params: SoftCubeParams,
) -> SoftCubePoint {
    let params = params.sanitized();

    let u = grid::grid_t(u_index, params.segments);
    let v = grid::grid_t(v_index, params.segments);

    let x = grid::local_axis(u) * 0.5;
    let y = grid::local_axis(v) * 0.5;

    let hard = face_point(face, x, y);
    let face_normal = face_normal(face);

    let rounded = rounded_box_project(hard, params.radius);
    let pillow = pillow_amount(x, y, params.pillow);

    let position = rounded + face_normal * pillow;
    let normal = soft_normal(position, face_normal);

    SoftCubePoint {
        position,
        normal,
        uv: [u, 1.0 - v],
    }
}

fn face_point(face: SoftCubeFace, x: f32, y: f32) -> Vec3 {
    match face {
        // Layer outward.
        SoftCubeFace::Top => Vec3::new(x, 0.5, y),

        // Layer inward.
        SoftCubeFace::Bottom => Vec3::new(x, -0.5, -y),

        // v - 1.
        SoftCubeFace::Front => Vec3::new(x, y, -0.5),

        // v + 1.
        SoftCubeFace::Back => Vec3::new(-x, y, 0.5),

        // u - 1.
        SoftCubeFace::Left => Vec3::new(-0.5, y, -x),

        // u + 1.
        SoftCubeFace::Right => Vec3::new(0.5, y, x),
    }
}

fn face_normal(face: SoftCubeFace) -> Vec3 {
    match face {
        SoftCubeFace::Top => Vec3::Y,
        SoftCubeFace::Bottom => -Vec3::Y,
        SoftCubeFace::Front => -Vec3::Z,
        SoftCubeFace::Back => Vec3::Z,
        SoftCubeFace::Left => -Vec3::X,
        SoftCubeFace::Right => Vec3::X,
    }
}

/// Rounded-box projection.
/// The central face remains broad and readable, while edges/corners become truly round.
fn rounded_box_project(p: Vec3, radius: f32) -> Vec3 {
    let half = Vec3::splat(0.5);
    let inner = half - Vec3::splat(radius);

    let clamped = p.clamp(-inner, inner);
    let delta = p - clamped;

    if delta.length_squared() <= 1e-8 {
        return p;
    }

    clamped + delta.normalize() * radius
}

/// Pillow is strongest at face center and fades before edges.
/// This gives the polished toy look without destroying voxel readability.
fn pillow_amount(x: f32, y: f32, strength: f32) -> f32 {
    if strength <= 0.0 {
        return 0.0;
    }

    let dx = (x.abs() / 0.5).clamp(0.0, 1.0);
    let dy = (y.abs() / 0.5).clamp(0.0, 1.0);

    let edge = dx.max(dy);
    let center = 1.0 - edge;

    smooth01(center).powf(1.35) * strength
}

fn smooth01(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Smooth normal.
/// It keeps a voxel identity while removing brutal corner angles.
fn soft_normal(position: Vec3, face_normal: Vec3) -> Vec3 {
    let radial = position.normalize_or_zero();

    if radial.length_squared() <= 1e-8 {
        return face_normal;
    }

    face_normal.lerp(radial, 0.72).normalize_or_zero()
}
