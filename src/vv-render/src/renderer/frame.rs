use glam::Vec3;
use glyphon::{Attrs, Buffer, Family, Metrics, Resolution, Shaping, TextArea, TextBounds};
use std::time::Instant;

use vv_gameplay::{Console, Player, PlayerGameplayState};
use vv_input::Controller;
use vv_mesh::MeshGen;
use vv_physics::Physics;
use vv_registry::CompiledContent;
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
        self.update_console_mesh(console.height_fraction);
        self.update_gameplay_ui_mesh(controller, gameplay, content);
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
        let mvp = controller.get_matrix(player, physics, w, h, &self.render_cfg);

        let atmosphere = AtmosphereUniform::from_config(&self.sky_state.to_atmosphere())
            .with_planet_geometry(planet.geometry);
        let sun_dir = atmosphere.sun_direction_vec3();
        let shadow_dist = 200.0f32;
        let proj_size = 60.0f32;
        let center = player.position;
        let mut sun_view = glam::Mat4::look_at_rh(center + sun_dir * shadow_dist, center, Vec3::Y);

        let shadow_map_size_f = self.render_cfg.shadow_map_size as f32;
        let texel_size = (2.0 * proj_size) / shadow_map_size_f;
        let shadow_origin = sun_view.transform_point3(center);
        let snap_x = (shadow_origin.x / texel_size).round() * texel_size - shadow_origin.x;
        let snap_y = (shadow_origin.y / texel_size).round() * texel_size - shadow_origin.y;
        sun_view = glam::Mat4::from_translation(Vec3::new(snap_x, snap_y, 0.0)) * sun_view;

        let sun_proj = glam::Mat4::orthographic_rh(
            -proj_size, proj_size, -proj_size, proj_size, -200.0, 500.0,
        );
        let light_vp = sun_proj * sun_view;

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
            planet: [planet.geometry.radius_m, atmosphere_height_m, 0.0, 0.0],
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

        let shadow_visible_chunks = self
            .chunks
            .values()
            .filter(|mesh| frustum.intersects_sphere(mesh.center, mesh.radius))
            .count();
        let shadow_visible_lods = self
            .lod_chunks
            .values()
            .filter(|mesh| frustum.intersects_sphere(mesh.center, mesh.radius))
            .count();
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

        // Shadow pass
        {
            let mut sp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shadow"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            sp.set_pipeline(&self.pipeline_shadow);
            sp.set_bind_group(0, &self.shadow_global_bind, &[]);
            for mesh in self.chunks.values() {
                if frustum.intersects_sphere(mesh.center, mesh.radius) {
                    sp.set_bind_group(1, &mesh.bind_group, &[]);
                    sp.set_vertex_buffer(0, mesh.v_buf.slice(..));
                    sp.set_index_buffer(mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    sp.draw_indexed(0..mesh.num_inds, 0, 0..1);
                }
            }
            for mesh in self.lod_chunks.values() {
                if frustum.intersects_sphere(mesh.center, mesh.radius) {
                    sp.set_bind_group(1, &mesh.bind_group, &[]);
                    sp.set_vertex_buffer(0, mesh.v_buf.slice(..));
                    sp.set_index_buffer(mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    sp.draw_indexed(0..mesh.num_inds, 0, 0..1);
                }
            }
        }

        // Main pass
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
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

            // Sky: draw first (behind all terrain). Uses depth_write_enabled=false +
            // depth_compare=Always so terrain always occludes it in subsequent draws.
            pass.set_pipeline(&self.pipeline_sky);
            pass.set_bind_group(0, &self.global_bind, &[]);
            pass.set_bind_group(1, &self.local_bind_identity, &[]);
            pass.draw(0..3, 0..1);

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
            if self.console_inds > 0 {
                pass.set_pipeline(&self.pipeline_ui);
                pass.set_bind_group(0, &self.global_bind_identity, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.console_v_buf.slice(..));
                pass.set_index_buffer(self.console_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.console_inds, 0, 0..1);
            }
            if self.ui_inds > 0 {
                pass.set_pipeline(&self.pipeline_ui);
                pass.set_bind_group(0, &self.global_bind_identity, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.ui_v_buf.slice(..));
                pass.set_index_buffer(self.ui_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.ui_inds, 0, 0..1);
            }
        }

        self.frame_count += 1;
        let now2 = std::time::Instant::now();
        if now2.duration_since(self.last_fps_time).as_secs_f32() >= 1.0 {
            self.current_fps = self.frame_count;
            self.frame_count = 0;
            self.last_fps_time = now2;
        }

        // Text pass
        {
            let mut text_areas: Vec<TextArea> = Vec::new();
            let mut text_buffers = Vec::new();

            if console.height_fraction > 0.0 {
                let console_h = (self.config.height as f32 / 2.0) * console.height_fraction;
                let start_y = console_h - 40.0;
                let line_h = 20.0;
                for (i, (line, color)) in console.history.iter().rev().enumerate() {
                    let y = start_y - i as f32 * line_h;
                    if y < 0.0 {
                        break;
                    }
                    let mut buf = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
                    buf.set_size(&mut self.font_system, w, h);
                    buf.set_text(
                        &mut self.font_system,
                        line,
                        Attrs::new()
                            .family(Family::Monospace)
                            .color(glyphon::Color::rgb(
                                (color[0] * 255.0) as u8,
                                (color[1] * 255.0) as u8,
                                (color[2] * 255.0) as u8,
                            )),
                        Shaping::Advanced,
                    );
                    text_buffers.push((buf, 10.0, y));
                }
                let ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let cur = if (ms / 500) % 2 == 0 { "_" } else { " " };
                let mut ibuf = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
                ibuf.set_size(&mut self.font_system, w, h);
                ibuf.set_text(
                    &mut self.font_system,
                    &format!("> {}{}", console.input_buffer, cur),
                    Attrs::new()
                        .family(Family::Monospace)
                        .color(glyphon::Color::rgb(255, 255, 0)),
                    Shaping::Advanced,
                );
                text_buffers.push((ibuf, 10.0, console_h - 20.0));
            }

            let fps_text = format!("FPS: {}", self.current_fps);
            let mut fps_buf = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
            fps_buf.set_size(&mut self.font_system, w, h);
            fps_buf.set_text(
                &mut self.font_system,
                &fps_text,
                Attrs::new()
                    .family(Family::Monospace)
                    .color(glyphon::Color::rgb(255, 255, 255)),
                Shaping::Advanced,
            );
            text_buffers.push((fps_buf, 10.0, 5.0));

            self.push_gameplay_text(controller, gameplay, &mut text_buffers);

            for (buf, x, y) in &text_buffers {
                text_areas.push(TextArea {
                    buffer: buf,
                    left: *x,
                    top: *y,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: w as i32,
                        bottom: h as i32,
                    },
                    default_color: glyphon::Color::rgb(255, 255, 255),
                });
            }

            if !text_areas.is_empty() {
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
                    label: Some("Text"),
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
                let _ = self.text_renderer.render(&self.text_atlas, &mut text_pass);
            }
        }

        self.queue.submit(std::iter::once(enc.finish()));
        out.present();
        self.text_atlas.trim();
        let overlay_draws = (!controller.first_person) as u32
            + (self.collision_inds > 0) as u32
            + (self.cursor_inds > 0) as u32
            + (self.break_inds > 0) as u32
            + (self.drop_inds > 0) as u32
            + (self.console_inds > 0) as u32
            + (self.ui_inds > 0) as u32;
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
}
