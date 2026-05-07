use super::Renderer;
use crate::rendering::Vertex;

impl<'a> Renderer<'a> {
    pub fn update_console_mesh(&mut self, t: f32) {
        if t <= 0.001 {
            self.console_inds = 0;
            return;
        }

        let height = t * 1.0;
        let bottom_y = 1.0 - height;

        let color = [0.1, 0.1, 0.15];
        let normal = [0.0, 0.0, 1.0];

        let verts = vec![
            Vertex {
                pos: [-1.0, 1.0, 0.0],
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
            Vertex {
                pos: [1.0, 1.0, 0.0],
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
            Vertex {
                pos: [-1.0, bottom_y, 0.0],
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
            Vertex {
                pos: [1.0, bottom_y, 0.0],
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
        ];

        let inds = vec![0, 2, 1, 1, 2, 3];

        self.queue
            .write_buffer(&self.console_v_buf, 0, bytemuck::cast_slice(&verts));
        self.queue
            .write_buffer(&self.console_i_buf, 0, bytemuck::cast_slice(&inds));
        self.console_inds = inds.len() as u32;
    }
}
