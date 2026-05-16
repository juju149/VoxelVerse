use super::hand_animation::{HandAnimation, HandPose};
use super::Renderer;
use crate::Vertex;
use vv_pack_compiler::{CompiledItem, CompiledItemGameplay};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerActionFeedback {
    Swing { strength: f32 },
    Hit { strength: f32 },
    Break { strength: f32 },
    Place,
}

impl<'a> Renderer<'a> {
    pub fn notify_player_action(&mut self, feedback: PlayerActionFeedback) {
        match feedback {
            PlayerActionFeedback::Swing { strength } => self.first_person_animation.swing(strength),
            PlayerActionFeedback::Hit { strength } => {
                self.first_person_animation.hit_recoil(strength)
            }
            PlayerActionFeedback::Break { strength } => {
                self.first_person_animation.break_accent(strength)
            }
            PlayerActionFeedback::Place => self.first_person_animation.swing(0.65),
        }
    }

    pub fn update_first_person_item(&mut self, dt: f32, selected_item: Option<&CompiledItem>) {
        self.first_person_animation.update(dt);
        let (verts, inds) =
            build_first_person_item_mesh(&self.first_person_animation, selected_item);
        self.queue
            .write_buffer(&self.first_person_v_buf, 0, bytemuck::cast_slice(&verts));
        self.queue
            .write_buffer(&self.first_person_i_buf, 0, bytemuck::cast_slice(&inds));
        self.first_person_inds = inds.len() as u32;
    }
}

pub(super) fn build_first_person_item_mesh(
    animation: &HandAnimation,
    selected_item: Option<&CompiledItem>,
) -> (Vec<Vertex>, Vec<u32>) {
    let pose = animation.pose();
    let mut verts = Vec::with_capacity(32);
    let mut inds = Vec::with_capacity(48);

    draw_rotated_rect(
        &mut verts,
        &mut inds,
        RectSpec {
            center: transform([0.78, -0.78], pose),
            size: [0.12 * pose.scale, 0.23 * pose.scale],
            rotation: pose.rotation - 0.18,
            color: [0.72, 0.50, 0.34],
        },
    );

    match selected_item.map(|item| &item.gameplay) {
        Some(CompiledItemGameplay::Tool(_)) => draw_tool(&mut verts, &mut inds, pose),
        Some(CompiledItemGameplay::PlaceBlock { .. }) => {
            draw_block_placeholder(&mut verts, &mut inds, pose)
        }
        Some(_) | None => {}
    }

    (verts, inds)
}

fn draw_tool(verts: &mut Vec<Vertex>, inds: &mut Vec<u32>, pose: HandPose) {
    let rot = pose.rotation - 0.78;
    draw_rotated_rect(
        verts,
        inds,
        RectSpec {
            center: transform([0.68, -0.68], pose),
            size: [0.035 * pose.scale, 0.36 * pose.scale],
            rotation: rot,
            color: [0.42, 0.27, 0.15],
        },
    );
    draw_rotated_rect(
        verts,
        inds,
        RectSpec {
            center: transform([0.61, -0.49], pose),
            size: [0.18 * pose.scale, 0.055 * pose.scale],
            rotation: rot,
            color: [0.58, 0.62, 0.64],
        },
    );
}

fn draw_block_placeholder(verts: &mut Vec<Vertex>, inds: &mut Vec<u32>, pose: HandPose) {
    draw_rotated_rect(
        verts,
        inds,
        RectSpec {
            center: transform([0.69, -0.66], pose),
            size: [0.13 * pose.scale, 0.13 * pose.scale],
            rotation: pose.rotation - 0.2,
            color: [0.42, 0.56, 0.36],
        },
    );
}

#[derive(Clone, Copy)]
struct RectSpec {
    center: [f32; 2],
    size: [f32; 2],
    rotation: f32,
    color: [f32; 3],
}

fn draw_rotated_rect(verts: &mut Vec<Vertex>, inds: &mut Vec<u32>, spec: RectSpec) {
    let base = verts.len() as u32;
    let hx = spec.size[0] * 0.5;
    let hy = spec.size[1] * 0.5;
    let (s, c) = spec.rotation.sin_cos();
    let points = [[-hx, -hy], [hx, -hy], [hx, hy], [-hx, hy]];
    for [x, y] in points {
        let px = spec.center[0] + x * c - y * s;
        let py = spec.center[1] + x * s + y * c;
        verts.push(Vertex {
            pos: [px, py, 0.0],
            uv: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            color: spec.color,
            tex_index: 0,
        });
    }
    inds.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn transform(base: [f32; 2], pose: HandPose) -> [f32; 2] {
    [base[0] + pose.offset[0], base[1] + pose.offset[1]]
}

#[cfg(test)]
mod tests {
    use super::build_first_person_item_mesh;
    use crate::renderer::hand_animation::HandAnimation;

    #[test]
    fn empty_hand_still_draws_hand() {
        let animation = HandAnimation::new();
        let (_, inds) = build_first_person_item_mesh(&animation, None);

        assert!(!inds.is_empty());
    }
}
