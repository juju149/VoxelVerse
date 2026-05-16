use super::Renderer;
use crate::Vertex;
use glam::Vec3;
use vv_math::CoordSystem;
use vv_voxel::VoxelCoord;
use vv_world::PlanetData;

const MAX_OVERLAY_BLOCKS: usize = 32;

impl<'a> Renderer<'a> {
    pub fn update_block_damage_overlay(
        &mut self,
        planet: &PlanetData,
        focused: Option<VoxelCoord>,
    ) {
        let (verts, inds) = build_block_damage_overlay_mesh(planet, focused);
        self.queue
            .write_buffer(&self.block_damage_v_buf, 0, bytemuck::cast_slice(&verts));
        self.queue
            .write_buffer(&self.block_damage_i_buf, 0, bytemuck::cast_slice(&inds));
        self.block_damage_inds = inds.len() as u32;
    }
}

pub(super) fn build_block_damage_overlay_mesh(
    planet: &PlanetData,
    focused: Option<VoxelCoord>,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut coords = Vec::new();
    if let Some(coord) = focused {
        coords.push(coord);
    }
    for (coord, _) in planet.block_damage.iter() {
        if Some(coord) != focused && coords.len() < MAX_OVERLAY_BLOCKS {
            coords.push(coord);
        }
    }

    let mut verts = Vec::with_capacity(coords.len() * 64);
    let mut inds = Vec::with_capacity(coords.len() * 64);
    for coord in coords {
        if let Some(fraction) = planet.block_damage_fraction(coord) {
            append_cracks_for_block(&mut verts, &mut inds, planet, coord, fraction);
        }
    }
    (verts, inds)
}

fn append_cracks_for_block(
    verts: &mut Vec<Vertex>,
    inds: &mut Vec<u32>,
    planet: &PlanetData,
    coord: VoxelCoord,
    fraction: f32,
) {
    let level = crack_level(fraction);
    if level == 0 {
        return;
    }

    for face in visible_overlay_faces() {
        let corners = face_corners(coord, face, planet);
        let normal = (corners[1] - corners[0])
            .cross(corners[2] - corners[0])
            .normalize_or_zero();
        let lift = normal * 0.018;
        for segment in crack_segments(level) {
            let a = bilerp(corners, segment[0], segment[1]) + lift;
            let b = bilerp(corners, segment[2], segment[3]) + lift;
            append_line(verts, inds, a, b, fraction);
        }
    }
}

pub(super) fn crack_level(fraction: f32) -> u8 {
    match fraction {
        f if f <= 0.2 => 0,
        f if f <= 0.4 => 1,
        f if f <= 0.6 => 2,
        f if f <= 0.8 => 3,
        _ => 4,
    }
}

fn visible_overlay_faces() -> [OverlayFace; 3] {
    [OverlayFace::Top, OverlayFace::Right, OverlayFace::Front]
}

#[derive(Clone, Copy)]
enum OverlayFace {
    Top,
    Right,
    Front,
}

fn face_corners(coord: VoxelCoord, face: OverlayFace, planet: &PlanetData) -> [Vec3; 4] {
    let p = |u: f32, v: f32, l: f32| {
        CoordSystem::get_vertex_pos_f32(
            coord.face,
            coord.u as f32 + u,
            coord.v as f32 + v,
            coord.layer as f32 + l,
            planet.profile,
        )
    };
    match face {
        OverlayFace::Top => [
            p(0.0, 0.0, 1.0),
            p(1.0, 0.0, 1.0),
            p(0.0, 1.0, 1.0),
            p(1.0, 1.0, 1.0),
        ],
        OverlayFace::Right => [
            p(1.0, 0.0, 0.0),
            p(1.0, 1.0, 0.0),
            p(1.0, 0.0, 1.0),
            p(1.0, 1.0, 1.0),
        ],
        OverlayFace::Front => [
            p(0.0, 1.0, 0.0),
            p(1.0, 1.0, 0.0),
            p(0.0, 1.0, 1.0),
            p(1.0, 1.0, 1.0),
        ],
    }
}

fn crack_segments(level: u8) -> &'static [[f32; 4]] {
    const L1: &[[f32; 4]] = &[[0.36, 0.52, 0.64, 0.47]];
    const L2: &[[f32; 4]] = &[[0.32, 0.48, 0.66, 0.50], [0.48, 0.50, 0.42, 0.28]];
    const L3: &[[f32; 4]] = &[
        [0.24, 0.46, 0.70, 0.52],
        [0.48, 0.51, 0.36, 0.22],
        [0.50, 0.51, 0.62, 0.78],
    ];
    const L4: &[[f32; 4]] = &[
        [0.18, 0.44, 0.74, 0.55],
        [0.46, 0.50, 0.30, 0.18],
        [0.52, 0.51, 0.70, 0.82],
        [0.40, 0.40, 0.66, 0.25],
    ];
    match level {
        0 => &[],
        1 => L1,
        2 => L2,
        3 => L3,
        _ => L4,
    }
}

fn bilerp(corners: [Vec3; 4], x: f32, y: f32) -> Vec3 {
    let a = corners[0].lerp(corners[1], x);
    let b = corners[2].lerp(corners[3], x);
    a.lerp(b, y)
}

fn append_line(verts: &mut Vec<Vertex>, inds: &mut Vec<u32>, a: Vec3, b: Vec3, fraction: f32) {
    let base = verts.len() as u32;
    let shade = (0.18 - 0.08 * fraction.clamp(0.0, 1.0)).max(0.05);
    let color = [shade, shade, shade];
    for p in [a, b] {
        verts.push(Vertex {
            pos: p.to_array(),
            uv: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            color,
            tex_index: 0,
        });
    }
    inds.extend_from_slice(&[base, base + 1]);
}

#[cfg(test)]
mod tests {
    use super::crack_level;

    #[test]
    fn low_fraction_has_no_visible_cracks() {
        assert_eq!(crack_level(0.1), 0);
    }

    #[test]
    fn stronger_damage_increases_level() {
        assert!(crack_level(0.85) > crack_level(0.35));
    }
}
