use super::terrain_renderer::TerrainRenderer;
use super::{GlobalUniform, LocalUniform, Renderer};
use crate::lod_animation::AnyKey;
use crate::snapshot::RenderFrameSnapshot;
use crate::types::ChunkMesh;
use vv_math::Frustum;

impl<'a> Renderer<'a> {
    pub fn render(&mut self, frame: &RenderFrameSnapshot<'_>) {
        let camera = &frame.camera;
        let planet = frame.planet;
        let hotbar = &frame.hotbar;
        let inventory = &frame.inventory;
        let inventory_ui = &frame.ui.inventory;
        let craft = &frame.craft;
        let console = &frame.console;
        let debug = &frame.debug;
        let render_started = std::time::Instant::now();
        self.update_console_mesh(console.height_fraction);
        if inventory_ui.is_open {
            self.hotbar_inds = 0;
            self.hotbar_cache_signature = None;
        } else {
            self.update_hotbar_mesh(hotbar, planet);
        }
        self.update_inventory_mesh(inventory, hotbar, inventory_ui, planet, craft);

        self.update_collision_debug_mesh(debug.show_collisions, camera.player_pos, planet);

        let out = match self.surface.get_current_texture() {
            Ok(o) => o,
            _ => return,
        };
        let view = out
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let surface_radius = planet.profile.surface_radius;
        let mut atmosphere =
            self.atmosphere
                .evaluate(surface_radius, planet.world_time, self.quality);
        if let Some(weather) = frame.weather {
            atmosphere.apply_weather(weather);
        }
        if let Some(celestial) = frame.celestial {
            atmosphere.apply_celestial(celestial);
        }
        let sun_dir = atmosphere.sun_dir;

        let shadow_dist = 200.0;
        let proj_size = 60.0;

        // basic LookAt
        let center = camera.player_pos;
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
        let mvp = camera.view_proj;

        // --- FRUSTUM CULLING LOGIC ---
        let current_frustum = Frustum::from_matrix(mvp);

        // determine which frustum to use for culling
        // if freeze is on, we use the stored one. if freeze is off, update the stored one (or just use current).
        let cull_frustum = if debug.freeze_culling {
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

        let cam_pos = camera.camera_pos;
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
        let quality_bits = self.quality.pack();

        // Pack weather params for the WGSL globals. Order kept in sync with
        // `weather_params` in include/camera/globals.wgsl.
        // x = precipitation intensity, y/z = horizontal wind direction,
        // w = precipitation kind (0=none, 1=rain, 2=snow, 3=sleet,
        //                         4=sand,  5=ash,  6=toxic_mist).
        let weather_params = match frame.weather {
            Some(w) => {
                let kind = match w.precipitation.kind {
                    vv_weather::PrecipitationKindSample::None => 0.0,
                    vv_weather::PrecipitationKindSample::Rain => 1.0,
                    vv_weather::PrecipitationKindSample::Snow => 2.0,
                    vv_weather::PrecipitationKindSample::Sleet => 3.0,
                    vv_weather::PrecipitationKindSample::Sand => 4.0,
                    vv_weather::PrecipitationKindSample::Ash => 5.0,
                    vv_weather::PrecipitationKindSample::ToxicMist => 6.0,
                };
                [
                    w.precipitation.intensity,
                    w.wind.direction.x,
                    w.wind.direction.z,
                    kind,
                ]
            }
            None => [0.0, 1.0, 0.0, 0.0],
        };

        // Pack celestial params. Order kept in sync with `celestial_params`
        // and `celestial_moon` in include/camera/globals.wgsl.
        let (celestial_params, celestial_moon) = match frame.celestial {
            Some(c) => {
                let (moon_dir, moon_radius) = match c.moons.first() {
                    Some(m) => (m.direction, m.angular_radius_rad),
                    None => (glam::Vec3::ZERO, 0.0),
                };
                (
                    [
                        c.eclipse_factor,
                        c.stars_visibility,
                        c.aurora_intensity,
                        c.sun_disc_angular_radius,
                    ],
                    [moon_dir.x, moon_dir.y, moon_dir.z, moon_radius],
                )
            }
            None => ([0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0]),
        };

        let global_uniform = |view_proj: glam::Mat4| GlobalUniform {
            view_proj: view_proj.to_cols_array(),
            light_view_proj: light_view_proj.to_cols_array(),
            cam_pos: [cam_pos.x, cam_pos.y, cam_pos.z, quality_bits],
            sun_dir: [
                atmosphere.sun_dir.x,
                atmosphere.sun_dir.y,
                atmosphere.sun_dir.z,
                atmosphere.fog_density,
            ],
            sky_horizon: [
                atmosphere.sky_horizon.x,
                atmosphere.sky_horizon.y,
                atmosphere.sky_horizon.z,
                atmosphere.time_of_day,
            ],
            sky_zenith: [
                atmosphere.sky_zenith.x,
                atmosphere.sky_zenith.y,
                atmosphere.sky_zenith.z,
                atmosphere.sun_intensity,
            ],
            render_params: [
                atmosphere.elapsed_seconds,
                quality_bits,
                self.config.width as f32,
                self.config.height as f32,
            ],
            atmosphere_params: [
                atmosphere.fog_density,
                atmosphere.height_fog_strength,
                atmosphere.volumetric_fog_strength,
                atmosphere.exposure,
            ],
            cloud_params: [
                atmosphere.cloud_steps,
                atmosphere.cloud_density,
                atmosphere.cloud_speed,
                atmosphere.cloud_coverage,
            ],
            water_params: [
                atmosphere.water.fresnel,
                atmosphere.water.specular,
                atmosphere.water.alpha,
                0.0,
            ],
            weather_params,
            celestial_params,
            celestial_moon,
        };

        // 1. Build and upload the main global uniform.
        let global_data = global_uniform(mvp);
        self.queue
            .write_buffer(&self.global_buf, 0, bytemuck::cast_slice(&[global_data]));

        // 2. Shadow global uniform uses the light matrix as view_proj.
        let shadow_uniform_data = global_uniform(light_view_proj);
        self.queue.write_buffer(
            &self.shadow_global_buf,
            0,
            bytemuck::cast_slice(&[shadow_uniform_data]),
        );

        let model_mat = camera.model_matrix;
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

            // Only near voxel chunks contribute to the sun's 60-unit ortho box.
            // LOD tiles live well outside the shadow frustum, so iterating them
            // here only wastes per-mesh `intersects_sphere` work on the CPU.
            for mesh in self.chunks.values() {
                if TerrainRenderer::behind_planet_horizon(
                    surface_radius,
                    cam_pos,
                    mesh.center,
                    mesh.radius,
                ) {
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
        self.render_sky(&mut enc);

        // --- PASS 2b: CELESTIAL (Phase 5.B) ---
        // Stars + moon + aurora additive overlay between sky and clouds so
        // clouds occlude them naturally. Self-skips when no celestial state
        // is supplied (every input is zero).
        self.render_celestial(&mut enc);

        // --- PASS 3: CLOUDS ---
        self.render_clouds(&mut enc);

        // --- PASS 4: MAIN RENDER ---
        let terrain_draw_started = std::time::Instant::now();
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.scene.view,
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

            if debug.is_wireframe {
                pass.set_pipeline(&self.pipeline_wire);
            } else {
                pass.set_pipeline(&self.pipeline_fill);
            }

            // Set atlas bind group once for the whole main pass.
            pass.set_bind_group(0, &self.global_bind, &[]);
            pass.set_bind_group(2, &self.atlas_bind, &[]);

            // DRAW LOD CHUNKS
            for mesh in self.lod_chunks.values() {
                if TerrainRenderer::behind_planet_horizon(
                    surface_radius,
                    cam_pos,
                    mesh.center,
                    mesh.radius,
                ) {
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
                if TerrainRenderer::behind_planet_horizon(
                    surface_radius,
                    cam_pos,
                    mesh.center,
                    mesh.radius,
                ) {
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

            if !camera.is_first_person {
                if debug.is_wireframe {
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

            main_draw_calls += self.draw_collision_debug(&mut pass);

            if self.block_damage_inds > 0 {
                pass.set_pipeline(&self.pipeline_line);
                pass.set_bind_group(0, &self.global_bind, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.block_damage_v_buf.slice(..));
                pass.set_index_buffer(self.block_damage_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.block_damage_inds, 0, 0..1);
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

            if camera.is_first_person {
                pass.set_pipeline(&self.pipeline_line);
                pass.set_bind_group(0, &self.global_bind_identity, &[]);
                pass.set_bind_group(1, &self.local_bind_identity, &[]);
                pass.set_vertex_buffer(0, self.cross_v_buf.slice(..));
                pass.set_index_buffer(self.cross_i_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.cross_inds, 0, 0..1);
                main_draw_calls += 1;
            }
        }
        self.last_terrain_draw_ms = terrain_draw_started.elapsed().as_secs_f32() * 1000.0;

        // --- PASS 5: VOLUMETRIC FOG VEIL ---
        self.render_volumetric_fog(&mut enc);

        // --- PASS 5b: PRECIPITATION (Phase 3.B) ---
        // Procedural rain/snow screen-space overlay. Self-skips in the
        // shader when `weather_params.x == 0`, so it costs ~nothing when no
        // weather state is supplied.
        self.render_precipitation(&mut enc);

        // --- PASS 6: FINAL COMPOSITE ---
        self.render_final_composite(&mut enc, &view);

        main_draw_calls += self.render_ui_mesh_pass(&mut enc, &view, camera.is_first_person);

        self.last_draw_calls = main_draw_calls;
        self.last_shadow_draw_calls = shadow_draw_calls;

        self.frame_stats.tick();

        self.render_text_pass(&mut enc, &view, frame, rendered_chunks, rendered_lods);

        self.queue.submit(std::iter::once(enc.finish()));
        out.present();
        self.text_atlas.trim();
        self.last_render_ms = render_started.elapsed().as_secs_f32() * 1000.0;
    }
}
