use super::Renderer;
use crate::ui::{ComponentState, InventoryButton, InventoryUiState, UiColor, UiRect};
use crate::Vertex;
use vv_pack_compiler::BlockMaterialLayers;

// =============================================================================
// Pixel-level shape primitives (rounded rects, circles, lines, ...)
// =============================================================================

impl<'a> Renderer<'a> {
    /// Component-state mapping shared by all slot drawers.
    pub(super) fn slot_state(
        &self,
        content: Option<vv_gameplay::HotbarSlot>,
        visible: bool,
        hovered: bool,
        selected: bool,
    ) -> ComponentState {
        if !visible {
            return ComponentState::Disabled;
        }
        if selected {
            return ComponentState::Selected;
        }
        if hovered {
            return ComponentState::Hovered;
        }
        if content.is_none() {
            return ComponentState::Empty;
        }
        ComponentState::Normal
    }

    pub(super) fn inventory_button_state(
        &self,
        ui: &InventoryUiState,
        button: InventoryButton,
    ) -> ComponentState {
        if matches!(ui.hovered_button, Some(b) if b == button) {
            ComponentState::Hovered
        } else {
            ComponentState::Normal
        }
    }

    pub(super) fn fill_rect(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        color: UiColor,
    ) {
        self.add_ui_rect_rgba(
            verts,
            inds,
            rect.x,
            rect.y,
            rect.x + rect.w,
            rect.y + rect.h,
            color,
        );
    }

    pub(super) fn fill_rounded_rect(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        color: UiColor,
        radius: f32,
    ) {
        let r = radius.max(0.0).min(rect.w * 0.5).min(rect.h * 0.5);
        if r <= 0.5 {
            self.fill_rect(verts, inds, rect, color);
            return;
        }
        // Central horizontal band.
        self.fill_rect(
            verts,
            inds,
            UiRect {
                x: rect.x,
                y: rect.y + r,
                w: rect.w,
                h: rect.h - r * 2.0,
            },
            color,
        );
        // Top / bottom strips between the corners.
        self.fill_rect(
            verts,
            inds,
            UiRect {
                x: rect.x + r,
                y: rect.y,
                w: rect.w - r * 2.0,
                h: r,
            },
            color,
        );
        self.fill_rect(
            verts,
            inds,
            UiRect {
                x: rect.x + r,
                y: rect.y + rect.h - r,
                w: rect.w - r * 2.0,
                h: r,
            },
            color,
        );
        // 4 corner arcs (screen Y points down).
        let pi = std::f32::consts::PI;
        let segs = corner_segments(r);
        self.fan_arc(
            verts,
            inds,
            rect.x + r,
            rect.y + r,
            r,
            pi,
            1.5 * pi,
            segs,
            color,
        );
        self.fan_arc(
            verts,
            inds,
            rect.x + rect.w - r,
            rect.y + r,
            r,
            1.5 * pi,
            2.0 * pi,
            segs,
            color,
        );
        self.fan_arc(
            verts,
            inds,
            rect.x + rect.w - r,
            rect.y + rect.h - r,
            r,
            0.0,
            0.5 * pi,
            segs,
            color,
        );
        self.fan_arc(
            verts,
            inds,
            rect.x + r,
            rect.y + rect.h - r,
            r,
            0.5 * pi,
            pi,
            segs,
            color,
        );
    }

    /// Hollow rounded-rect outline. Four straight rim strips on the edges
    /// + four corner "ring arcs" (triangle strip between an outer arc and
    /// an inner arc). No body fill is drawn so the caller must paint the
    /// inside separately *before* stroking.
    pub(super) fn stroke_rounded_rect(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        color: UiColor,
        width: f32,
        radius: f32,
    ) {
        let w = width.max(1.0);
        let r = radius.max(0.0).min(rect.w * 0.5).min(rect.h * 0.5);
        let x0 = rect.x;
        let y0 = rect.y;
        let x1 = rect.x + rect.w;
        let y1 = rect.y + rect.h;

        // Edge strips (excluding corners).
        let middle_w = (rect.w - r * 2.0).max(0.0);
        let middle_h = (rect.h - r * 2.0).max(0.0);
        if middle_w > 0.0 {
            self.fill_rect(
                verts,
                inds,
                UiRect {
                    x: x0 + r,
                    y: y0,
                    w: middle_w,
                    h: w,
                },
                color,
            );
            self.fill_rect(
                verts,
                inds,
                UiRect {
                    x: x0 + r,
                    y: y1 - w,
                    w: middle_w,
                    h: w,
                },
                color,
            );
        }
        if middle_h > 0.0 {
            self.fill_rect(
                verts,
                inds,
                UiRect {
                    x: x0,
                    y: y0 + r,
                    w,
                    h: middle_h,
                },
                color,
            );
            self.fill_rect(
                verts,
                inds,
                UiRect {
                    x: x1 - w,
                    y: y0 + r,
                    w,
                    h: middle_h,
                },
                color,
            );
        }

        // Corner ring arcs.
        if r > 0.5 {
            let inner_r = (r - w).max(0.0);
            let pi = std::f32::consts::PI;
            self.ring_arc(verts, inds, x0 + r, y0 + r, r, inner_r, pi, 1.5 * pi, color);
            self.ring_arc(
                verts,
                inds,
                x1 - r,
                y0 + r,
                r,
                inner_r,
                1.5 * pi,
                2.0 * pi,
                color,
            );
            self.ring_arc(
                verts,
                inds,
                x1 - r,
                y1 - r,
                r,
                inner_r,
                0.0,
                0.5 * pi,
                color,
            );
            self.ring_arc(verts, inds, x0 + r, y1 - r, r, inner_r, 0.5 * pi, pi, color);
        }
    }

    fn ring_arc(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        cx: f32,
        cy: f32,
        r_outer: f32,
        r_inner: f32,
        a0: f32,
        a1: f32,
        color: UiColor,
    ) {
        let segments = corner_segments(r_outer).max(3);
        let mut prev_outer = self.push_vert(
            verts,
            cx + r_outer * a0.cos(),
            cy + r_outer * a0.sin(),
            color.rgb,
        );
        let mut prev_inner = self.push_vert(
            verts,
            cx + r_inner * a0.cos(),
            cy + r_inner * a0.sin(),
            color.rgb,
        );
        for i in 1..=segments {
            let angle = a0 + (a1 - a0) * (i as f32 / segments as f32);
            let cos = angle.cos();
            let sin = angle.sin();
            let next_outer =
                self.push_vert(verts, cx + r_outer * cos, cy + r_outer * sin, color.rgb);
            let next_inner =
                self.push_vert(verts, cx + r_inner * cos, cy + r_inner * sin, color.rgb);
            inds.extend([prev_outer, prev_inner, next_inner]);
            inds.extend([prev_outer, next_inner, next_outer]);
            prev_outer = next_outer;
            prev_inner = next_inner;
        }
    }

    pub(super) fn fan_arc(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        cx: f32,
        cy: f32,
        r: f32,
        a0: f32,
        a1: f32,
        segments: usize,
        color: UiColor,
    ) {
        let center = self.push_vert(verts, cx, cy, color.rgb);
        let mut prev = self.push_vert(verts, cx + r * a0.cos(), cy + r * a0.sin(), color.rgb);
        let segments = segments.max(2);
        for i in 1..=segments {
            let angle = a0 + (a1 - a0) * (i as f32 / segments as f32);
            let next = self.push_vert(verts, cx + r * angle.cos(), cy + r * angle.sin(), color.rgb);
            inds.extend([center, prev, next]);
            prev = next;
        }
    }

    pub(super) fn fill_circle(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        cx: f32,
        cy: f32,
        r: f32,
        color: UiColor,
    ) {
        let segments = corner_segments(r).max(10);
        let center = self.push_vert(verts, cx, cy, color.rgb);
        let mut prev = self.push_vert(verts, cx + r, cy, color.rgb);
        let two_pi = std::f32::consts::TAU;
        for i in 1..=segments {
            let angle = two_pi * (i as f32 / segments as f32);
            let next = self.push_vert(verts, cx + r * angle.cos(), cy + r * angle.sin(), color.rgb);
            inds.extend([center, prev, next]);
            prev = next;
        }
    }

    pub(super) fn stroke_circle(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        cx: f32,
        cy: f32,
        r: f32,
        width: f32,
        color: UiColor,
    ) {
        let segments = corner_segments(r).max(12);
        let two_pi = std::f32::consts::TAU;
        let r_outer = r;
        let r_inner = (r - width).max(0.0);
        let mut prev_outer = self.push_vert(verts, cx + r_outer, cy, color.rgb);
        let mut prev_inner = self.push_vert(verts, cx + r_inner, cy, color.rgb);
        for i in 1..=segments {
            let angle = two_pi * (i as f32 / segments as f32);
            let cos = angle.cos();
            let sin = angle.sin();
            let next_outer =
                self.push_vert(verts, cx + r_outer * cos, cy + r_outer * sin, color.rgb);
            let next_inner =
                self.push_vert(verts, cx + r_inner * cos, cy + r_inner * sin, color.rgb);
            inds.extend([prev_outer, prev_inner, next_inner]);
            inds.extend([prev_outer, next_inner, next_outer]);
            prev_outer = next_outer;
            prev_inner = next_inner;
        }
    }

    pub(super) fn draw_line_thick(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        thickness: f32,
        color: UiColor,
    ) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt().max(1e-3);
        let nx = -dy / len;
        let ny = dx / len;
        let half = thickness * 0.5;
        let (ax, ay) = (x0 + nx * half, y0 + ny * half);
        let (bx, by) = (x1 + nx * half, y1 + ny * half);
        let (cx, cy) = (x1 - nx * half, y1 - ny * half);
        let (dx2, dy2) = (x0 - nx * half, y0 - ny * half);
        self.add_ui_quad(
            verts,
            inds,
            (ax, ay),
            (bx, by),
            (cx, cy),
            (dx2, dy2),
            color.rgb,
        );
    }

    pub(super) fn draw_diagonal(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        rect: UiRect,
        inset: f32,
        thickness: f32,
        color: UiColor,
        flip: bool,
    ) {
        let (x0, y0, x1, y1) = if flip {
            (
                rect.x + rect.w - inset,
                rect.y + inset,
                rect.x + inset,
                rect.y + rect.h - inset,
            )
        } else {
            (
                rect.x + inset,
                rect.y + inset,
                rect.x + rect.w - inset,
                rect.y + rect.h - inset,
            )
        };
        self.draw_line_thick(verts, inds, x0, y0, x1, y1, thickness, color);
    }

    /// Draw an isometric voxel block using 3 shaded faces.
    ///
    /// When `texture` is `Some`, each face samples its assigned atlas layer
    /// and the vertex color carries pure grayscale shading. The UI shader
    /// expects 1-based atlas indices (0 = "no material"), so each layer is
    /// shifted by +1 at this boundary — the registry stores them 0-based.
    ///
    /// When `texture` is `None`, the cube is flat-colored from `base_rgb`.
    pub(super) fn draw_iso_block(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        slot: UiRect,
        base_rgb: [f32; 3],
        dim: f32,
        texture: Option<BlockMaterialLayers>,
    ) {
        let cx = slot.x + slot.w * 0.5;
        let cy = slot.y + slot.h * 0.5;
        let span = slot.w.min(slot.h) * 0.70;
        let u = span / 4.0;

        let textured = texture.is_some();
        let face_color = |factor: f32| -> [f32; 3] {
            let m = (factor * dim).clamp(0.0, 1.6);
            if textured {
                [m.min(1.0); 3]
            } else {
                [
                    (base_rgb[0] * m * 1.10).min(1.0),
                    (base_rgb[1] * m * 1.10).min(1.0),
                    (base_rgb[2] * m * 1.10).min(1.0),
                ]
            }
        };

        // UI shader is 1-based; world atlas index 0 → tex_index 1.
        let face_tex = |layer: u32| -> u32 {
            if textured {
                layer + 1
            } else {
                0
            }
        };

        let layers = texture.unwrap_or_default();
        let top_color = face_color(1.18);
        let left_color = face_color(0.70);
        let right_color = face_color(0.95);

        let p_top = (cx, cy - 2.0 * u);
        let p_right = (cx + 2.0 * u, cy - u);
        let p_front = (cx, cy);
        let p_left = (cx - 2.0 * u, cy - u);

        // Top face: p_top(TL) → p_right(TR) → p_front(BR) → p_left(BL)
        self.add_ui_quad_tex(
            verts,
            inds,
            [p_top, p_right, p_front, p_left],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            top_color,
            face_tex(layers.top),
        );

        let p_bottom = (cx, cy + 2.0 * u);
        let p_left_bottom = (cx - 2.0 * u, cy + u);

        // Left face uses the block's "left" (nx) material — the iso cube's
        // visible left side. UV is mapped so the texture's top edge sits at
        // the cube's top edge.
        self.add_ui_quad_tex(
            verts,
            inds,
            [p_left, p_front, p_bottom, p_left_bottom],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            left_color,
            face_tex(layers.left),
        );

        let p_right_bottom = (cx + 2.0 * u, cy + u);

        // Right face uses the block's "right" (px) material.
        self.add_ui_quad_tex(
            verts,
            inds,
            [p_right, p_right_bottom, p_bottom, p_front],
            [[1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]],
            right_color,
            face_tex(layers.right),
        );

        // Top-front rim highlight (always flat, no texture).
        let glint_color = [
            (base_rgb[0] * 1.35).min(1.0),
            (base_rgb[1] * 1.35).min(1.0),
            (base_rgb[2] * 1.35).min(1.0),
        ];
        let highlight_w = (u * 0.18).max(1.5);
        self.add_ui_quad(
            verts,
            inds,
            p_top,
            p_right,
            (p_right.0 - u * 0.08, p_right.1 + highlight_w * 0.6),
            (p_top.0 + u * 0.08, p_top.1 + highlight_w * 0.6),
            glint_color,
        );
    }

    /// Like `add_ui_quad` but each corner carries its own UV and a shared
    /// `tex_index` for atlas sampling. Use `tex_index = 0` to fall back to
    /// the vertex color (same sentinel as flat geometry).
    fn add_ui_quad_tex(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        corners: [(f32, f32); 4],
        uvs: [[f32; 2]; 4],
        rgb: [f32; 3],
        tex_index: u32,
    ) {
        let i0 = self.push_vert_uv(verts, corners[0].0, corners[0].1, rgb, uvs[0], tex_index);
        let i1 = self.push_vert_uv(verts, corners[1].0, corners[1].1, rgb, uvs[1], tex_index);
        let i2 = self.push_vert_uv(verts, corners[2].0, corners[2].1, rgb, uvs[2], tex_index);
        let i3 = self.push_vert_uv(verts, corners[3].0, corners[3].1, rgb, uvs[3], tex_index);
        inds.extend([i0, i1, i2, i0, i2, i3]);
    }

    fn push_vert_uv(
        &self,
        verts: &mut Vec<Vertex>,
        x: f32,
        y: f32,
        rgb: [f32; 3],
        uv: [f32; 2],
        tex_index: u32,
    ) -> u32 {
        let width = self.config.width.max(1) as f32;
        let height = self.config.height.max(1) as f32;
        let pos = [(x / width) * 2.0 - 1.0, 1.0 - (y / height) * 2.0, 0.0];
        let idx = verts.len() as u32;
        verts.push(Vertex {
            pos,
            uv,
            color: rgb,
            normal: [0.0, 0.0, 1.0],
            tex_index,
        });
        idx
    }

    pub(super) fn add_ui_quad(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        a: (f32, f32),
        b: (f32, f32),
        c: (f32, f32),
        d: (f32, f32),
        rgb: [f32; 3],
    ) {
        let i0 = self.push_vert(verts, a.0, a.1, rgb);
        let i1 = self.push_vert(verts, b.0, b.1, rgb);
        let i2 = self.push_vert(verts, c.0, c.1, rgb);
        let i3 = self.push_vert(verts, d.0, d.1, rgb);
        inds.extend([i0, i1, i2, i0, i2, i3]);
    }

    pub(super) fn add_ui_rect_rgba(
        &self,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: UiColor,
    ) {
        let rgb = color.rgb;
        let i0 = self.push_vert(verts, x0, y0, rgb);
        let i1 = self.push_vert(verts, x1, y0, rgb);
        let i2 = self.push_vert(verts, x0, y1, rgb);
        let i3 = self.push_vert(verts, x1, y1, rgb);
        inds.extend([i0, i2, i1, i1, i2, i3]);
    }

    fn push_vert(&self, verts: &mut Vec<Vertex>, x: f32, y: f32, rgb: [f32; 3]) -> u32 {
        let width = self.config.width.max(1) as f32;
        let height = self.config.height.max(1) as f32;
        let pos = [(x / width) * 2.0 - 1.0, 1.0 - (y / height) * 2.0, 0.0];
        let idx = verts.len() as u32;
        verts.push(Vertex {
            pos,
            uv: [0.0, 0.0],
            color: rgb,
            normal: [0.0, 0.0, 1.0],
            tex_index: 0,
        });
        idx
    }
}

/// Pick a triangle count for a rounded-corner arc. Bigger radii get more
/// segments so the silhouette stays smooth on 4K displays.
fn corner_segments(radius: f32) -> usize {
    if radius <= 4.0 {
        4
    } else if radius <= 10.0 {
        6
    } else if radius <= 20.0 {
        8
    } else {
        10
    }
}

/// Compute the three equipment-slot rects and their labels inside the given
/// left panel. Each slot is left-aligned with a right-side label. Used by
/// both the mesh drawer and the text-spec emitter so the positions always
/// agree.
pub(super) fn equip_slot_rects(left_panel: UiRect, scale: f32) -> [(UiRect, &'static str); 3] {
    let slot_size = (left_panel.w * 0.36).min(52.0 * scale);
    let gap = slot_size * 0.40;
    let total_h = slot_size * 3.0 + gap * 2.0;
    let top = left_panel.y + (left_panel.h - total_h) * 0.5;
    let left = left_panel.x + left_panel.w * 0.12;
    [
        (
            UiRect {
                x: left,
                y: top,
                w: slot_size,
                h: slot_size,
            },
            "Casque",
        ),
        (
            UiRect {
                x: left,
                y: top + slot_size + gap,
                w: slot_size,
                h: slot_size,
            },
            "Plastron",
        ),
        (
            UiRect {
                x: left,
                y: top + (slot_size + gap) * 2.0,
                w: slot_size,
                h: slot_size,
            },
            "Bottes",
        ),
    ]
}
