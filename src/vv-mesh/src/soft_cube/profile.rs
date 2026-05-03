use glam::Vec3;

use super::{
    grid, seal::seal_hidden_edges, SoftCubeEdgeMask, SoftCubeFace, SoftCubeParams, SoftCubePoint,
};

pub(crate) fn sample_soft_cube(
    face: SoftCubeFace,
    u_index: u8,
    v_index: u8,
    params: SoftCubeParams,
    edge_mask: SoftCubeEdgeMask,
) -> SoftCubePoint {
    let params = params.sanitized();

    let u = grid::grid_t(u_index, params.segments);
    let v = grid::grid_t(v_index, params.segments);

    let local_u = grid::local_axis(u) * 0.5;
    let local_v = grid::local_axis(v) * 0.5;

    let hard = face_point(face, local_u, local_v);
    let face_normal = face_normal(face);
    let exposure = AxisExposure::from_face(face, edge_mask);

    let rounded = rounded_box_project(hard, params.radius, exposure);
    let pillow = pillow_amount(local_u, local_v, params.pillow);

    let mut point = SoftCubePoint {
        position: rounded.position + face_normal * pillow,
        normal: rounded.normal,
        uv: [u, 1.0 - v],
    };

    seal_hidden_edges(
        face,
        &mut point,
        edge_mask,
        u_index,
        v_index,
        params.segments,
    );

    point
}

pub(crate) fn sample_soft_cube_uv(
    face: SoftCubeFace,
    u: f32,
    v: f32,
    params: SoftCubeParams,
    edge_mask: SoftCubeEdgeMask,
) -> SoftCubePoint {
    let params = params.sanitized();

    let u = u.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);

    let local_u = grid::local_axis(u) * 0.5;
    let local_v = grid::local_axis(v) * 0.5;

    let hard = face_point(face, local_u, local_v);
    let face_normal = face_normal(face);
    let exposure = AxisExposure::from_face(face, edge_mask);

    let rounded = rounded_box_project(hard, params.radius, exposure);
    let pillow = pillow_amount(local_u, local_v, params.pillow);

    SoftCubePoint {
        position: rounded.position + face_normal * pillow,
        normal: rounded.normal,
        uv: [u, 1.0 - v],
    }
}
fn face_point(face: SoftCubeFace, x: f32, y: f32) -> Vec3 {
    match face {
        SoftCubeFace::Top => Vec3::new(x, 0.5, y),
        SoftCubeFace::Bottom => Vec3::new(x, -0.5, -y),
        SoftCubeFace::Front => Vec3::new(x, y, -0.5),
        SoftCubeFace::Back => Vec3::new(-x, y, 0.5),
        SoftCubeFace::Left => Vec3::new(-0.5, y, -x),
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

#[derive(Debug, Clone, Copy)]
struct AxisExposure {
    x_neg: bool,
    x_pos: bool,
    y_neg: bool,
    y_pos: bool,
    z_neg: bool,
    z_pos: bool,
}

impl AxisExposure {
    fn from_face(face: SoftCubeFace, edge: SoftCubeEdgeMask) -> Self {
        match face {
            SoftCubeFace::Top => Self {
                x_neg: edge.min_u,
                x_pos: edge.max_u,
                y_neg: false,
                y_pos: true,
                z_neg: edge.min_v,
                z_pos: edge.max_v,
            },
            SoftCubeFace::Bottom => Self {
                x_neg: edge.min_u,
                x_pos: edge.max_u,
                y_neg: true,
                y_pos: false,
                z_neg: edge.max_v,
                z_pos: edge.min_v,
            },
            SoftCubeFace::Front => Self {
                x_neg: edge.min_u,
                x_pos: edge.max_u,
                y_neg: edge.min_v,
                y_pos: edge.max_v,
                z_neg: true,
                z_pos: false,
            },
            SoftCubeFace::Back => Self {
                x_neg: edge.max_u,
                x_pos: edge.min_u,
                y_neg: edge.min_v,
                y_pos: edge.max_v,
                z_neg: false,
                z_pos: true,
            },
            SoftCubeFace::Left => Self {
                x_neg: true,
                x_pos: false,
                y_neg: edge.min_v,
                y_pos: edge.max_v,
                z_neg: edge.max_u,
                z_pos: edge.min_u,
            },
            SoftCubeFace::Right => Self {
                x_neg: false,
                x_pos: true,
                y_neg: edge.min_v,
                y_pos: edge.max_v,
                z_neg: edge.min_u,
                z_pos: edge.max_u,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RoundedSample {
    position: Vec3,
    normal: Vec3,
}

/// Rounded-box projection with edge awareness.
///
/// If a side is not exposed, that axis is not allowed to retract or curve.
fn rounded_box_project(p: Vec3, radius: f32, exposure: AxisExposure) -> RoundedSample {
    let inner = 0.5 - radius;

    let (cx, dx) = rounded_axis(p.x, inner, exposure.x_neg, exposure.x_pos);
    let (cy, dy) = rounded_axis(p.y, inner, exposure.y_neg, exposure.y_pos);
    let (cz, dz) = rounded_axis(p.z, inner, exposure.z_neg, exposure.z_pos);

    let clamped = Vec3::new(cx, cy, cz);
    let delta = Vec3::new(dx, dy, dz);

    if delta.length_squared() <= 1e-8 {
        return RoundedSample {
            position: p,
            normal: dominant_axis_normal(p),
        };
    }

    let normal = delta.normalize();
    let position = clamped + normal * radius;

    RoundedSample { position, normal }
}

fn rounded_axis(
    value: f32,
    inner: f32,
    expose_negative: bool,
    expose_positive: bool,
) -> (f32, f32) {
    if value > inner && expose_positive {
        return (inner, value - inner);
    }

    if value < -inner && expose_negative {
        return (-inner, value + inner);
    }

    (value, 0.0)
}

fn dominant_axis_normal(p: Vec3) -> Vec3 {
    let ax = p.x.abs();
    let ay = p.y.abs();
    let az = p.z.abs();

    if ay >= ax && ay >= az {
        return Vec3::Y * p.y.signum();
    }

    if ax >= az {
        return Vec3::X * p.x.signum();
    }

    Vec3::Z * p.z.signum()
}

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
