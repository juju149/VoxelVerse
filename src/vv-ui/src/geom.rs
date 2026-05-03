#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UiPoint {
    pub x: f32,
    pub y: f32,
}

impl UiPoint {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UiSize {
    pub width: f32,
    pub height: f32,
}

impl UiSize {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn min_side(self) -> f32 {
        self.width.min(self.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl UiRect {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_min_size(min: UiPoint, size: UiSize) -> Self {
        Self::new(min.x, min.y, size.width, size.height)
    }

    pub fn left(self) -> f32 {
        self.x
    }

    pub fn right(self) -> f32 {
        self.x + self.width
    }

    pub fn top(self) -> f32 {
        self.y
    }

    pub fn bottom(self) -> f32 {
        self.y + self.height
    }

    pub fn center(self) -> UiPoint {
        UiPoint::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    pub fn size(self) -> UiSize {
        UiSize::new(self.width, self.height)
    }

    pub fn contains(self, point: UiPoint) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }

    pub fn inset(self, insets: UiEdgeInsets) -> Self {
        Self::new(
            self.x + insets.left,
            self.y + insets.top,
            (self.width - insets.horizontal()).max(0.0),
            (self.height - insets.vertical()).max(0.0),
        )
    }

    pub fn expand(self, amount: f32) -> Self {
        Self::new(
            self.x - amount,
            self.y - amount,
            self.width + amount * 2.0,
            self.height + amount * 2.0,
        )
    }

    pub fn translate(self, dx: f32, dy: f32) -> Self {
        Self::new(self.x + dx, self.y + dy, self.width, self.height)
    }

    pub fn split_left(self, width: f32) -> (Self, Self) {
        let left_width = width.clamp(0.0, self.width);
        let left = Self::new(self.x, self.y, left_width, self.height);
        let right = Self::new(
            self.x + left_width,
            self.y,
            self.width - left_width,
            self.height,
        );
        (left, right)
    }

    pub fn split_right(self, width: f32) -> (Self, Self) {
        let right_width = width.clamp(0.0, self.width);
        let left = Self::new(self.x, self.y, self.width - right_width, self.height);
        let right = Self::new(
            self.x + self.width - right_width,
            self.y,
            right_width,
            self.height,
        );
        (left, right)
    }

    pub fn split_top(self, height: f32) -> (Self, Self) {
        let top_height = height.clamp(0.0, self.height);
        let top = Self::new(self.x, self.y, self.width, top_height);
        let bottom = Self::new(
            self.x,
            self.y + top_height,
            self.width,
            self.height - top_height,
        );
        (top, bottom)
    }

    pub fn split_bottom(self, height: f32) -> (Self, Self) {
        let bottom_height = height.clamp(0.0, self.height);
        let top = Self::new(self.x, self.y, self.width, self.height - bottom_height);
        let bottom = Self::new(
            self.x,
            self.y + self.height - bottom_height,
            self.width,
            bottom_height,
        );
        (top, bottom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UiEdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl UiEdgeInsets {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}
