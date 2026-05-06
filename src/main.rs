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
use crate::gameplay::{
    BlockActionIntent, BlockInteraction, BlockSelection, BlockSelectionMode, PlanetResize,
    PlanetResizeIntent, Player, PlayerController,
};
use crate::input::Controller;
use crate::rendering::Renderer;
use crate::world::PlanetData;
use std::time::Instant;
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent}; // Added DeviceEvent
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
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
    let mut planet = PlanetData::new(1000);

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

                    controller.process_events(&event, &player);

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
                                    let ray = controller.view_ray(
                                        &player,
                                        renderer.config.width as f32,
                                        renderer.config.height as f32,
                                    );
                                    BlockSelection::trace(
                                        ray,
                                        controller.interaction_reach(),
                                        &planet,
                                        BlockSelectionMode::Placement,
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
                            let resize = match event.physical_key {
                                PhysicalKey::Code(KeyCode::BracketRight) => {
                                    Some(PlanetResizeIntent::Grow)
                                }
                                PhysicalKey::Code(KeyCode::BracketLeft) => {
                                    Some(PlanetResizeIntent::Shrink)
                                }
                                _ => None,
                            };

                            if let Some(intent) = resize {
                                PlanetResize::apply(intent, &mut planet, &mut player);
                                renderer.force_reload_all(&planet, player.position);
                                renderer.log_memory(&planet);
                                renderer.window.request_redraw();
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
                        let player_input = controller.sample_player_input();
                        PlayerController::update(&mut player, &planet, player_input, dt);

                        let width = renderer.config.width as f32;
                        let height = renderer.config.height as f32;
                        let ray = controller.view_ray(&player, width, height);
                        let ray_result = BlockSelection::trace(
                            ray,
                            controller.interaction_reach(),
                            &planet,
                            BlockSelectionMode::HitSolid,
                        );
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
