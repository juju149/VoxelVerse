mod content;
mod diagnostics;
mod gameplay;
mod generation;
mod input;
mod math;
mod meshing;
mod physics;
mod rendering;
mod voxel;
mod world;

use crate::diagnostics::{Console, SystemDiagnostics};
use crate::gameplay::{BlockActionIntent, BlockInteraction, Player};
use crate::generation::CoordSystem;
use crate::input::Controller;
use crate::rendering::Renderer;
use crate::world::PlanetData;
use std::time::Instant;
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent}; // Added DeviceEvent
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, WindowBuilder};

fn main() {
    SystemDiagnostics::print_startup_info();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("voxanet")
        .build(&event_loop)
        .unwrap();

    let mut renderer = pollster::block_on(Renderer::new(&window));
    let mut controller = Controller::new();
    let mut player = Player::new();
    let mut planet = PlanetData::new(1000); // Keep high resolution

    let mut console = Console::new();
    console.log("Welcome to voxanet.", [0.0, 1.0, 0.0]);
    console.log("Press ` to open console.", [1.0, 1.0, 1.0]);

    player.spawn(planet.spawn_position());
    let mut last_time = Instant::now();
    let mut current_mode_first_person = false;

    event_loop
        .run(move |event, target| {
            match event {
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } if !console.is_open => {
                    controller.process_mouse_motion(delta);
                }

                Event::WindowEvent { event, window_id } if window_id == renderer.window.id() => {
                    // CONSOLE INPUT INTERCEPTION
                    if console.is_open {
                        if let WindowEvent::KeyboardInput {
                            event: key_event, ..
                        } = event
                        {
                            if key_event.state == ElementState::Pressed {
                                match key_event.physical_key {
                                    PhysicalKey::Code(KeyCode::Backquote) => console.toggle(),
                                    PhysicalKey::Code(KeyCode::Enter) => {
                                        console.submit(&mut player)
                                    }
                                    PhysicalKey::Code(KeyCode::Backspace) => {
                                        console.handle_backspace()
                                    }
                                    _ => {
                                        if let Some(txt) = &key_event.text {
                                            // Append text to console buffer
                                            for c in txt.chars() {
                                                console.handle_char(c);
                                            }
                                        }
                                    }
                                }
                            }
                            return;
                        }
                    }

                    if let WindowEvent::KeyboardInput {
                        event: key_event, ..
                    } = &event
                    {
                        if key_event.state == ElementState::Pressed {
                            if let PhysicalKey::Code(KeyCode::Backquote) = key_event.physical_key {
                                console.toggle();
                                return;
                            }
                        }
                    }

                    if console.is_open {
                        match event {
                            WindowEvent::CloseRequested => target.exit(),
                            WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
                            WindowEvent::RedrawRequested => {
                                renderer.render(&controller, &player, &planet, &console);
                            }
                            _ => {}
                        }
                        return;
                    }

                    controller.process_events(&event, &mut player, &planet);

                    match event {
                        WindowEvent::CloseRequested => target.exit(),
                        WindowEvent::Resized(size) => renderer.resize(size.width, size.height),

                        WindowEvent::MouseInput {
                            state: ElementState::Pressed,
                            button,
                            ..
                        } => {
                            let intent = match button {
                                MouseButton::Right => Some(BlockActionIntent::Place),
                                MouseButton::Left => Some(BlockActionIntent::Mine),
                                _ => None,
                            };

                            if let Some(intent) = intent {
                                let placement = if intent == BlockActionIntent::Place {
                                    controller
                                        .raycast(
                                            &player,
                                            &planet,
                                            renderer.config.width as f32,
                                            renderer.config.height as f32,
                                            true,
                                        )
                                        .map(|(id, _)| id)
                                } else {
                                    None
                                };

                                if let Some(action) = BlockInteraction::resolve(
                                    intent,
                                    controller.cursor_id,
                                    placement,
                                ) {
                                    let changed = BlockInteraction::apply(action, &mut planet);
                                    renderer.refresh_neighbors(changed, &planet);
                                    renderer.window.request_redraw();
                                } else if controller.cursor_id.is_none() && controller.first_person
                                {
                                    let _ = renderer.window.set_cursor_grab(CursorGrabMode::Locked);
                                    renderer.window.set_cursor_visible(false);
                                }
                            }
                        }

                        WindowEvent::KeyboardInput { event, .. }
                            if event.state == ElementState::Pressed =>
                        {
                            if let Key::Character(ref s) = event.logical_key {
                                if s == "]" || s == "[" {
                                    if s == "]" {
                                        planet.resize(true);
                                    } else {
                                        planet.resize(false);
                                    }

                                    let new_res = planet.resolution;
                                    let current_dir = if player.position.length() > 0.1 {
                                        player.position.normalize()
                                    } else {
                                        glam::Vec3::Y
                                    };
                                    let probe_dist = planet.profile.surface_radius;
                                    let dummy_pos = current_dir * probe_dist;

                                    let spawn_radius = if let Some(id) =
                                        CoordSystem::pos_to_id(dummy_pos, new_res)
                                    {
                                        let h = planet.terrain.get_height(id.face, id.u, id.v);
                                        planet.profile.layer_radius(h + 1)
                                            + planet.profile.spawn_clearance()
                                    } else {
                                        (new_res as f32 / 2.0) + 20.0
                                    };

                                    player.position = current_dir * spawn_radius;
                                    player.velocity = glam::Vec3::ZERO;

                                    renderer.force_reload_all(&planet, player.position);
                                    renderer.log_memory(&planet);
                                    renderer.window.request_redraw();
                                }
                            }
                        }

                        WindowEvent::RedrawRequested => {
                            renderer.render(&controller, &player, &planet, &console);
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    let now = Instant::now();
                    let dt = (now - last_time).as_secs_f32();
                    last_time = now;

                    if controller.first_person != current_mode_first_person {
                        current_mode_first_person = controller.first_person;
                        if current_mode_first_person && !console.is_open {
                            let _ = renderer.window.set_cursor_grab(CursorGrabMode::Locked);
                            renderer.window.set_cursor_visible(false);
                        } else {
                            let _ = renderer.window.set_cursor_grab(CursorGrabMode::None);
                            renderer.window.set_cursor_visible(true);
                        }
                    }

                    console.update_animation(dt);

                    if !console.is_open {
                        controller.update_player(&mut player, &planet, dt);

                        let width = renderer.config.width as f32;
                        let height = renderer.config.height as f32;
                        let ray_result = controller.raycast(&player, &planet, width, height, false);
                        controller.cursor_id = ray_result.map(|(id, _)| id);
                    } else {
                        controller.clear_transient_input();
                        controller.cursor_id = None;
                        let _ = renderer.window.set_cursor_grab(CursorGrabMode::None);
                        renderer.window.set_cursor_visible(true);
                    }

                    renderer.update_cursor(&planet, controller.cursor_id);
                    renderer.update_view(player.position, &planet);
                    renderer.window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}
