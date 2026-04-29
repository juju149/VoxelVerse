mod diagnostics;
use diagnostics::SystemDiagnostics;

use glam::Vec3;
use std::collections::BTreeMap;
use std::env;
use std::path::Path;
use std::time::Instant;
use winit::event::{DeviceEvent, ElementState, Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

use vv_compiler::compile_assets_root;
use vv_config::EngineConfig;
use vv_diagnostics::{EngineDiagnostics, LogDomain, LogLevel, PerfPhase, PhaseTimer};
use vv_gameplay::{
    Console, InteractionTarget, InventoryPointerIntent, Player, PlayerGameplayState, PlayerIntent,
};
use vv_input::{Controller, CursorFocus, UiPointerEvent};
use vv_physics::Physics;
use vv_planet::CoordSystem;
use vv_registry::{CompiledContent, PlanetTypeSource, WorldSettingsSource};
use vv_render::Renderer;
use vv_world_gen::PlanetTerrain;
use vv_world_runtime::PlanetData;

fn main() {
    let mut diagnostics = EngineDiagnostics::from_env();
    diagnostics.log(
        LogLevel::Info,
        LogDomain::Startup,
        format!(
            "diagnostics mode={} level={:?} env VV_DIAGNOSTICS=normal|debug|perf VV_LOG=trace|debug|info|warn|error",
            diagnostics.config().mode.as_str(),
            diagnostics.config().min_level,
        ),
    );
    SystemDiagnostics::print_startup_info(diagnostics.config());

    // --- Configuration ------------------------------------------------------
    // All engine parameters live here. Change a value; it propagates everywhere.
    let config = EngineConfig::default();
    diagnostics.log(
        LogLevel::Info,
        LogDomain::Config,
        format!(
            "world seed={} lod_grid={} shadow_map={} fov_fp={} fov_orbit={} physics gravity={} core_layers={}",
            config.worldgen.noise_seed,
            config.lod.tile_grid_res,
            config.render.shadow_map_size,
            config.render.fov_first_person_deg,
            config.render.fov_orbit_deg,
            config.physics.gravity,
            config.physics.core_protection_layers,
        ),
    );
    let compile_timer = PhaseTimer::start(PerfPhase::Worldgen);
    let compiled_content =
        compile_assets_root(Path::new("assets")).expect("assets packs should compile");
    diagnostics.record_startup_phase(PerfPhase::Worldgen, compile_timer.finish().duration);
    diagnostics.log(
        LogLevel::Info,
        LogDomain::Startup,
        format!(
            "content compiled blocks={} items={} recipes={} biomes={} flora={} ores={} fauna={}",
            compiled_content.blocks.len(),
            compiled_content.items.len(),
            compiled_content.recipes.len(),
            compiled_content.biomes.len(),
            compiled_content.flora.len(),
            compiled_content.ores.len(),
            compiled_content.fauna.len(),
        ),
    );
    let terrain_timer = PhaseTimer::start(PerfPhase::Worldgen);
    let terrain = PlanetTerrain::generate(
        &config.worldgen,
        &compiled_content.worldgen_content(),
        compiled_content.world_content().world_settings(),
    )
    .expect("compiled worldgen content should generate terrain");
    diagnostics.record_startup_phase(PerfPhase::Worldgen, terrain_timer.finish().duration);
    let planet_geometry = terrain.geometry();
    let block_content = compiled_content.to_block_content();

    // --- Window & event loop ------------------------------------------------
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("VoxelVerse")
        .build(&event_loop)
        .unwrap();

    // --- Core systems -------------------------------------------------------
    let physics = Physics::new(config.physics.clone());
    let mut renderer = pollster::block_on(Renderer::new(
        &window,
        &config,
        block_content.clone(),
        diagnostics.config(),
    ));
    let mut controller = Controller::new(&config.player);
    let mut cursor_focus = CursorFocus::default();
    let mut player = Player::new(&config.player);
    let mut gameplay = PlayerGameplayState::new(config.player.reach_distance);
    let mut console = Console::new();
    console.log("Welcome to VoxelVerse.", [0.0, 1.0, 0.0]);
    console.log("Press ` to open the console.", [1.0, 1.0, 1.0]);

    // --- Planet -------------------------------------------------------------
    diagnostics.log(
        LogLevel::Info,
        LogDomain::World,
        format!(
            "building planet radius={:.1}m voxel={:.3}m resolution={}",
            planet_geometry.radius_m, planet_geometry.voxel_size_m, planet_geometry.resolution
        ),
    );
    let mut planet = PlanetData::new(
        planet_geometry,
        terrain,
        config.physics.core_protection_layers,
    );
    log_planet_info(&diagnostics, &config, &compiled_content, &planet);

    // --- Player spawn -------------------------------------------------------
    let center = planet.resolution / 2;
    let ground_h = planet.terrain.get_height(0, center, center);
    let spawn_r = CoordSystem::get_layer_radius(ground_h, planet.geometry)
        + config.player.spawn_height_offset;
    player.spawn(Vec3::new(0.0, spawn_r, 0.0));

    // --- Main loop ----------------------------------------------------------
    let mut last_time = Instant::now();
    let exit_after_frames = env::var("VV_EXIT_AFTER_FRAMES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok());
    let mut rendered_frames = 0u64;

    event_loop
        .run(move |event, target| {
            let now = Instant::now();
            let dt = (now - last_time).as_secs_f32();
            last_time = now;

            renderer.begin_diagnostic_frame();
            let mut frame = diagnostics.begin_frame(dt);

            let input_timer = PhaseTimer::start(PerfPhase::Input);
            cursor_focus.apply(
                renderer.window,
                controller.first_person,
                console.is_open || gameplay.inventory_open,
            );
            frame.record(input_timer.finish());

            // Physics + view update
            if !console.is_open && !gameplay.inventory_open {
                let physics_timer = PhaseTimer::start(PerfPhase::Physics);
                controller.update_player(&mut player, &planet, &physics, dt);
                frame.record(physics_timer.finish());
            }

            let w = renderer.config.width as f32;
            let h = renderer.config.height as f32;
            let ray = controller.raycast(&player, &planet, &physics, w, h, &config.render, false);
            let interaction_target =
                ray.map(|(block, distance)| InteractionTarget { block, distance });
            let placement_target = controller
                .raycast(&player, &planet, &physics, w, h, &config.render, true)
                .map(|(id, _)| id);
            let mut intent = controller.take_gameplay_intent();
            let ui_pointer_events = controller.take_ui_pointer_events();
            if console.is_open {
                intent = PlayerIntent::default();
            } else {
                for event in ui_pointer_events {
                    if !gameplay.inventory_open {
                        continue;
                    }
                    match event {
                        UiPointerEvent::PrimaryPressed(pos) => {
                            if let Some(slot) = renderer.inventory_slot_at(&gameplay, pos) {
                                intent
                                    .inventory_pointers
                                    .push(InventoryPointerIntent::BeginDrag(slot));
                            }
                        }
                        UiPointerEvent::PrimaryReleased(pos) => {
                            if let Some(recipe) =
                                renderer.inventory_recipe_at(&gameplay, &compiled_content, pos)
                            {
                                intent.craft_recipe = Some(recipe);
                            } else {
                                let slot = renderer.inventory_slot_at(&gameplay, pos);
                                intent
                                    .inventory_pointers
                                    .push(InventoryPointerIntent::EndDrag(slot));
                            }
                        }
                    }
                }
                if gameplay.inventory_open {
                    intent.mine_held = false;
                    intent.place_pressed = false;
                }
            }
            let gameplay_timer = PhaseTimer::start(PerfPhase::Gameplay);
            let gameplay_events = gameplay.update(
                dt,
                player.position,
                interaction_target,
                placement_target,
                intent,
                &mut planet,
                &compiled_content,
            );
            frame.record(gameplay_timer.finish());
            let mesh_timer = PhaseTimer::start(PerfPhase::Meshing);
            for id in gameplay_events.changed_blocks {
                renderer.refresh_neighbors(id, &planet);
            }
            frame.record(mesh_timer.finish());
            controller.cursor_id = gameplay.target.map(|target| target.block);
            renderer.update_cursor(&planet, controller.cursor_id);
            let view_timer = PhaseTimer::start(PerfPhase::ViewVisibility);
            renderer.update_view(player.position, &planet);
            frame.record(view_timer.finish());
            frame.record_duration(PerfPhase::LodCoverage, renderer.lod_coverage_time());
            frame.record_duration(PerfPhase::ChunkStreaming, renderer.chunk_streaming_time());

            let ui_timer = PhaseTimer::start(PerfPhase::Ui);
            console.update_animation(dt);
            frame.record(ui_timer.finish());

            match event {
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    controller.process_mouse_motion(delta);
                }

                Event::WindowEvent { event, window_id } if window_id == renderer.window.id() => {
                    // Console intercepts input when open
                    if console.is_open {
                        match &event {
                            WindowEvent::KeyboardInput { event: ke, .. }
                                if ke.state == ElementState::Pressed =>
                            {
                                match ke.physical_key {
                                    PhysicalKey::Code(KeyCode::Backquote) => console.toggle(),
                                    PhysicalKey::Code(KeyCode::Enter) => {
                                        console.submit(&mut player)
                                    }
                                    PhysicalKey::Code(KeyCode::Backspace) => {
                                        console.handle_backspace()
                                    }
                                    _ => {
                                        if let Some(txt) = &ke.text {
                                            for c in txt.chars() {
                                                console.handle_char(c);
                                            }
                                        }
                                    }
                                }
                                return;
                            }
                            _ => {}
                        }
                    }

                    // Backtick toggles console regardless of mode
                    if let WindowEvent::KeyboardInput { event: ke, .. } = &event {
                        if ke.state == ElementState::Pressed {
                            if let PhysicalKey::Code(KeyCode::Backquote) = ke.physical_key {
                                console.toggle();
                                return;
                            }
                        }
                    }

                    controller.process_events(&event, &mut player);

                    match event {
                        WindowEvent::CloseRequested => target.exit(),
                        WindowEvent::Resized(size) => renderer.resize(size.width, size.height),

                        WindowEvent::MouseInput { .. } => {}

                        WindowEvent::KeyboardInput { event, .. }
                            if event.state == ElementState::Pressed =>
                        {
                            if let Key::Character(ref s) = event.logical_key {
                                if s == "]" || s == "[" {
                                    let increase = s == "]";
                                    let new_geometry = planet.next_geometry(increase);
                                    diagnostics.log(
                                        LogLevel::Info,
                                        LogDomain::World,
                                        format!(
                                            "resizing planet old_resolution={} new_resolution={} direction={}",
                                            planet.resolution,
                                            new_geometry.resolution,
                                            if increase { "increase" } else { "decrease" }
                                        ),
                                    );
                                    let resize_timer = PhaseTimer::start(PerfPhase::Worldgen);
                                    let new_terrain = PlanetTerrain::generate_for_geometry(
                                        new_geometry,
                                        &config.worldgen,
                                        &compiled_content.worldgen_content(),
                                    )
                                    .expect("compiled worldgen content should regenerate terrain");
                                    diagnostics.record_startup_phase(
                                        PerfPhase::Worldgen,
                                        resize_timer.finish().duration,
                                    );
                                    planet.apply_resize(new_geometry, new_terrain);

                                    let dir = if player.position.length() > 0.1 {
                                        player.position.normalize()
                                    } else {
                                        Vec3::Y
                                    };
                                    let probe = dir * planet.geometry.radius_m;
                                    let spawn_radius =
                                        CoordSystem::pos_to_id(probe, planet.geometry)
                                            .map(|id| {
                                                CoordSystem::get_layer_radius(
                                                    planet.terrain.get_height(id.face, id.u, id.v),
                                                    planet.geometry,
                                                ) + config.player.spawn_height_offset
                                            })
                                            .unwrap_or(planet.geometry.radius_m + 20.0);

                                    player.position = dir * spawn_radius;
                                    player.velocity = Vec3::ZERO;

                                    renderer.force_reload_all(&planet, player.position);
                                    renderer.log_memory(&planet);
                                    renderer.window.request_redraw();
                                }
                            }
                        }

                        WindowEvent::RedrawRequested => {
                            let render_timer = PhaseTimer::start(PerfPhase::Render);
                            renderer.render(
                                &controller,
                                &player,
                                &physics,
                                &planet,
                                &console,
                                &gameplay,
                                &compiled_content,
                            );
                            frame.record(render_timer.finish());
                            frame
                                .record_duration(PerfPhase::RenderPrep, renderer.render_prep_time());
                            let snapshot = renderer.diagnostic_snapshot(&planet, &gameplay);
                            frame.record_duration(PerfPhase::GpuUpload, snapshot.gpu.upload_time);
                            diagnostics.finish_frame(frame, snapshot);
                            rendered_frames += 1;
                            if exit_after_frames.is_some_and(|limit| rendered_frames >= limit) {
                                target.exit();
                            }
                        }
                        _ => {}
                    }
                }

                Event::AboutToWait => renderer.window.request_redraw(),
                _ => {}
            }
        })
        .unwrap();
}

fn log_planet_info(
    diagnostics: &EngineDiagnostics,
    config: &EngineConfig,
    content: &CompiledContent,
    planet: &PlanetData,
) {
    let world = content.world_content();
    let worldgen = content.worldgen_content();
    let planet_type = worldgen
        .default_planet_type()
        .and_then(|id| worldgen.planet_type(id));
    let planet_type_key = planet_type
        .map(|view| view.key.to_string())
        .unwrap_or_else(|| "<missing>".to_owned());

    let sample_steps = 24;
    let mut biome_counts = BTreeMap::<String, u32>::new();
    let mut min_height = u32::MAX;
    let mut max_height = 0;
    let step = (planet.resolution / sample_steps).max(1);

    for face in 0..6 {
        let mut u = 0;
        while u < planet.resolution {
            let mut v = 0;
            while v < planet.resolution {
                let height = planet.terrain.get_height(face, u, v);
                min_height = min_height.min(height);
                max_height = max_height.max(height);
                let biome_id = planet.terrain.get_biome(face, u, v);
                let biome_key = content
                    .biomes
                    .key(biome_id)
                    .map(|key| key.to_string())
                    .unwrap_or_else(|| format!("<unknown:{biome_id:?}>"));
                *biome_counts.entry(biome_key).or_default() += 1;
                v += step;
            }
            u += step;
        }
    }

    diagnostics.log(
        LogLevel::Info,
        LogDomain::World,
        format!(
            "planet type={} seed={} resolution={} radius={:.1}m voxel={:.3}m core_lock={} height_layers={}..{}",
            planet_type_key,
            config.worldgen.noise_seed,
            planet.resolution,
            planet.geometry.radius_m,
            world.world_settings().voxel_size_m,
            planet.core_protection_layers,
            min_height,
            max_height,
        ),
    );
    let biome_summary = biome_counts
        .into_iter()
        .map(|(biome, count)| format!("{biome}:{count}"))
        .collect::<Vec<_>>()
        .join(", ");
    diagnostics.log(
        LogLevel::Info,
        LogDomain::Worldgen,
        format!("sampled_biomes {}", biome_summary),
    );
}
