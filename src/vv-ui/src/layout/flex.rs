use crate::{UiEdgeInsets, UiRect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiFlexDirection {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiFlexLayout {
    pub bounds: UiRect,
    pub direction: UiFlexDirection,
    pub gap: f32,
    pub padding: UiEdgeInsets,
}

impl UiFlexLayout {
    pub fn row(bounds: UiRect) -> Self {
        Self {
            bounds,
            direction: UiFlexDirection::Row,
            gap: 0.0,
            padding: UiEdgeInsets::ZERO,
        }
    }

    pub fn column(bounds: UiRect) -> Self {
        Self {
            bounds,
            direction: UiFlexDirection::Column,
            gap: 0.0,
            padding: UiEdgeInsets::ZERO,
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    pub fn padding(mut self, padding: UiEdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    pub fn fixed(self, sizes: &[f32]) -> Vec<UiRect> {
        let inner = self.bounds.inset(self.padding);
        let mut rects = Vec::with_capacity(sizes.len());
        let mut cursor = match self.direction {
            UiFlexDirection::Row => inner.x,
            UiFlexDirection::Column => inner.y,
        };

        for size in sizes {
            let size = (*size).max(0.0);
            let rect = match self.direction {
                UiFlexDirection::Row => {
                    let rect = UiRect::new(cursor, inner.y, size, inner.height);
                    cursor += size + self.gap;
                    rect
                }
                UiFlexDirection::Column => {
                    let rect = UiRect::new(inner.x, cursor, inner.width, size);
                    cursor += size + self.gap;
                    rect
                }
            };
            rects.push(rect);
        }

        rects
    }

    pub fn equal(self, count: usize) -> Vec<UiRect> {
        if count == 0 {
            return Vec::new();
        }

        let inner = self.bounds.inset(self.padding);
        let total_gap = self.gap * count.saturating_sub(1) as f32;

        match self.direction {
            UiFlexDirection::Row => {
                let width = ((inner.width - total_gap) / count as f32).max(0.0);
                (0..count)
                    .map(|i| {
                        UiRect::new(
                            inner.x + i as f32 * (width + self.gap),
                            inner.y,
                            width,
                            inner.height,
                        )
                    })
                    .collect()
            }
            UiFlexDirection::Column => {
                let height = ((inner.height - total_gap) / count as f32).max(0.0);
                (0..count)
                    .map(|i| {
                        UiRect::new(
                            inner.x,
                            inner.y + i as f32 * (height + self.gap),
                            inner.width,
                            height,
                        )
                    })
                    .collect()
            }
        }
    }
}
