use super::{GlobalUniform, LocalUniform, Renderer};
use crate::diagnostics::Console;
use crate::gameplay::{Hotbar, Inventory, Player};
use crate::input::Controller;
use crate::math::Frustum;
use crate::meshing::MeshGen;
use crate::rendering::lod_animation::AnyKey;
use crate::rendering::types::{ChunkMesh, Vertex};
use crate::ui::InventoryUiState;
use crate::world::PlanetData;
use glyphon::{Attrs, Buffer, Family, Metrics, Resolution, Shaping, TextArea, TextBounds};

impl<'a> Renderer<'a> {
    pub fn render_loading(&mut self, progress: f32, message: &str) {
        let progress = progress.clamp(0.0, 1.0);
        self.window
            .set_title(&format!("VoxelVerse - chargement {:.0}%", progress * 100.0));

        let Ok(out) = self.surface.get_current_texture() else {
            return;
        };
        let view = out
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let _pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Loading Clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.03,
                            g: 0.04,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        let filled = (progress * 28.0).round() as usize;
        let bar = format!(
            "[{}{}] {:.0}%",
            "#".repeat(filled),
            "-".repeat(28usize.saturating_sub(filled)),
            progress * 100.0
        );
        let text = format!("VoxelVerse\n{}\n{}", message, bar);
        let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(24.0, 34.0));
        buffer.set_size(
            &mut self.font_system,
            self.config.width as f32,
            self.config.height as f32,
        );
        buffer.set_text(
            &mut self.font_system,
            &text,
            Attrs::new()
                .family(Family::Monospace)
                .color(glyphon::Color::rgb(230, 240, 235)),
            Shaping::Advanced,
        );

        let text_area = TextArea {
            buffer: &buffer,
            left: 48.0,
            top: (self.config.height as f32 * 0.5 - 70.0).max(40.0),
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: 0,
                right: self.config.width as i32,
                bottom: self.config.height as i32,
            },
            default_color: glyphon::Color::rgb(255, 255, 255),
        };

        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.text_atlas,
                Resolution {
                    width: self.config.width,
                    height: self.config.height,
                },
                vec![text_area],
                &mut self.swash_cache,
            )
            .ok();

        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Loading Text"),
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
            let _ = self.text_renderer.render(&self.text_atlas, &mut pass);
        }

        self.queue.submit(std::iter::once(enc.finish()));
        out.present();
        self.device.poll(wgpu::Maintain::Wait);
    }

    pub fn render(
        &mut self,
        controller: &Controller,
        player: &Player,
        planet: &PlanetData,
        hotbar: &Hotbar,
        inventory: &Inventory,
        inventory_ui: &InventoryUiState,
        console: &Console,
    ) {
        let render_started = std::time::Instant::now();
        self.update_console_mesh(console.height_fraction);
        if inventory_ui.is_open {
            self.hotbar_inds = 0;
        } else {
            self.update_hotbar_mesh(hotbar, planet);
        }
        self.update_inventory_mesh(inventory, hotbar, inventory_ui, planet);

        if controller.show_collisions {
            let mesh = MeshGen::generate_collision_debug(player.position, planet);
            let gpu_v: Vec<Vertex> = mesh.vertices.iter().copied().map(Vertex::from).collect();
            self.queue
                .write_buffer(&self.collision_v_buf, 0, bytemuck::cast_slice(&gpu_v));
            self.queue.write_buffer(
                &self.collision_i_buf,
                0,
                bytemuck::cast_slice(&mesh.indices),
            );
            self.collision_inds = mesh.indices.len() as u32;
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

        // -- sun direction: low angle for long dramatic shadows --
        let sun_dir = glam::Vec3::new(0.38, 0.62, 0.28).normalize();
        let shadow_dist = 200.0;
        let proj_size = 60.0;

        // Fog density scales with planet surface radius so that all planet sizes
        // have visually appropriate atmospheric depth.
        // Formula: 0.75 / surface_radius → same angular density at every scale.
        let fog_density = 0.75 / planet.profile.surface_radius.max(1.0);

        // basic LookAt
        let center = player.position;
        let mut sun_view =
            glam::Mat4::look_at_rh(center + (sun_dir * shadow_dist), center, glam::Vec3::Y);

        // texel Snapping
        // project the center position into light space, snap it to a pixel,
        // and then offset the view matrix by the difference.
        let texel_size = (2.0 * proj_size) / self.shadow_map_size as f32;

        let shadow_origin = sun_view.transform_point3(center);
        let snapped_x = (shadow_origin.x / texel_size).round() * texel_size;
        let snapped_y = (shadow_origin.y / texel_size).round() * texel_size;

        let snap_offset_x = snapped_x - shadow_origin.x;
        let snap_offset_y = snapped_y - shadow_origin.y;

        // apply snap to the view matrix
        let snap_mat =
            glam::Mat4::from_translation(glam::Vec3::new(snap_offset_x, snap_offset_y, 0.0));
        sun_view = snap_mat * sun_view;

        // projection
        let sun_proj = glam::Mat4::orthographic_rh(
            -proj_size, proj_size, -proj_size, proj_size, -200.0, 500.0,
        );

        let light_view_proj = sun_proj * sun_view;

        // -- Camera Matrix --
        let mvp =
            controller.get_matrix(player, self.config.width as f32, self.config.height as f32);

        // --- FRUSTUM CULLING LOGIC ---
        let current_frustum = Frustum::from_matrix(mvp);

        // determine which frustum to use for culling
        // if freeze is on, we use the stored one. if freeze is off, update the stored one (or just use current).
        let cull_frustum = if controller.freeze_culling {
            if self.frozen_frustum.is_none() {
                self.frozen_frustum = Some(Frustum::from_matrix(mvp));
            }
            self.frozen_frustum.as_ref().unwrap()
        } else {
            self.frozen_frustum = None;
            &current_frustum
        };

        // debug Stats
        let mut rendered_lods = 0;
        let mut rendered_chunks = 0;
        let mut main_draw_calls = 0usize;
        let mut shadow_draw_calls = 0usize;

        let cam_pos = controller.get_camera_pos(player);
        let frustum = Frustum::from_matrix(mvp);

        // Build a separate frustum from the sun's light-space matrix so the
        // shadow pass culls against the *sun*'s 60-unit ortho box, not the
        // (much larger) camera frustum.  Without this, the shadow pass redraws
        // hundreds of chunks that don't even contribute to the shadow map.
        let sun_frustum = Frustum::from_matrix(light_view_proj);

        // Horizon (back-of-planet) test for spherical planets centred at the
        // world origin.  Returns `true` when the chunk's bounding sphere is
        // fully past the geometric horizon — guaranteed invisible because the
        // planet body itself occludes it.  Conservative: skipped when the
        // camera is inside / very close to the surface.
        let surface_radius = planet.profile.surface_radius;
        let cam_dist = cam_pos.length();
        let horizon_active = cam_dist > surface_radius * 1.001;
        let cos_horizon = if horizon_active {
            surface_radius / cam_dist
        } else {
            -1.0
        };
        let cam_dir = if cam_dist > 1e-3 {
            cam_pos / cam_dist
        } else {
            glam::Vec3::Y
        };
        let behind_horizon = |center: glam::Vec3, radius: f32| -> bool {
            if !horizon_active {
                return false;
            }
            let dist = center.length();
            if dist < 1e-3 {
                return false;
            }
            let cos_angle = cam_dir.dot(center) / dist;
            // Subtract chunk angular radius (×1.5 safety) so we never cull a
            // chunk whose silhouette could still poke above the horizon.
            let angular_radius = radius / dist;
            cos_angle < cos_horizon - 1.5 * angular_radius
        };

        // 1. Compute atmosphere colors from sun elevation.
        //    sun_dir.y is the sine of the sun's elevation angle.
        let sun_elevation = sun_dir.y.clamp(-1.0_f32, 1.0);
        let above_horizon = sun_elevation.max(0.0_f32);
        // dawn_factor peaks near the horizon (sunrise/sunset) and falls off at noon/night
        let dawn_factor =
            (1.0 - (sun_elevation.abs() * 3.5).min(1.0)).powi(2) * (above_horizon * 2.0).min(1.0);

        let h_noon = glam::Vec3::new(0.72, 0.84, 1.00);
        let h_dawn = glam::Vec3::new(0.96, 0.58, 0.26);
        let h_night = glam::Vec3::new(0.02, 0.03, 0.08);
        let sky_horizon_rgb = if sun_elevation >= 0.0 {
            // Blend toward dawn/dusk colors when sun is near horizon
            let t = dawn_factor;
            glam::Vec3::new(
                h_noon.x * (1.0 - t) + h_dawn.x * t,
                h_noon.y * (1.0 - t) + h_dawn.y * t,
                h_noon.z * (1.0 - t) + h_dawn.z * t,
            )
        } else {
            let night_t = (-sun_elevation * 4.0).min(1.0);
            glam::Vec3::new(
                h_dawn.x * (1.0 - night_t) + h_night.x * night_t,
                h_dawn.y * (1.0 - night_t) + h_night.y * night_t,
                h_dawn.z * (1.0 - night_t) + h_night.z * night_t,
            )
        };

        let day_t = above_horizon.powf(0.5);
        let z_day = glam::Vec3::new(0.12, 0.28, 0.76);
        let z_night = glam::Vec3::new(0.01, 0.01, 0.05);
        let sky_zenith_rgb = glam::Vec3::new(
            z_day.x * day_t + z_night.x * (1.0 - day_t),
            z_day.y * day_t + z_night.y * (1.0 - day_t),
            z_day.z * day_t + z_night.z * (1.0 - day_t),
        );

        let sun_intensity = above_horizon.powf(0.3).min(1.0);
        // time_of_day: placeholder (0=midnight, 0.25=sunrise, 0.5=noon, 0.75=sunset)
        let time_of_day = 0.5_f32;

        // 2. Build and upload the main global uniform.
        let global_data = GlobalUniform {
            view_proj: mvp.to_cols_array(),
            light_view_proj: light_view_proj.to_cols_array(),
            cam_pos: [cam_pos.x, cam_pos.y, cam_pos.z, self.quality.pack()],
            // w component carries per-planet fog density (read in shader via sun_dir.w).
            sun_dir: [sun_dir.x, sun_dir.y, sun_dir.z, fog_density],
            sky_horizon: [
                sky_horizon_rgb.x,
                sky_horizon_rgb.y,
                sky_horizon_rgb.z,
                time_of_day,
            ],
            sky_zenith: [
                sky_zenith_rgb.x,
                sky_zenith_rgb.y,
                sky_zenith_rgb.z,
                sun_intensity,
            ],
        };
        self.queue
            .write_buffer(&self.global_buf, 0, bytemuck::cast_slice(&[global_data]));

        // 3. Shadow global uniform uses the light matrix as view_proj.
        let shadow_uniform_data = GlobalUniform {
            view_proj: light_view_proj.to_cols_array(),
            light_view_proj: light_view_proj.to_cols_array(),
            cam_pos: [cam_pos.x, cam_pos.y, cam_pos.z, self.quality.pack()],
            sun_dir: [sun_dir.x, sun_dir.y, sun_dir.z, fog_density],
            sky_horizon: [
                sky_horizon_rgb.x,
                sky_horizon_rgb.y,
                sky_horizon_rgb.z,
                time_of_day,
            ],
            sky_zenith: [
                sky_zenith_rgb.x,
                sky_zenith_rgb.y,
                sky_zenith_rgb.z,
                sun_intensity,
            ],
        };
        self.queue.write_buffer(
            &self.shadow_global_buf,
            0,
            bytemuck::cast_slice(&[shadow_uniform_data]),
        );

        let model_mat = player.get_model_matrix();
        self.queue.write_buffer(
            &self.local_buf_player,
            0,
            bytemuck::cast_slice(model_mat.as_ref()),
        );

        let now = std::time::Instant::now();
        let edge_rounding_radius = |key: AnyKey| match key {
            AnyKey::Voxel(_) => planet.profile.edge_rounding_radius_voxels,
            AnyKey::Lod(_) => 0.0,
        };
        let dying_status = self.animator.update_dying(now);
        for (key, alpha) in dying_status {
            if let Some(state) = self.animator.dying_chunks.get(&key) {
                let data = LocalUniform {
                    model: glam::Mat4::IDENTITY.to_cols_array(),
                    params: [alpha, edge_rounding_radius(key), 0.0, 0.0],
                };
                self.queue
                    .write_buffer(&state.mesh.uniform_buf, 0, bytemuck::cast_slice(&[data]));
            }
        }

        let queue = &self.queue;
        let animator = &mut self.animator;

        let mut update_opacity = |key: AnyKey, mesh: &ChunkMesh| {
            let alpha = animator.get_opacity(key, now);
            if alpha < 1.0 {
                let data = LocalUniform {
                    model: glam::Mat4::IDENTITY.to_cols_array(),
                    params: [alpha, edge_rounding_radius(key), 0.0, 0.0],
                };
                queue.write_buffer(&mesh.uniform_buf, 0, bytemuck::cast_slice(&[data]));
            } else if animator.spawning_chunks.contains_key(&key) {
                let data = LocalUniform {
                    model: glam::Mat4::IDENTITY.to_cols_array(),
                    params: [1.0, edge_rounding_radius(key), 0.0, 0.0],
                };
                queue.write_buffer(&mesh.uniform_buf, 0, bytemuck::cast_slice(&[data]));
                animator.spawning_chunks.remove(&key);
            }
        };

        for (key, mesh) in &self.lod_chunks {
            update_opacity(AnyKey::Lod(*key), mesh);
        }
        for (key, mesh) in &self.chunks {
            update_opacity(AnyKey::Voxel(*key), mesh);
        }

        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // --- PASS 1: SHADOW MAP GENERATION ---
        {
            let mut shadow_pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shadow Pass"),
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

            shadow_pass.set_pipeline(&self.pipeline_shadow);
            shadow_pass.set_bind_group(0, &self.shadow_global_bind, &[]);
            shadow_pass.set_bind_group(2, &self.atlas_bind, &[]);

            for mesh in self.chunks.values() {
                if behind_horizon(mesh.center, mesh.radius) {
                    continue;
                }
                if sun_frustum.intersects_sphere(mesh.center, mesh.radius) {
                    shadow_pass.set_bind_group(1, &mesh.bind_group, &[]);
                    shadow_pass.set_vertex_buffer(0, mesh.v_buf.slice(..));
                    shadow_pass.set_index_buffer(mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    shadow_pass.draw_indexed(0..mesh.num_inds, 0, 0..1);
                    shadow_draw_calls += 1;
                }
            }
            for mesh in self.lod_chunks.values() {
                if behind_horizon(mesh.center, mesh.radius) {
                    continue;
                }
                if sun_frustum.intersects_sphere(mesh.center, mesh.radius) {
                    shadow_pass.set_bind_group(1, &mesh.bind_group, &[]);
                    shadow_pass.set_vertex_buffer(0, mesh.v_buf.slice(..));
                    shadow_pass.set_index_buffer(mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    shadow_pass.draw_indexed(0..mesh.num_inds, 0, 0..1);
                    shadow_draw_calls += 1;
                }
            }
        }

        // --- PASS 2: SKY ---
        // Renders the atmospheric sky as a fullscreen triangle before terrain so
        // background pixels (horizon, space above planet) show the sky.
        {
            let mut sky_pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sky Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None, // sky writes no depth
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            sky_pass.set_pipeline(&self.pipeline_sky);
            sky_pass.set_bind_group(0, &self.sky_global_bind, &[]);
            sky_pass.draw(0..3, 0..1); // fullscreen triangle from vertex_index
        }

        // --- PASS 3: MAIN RENDER ---
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Sky was rendered in the previous pass — keep it as background.
                        load: wgpu::LoadOp::Load,
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

            if controller.is_wireframe {
                pass.set_pipeline(&self.pipeline_wire);
            } else {
                pass.set_pipeline(&self.pipeline_fill);
            }

            // Set atlas bind group once for the whole main pass.
            pass.set_bind_group(0, &self.global_bind, &[]);
            pass.set_bind_group(2, &self.atlas_bind, &[]);

            // DRAW LOD CHUNKS
            for mesh in self.lod_chunks.values() {
                if behind_horizon(mesh.center, mesh.radius) {
                    continue;
                }
                if cull_frustum.intersects_sphere(mesh.center, mesh.radius) {
                    rendered_lods += 1; // Count
                    pass.set_bind_group(1, &mesh.bind_group, &[]);
                    pass.set_vertex_buffer(0, mesh.v_buf.slice(..));
                    pass.set_index_buffer(mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..mesh.num_inds, 0, 0..1);
                    main_draw_calls += 1;
                }
            }

            // DRAW VOXEL CHUNKS
            for mesh in self.chunks.values() {
                if behind_horizon(mesh.center, mesh.radius) {
                    continue;
                }
                if cull_frustum.intersects_sphere(mesh.center, mesh.radius) {
                    rendered_chunks += 1; // Count
                    pass.set_bind_group(1, &mesh.bind_group, &[]);
                    pass.set_vertex_buffer(0, mesh.v_buf.slice(..));
                    pass.set_index_buffer(mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..mesh.num_inds, 0, 0..1);
                    main_draw_calls += 1;
                }
            }

            // DRAW DYING ANIMATIONS
            for state in self.animator.dying_chunks.values() {
                if frustum.intersects_sphere(state.mesh.center, state.mesh.radius) {
                    pass.set_bind_group(1, &state.mesh.bind_group, &[]);
                    pass.set_vertex_buffer(0, state.mesh.v_buf.slice(..));
                    pass.set_index_buffer(state.mesh.i_buf.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..state.mesh.num_inds, 0, 0..1);
                    main_draw_calls += 1;
                }
            }

            if !controller.first_person {
                if controller.is_wireframe {
                    pass.set_pipeline(&self.pipeline_wire);
                } else {
                    pass.set_pipeline(&self.pipeline_fill);
                }
                pass.set_bind_group(1, &self.local_bind_player, &[]);
                pass.set_vertex_buffer(0, self.player_v_buf.slice(..));
                pass.set_index_buffer(self.player_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.player_inds, 0, 0..1);
                main_draw_calls += 1;
            }

            if self.collision_inds > 0 {
                pass.set_pipeline(&self.pipeline_line); // Use line pipeline
                pass.set_bind_group(0, &self.global_bind, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.collision_v_buf.slice(..));
                pass.set_index_buffer(self.collision_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.collision_inds, 0, 0..1);
                main_draw_calls += 1;
            }

            if self.cursor_inds > 0 {
                pass.set_pipeline(&self.pipeline_fill);
                pass.set_bind_group(0, &self.global_bind, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.cursor_v_buf.slice(..));
                pass.set_index_buffer(self.cursor_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.cursor_inds, 0, 0..1);
                main_draw_calls += 1;
            }

            if controller.first_person {
                pass.set_pipeline(&self.pipeline_line);
                pass.set_bind_group(0, &self.global_bind_identity, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.cross_v_buf.slice(..));
                pass.set_index_buffer(self.cross_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.cross_inds, 0, 0..1);
                main_draw_calls += 1;
            }

            if self.hotbar_inds > 0 {
                pass.set_pipeline(&self.pipeline_ui);
                pass.set_bind_group(0, &self.global_bind_identity, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_bind_group(2, &self.atlas_bind, &[]);
                pass.set_vertex_buffer(0, self.hotbar_v_buf.slice(..));
                pass.set_index_buffer(self.hotbar_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.hotbar_inds, 0, 0..1);
                main_draw_calls += 1;
            }

            if self.inventory_inds > 0 {
                pass.set_pipeline(&self.pipeline_ui);
                pass.set_bind_group(0, &self.global_bind_identity, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_bind_group(2, &self.atlas_bind, &[]);
                pass.set_vertex_buffer(0, self.inventory_v_buf.slice(..));
                pass.set_index_buffer(self.inventory_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.inventory_inds, 0, 0..1);
                main_draw_calls += 1;
            }

            if self.console_inds > 0 {
                pass.set_pipeline(&self.pipeline_ui);
                pass.set_bind_group(0, &self.global_bind_identity, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.console_v_buf.slice(..));
                pass.set_index_buffer(self.console_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.console_inds, 0, 0..1);
                main_draw_calls += 1;
            }
        }

        self.last_draw_calls = main_draw_calls;
        self.last_shadow_draw_calls = shadow_draw_calls;

        self.frame_stats.tick();

        // --- PASS 3: TEXT RENDER ---
        // run this pass every frame to show FPS
        {
            let mut text_buffers = Vec::new();
            if console.height_fraction > 0.0 {
                let console_pixel_height =
                    (self.config.height as f32 / 2.0) * console.height_fraction;
                let start_y = console_pixel_height - 40.0;
                let line_height = 20.0;

                for (i, (line_text, color)) in console.history.iter().rev().enumerate() {
                    let y = start_y - (i as f32 * line_height);
                    if y < 0.0 {
                        break;
                    }

                    let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
                    buffer.set_size(
                        &mut self.font_system,
                        self.config.width as f32,
                        self.config.height as f32,
                    );
                    buffer.set_text(
                        &mut self.font_system,
                        line_text,
                        Attrs::new()
                            .family(Family::Monospace)
                            .color(glyphon::Color::rgb(
                                (color[0] * 255.0) as u8,
                                (color[1] * 255.0) as u8,
                                (color[2] * 255.0) as u8,
                            )),
                        Shaping::Advanced,
                    );
                    text_buffers.push((buffer, 10.0, y));
                }

                let input_y = console_pixel_height - 20.0;
                let mut input_buf = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
                input_buf.set_size(
                    &mut self.font_system,
                    self.config.width as f32,
                    self.config.height as f32,
                );
                let time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let cursor = if (time / 500).is_multiple_of(2) {
                    "_"
                } else {
                    " "
                };
                input_buf.set_text(
                    &mut self.font_system,
                    &format!("> {}{}", console.input_buffer, cursor),
                    Attrs::new()
                        .family(Family::Monospace)
                        .color(glyphon::Color::rgb(255, 255, 0)),
                    Shaping::Advanced,
                );
                text_buffers.push((input_buf, 10.0, input_y));
            }

            for spec in self.hotbar_text_specs(hotbar) {
                let size = spec.size.max(8.0);
                let line = (size * 1.25).max(size + 2.0);
                let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(size, line));
                buffer.set_size(
                    &mut self.font_system,
                    self.config.width as f32,
                    self.config.height as f32,
                );
                buffer.set_text(
                    &mut self.font_system,
                    &spec.text,
                    Attrs::new()
                        .family(Family::Monospace)
                        .color(glyphon::Color::rgb(
                            spec.color[0],
                            spec.color[1],
                            spec.color[2],
                        )),
                    Shaping::Advanced,
                );
                text_buffers.push((buffer, spec.left, spec.top));
            }

            // Inventory text overlay (titles, labels, search content, badges).
            for spec in self.inventory_text_specs(inventory, hotbar, inventory_ui, planet) {
                let size = spec.size.max(8.0);
                let line = (size * 1.25).max(size + 2.0);
                let mut buffer =
                    Buffer::new(&mut self.font_system, Metrics::new(size, line));
                buffer.set_size(
                    &mut self.font_system,
                    self.config.width as f32,
                    self.config.height as f32,
                );
                buffer.set_text(
                    &mut self.font_system,
                    &spec.text,
                    Attrs::new()
                        .family(Family::Monospace)
                        .color(glyphon::Color::rgb(
                            spec.color[0],
                            spec.color[1],
                            spec.color[2],
                        )),
                    Shaping::Advanced,
                );
                text_buffers.push((buffer, spec.left, spec.top));
            }

            // 2. FPS Text
            let mut fps_buffer = Buffer::new(&mut self.font_system, Metrics::new(20.0, 24.0));
            fps_buffer.set_size(
                &mut self.font_system,
                self.config.width as f32,
                self.config.height as f32,
            );
            fps_buffer.set_text(
                &mut self.font_system,
                &format!("FPS: {}", self.frame_stats.fps()),
                Attrs::new()
                    .family(Family::Monospace)
                    .color(glyphon::Color::rgb(0, 255, 0)),
                Shaping::Advanced,
            );

            let mut debug_buf = Buffer::new(&mut self.font_system, Metrics::new(14.0, 18.0));

            if player.debug_mode {
                let status = if controller.freeze_culling {
                    "FROZEN"
                } else {
                    "ACTIVE"
                };
                let stats = self.render_stats(rendered_chunks, rendered_lods);
                let target = controller
                    .cursor_id
                    .map(|id| format!("f{} l{} u{} v{}", id.face, id.layer, id.u, id.v));
                let info = stats.debug_overlay(
                    status,
                    self.frame_stats.frame_time_ms(),
                    player.position.to_array(),
                    target,
                );

                debug_buf.set_size(
                    &mut self.font_system,
                    self.config.width as f32,
                    self.config.height as f32,
                );
                debug_buf.set_text(
                    &mut self.font_system,
                    &info,
                    Attrs::new()
                        .family(Family::Monospace)
                        .color(glyphon::Color::rgb(200, 200, 200)),
                    Shaping::Advanced,
                );
            }

            // create text areas
            let mut text_areas: Vec<TextArea> = text_buffers
                .iter()
                .map(|(buf, x, y)| TextArea {
                    buffer: buf,
                    left: *x,
                    top: *y,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: self.config.width as i32,
                        bottom: self.config.height as i32,
                    },
                    default_color: glyphon::Color::rgb(255, 255, 255),
                })
                .collect();

            text_areas.push(TextArea {
                buffer: &fps_buffer,
                left: self.config.width as f32 - 120.0,
                top: 10.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: self.config.width as i32,
                    bottom: self.config.height as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            });

            if player.debug_mode {
                text_areas.push(TextArea {
                    buffer: &debug_buf,
                    left: self.config.width as f32 - 180.0,
                    top: 40.0,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: self.config.width as i32,
                        bottom: self.config.height as i32,
                    },
                    default_color: glyphon::Color::rgb(255, 255, 255),
                });
            }

            self.text_renderer
                .prepare(
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
                )
                .unwrap();

            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Text Pass"),
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

            self.text_renderer
                .render(&self.text_atlas, &mut pass)
                .unwrap();
        }

        self.queue.submit(std::iter::once(enc.finish()));
        out.present();
        self.text_atlas.trim();
        self.last_render_ms = render_started.elapsed().as_secs_f32() * 1000.0;
    }
}
