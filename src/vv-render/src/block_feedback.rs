use glam::Vec3;
use vv_core::BlockId;
use vv_mesh::Vertex;
use vv_planet::{CoordSystem, PlanetGeometry};
use vv_world_runtime::PlanetData;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SelectionOutlineStyle {
    pub(crate) radius: f32,
    pub(crate) inset: f32,
    pub(crate) lift: f32,
    pub(crate) color: [f32; 3],
}

impl Default for SelectionOutlineStyle {
    fn default() -> Self {
        Self {
            radius: 0.006,
            inset: 0.012,
            lift: 0.006,
            color: [0.72, 0.92, 1.0],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BlockBreakStyle {
    pub(crate) min_radius: f32,
    pub(crate) max_radius: f32,
    pub(crate) lift: f32,
    pub(crate) color: [f32; 3],
}

impl Default for BlockBreakStyle {
    fn default() -> Self {
        Self {
            min_radius: 0.004,
            max_radius: 0.010,
            lift: 0.010,
            color: [0.08, 0.105, 0.12],
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct FeedbackMesh {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
}

pub(crate) fn selection_outline_mesh(
    planet: &PlanetData,
    id: BlockId,
    style: SelectionOutlineStyle,
) -> FeedbackMesh {
    let corners = block_corners(id, planet.geometry, style.inset);
    let edges = [
        (0, 1),
        (1, 3),
        (3, 2),
        (2, 0),
        (4, 5),
        (5, 7),
        (7, 6),
        (6, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    let block_center = corners.iter().copied().sum::<Vec3>() / corners.len() as f32;
    let mut mesh = FeedbackMesh::default();
    let mut idx = 0u32;
    for (start, end) in edges {
        let a = corners[start];
        let b = corners[end];
        let normal = ((a + b) * 0.5 - block_center).normalize_or_zero();
        push_segment(
            &mut mesh.vertices,
            &mut mesh.indices,
            &mut idx,
            a + normal * style.lift,
            b + normal * style.lift,
            style.radius,
            style.color,
        );
    }

    mesh
}

pub(crate) fn block_break_mesh(
    planet: &PlanetData,
    id: BlockId,
    progress: f32,
    style: BlockBreakStyle,
) -> FeedbackMesh {
    if progress <= 0.01 || !planet.exists(id) {
        return FeedbackMesh::default();
    }

    let progress = progress.clamp(0.0, 1.0);
    let eased = 1.0 - (1.0 - progress) * (1.0 - progress);
    let corners = block_corners(id, planet.geometry, 0.018);
    let faces = [
        [4, 5, 7, 6],
        [0, 1, 3, 2],
        [0, 4, 6, 2],
        [1, 5, 7, 3],
        [2, 3, 7, 6],
        [0, 1, 5, 4],
    ];
    let strokes: [((f32, f32), (f32, f32)); 14] = [
        ((0.50, 0.50), (0.32, 0.36)),
        ((0.50, 0.50), (0.68, 0.38)),
        ((0.50, 0.50), (0.43, 0.67)),
        ((0.50, 0.50), (0.62, 0.66)),
        ((0.32, 0.36), (0.22, 0.29)),
        ((0.32, 0.36), (0.24, 0.48)),
        ((0.68, 0.38), (0.80, 0.31)),
        ((0.68, 0.38), (0.78, 0.51)),
        ((0.43, 0.67), (0.31, 0.78)),
        ((0.43, 0.67), (0.50, 0.82)),
        ((0.62, 0.66), (0.74, 0.76)),
        ((0.24, 0.48), (0.15, 0.58)),
        ((0.80, 0.31), (0.89, 0.24)),
        ((0.74, 0.76), (0.86, 0.84)),
    ];
    let visible = ((eased * strokes.len() as f32).ceil() as usize).clamp(1, strokes.len());
    let radius = style.min_radius + (style.max_radius - style.min_radius) * progress;

    let mut mesh = FeedbackMesh::default();
    let mut idx = 0u32;
    for face in faces {
        let face_corners = [
            corners[face[0]],
            corners[face[1]],
            corners[face[2]],
            corners[face[3]],
        ];
        let normal = (face_corners[1] - face_corners[0])
            .cross(face_corners[2] - face_corners[0])
            .normalize_or_zero();
        for ((sx, sy), (ex, ey)) in strokes.iter().take(visible) {
            let start = face_point(face_corners, *sx, *sy);
            let raw_end = face_point(face_corners, *ex, *ey);
            let end = start + (raw_end - start) * eased.clamp(0.18, 1.0);
            push_segment(
                &mut mesh.vertices,
                &mut mesh.indices,
                &mut idx,
                start + normal * style.lift,
                end + normal * style.lift,
                radius,
                style.color,
            );
        }
    }

    mesh
}

fn block_corners(id: BlockId, geometry: PlanetGeometry, inset: f32) -> [Vec3; 8] {
    let p =
        |u, v, l| CoordSystem::get_vertex_pos(id.face, id.u + u, id.v + v, id.layer + l, geometry);
    let raw = [
        p(0, 0, 0),
        p(1, 0, 0),
        p(0, 1, 0),
        p(1, 1, 0),
        p(0, 0, 1),
        p(1, 0, 1),
        p(0, 1, 1),
        p(1, 1, 1),
    ];
    let center = raw.iter().copied().sum::<Vec3>() / raw.len() as f32;
    raw.map(|corner| corner.lerp(center, inset))
}

fn face_point(corners: [Vec3; 4], u: f32, v: f32) -> Vec3 {
    let bottom = corners[0].lerp(corners[1], u);
    let top = corners[3].lerp(corners[2], u);
    bottom.lerp(top, v)
}

fn push_segment(
    verts: &mut Vec<Vertex>,
    inds: &mut Vec<u32>,
    idx: &mut u32,
    a: Vec3,
    b: Vec3,
    radius: f32,
    color: [f32; 3],
) {
    let dir = b - a;
    if dir.length_squared() <= f32::EPSILON {
        return;
    }

    let dir = dir.normalize();
    let ref_up = if dir.dot(Vec3::Y).abs() > 0.9 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let right = dir.cross(ref_up).normalize_or_zero() * radius;
    let up = dir.cross(right).normalize_or_zero() * radius;
    let base = *idx;
    for off in [-right - up, right - up, right + up, -right + up] {
        let normal = off.normalize_or_zero().to_array();
        verts.push(Vertex::untextured((a + off).to_array(), color, normal));
        verts.push(Vertex::untextured((b + off).to_array(), color, normal));
    }
    for (i0, i1, i2, i3) in [(0u32, 1, 3, 2), (2, 3, 5, 4), (4, 5, 7, 6), (6, 7, 1, 0)] {
        inds.push(base + i0);
        inds.push(base + i1);
        inds.push(base + i2);
        inds.push(base + i2);
        inds.push(base + i3);
        inds.push(base + i0);
    }
    *idx += 8;
}
