mod diagnostics;
use diagnostics::SystemDiagnostics;

use glam::Vec3;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;
use winit::event::{DeviceEvent, ElementState, Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

use vv_compiler::compile_assets_root;
use vv_config::EngineConfig;
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
    SystemDiagnostics::print_startup_info();

    // --- Configuration ------------------------------------------------------
    // All engine parameters live here. Change a value; it propagates everywhere.
    let config = EngineConfig::default();
    let compiled_content =
        compile_assets_root(Path::new("assets")).expect("assets packs should compile");
    let terrain = PlanetTerrain::generate(
        config.planet_resolution,
        &config.worldgen,
        &compiled_content.worldgen_content(),
        compiled_content
            .world_content()
            .world_settings()
            .voxel_size_m,
    )
    .expect("compiled worldgen content should generate terrain");
    let block_content = compiled_content.to_block_content();

    // --- Window & event loop ------------------------------------------------
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("VoxelVerse")
        .build(&event_loop)
        .unwrap();

    // --- Core systems -------------------------------------------------------
    let physics = Physics::new(config.physics.clone());
    let mut renderer = pollster::block_on(Renderer::new(&window, &config, block_content.clone()));
    let mut controller = Controller::new(&config.player);
    let mut cursor_focus = CursorFocus::default();
    let mut player = Player::new(&config.player);
    let mut gameplay = PlayerGameplayState::new(config.player.reach_distance);
    let mut console = Console::new();
    console.log("Welcome to VoxelVerse.", [0.0, 1.0, 0.0]);
    console.log("Press ` to open the console.", [1.0, 1.0, 1.0]);

    // --- Planet -------------------------------------------------------------
    println!("Building planet (resolution {})…", config.planet_resolution);
    let mut planet = PlanetData::new(
        config.planet_resolution,
        terrain,
        config.physics.core_protection_layers,
    );
    log_planet_info(&config, &compiled_content, &planet);

    // --- Player spawn -------------------------------------------------------
    let center = planet.resolution / 2;
    let ground_h = planet.terrain.get_height(0, center, center);
    let spawn_r = CoordSystem::get_layer_radius(ground_h, planet.resolution)
        + config.player.spawn_height_offset;
    player.spawn(Vec3::new(0.0, spawn_r, 0.0));

    // --- Main loop ----------------------------------------------------------
    let mut last_time = Instant::now();

    event_loop
        .run(move |event, target| {
            let now = Instant::now();
            let dt = (now - last_time).as_secs_f32();
            last_time = now;

            cursor_focus.apply(
                renderer.window,
                controller.first_person,
                console.is_open || gameplay.inventory_open,
            );

            // Physics + view update
            if !console.is_open && !gameplay.inventory_open {
                controller.update_player(&mut player, &planet, &physics, dt);
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
            let gameplay_events = gameplay.update(
                dt,
                player.position,
                interaction_target,
                placement_target,
                intent,
                &mut planet,
                &compiled_content,
            );
            for id in gameplay_events.changed_blocks {
                renderer.refresh_neighbors(id, &planet);
            }
            controller.cursor_id = gameplay.target.map(|target| target.block);
            renderer.update_cursor(&planet, controller.cursor_id);
            renderer.update_view(player.position, &planet);

            console.update_animation(dt);

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
                                    let new_res = planet.next_resolution(increase);
                                    let new_terrain = PlanetTerrain::generate(
                                        new_res,
                                        &config.worldgen,
                                        &compiled_content.worldgen_content(),
                                        compiled_content
                                            .world_content()
                                            .world_settings()
                                            .voxel_size_m,
                                    )
                                    .expect("compiled worldgen content should regenerate terrain");
                                    planet.apply_resize(new_res, new_terrain);

                                    let dir = if player.position.length() > 0.1 {
                                        player.position.normalize()
                                    } else {
                                        Vec3::Y
                                    };
                                    let probe = dir * (new_res as f32 / 2.0);
                                    let spawn_radius = CoordSystem::pos_to_id(probe, new_res)
                                        .map(|id| {
                                            CoordSystem::get_layer_radius(
                                                planet.terrain.get_height(id.face, id.u, id.v),
                                                new_res,
                                            ) + config.player.spawn_height_offset
                                        })
                                        .unwrap_or(new_res as f32 / 2.0 + 20.0);

                                    player.position = dir * spawn_radius;
                                    player.velocity = Vec3::ZERO;

                                    renderer.force_reload_all(&planet, player.position);
                                    renderer.log_memory(&planet);
                                    renderer.window.request_redraw();
                                }
                            }
                        }

                        WindowEvent::RedrawRequested => {
                            renderer.render(
                                &controller,
                                &player,
                                &physics,
                                &planet,
                                &console,
                                &gameplay,
                                &compiled_content,
                            );
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

fn log_planet_info(config: &EngineConfig, content: &CompiledContent, planet: &PlanetData) {
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

    println!("--- Planet Info ---");
    println!("Type      : {}", planet_type_key);
    println!("Seed      : {}", config.worldgen.noise_seed);
    println!("Resolution: {}", planet.resolution);
    println!("Voxel size: {:.2} m", world.world_settings().voxel_size_m);
    println!("Core lock : {} layers", planet.core_protection_layers);
    println!("Height    : {}..{} layers", min_height, max_height);
    println!(
        "Content   : {} blocks, {} items, {} recipes, {} biomes, {} flora, {} ores, {} fauna",
        content.blocks.len(),
        content.items.len(),
        content.recipes.len(),
        content.biomes.len(),
        content.flora.len(),
        content.ores.len(),
        content.fauna.len()
    );
    println!("Biomes generated on sampled surface:");
    for (biome, count) in biome_counts {
        println!("  - {}: {} samples", biome, count);
    }
    println!("-------------------");
}
