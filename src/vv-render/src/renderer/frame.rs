use glam::Vec3;
use glyphon::{Attrs, Buffer, Family, Metrics, Resolution, Shaping, TextArea, TextBounds};
use std::time::Instant;

use vv_gameplay::{Console, Player, PlayerGameplayState};
use vv_input::Controller;
use vv_mesh::MeshGen;
use vv_physics::Physics;
use vv_registry::CompiledContent;
use vv_ui::{UiTextAlign, UiTextCommand};
use vv_world_runtime::PlanetData;

use crate::{atmosphere::AtmosphereUniform, AnyKey, Frustum};

use super::types::{GlobalUniform, LocalUniform};
use super::Renderer;

impl<'a> Renderer<'a> {
    pub fn render(
        &mut self,
        controller: &Controller,
        player: &Player,
        physics: &Physics,
        planet: &PlanetData,
        console: &Console,
        gameplay: &PlayerGameplayState,
        content: &CompiledContent,
    ) {
        let prep_start = Instant::now();

        self.update_block_break_feedback(planet, gameplay);
        self.update_dropped_item_mesh(gameplay, content);

        if controller.show_collisions {
            let (v, i) = MeshGen::generate_collision_debug(player.position, planet);
            self.queue
                .write_buffer(&self.collision_v_buf, 0, bytemuck::cast_slice(&v));
            self.queue
                .write_buffer(&self.collision_i_buf, 0, bytemuck::cast_slice(&i));
            self.collision_inds = i.len() as u32;
        } else {
            self.collision_inds = 0;
        }

        let out = match self.surface.get_current_texture() {
            Ok(o) => o,
            _ => return,
        };

        let view = out
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let w = self.config.width as f32;
        let h = self.config.height as f32;

        let ui_frame = self.build_renderer_ui_frame(controller, console, gameplay, content);
        self.ui_renderer
            .update(&self.device, &self.queue, &ui_frame);

        let mvp = controller.get_matrix(player, physics, w, h, &self.render_cfg);

        let atmosphere = AtmosphereUniform::from_config(&self.render_cfg.atmosphere)
            .with_planet_geometry(planet.geometry);
        let light_vp = glam::Mat4::IDENTITY;

        let cam_pos = controller.get_camera_pos(player, physics);
        let frustum = Frustum::from_matrix(mvp);

        let cull_frustum_val;
        let cull_frustum: &Frustum = if controller.freeze_culling {
            if self.frozen_frustum.is_none() {
                self.frozen_frustum = Some(Frustum::from_matrix(mvp));
            }
            self.frozen_frustum.as_ref().unwrap()
        } else {
            self.frozen_frustum = None;
            cull_frustum_val = Frustum::from_matrix(mvp);
            &cull_frustum_val
        };

        let atmosphere_height_m = (planet.geometry.radius_m * 0.28).clamp(8_000.0, 120_000.0);

        let global_data = GlobalUniform {
            view_proj: mvp.to_cols_array(),
            light_view_proj: light_vp.to_cols_array(),
            cam_pos: [cam_pos.x, cam_pos.y, cam_pos.z, 1.0],
            atmosphere,
            inv_view_proj: mvp.inverse().to_cols_array(),
            planet: [
                planet.geometry.radius_m,
                atmosphere_height_m,
                self.render_cfg.shadow_mode.as_shader_id(),
                0.0,
            ],
        };

        self.queue
            .write_buffer(&self.global_buf, 0, bytemuck::cast_slice(&[global_data]));

        self.queue.write_buffer(
            &self.shadow_global_buf,
            0,
            bytemuck::cast_slice(&[GlobalUniform {
                view_proj: light_vp.to_cols_array(),
                ..global_data
            }]),
        );

        let model_mat = player.get_model_matrix();
        self.queue.write_buffer(
            &self.local_buf_player,
            0,
            bytemuck::cast_slice(model_mat.as_ref()),
        );

        let r = planet.resolution as f32 / 2.0;
        self.queue.write_buffer(
            &self.local_buf_guide,
            0,
            bytemuck::cast_slice(glam::Mat4::from_scale(Vec3::splat(r)).as_ref()),
        );

        let now = std::time::Instant::now();
        let dying = self.animator.update_dying(now);

        for (key, alpha) in dying {
            if let Some(state) = self.animator.dying_chunks.get(&key) {
                let d = LocalUniform {
                    model: glam::Mat4::IDENTITY.to_cols_array(),
                    params: [alpha, 1.0, 0.0, 0.0],
                };
                self.queue
                    .write_buffer(&state.mesh.uniform_buf, 0, bytemuck::cast_slice(&[d]));
            }
        }

        let queue = &self.queue;
        let animator = &mut self.animator;

        for (key, mesh) in &self.lod_chunks {
            let alpha = animator.get_opacity(AnyKey::Lod(*key), now);
            if alpha < 1.0 || animator.spawning_chunks.contains_key(&AnyKey::Lod(*key)) {
                let d = LocalUniform {
                    model: glam::Mat4::IDENTITY.to_cols_array(),
                    params: [alpha.min(1.0), 0.0, 0.0, 0.0],
                };
                queue.write_buffer(&mesh.uniform_buf, 0, bytemuck::cast_slice(&[d]));
                if alpha >= 1.0 {
                    animator.spawning_chunks.remove(&AnyKey::Lod(*key));
                }
            }
        }

        for (key, mesh) in &self.chunks {
            let alpha = animator.get_opacity(AnyKey::Voxel(*key), now);
            if alpha < 1.0 || animator.spawning_chunks.contains_key(&AnyKey::Voxel(*key)) {
                let d = LocalUniform {
                    model: glam::Mat4::IDENTITY.to_cols_array(),
                    params: [alpha.min(1.0), 0.0, 0.0, 0.0],
                };
                queue.write_buffer(&mesh.uniform_buf, 0, bytemuck::cast_slice(&[d]));
                if alpha >= 1.0 {
                    animator.spawning_chunks.remove(&AnyKey::Voxel(*key));
                }
            }
        }

        self.frame_telemetry.render_prep_time += prep_start.elapsed();

        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let shadow_visible_chunks = 0usize;
        let shadow_visible_lods = 0usize;
        let main_visible_chunks = self
            .chunks
            .values()
            .filter(|mesh| cull_frustum.intersects_sphere(mesh.center, mesh.radius))
            .count();
        let main_visible_lods = self
            .lod_chunks
            .values()
            .filter(|mesh| cull_frustum.intersects_sphere(mesh.center, mesh.radius))
            .count();
        let dying_visible = self
            .animator
            .dying_chunks
            .values()
            .filter(|state| frustum.intersects_sphere(state.mesh.center, state.mesh.radius))
            .count();
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(atmosphere.clear_color()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let terrain_pipeline = if controller.is_wireframe {
                &self.pipeline_wire
            } else {
                &self.pipeline_fill
            };
            pass.set_pipeline(terrain_pipeline);
            pass.set_bind_group(0, &self.global_bind, &[]);

            for mesh in self.lod_chunks.values() {
                if cull_frustum.intersects_sphere(mesh.center, mesh.radius) {
                    pass.set_bind_group(1, &mesh.bind_group, &[]);
                    pass.set_vertex_buffer(0, mesh.v_buf.slice(..));
                    pass.set_index_buffer(mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..mesh.num_inds, 0, 0..1);
                }
            }

            for mesh in self.chunks.values() {
                if cull_frustum.intersects_sphere(mesh.center, mesh.radius) {
                    pass.set_bind_group(1, &mesh.bind_group, &[]);
                    pass.set_vertex_buffer(0, mesh.v_buf.slice(..));
                    pass.set_index_buffer(mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..mesh.num_inds, 0, 0..1);
                }
            }

            for state in self.animator.dying_chunks.values() {
                if frustum.intersects_sphere(state.mesh.center, state.mesh.radius) {
                    pass.set_bind_group(1, &state.mesh.bind_group, &[]);
                    pass.set_vertex_buffer(0, state.mesh.v_buf.slice(..));
                    pass.set_index_buffer(state.mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..state.mesh.num_inds, 0, 0..1);
                }
            }

            if !controller.first_person {
                pass.set_pipeline(terrain_pipeline);
                pass.set_bind_group(1, &self.local_bind_player, &[]);
                pass.set_vertex_buffer(0, self.player_v_buf.slice(..));
                pass.set_index_buffer(self.player_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.player_inds, 0, 0..1);
            }

            if self.collision_inds > 0 {
                pass.set_pipeline(&self.pipeline_line);
                pass.set_bind_group(0, &self.global_bind, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.collision_v_buf.slice(..));
                pass.set_index_buffer(self.collision_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.collision_inds, 0, 0..1);
            }

            if self.cursor_inds > 0 {
                pass.set_pipeline(&self.pipeline_feedback);
                pass.set_bind_group(0, &self.global_bind, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.cursor_v_buf.slice(..));
                pass.set_index_buffer(self.cursor_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.cursor_inds, 0, 0..1);
            }

            if self.break_inds > 0 {
                pass.set_pipeline(&self.pipeline_feedback);
                pass.set_bind_group(0, &self.global_bind, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.break_v_buf.slice(..));
                pass.set_index_buffer(self.break_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.break_inds, 0, 0..1);
            }

            if self.drop_inds > 0 {
                pass.set_pipeline(&self.pipeline_fill);
                pass.set_bind_group(0, &self.global_bind, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.drop_v_buf.slice(..));
                pass.set_index_buffer(self.drop_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.drop_inds, 0, 0..1);
            }
        }

        {
            let mut ui_pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("VoxelVerse UI"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.ui_renderer.draw(&mut ui_pass);
        }

        self.frame_count += 1;
        let now2 = std::time::Instant::now();
        if now2.duration_since(self.last_fps_time).as_secs_f32() >= 1.0 {
            self.current_fps = self.frame_count;
            self.frame_count = 0;
            self.last_fps_time = now2;
        }

        self.render_ui_text(&mut enc, &view, w, h);

        self.queue.submit(std::iter::once(enc.finish()));
        out.present();
        self.text_atlas.trim();

        let overlay_draws = (!controller.first_person) as u32
            + (self.collision_inds > 0) as u32
            + (self.cursor_inds > 0) as u32
            + (self.break_inds > 0) as u32
            + (self.drop_inds > 0) as u32
            + (self.ui_renderer.index_count() > 0) as u32;

        self.frame_telemetry.gpu.draw_calls = (shadow_visible_chunks
            + shadow_visible_lods
            + main_visible_chunks
            + main_visible_lods
            + dying_visible) as u32
            + overlay_draws;

        self.frame_telemetry.gpu.visible_chunks = main_visible_chunks;
        self.frame_telemetry.gpu.visible_lods = main_visible_lods;
        self.frame_telemetry.gpu.active_buffers =
            (self.chunks.len() + self.lod_chunks.len() + self.animator.dying_chunks.len()) * 3;
    }

    fn render_ui_text(
        &mut self,
        enc: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        screen_w: f32,
        screen_h: f32,
    ) {
        let mut text_buffers = Vec::new();

        for item in self.ui_renderer.text_items() {
            let cmd = &item.command;

            if cmd.text.is_empty() || cmd.color.a <= 0.001 {
                continue;
            }

            let mut buf = Buffer::new(
                &mut self.font_system,
                Metrics::new(cmd.size, cmd.size + 4.0),
            );

            buf.set_size(
                &mut self.font_system,
                cmd.rect.width.max(1.0),
                cmd.rect.height.max(cmd.size + 4.0),
            );

            buf.set_text(
                &mut self.font_system,
                &cmd.text,
                Attrs::new()
                    .family(Family::Monospace)
                    .color(glyphon::Color::rgb(
                        color_channel(cmd.color.r),
                        color_channel(cmd.color.g),
                        color_channel(cmd.color.b),
                    )),
                Shaping::Advanced,
            );

            let aligned_x = aligned_text_x(cmd);

            text_buffers.push((buf, aligned_x, cmd.rect.y));
        }

        if text_buffers.is_empty() {
            return;
        }

        let mut text_areas = Vec::with_capacity(text_buffers.len());

        for (buf, x, y) in &text_buffers {
            text_areas.push(TextArea {
                buffer: buf,
                left: *x,
                top: *y,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: screen_w as i32,
                    bottom: screen_h as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            });
        }

        let _ = self.text_renderer.prepare(
            &self.device,
            &self.queue,
            &mut self.font_system,
            &mut self.text_atlas,
            Resolution {
                width: self.config.width,
                height: self.config.height,
            },
            text_areas,
            &mut self.swash_cache,
        );

        let mut text_pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("VoxelVerse UI Text"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let _ = self.text_renderer.render(&self.text_atlas, &mut text_pass);
    }
}

fn aligned_text_x(cmd: &UiTextCommand) -> f32 {
    match cmd.align {
        UiTextAlign::Left => cmd.rect.x,
        UiTextAlign::Center => {
            let estimated = estimate_text_width(&cmd.text, cmd.size);
            cmd.rect.x + ((cmd.rect.width - estimated) * 0.5).max(0.0)
        }
        UiTextAlign::Right => {
            let estimated = estimate_text_width(&cmd.text, cmd.size);
            cmd.rect.right() - estimated.min(cmd.rect.width)
        }
    }
}

fn estimate_text_width(text: &str, size: f32) -> f32 {
    text.chars().map(|ch| glyph_width(ch, size)).sum()
}

fn glyph_width(ch: char, size: f32) -> f32 {
    match ch {
        ' ' => size * 0.35,
        'i' | 'l' | 'I' | '!' | '|' | '.' | ',' | ':' | ';' => size * 0.36,
        'm' | 'w' | 'M' | 'W' => size * 0.82,
        '×' | '+' | '−' | '-' | '↕' | '⌕' | '▣' => size * 0.72,
        c if c.is_ascii_digit() => size * 0.56,
        c if c.is_ascii_uppercase() => size * 0.68,
        _ => size * 0.58,
    }
}

fn color_channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
