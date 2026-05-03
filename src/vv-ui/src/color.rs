#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl UiColor {
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgba(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::rgba(0.0, 0.0, 0.0, 1.0);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    pub fn multiply_alpha(self, factor: f32) -> Self {
        Self {
            a: self.a * factor,
            ..self
        }
    }

    pub fn lighten(self, amount: f32) -> Self {
        let t = amount.clamp(0.0, 1.0);
        Self {
            r: self.r + (1.0 - self.r) * t,
            g: self.g + (1.0 - self.g) * t,
            b: self.b + (1.0 - self.b) * t,
            a: self.a,
        }
    }

    pub fn darken(self, amount: f32) -> Self {
        let t = 1.0 - amount.clamp(0.0, 1.0);
        Self {
            r: self.r * t,
            g: self.g * t,
            b: self.b * t,
            a: self.a,
        }
    }

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Default for UiColor {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiGradient {
    pub top: UiColor,
    pub bottom: UiColor,
}

impl UiGradient {
    pub const fn vertical(top: UiColor, bottom: UiColor) -> Self {
        Self { top, bottom }
    }

    pub const fn solid(color: UiColor) -> Self {
        Self {
            top: color,
            bottom: color,
        }
    }
}
