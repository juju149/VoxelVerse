use glam::Vec3;

use vv_gameplay::PlayerGameplayState;
use vv_mesh::Vertex;
use vv_registry::{BlockRenderSource, CompiledContent, CompiledItemKind, ItemId};
use vv_world_runtime::PlanetData;

use crate::block_feedback::{block_break_mesh, BlockBreakStyle};

use super::Renderer;

impl<'a> Renderer<'a> {
    pub(super) fn update_block_break_feedback(
        &mut self,
        planet: &PlanetData,
        gameplay: &PlayerGameplayState,
    ) {
        let progress = gameplay.mining.progress;

        let Some(id) = gameplay.mining.target else {
            self.break_inds = 0;
            return;
        };

        let mesh = block_break_mesh(planet, id, progress, BlockBreakStyle::default());

        if mesh.indices.is_empty() {
            self.break_inds = 0;
            return;
        }

        self.queue
            .write_buffer(&self.break_v_buf, 0, bytemuck::cast_slice(&mesh.vertices));
        self.queue
            .write_buffer(&self.break_i_buf, 0, bytemuck::cast_slice(&mesh.indices));

        self.break_inds = mesh.indices.len() as u32;
    }

    pub(super) fn update_dropped_item_mesh(
        &mut self,
        gameplay: &PlayerGameplayState,
        content: &CompiledContent,
    ) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let mut idx = 0u32;

        for drop in gameplay.dropped_items.iter().take(128) {
            let color = self.dropped_item_color(drop.stack.item, content);
            Self::push_dropped_item_cube(
                &mut verts,
                &mut inds,
                &mut idx,
                drop.position,
                0.28,
                color,
            );
        }

        if !verts.is_empty() {
            self.queue
                .write_buffer(&self.drop_v_buf, 0, bytemuck::cast_slice(&verts));
            self.queue
                .write_buffer(&self.drop_i_buf, 0, bytemuck::cast_slice(&inds));
        }

        self.drop_inds = inds.len() as u32;
    }

    fn dropped_item_color(&self, item: ItemId, content: &CompiledContent) -> [f32; 3] {
        let Some(item) = content.items.get(item) else {
            return [0.75, 0.75, 0.75];
        };

        match item.kind {
            CompiledItemKind::Block { block } => self
                .block_content
                .block_render(block)
                .map(|render| render.color)
                .unwrap_or([0.75, 0.75, 0.75]),
            CompiledItemKind::Placeable { .. } => [0.95, 0.72, 0.35],
            CompiledItemKind::Tool { .. } => [0.72, 0.78, 0.85],
            CompiledItemKind::Armor => [0.62, 0.72, 0.90],
            CompiledItemKind::Food => [0.72, 0.90, 0.48],
            CompiledItemKind::Resource => [0.72, 0.68, 0.58],
        }
    }

    fn push_dropped_item_cube(
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        center: Vec3,
        size: f32,
        color: [f32; 3],
    ) {
        let h = size * 0.5;

        let p = [
            center + Vec3::new(-h, -h, -h),
            center + Vec3::new(h, -h, -h),
            center + Vec3::new(h, h, -h),
            center + Vec3::new(-h, h, -h),
            center + Vec3::new(-h, -h, h),
            center + Vec3::new(h, -h, h),
            center + Vec3::new(h, h, h),
            center + Vec3::new(-h, h, h),
        ];

        let faces = [
            ([0, 1, 2, 3], [0.0, 0.0, -1.0]),
            ([5, 4, 7, 6], [0.0, 0.0, 1.0]),
            ([4, 0, 3, 7], [-1.0, 0.0, 0.0]),
            ([1, 5, 6, 2], [1.0, 0.0, 0.0]),
            ([3, 2, 6, 7], [0.0, 1.0, 0.0]),
            ([4, 5, 1, 0], [0.0, -1.0, 0.0]),
        ];

        for (face, normal) in faces {
            let base = *idx;

            for i in face {
                verts.push(Vertex::untextured(p[i].to_array(), color, normal));
            }

            inds.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
            *idx += 4;
        }
    }
}
