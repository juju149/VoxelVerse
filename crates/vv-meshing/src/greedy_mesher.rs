use crate::cpu_mesh::CpuVertex;
use crate::voxel_mesher::face_culling::{Coord, VoxelAccessor};
use crate::voxel_mesher::face_emitter::{grid_from_profile, QuadFace};
use crate::voxel_mesher::material_packing::{pack_material_edges, FaceEdgeMask};
use crate::voxel_mesher::MeshGen;
use std::collections::{HashMap, HashSet};
use vv_math::CoordSystem;
use vv_voxel::{PlanetProfile, VoxelId};

pub(crate) struct GreedyMesher;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum FaceDir {
    Top,
    Bottom,
    Front,
    Back,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct PlaneKey {
    dir: FaceDir,
    fixed: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct MergeKey {
    voxel: VoxelId,
    packed_tex_index: u32,
    color: [u8; 3],
    dir: FaceDir,
}

#[derive(Clone, Copy)]
struct FaceCell {
    a: u32,
    b: u32,
    coord: Coord,
    merge: MergeKey,
}

#[derive(Clone, Copy)]
struct QuadRun {
    cell: FaceCell,
    width: u32,
    height: u32,
}

impl GreedyMesher {
    pub fn append_opaque_cubes(
        accessor: &VoxelAccessor<'_>,
        candidates: &[Coord],
        profile: PlanetProfile,
        verts: &mut Vec<CpuVertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
    ) {
        let mut planes: HashMap<PlaneKey, Vec<FaceCell>> = HashMap::new();
        for &coord in candidates {
            if !accessor.uses_greedy(coord.layer, coord.u, coord.v) {
                continue;
            }
            collect_faces(accessor, coord, &mut planes);
        }
        for (plane, mut cells) in planes {
            cells.sort_by_key(|cell| (cell.b, cell.a));
            emit_plane(plane, &cells, profile, verts, inds, idx);
        }
    }
}

fn collect_faces(
    accessor: &VoxelAccessor<'_>,
    coord: Coord,
    planes: &mut HashMap<PlaneKey, Vec<FaceCell>>,
) {
    let check = |dl: i32, du: i32, dv: i32| -> bool {
        accessor.check_solid(
            coord.layer as i32 + dl,
            coord.u as i32 + du,
            coord.v as i32 + dv,
        )
    };

    push_face(accessor, coord, FaceDir::Top, !check(1, 0, 0), planes);
    push_face(accessor, coord, FaceDir::Bottom, !check(-1, 0, 0), planes);
    push_face(accessor, coord, FaceDir::Front, !check(0, 0, -1), planes);
    push_face(accessor, coord, FaceDir::Back, !check(0, 0, 1), planes);
    push_face(accessor, coord, FaceDir::Left, !check(0, -1, 0), planes);
    push_face(accessor, coord, FaceDir::Right, !check(0, 1, 0), planes);
}

fn push_face(
    accessor: &VoxelAccessor<'_>,
    coord: Coord,
    dir: FaceDir,
    visible: bool,
    planes: &mut HashMap<PlaneKey, Vec<FaceCell>>,
) {
    if !visible {
        return;
    }
    let Some((plane, a, b)) = face_plane_coord(coord, dir) else {
        return;
    };
    let merge = make_merge_key(accessor, coord, dir);
    planes
        .entry(plane)
        .or_default()
        .push(FaceCell { a, b, coord, merge });
}

fn emit_plane(
    plane: PlaneKey,
    cells: &[FaceCell],
    profile: PlanetProfile,
    verts: &mut Vec<CpuVertex>,
    inds: &mut Vec<u32>,
    idx: &mut u32,
) {
    let grid: HashMap<(u32, u32), FaceCell> =
        cells.iter().map(|cell| ((cell.a, cell.b), *cell)).collect();
    let mut visited = HashSet::with_capacity(cells.len());

    for cell in cells {
        if visited.contains(&(cell.a, cell.b)) {
            continue;
        }
        let width = run_width(cell, &grid, &visited);
        let height = run_height(cell, width, &grid, &visited);
        for db in 0..height {
            for da in 0..width {
                visited.insert((cell.a + da, cell.b + db));
            }
        }
        emit_quad(
            plane,
            QuadRun {
                cell: *cell,
                width,
                height,
            },
            profile,
            verts,
            inds,
            idx,
        );
    }
}

fn run_width(
    origin: &FaceCell,
    grid: &HashMap<(u32, u32), FaceCell>,
    visited: &HashSet<(u32, u32)>,
) -> u32 {
    let mut width = 1;
    loop {
        let key = (origin.a + width, origin.b);
        if visited.contains(&key) {
            break;
        }
        if grid.get(&key).is_some_and(|c| c.merge == origin.merge) {
            width += 1;
        } else {
            break;
        }
    }
    width
}

fn run_height(
    origin: &FaceCell,
    width: u32,
    grid: &HashMap<(u32, u32), FaceCell>,
    visited: &HashSet<(u32, u32)>,
) -> u32 {
    let mut height = 1;
    'h: loop {
        for da in 0..width {
            let key = (origin.a + da, origin.b + height);
            if visited.contains(&key) || !grid.get(&key).is_some_and(|c| c.merge == origin.merge) {
                break 'h;
            }
        }
        height += 1;
    }
    height
}

fn emit_quad(
    plane: PlaneKey,
    run: QuadRun,
    profile: PlanetProfile,
    verts: &mut Vec<CpuVertex>,
    inds: &mut Vec<u32>,
    idx: &mut u32,
) {
    let cell = run.cell;
    let grid = grid_from_profile(profile);
    let face = cell.coord.face;
    let p = |u: u32, v: u32, l: u32| CoordSystem::get_vertex_pos(face, u, v, l, grid);
    let a0 = cell.a;
    let a1 = cell.a + run.width;
    let b0 = cell.b;
    let b1 = cell.b + run.height;
    let f = plane.fixed;

    let pos = match plane.dir {
        FaceDir::Top => [p(a0, b0, f), p(a1, b0, f), p(a1, b1, f), p(a0, b1, f)],
        FaceDir::Bottom => [p(a0, b1, f), p(a1, b1, f), p(a1, b0, f), p(a0, b0, f)],
        FaceDir::Front => [p(a0, f, b0), p(a1, f, b0), p(a1, f, b1), p(a0, f, b1)],
        FaceDir::Back => [p(a1, f, b0), p(a0, f, b0), p(a0, f, b1), p(a1, f, b1)],
        FaceDir::Left => [p(f, a1, b0), p(f, a0, b0), p(f, a0, b1), p(f, a1, b1)],
        FaceDir::Right => [p(f, a0, b0), p(f, a1, b0), p(f, a1, b1), p(f, a0, b1)],
    };

    let color = decode_color(cell.merge.color);
    MeshGen::quad_tiled(
        verts,
        inds,
        idx,
        QuadFace {
            pos,
            colors: [color; 4],
            force_radial: matches!(plane.dir, FaceDir::Top),
            packed_tex_index: cell.merge.packed_tex_index,
            flip_u: false,
            flip_v: !matches!(plane.dir, FaceDir::Top),
        },
        [run.width as f32, run.height as f32],
    );
}

fn face_plane_coord(coord: Coord, dir: FaceDir) -> Option<(PlaneKey, u32, u32)> {
    let fixed = match dir {
        FaceDir::Top => coord.layer.checked_add(1)?,
        FaceDir::Bottom => coord.layer,
        FaceDir::Front => coord.v,
        FaceDir::Back => coord.v.checked_add(1)?,
        FaceDir::Left => coord.u,
        FaceDir::Right => coord.u.checked_add(1)?,
    };
    let (a, b) = match dir {
        FaceDir::Top | FaceDir::Bottom => (coord.u, coord.v),
        FaceDir::Front | FaceDir::Back => (coord.u, coord.layer),
        FaceDir::Left | FaceDir::Right => (coord.v, coord.layer),
    };
    Some((PlaneKey { dir, fixed }, a, b))
}

fn make_merge_key(accessor: &VoxelAccessor<'_>, coord: Coord, dir: FaceDir) -> MergeKey {
    let voxel = accessor.voxel_id(coord.layer, coord.u, coord.v);
    let visual = accessor.materials.visual(voxel);
    let layer_idx = match dir {
        FaceDir::Top => visual.layers.top,
        FaceDir::Bottom => visual.layers.bottom,
        FaceDir::Front => visual.layers.front,
        FaceDir::Back => visual.layers.back,
        FaceDir::Left => visual.layers.left,
        FaceDir::Right => visual.layers.right,
    };
    let natural_h = accessor.surface_height(coord.u, coord.v);
    let light = if coord.layer >= natural_h {
        1.0_f32
    } else if matches!(dir, FaceDir::Bottom) {
        0.4
    } else {
        0.8
    };
    let color = [
        visual.tint[0] * light,
        visual.tint[1] * light,
        visual.tint[2] * light,
    ];
    MergeKey {
        voxel,
        packed_tex_index: pack_material_edges(layer_idx, FaceEdgeMask::default()),
        color: encode_color(color),
        dir,
    }
}

fn encode_color(c: [f32; 3]) -> [u8; 3] {
    [
        (c[0] * 255.0) as u8,
        (c[1] * 255.0) as u8,
        (c[2] * 255.0) as u8,
    ]
}

fn decode_color(c: [u8; 3]) -> [f32; 3] {
    [
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
    ]
}
