mod args;
mod block_selector;
mod camera;
mod debug_mode;
mod renderer;
mod scene;
mod viewer_screenshot;
mod viewer_ui;

use std::str::FromStr;
use std::time::Instant;

use winit::event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

use vv_compiler::compile_assets_root;
use vv_registry::ContentKey;

use args::{Scene, ViewerArgs, ViewerState};
use block_selector::BlockSelector;
use debug_mode::DebugMode;
use renderer::ViewerRenderer;
use scene::{build_grid, build_scene, layout_scene};

fn main() {
    let args = match ViewerArgs::parse() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            std::process::exit(1);
        }
    };

    println!("[vv-viewer] loading packs from {:?}", args.assets_root);
    let mut content = match compile_assets_root(&args.assets_root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[vv-viewer] pack compilation failed:");
            for diag in e.diagnostics() {
                eprintln!("  {diag:?}");
            }
            std::process::exit(1);
        }
    };
    println!(
        "[vv-viewer] compiled {} blocks, {} block visuals",
        content.blocks.len(),
        content.block_visuals.len()
    );

    // Build block selector from content.
    let mut selector = BlockSelector::from_content(&content);

    // Resolve initial blocks from CLI args (or pick first block in interactive mode).
    let initial_keys: Vec<ContentKey> = if args.block_keys.is_empty() {
        // Interactive: start with first available block.
        content
            .blocks
            .keys()
            .first()
            .cloned()
            .map(|k| vec![k])
            .unwrap_or_default()
    } else {
        args.block_keys
            .iter()
            .filter_map(|k| ContentKey::from_str(k).ok())
            .collect()
    };

    // Select first initial key in the selector.
    if let Some(first_key) = initial_keys.first() {
        selector.select_by_key(first_key);
    }

    // Build initial scene using the selected block.
    let mut state = ViewerState::default();
    state.scene = args.scene;
    if args.screenshot {
        state.needs_screenshot = true;
    }

    // Determine initial block ids for scene layout.
    let initial_block_ids: Vec<(vv_registry::BlockId, String)> = initial_keys
        .iter()
        .filter_map(|key| content.blocks.id(key).map(|id| (id, key.to_string())))
        .collect();

    let block_content = content.to_block_content();
    let initial_layout = layout_scene(state.scene, &initial_block_ids);
    let scene_mesh = build_scene(&initial_layout, &block_content);
    let grid_extent = scene_mesh.extent as f32 * 0.5;
    let (grid_verts, grid_inds) = build_grid(grid_extent + 1.0);

    let title = initial_keys
        .iter()
        .map(|k| k.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    // Create window + renderer.
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title(format!("vv-viewer \u{2014} {title}"))
        .with_inner_size(winit::dpi::LogicalSize::new(1400u32, 900u32))
        .build(&event_loop)
        .unwrap();

    let mut render = pollster::block_on(ViewerRenderer::new(
        &window,
        &content,
        &args.assets_root,
        &scene_mesh.vertices,
        &scene_mesh.indices,
        scene_mesh.extent,
        &grid_verts,
        &grid_inds,
    ));

    let mut mouse_drag = false;
    let _screenshot_dir = std::path::PathBuf::from("target/viewer-screenshots");
    let mut last_frame = Instant::now();

    // Helper to rebuild scene from current selector + state.
    let rebuild_scene =
        |state: &ViewerState, selector: &BlockSelector, content: &vv_registry::CompiledContent| {
            let block_ids: Vec<(vv_registry::BlockId, String)> = selector
                .selected()
                .and_then(|e| {
                    content
                        .blocks
                        .id(&e.key)
                        .map(|id| vec![(id, e.key.to_string())])
                })
                .unwrap_or_default();
            let layout = layout_scene(state.scene, &block_ids);
            let bc = content.to_block_content();
            let mesh = build_scene(&layout, &bc);
            let extent_f = mesh.extent as f32 * 0.5;
            let (gv, gi) = build_grid(extent_f + 1.0);
            (mesh, gv, gi)
        };

    println!(
        "[vv-viewer] ready | 1-9 debug modes | G grid | R reload | S screenshot | F reset cam | Space turntable"
    );

    event_loop
        .run(move |event, target| {
            // Advance turntable.
            let now = Instant::now();
            let dt = now.duration_since(last_frame).as_secs_f32();
            last_frame = now;
            if state.turntable {
                state.turntable_angle += dt * 45.0_f32.to_radians();
                render.camera.set_azimuth(state.turntable_angle);
            }

            match event {
                Event::AboutToWait => {
                    render.window.request_redraw();
                }

                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta: (dx, dy) },
                    ..
                } => {
                    if mouse_drag {
                        render.camera.pan_orbit(dx as f32, dy as f32);
                    }
                }

                Event::WindowEvent { event: ref wev, .. } => {
                    // Let egui handle the event first.
                    let consumed = render.on_window_event(wev);

                    match wev {
                        WindowEvent::CloseRequested => target.exit(),

                        WindowEvent::Resized(size) => {
                            render.resize(size.width, size.height);
                        }

                        WindowEvent::RedrawRequested => {
                            // Handle screenshot request.
                            if state.needs_screenshot {
                                state.needs_screenshot = false;
                                if let Some(entry) = selector.selected() {
                                    render.screenshot(&state, &entry.key);
                                }
                            }

                            // Render 3D + egui, get UI actions.
                            let actions = render.render(&mut state, &mut selector, &content);

                            // Handle UI actions.
                            if let Some(key) = actions.new_block {
                                selector.select_by_key(&key);
                                let (mesh, _gv, _gi) = rebuild_scene(&state, &selector, &content);
                                render.update_scene(&mesh.vertices, &mesh.indices, mesh.extent);
                                let _ = &mesh; // grid remains the same for now
                                render.camera.reset(mesh.extent.max(1));
                            }
                            if actions.scene_changed {
                                let (mesh, _gv2, _gi2) = rebuild_scene(&state, &selector, &content);
                                render.update_scene(&mesh.vertices, &mesh.indices, mesh.extent);
                                render.camera.reset(mesh.extent.max(1));
                            }
                            if actions.reload_requested {
                                reload_content(
                                    &args.assets_root,
                                    &mut content,
                                    &mut selector,
                                    &state,
                                    &mut render,
                                );
                            }
                            if actions.screenshot_requested {
                                state.needs_screenshot = true;
                            }
                        }

                        WindowEvent::MouseWheel { delta, .. } if !consumed => {
                            let scroll = match delta {
                                MouseScrollDelta::LineDelta(_, y) => *y,
                                MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 60.0,
                            };
                            render.camera.zoom(scroll);
                        }

                        WindowEvent::MouseInput {
                            button,
                            state: btn_state,
                            ..
                        } if !consumed => {
                            if *button == MouseButton::Left {
                                mouse_drag = *btn_state == ElementState::Pressed;
                            }
                        }

                        WindowEvent::KeyboardInput { event: ke, .. }
                            if ke.state == ElementState::Pressed && !consumed =>
                        {
                            // Handle R key directly (needs mutable content).
                            if ke.physical_key == PhysicalKey::Code(KeyCode::KeyR) {
                                reload_content(
                                    &args.assets_root,
                                    &mut content,
                                    &mut selector,
                                    &state,
                                    &mut render,
                                );
                            } else {
                                handle_key(
                                    ke,
                                    &mut state,
                                    &mut selector,
                                    &mut render,
                                    &content,
                                    &mut mouse_drag,
                                );
                            }
                        }

                        _ => {}
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}

fn handle_key(
    ke: &winit::event::KeyEvent,
    state: &mut ViewerState,
    selector: &mut BlockSelector,
    render: &mut ViewerRenderer<'_>,
    content: &vv_registry::CompiledContent,
    _mouse_drag: &mut bool,
) {
    let rebuild =
        |state: &ViewerState, selector: &BlockSelector, render: &mut ViewerRenderer<'_>| {
            let block_ids: Vec<(vv_registry::BlockId, String)> = selector
                .selected()
                .and_then(|e| {
                    content
                        .blocks
                        .id(&e.key)
                        .map(|id| vec![(id, e.key.to_string())])
                })
                .unwrap_or_default();
            let layout = layout_scene(state.scene, &block_ids);
            let bc = content.to_block_content();
            let mesh = build_scene(&layout, &bc);
            render.update_scene(&mesh.vertices, &mesh.indices, mesh.extent);
            mesh.extent
        };

    match ke.physical_key {
        PhysicalKey::Code(KeyCode::Digit1) | PhysicalKey::Code(KeyCode::Numpad1) => {
            state.debug_mode = DebugMode::Beauty;
        }
        PhysicalKey::Code(KeyCode::Digit2) | PhysicalKey::Code(KeyCode::Numpad2) => {
            state.debug_mode = DebugMode::FlatColor;
        }
        PhysicalKey::Code(KeyCode::Digit3) | PhysicalKey::Code(KeyCode::Numpad3) => {
            state.debug_mode = DebugMode::Palette;
        }
        PhysicalKey::Code(KeyCode::Digit4) | PhysicalKey::Code(KeyCode::Numpad4) => {
            state.debug_mode = DebugMode::Noise;
        }
        PhysicalKey::Code(KeyCode::Digit5) | PhysicalKey::Code(KeyCode::Numpad5) => {
            state.debug_mode = DebugMode::AoOnly;
        }
        PhysicalKey::Code(KeyCode::Digit6) | PhysicalKey::Code(KeyCode::Numpad6) => {
            state.debug_mode = DebugMode::FaceId;
        }
        PhysicalKey::Code(KeyCode::Digit7) | PhysicalKey::Code(KeyCode::Numpad7) => {
            state.debug_mode = DebugMode::Uv;
        }
        PhysicalKey::Code(KeyCode::Digit8) | PhysicalKey::Code(KeyCode::Numpad8) => {
            state.debug_mode = DebugMode::EdgesOnly;
        }
        PhysicalKey::Code(KeyCode::Digit9) | PhysicalKey::Code(KeyCode::Numpad9) => {
            state.debug_mode = DebugMode::NoVariation;
        }
        PhysicalKey::Code(KeyCode::KeyG) => {
            state.show_grid = !state.show_grid;
        }
        PhysicalKey::Code(KeyCode::KeyF) => {
            render.camera.reset(render.scene_extent.max(1));
        }
        PhysicalKey::Code(KeyCode::KeyR) => {
            // Handled in event loop (needs mutable content).
        }
        PhysicalKey::Code(KeyCode::KeyS) => {
            state.needs_screenshot = true;
        }
        PhysicalKey::Code(KeyCode::Space) => {
            state.turntable = !state.turntable;
        }
        PhysicalKey::Code(KeyCode::KeyW) => {
            let scenes = Scene::ALL;
            let idx = scenes.iter().position(|&s| s == state.scene).unwrap_or(0);
            state.scene = scenes[(idx + 1) % scenes.len()];
            rebuild(state, selector, render);
        }
        _ => {}
    }
}

fn reload_content(
    assets_root: &std::path::Path,
    content: &mut vv_registry::CompiledContent,
    selector: &mut BlockSelector,
    state: &ViewerState,
    render: &mut ViewerRenderer<'_>,
) {
    println!("[vv-viewer] reloading packs...");
    match compile_assets_root(assets_root) {
        Ok(new_content) => {
            println!(
                "[vv-viewer] reloaded: {} blocks, {} visuals",
                new_content.blocks.len(),
                new_content.block_visuals.len()
            );
            *content = new_content;
            selector.rebuild_from_content(content);
            render.update_content(content);

            // Rebuild scene from selector.
            let block_ids: Vec<(vv_registry::BlockId, String)> = selector
                .selected()
                .and_then(|e| {
                    content
                        .blocks
                        .id(&e.key)
                        .map(|id| vec![(id, e.key.to_string())])
                })
                .unwrap_or_default();
            let layout = layout_scene(state.scene, &block_ids);
            let bc = content.to_block_content();
            let mesh = build_scene(&layout, &bc);
            render.update_scene(&mesh.vertices, &mesh.indices, mesh.extent);
        }
        Err(e) => {
            eprintln!("[vv-viewer] reload failed:");
            for d in e.diagnostics() {
                eprintln!("  {d:?}");
            }
        }
    }
}
