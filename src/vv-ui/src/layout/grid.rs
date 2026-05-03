use crate::{UiEdgeInsets, UiRect};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiGridLayout {
    pub bounds: UiRect,
    pub columns: usize,
    pub rows: usize,
    pub gap_x: f32,
    pub gap_y: f32,
    pub padding: UiEdgeInsets,
}

impl UiGridLayout {
    pub fn new(bounds: UiRect, columns: usize, rows: usize) -> Self {
        Self {
            bounds,
            columns,
            rows,
            gap_x: 0.0,
            gap_y: 0.0,
            padding: UiEdgeInsets::ZERO,
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        let gap = gap.max(0.0);
        self.gap_x = gap;
        self.gap_y = gap;
        self
    }

    pub fn gap_xy(mut self, gap_x: f32, gap_y: f32) -> Self {
        self.gap_x = gap_x.max(0.0);
        self.gap_y = gap_y.max(0.0);
        self
    }

    pub fn padding(mut self, padding: UiEdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    pub fn cell(self, column: usize, row: usize) -> Option<UiRect> {
        if self.columns == 0 || self.rows == 0 || column >= self.columns || row >= self.rows {
            return None;
        }

        let inner = self.bounds.inset(self.padding);
        let total_gap_x = self.gap_x * self.columns.saturating_sub(1) as f32;
        let total_gap_y = self.gap_y * self.rows.saturating_sub(1) as f32;
        let cell_w = ((inner.width - total_gap_x) / self.columns as f32).max(0.0);
        let cell_h = ((inner.height - total_gap_y) / self.rows as f32).max(0.0);

        Some(UiRect::new(
            inner.x + column as f32 * (cell_w + self.gap_x),
            inner.y + row as f32 * (cell_h + self.gap_y),
            cell_w,
            cell_h,
        ))
    }

    pub fn cells(self) -> Vec<UiRect> {
        let mut cells = Vec::with_capacity(self.columns * self.rows);
        for row in 0..self.rows {
            for column in 0..self.columns {
                if let Some(cell) = self.cell(column, row) {
                    cells.push(cell);
                }
            }
        }
        cells
    }
}
