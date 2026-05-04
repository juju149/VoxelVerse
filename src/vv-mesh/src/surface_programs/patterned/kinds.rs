use vv_registry::{
    RUNTIME_PATTERN_CRACKED_CELLS, RUNTIME_PATTERN_NATURAL_CELLS, RUNTIME_PATTERN_RUNNING_BOND,
    RUNTIME_PATTERN_STRIPS,
};

use super::{config::PatternedMeshConfig, noise, PatternedCell};

pub(crate) fn build_cells(config: PatternedMeshConfig, face_seed: u32) -> Vec<PatternedCell> {
    match config.kind {
        RUNTIME_PATTERN_STRIPS => strips(config, face_seed),
        RUNTIME_PATTERN_RUNNING_BOND => running_bond(config, face_seed),
        RUNTIME_PATTERN_NATURAL_CELLS | RUNTIME_PATTERN_CRACKED_CELLS => {
            natural_cells(config, face_seed)
        }
        _ => grid(config, face_seed),
    }
}

fn grid(config: PatternedMeshConfig, face_seed: u32) -> Vec<PatternedCell> {
    let rows = config.rows.max(1);
    let columns = config.columns.max(1);
    let mut cells = Vec::with_capacity((rows * columns) as usize);

    for row in 0..rows {
        for column in 0..columns {
            cells.push(cell_in_grid(config, face_seed, row, column, 0.0, 0.0, 0.0));
        }
    }

    cells
}

fn running_bond(config: PatternedMeshConfig, face_seed: u32) -> Vec<PatternedCell> {
    let rows = config.rows.max(1);
    let columns = config.columns.max(1);
    let mut cells = Vec::with_capacity((rows * columns) as usize);

    for row in 0..rows {
        let stagger = if row % 2 == 0 { 0.0 } else { 0.5 };
        for column in 0..columns {
            cells.push(cell_in_grid(
                config, face_seed, row, column, stagger, 0.0, 0.0,
            ));
        }
    }

    cells
}

fn strips(config: PatternedMeshConfig, face_seed: u32) -> Vec<PatternedCell> {
    let columns = config.columns.max(1);
    let mut cells = Vec::with_capacity(columns as usize);

    for column in 0..columns {
        let u0 = column as f32 / columns as f32;
        let u1 = (column + 1) as f32 / columns as f32;
        cells.push(cell(config, face_seed, 0, column, [u0, 0.0], [u1, 1.0]));
    }

    cells
}

fn natural_cells(config: PatternedMeshConfig, face_seed: u32) -> Vec<PatternedCell> {
    let rows = config.rows.max(1);
    let columns = config.columns.max(1);
    let mut cells = Vec::with_capacity((rows * columns) as usize);

    for row in 0..rows {
        for column in 0..columns {
            let jx = (noise::hash01(face_seed, row, column, 11) - 0.5) * 0.10;
            let jy = (noise::hash01(face_seed, row, column, 17) - 0.5) * 0.10;
            let shrink = noise::hash01(face_seed, row, column, 23) * 0.020;
            cells.push(cell_in_grid(
                config,
                face_seed,
                row,
                column,
                0.0,
                jx,
                jy + shrink,
            ));
        }
    }

    cells
}

fn cell_in_grid(
    config: PatternedMeshConfig,
    face_seed: u32,
    row: u32,
    column: u32,
    stagger_x: f32,
    jitter_x: f32,
    jitter_y: f32,
) -> PatternedCell {
    let rows = config.rows.max(1);
    let columns = config.columns.max(1);

    let cell_w = 1.0 / columns as f32;
    let cell_h = 1.0 / rows as f32;

    let mut u0 = (column as f32 + stagger_x + jitter_x) * cell_w;
    let mut u1 = u0 + cell_w * (1.0 - jitter_y.abs() * 0.22);

    if u0 >= 1.0 {
        u0 -= 1.0;
        u1 -= 1.0;
    }

    let v0 = (row as f32 * cell_h + jitter_y * cell_h).clamp(0.0, 1.0);
    let v1 = ((row + 1) as f32 * cell_h + jitter_y * cell_h).clamp(0.0, 1.0);

    cell(
        config,
        face_seed,
        row,
        column,
        [u0.clamp(0.0, 1.0), v0],
        [u1.clamp(0.0, 1.0), v1],
    )
}

fn cell(
    config: PatternedMeshConfig,
    face_seed: u32,
    row: u32,
    column: u32,
    uv_min: [f32; 2],
    uv_max: [f32; 2],
) -> PatternedCell {
    let h = noise::hash01(face_seed ^ config.seed, row, column, config.kind);
    let height = (h - 0.5) * config.height_variation;
    let color_variation = (h - 0.5) * 2.0 * config.color_variation;

    PatternedCell {
        uv_min,
        uv_max,
        depth: config.gap_depth + height,
        bevel: config.cell_bevel,
        color_variation,
    }
}
