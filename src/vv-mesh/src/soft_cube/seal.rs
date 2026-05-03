use super::{SoftCubeEdgeMask, SoftCubeFace, SoftCubePoint};

/// Seals hidden edges so connected voxels remain perfectly closed.
///
/// A hidden edge must be 100% hard cube:
/// - exact border position
/// - exact face plane
/// - exact face normal
///
/// Without this, rounded cubes create tiny visual seams between neighboring blocks.
///
/// Only the in-plane coordinates are clamped here. The face-normal coordinate and
/// surface normal are left as computed by rounded_box_project, which already handles
/// mixed corners (one edge sealed, one exposed) correctly via the exposure flags.
pub(crate) fn seal_hidden_edges(
    face: SoftCubeFace,
    point: &mut SoftCubePoint,
    edge_mask: SoftCubeEdgeMask,
    u_index: u8,
    v_index: u8,
    segments: u8,
) {
    let on_min_u = u_index == 0;
    let on_max_u = u_index == segments;
    let on_min_v = v_index == 0;
    let on_max_v = v_index == segments;

    match face {
        SoftCubeFace::Top => seal_top(point, edge_mask, on_min_u, on_max_u, on_min_v, on_max_v),
        SoftCubeFace::Bottom => {
            seal_bottom(point, edge_mask, on_min_u, on_max_u, on_min_v, on_max_v)
        }
        SoftCubeFace::Front => seal_front(point, edge_mask, on_min_u, on_max_u, on_min_v, on_max_v),
        SoftCubeFace::Back => seal_back(point, edge_mask, on_min_u, on_max_u, on_min_v, on_max_v),
        SoftCubeFace::Left => seal_left(point, edge_mask, on_min_u, on_max_u, on_min_v, on_max_v),
        SoftCubeFace::Right => seal_right(point, edge_mask, on_min_u, on_max_u, on_min_v, on_max_v),
    }
}

fn seal_top(
    point: &mut SoftCubePoint,
    edge: SoftCubeEdgeMask,
    on_min_u: bool,
    on_max_u: bool,
    on_min_v: bool,
    on_max_v: bool,
) {
    if on_min_u && !edge.min_u {
        point.position.x = -0.5;
    }
    if on_max_u && !edge.max_u {
        point.position.x = 0.5;
    }
    if on_min_v && !edge.min_v {
        point.position.z = -0.5;
    }
    if on_max_v && !edge.max_v {
        point.position.z = 0.5;
    }
}

fn seal_bottom(
    point: &mut SoftCubePoint,
    edge: SoftCubeEdgeMask,
    on_min_u: bool,
    on_max_u: bool,
    on_min_v: bool,
    on_max_v: bool,
) {
    if on_min_u && !edge.min_u {
        point.position.x = -0.5;
    }
    if on_max_u && !edge.max_u {
        point.position.x = 0.5;
    }
    if on_min_v && !edge.min_v {
        point.position.z = 0.5;
    }
    if on_max_v && !edge.max_v {
        point.position.z = -0.5;
    }
}

fn seal_front(
    point: &mut SoftCubePoint,
    edge: SoftCubeEdgeMask,
    on_min_u: bool,
    on_max_u: bool,
    on_min_v: bool,
    on_max_v: bool,
) {
    if on_min_u && !edge.min_u {
        point.position.x = -0.5;
    }
    if on_max_u && !edge.max_u {
        point.position.x = 0.5;
    }
    if on_min_v && !edge.min_v {
        point.position.y = -0.5;
    }
    if on_max_v && !edge.max_v {
        point.position.y = 0.5;
    }
}

fn seal_back(
    point: &mut SoftCubePoint,
    edge: SoftCubeEdgeMask,
    on_min_u: bool,
    on_max_u: bool,
    on_min_v: bool,
    on_max_v: bool,
) {
    if on_min_u && !edge.min_u {
        point.position.x = 0.5;
    }
    if on_max_u && !edge.max_u {
        point.position.x = -0.5;
    }
    if on_min_v && !edge.min_v {
        point.position.y = -0.5;
    }
    if on_max_v && !edge.max_v {
        point.position.y = 0.5;
    }
}

fn seal_left(
    point: &mut SoftCubePoint,
    edge: SoftCubeEdgeMask,
    on_min_u: bool,
    on_max_u: bool,
    on_min_v: bool,
    on_max_v: bool,
) {
    if on_min_u && !edge.min_u {
        point.position.z = 0.5;
    }
    if on_max_u && !edge.max_u {
        point.position.z = -0.5;
    }
    if on_min_v && !edge.min_v {
        point.position.y = -0.5;
    }
    if on_max_v && !edge.max_v {
        point.position.y = 0.5;
    }
}

fn seal_right(
    point: &mut SoftCubePoint,
    edge: SoftCubeEdgeMask,
    on_min_u: bool,
    on_max_u: bool,
    on_min_v: bool,
    on_max_v: bool,
) {
    if on_min_u && !edge.min_u {
        point.position.z = -0.5;
    }
    if on_max_u && !edge.max_u {
        point.position.z = 0.5;
    }
    if on_min_v && !edge.min_v {
        point.position.y = -0.5;
    }
    if on_max_v && !edge.max_v {
        point.position.y = 0.5;
    }
}
