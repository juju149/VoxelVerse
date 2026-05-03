use vv_ui::{
    UiBorder, UiColor, UiCommand, UiFrame, UiGradient, UiLayer, UiRect, UiShadow, UiTextCommand,
};

use super::{UiTextItem, UiVertex};

#[derive(Debug, Clone, Default)]
pub struct UiMesh {
    pub vertices: Vec<UiVertex>,
    pub indices: Vec<u32>,
    pub text_items: Vec<UiTextItem>,
}

impl UiMesh {
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.text_items.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() && self.indices.is_empty() && self.text_items.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct UiMeshBuilder {
    screen: UiRect,
    clip_stack: Vec<UiRect>,
    mesh: UiMesh,
}

impl UiMeshBuilder {
    pub fn new(screen: UiRect) -> Self {
        Self {
            screen,
            clip_stack: Vec::new(),
            mesh: UiMesh::default(),
        }
    }

    pub fn build(frame: &UiFrame) -> UiMesh {
        let mut builder = Self::new(frame.screen());

        for command in frame.commands() {
            builder.push_command(command);
        }

        builder.mesh
    }

    fn push_command(&mut self, command: &UiCommand) {
        match command {
            UiCommand::Rect {
                layer: _,
                rect,
                color,
                radius,
                border,
                shadow,
            } => {
                self.push_shadow(*rect, *shadow);
                self.push_solid_rect(*rect, *color, *radius);
                self.push_border(*rect, *border);
            }
            UiCommand::GradientRect {
                layer: _,
                rect,
                gradient,
                radius,
                border,
                shadow,
            } => {
                self.push_shadow(*rect, *shadow);
                self.push_gradient_rect(*rect, *gradient, *radius);
                self.push_border(*rect, *border);
            }
            UiCommand::Image {
                layer: _,
                rect,
                image: _,
                tint,
                radius,
            } => {
                self.push_solid_rect(*rect, *tint, *radius);
            }
            UiCommand::Icon {
                layer: _,
                rect,
                icon: _,
                color,
            } => {
                self.push_solid_rect(*rect, *color, 0.0);
            }
            UiCommand::Text { layer, command } => {
                self.push_text(*layer, command.clone());
            }
            UiCommand::ClipStart { layer: _, rect } => {
                let next = match self.current_clip() {
                    Some(current) => intersect_rect(current, *rect),
                    None => Some(*rect),
                };

                if let Some(next) = next {
                    self.clip_stack.push(next);
                } else {
                    self.clip_stack.push(UiRect::ZERO);
                }
            }
            UiCommand::ClipEnd { layer: _ } => {
                self.clip_stack.pop();
            }
        }
    }

    fn push_text(&mut self, layer: UiLayer, command: UiTextCommand) {
        let Some(rect) = self.apply_clip(command.rect) else {
            return;
        };

        let clipped = UiTextCommand { rect, ..command };
        self.mesh.text_items.push(UiTextItem::new(layer, clipped));
    }

    fn push_shadow(&mut self, rect: UiRect, shadow: UiShadow) {
        if shadow.color.a <= 0.001 {
            return;
        }

        let shadow_rect = rect
            .expand(shadow.spread + shadow.blur * 0.12)
            .translate(shadow.offset_x, shadow.offset_y);

        self.push_solid_rect(shadow_rect, shadow.color, 0.0);
    }

    fn push_border(&mut self, rect: UiRect, border: UiBorder) {
        if border.width <= 0.0 || border.color.a <= 0.001 {
            return;
        }

        let w = border.width.max(1.0);

        let top = UiRect::new(rect.x, rect.y, rect.width, w);
        let bottom = UiRect::new(rect.x, rect.bottom() - w, rect.width, w);
        let left = UiRect::new(rect.x, rect.y, w, rect.height);
        let right = UiRect::new(rect.right() - w, rect.y, w, rect.height);

        self.push_solid_rect(top, border.color, 0.0);
        self.push_solid_rect(bottom, border.color, 0.0);
        self.push_solid_rect(left, border.color, 0.0);
        self.push_solid_rect(right, border.color, 0.0);
    }

    fn push_solid_rect(&mut self, rect: UiRect, color: UiColor, radius: f32) {
        self.push_gradient_rect(rect, UiGradient::solid(color), radius);
    }

    fn push_gradient_rect(&mut self, rect: UiRect, gradient: UiGradient, radius: f32) {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }

        let Some(rect) = self.apply_clip(rect) else {
            return;
        };

        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }

        let base = self.mesh.vertices.len() as u32;

        let x0 = rect.x;
        let x1 = rect.right();
        let y0 = rect.y;
        let y1 = rect.bottom();

        let p0 = self.to_clip(x0, y0);
        let p1 = self.to_clip(x1, y0);
        let p2 = self.to_clip(x1, y1);
        let p3 = self.to_clip(x0, y1);

        let params = [radius, rect.width, rect.height, 0.0];

        self.mesh.vertices.extend_from_slice(&[
            UiVertex::new(p0, [0.0, 0.0], gradient.top.to_array(), params),
            UiVertex::new(p1, [1.0, 0.0], gradient.top.to_array(), params),
            UiVertex::new(p2, [1.0, 1.0], gradient.bottom.to_array(), params),
            UiVertex::new(p3, [0.0, 1.0], gradient.bottom.to_array(), params),
        ]);

        self.mesh
            .indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn to_clip(&self, x: f32, y: f32) -> [f32; 2] {
        let screen_w = self.screen.width.max(1.0);
        let screen_h = self.screen.height.max(1.0);

        [x / screen_w * 2.0 - 1.0, 1.0 - y / screen_h * 2.0]
    }

    fn current_clip(&self) -> Option<UiRect> {
        self.clip_stack.last().copied()
    }

    fn apply_clip(&self, rect: UiRect) -> Option<UiRect> {
        match self.current_clip() {
            Some(clip) => intersect_rect(rect, clip),
            None => Some(rect),
        }
    }
}

fn intersect_rect(a: UiRect, b: UiRect) -> Option<UiRect> {
    let x0 = a.left().max(b.left());
    let y0 = a.top().max(b.top());
    let x1 = a.right().min(b.right());
    let y1 = a.bottom().min(b.bottom());

    if x1 <= x0 || y1 <= y0 {
        return None;
    }

    Some(UiRect::new(x0, y0, x1 - x0, y1 - y0))
}
