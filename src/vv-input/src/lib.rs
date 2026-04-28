use glam::{Mat4, Vec2, Vec3};
use vv_config::PlayerConfig;
use vv_core::BlockId;
use vv_gameplay::{Player, PlayerIntent};
use vv_physics::Physics;
use vv_planet::CoordSystem;
use vv_world_runtime::PlanetData;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Input state, camera control, and player action relay.
///
/// `Controller` translates raw device events into abstract player intent
/// and drives the `Player` update each frame.
pub struct Controller {
    // Camera orbit state (third-person mode)
    pub cam_dist: f32,
    pub cam_yaw: f32,
    pub cam_pitch: f32,

    // Input state
    pub mouse_pos: Vec2,
    pub mouse_delta: (f32, f32),
    pub is_orbiting: bool,
    pub is_wireframe: bool,
    pub show_collisions: bool,
    pub fly_mode: bool,
    pub sprint: bool,
    pub freeze_culling: bool,
    pub cursor_id: Option<BlockId>,
    pub first_person: bool,
    mine_held: bool,
    place_pressed: bool,
    hotbar_delta: i32,
    hotbar_slot: Option<usize>,
    toggle_inventory: bool,

    reach_distance: f32,
    keys: [bool; 5], // W A S D Space
}

impl Controller {
    pub fn new(cfg: &PlayerConfig) -> Self {
        Self {
            cam_dist: 200.0,
            cam_yaw: 0.0,
            cam_pitch: 0.5,
            mouse_pos: Vec2::ZERO,
            mouse_delta: (0.0, 0.0),
            is_orbiting: false,
            cursor_id: None,
            is_wireframe: false,
            show_collisions: false,
            fly_mode: false,
            freeze_culling: false,
            sprint: false,
            first_person: true,
            mine_held: false,
            place_pressed: false,
            hotbar_delta: 0,
            hotbar_slot: None,
            toggle_inventory: false,
            reach_distance: cfg.reach_distance,
            keys: [false; 5],
        }
    }

    // --- Per-frame update ---------------------------------------------------

    pub fn update_player(
        &mut self,
        player: &mut Player,
        planet: &PlanetData,
        physics: &Physics,
        dt: f32,
    ) {
        let mut input = Vec3::ZERO;
        if self.keys[0] {
            input.z -= 1.0;
        }
        if self.keys[1] {
            input.x -= 1.0;
        }
        if self.keys[2] {
            input.z += 1.0;
        }
        if self.keys[3] {
            input.x += 1.0;
        }
        let jump = self.keys[4];

        let rotation_delta = if self.first_person {
            self.mouse_delta
        } else {
            (0.0, 0.0)
        };
        player.update(
            dt,
            planet,
            physics,
            input,
            jump,
            rotation_delta,
            self.fly_mode,
            self.sprint,
        );
        self.mouse_delta = (0.0, 0.0);
    }

    pub fn take_gameplay_intent(&mut self) -> PlayerIntent {
        let intent = PlayerIntent {
            mine_held: self.mine_held,
            place_pressed: self.place_pressed,
            hotbar_delta: self.hotbar_delta,
            hotbar_slot: self.hotbar_slot,
            toggle_inventory: self.toggle_inventory,
        };
        self.place_pressed = false;
        self.hotbar_delta = 0;
        self.hotbar_slot = None;
        self.toggle_inventory = false;
        intent
    }

    // --- Camera -------------------------------------------------------------

    pub fn get_camera_pos(&self, player: &Player, physics: &Physics) -> Vec3 {
        if self.first_person {
            player.position + Physics::get_up_vector(player.position) * physics.eye_height
        } else {
            player.position + Physics::get_up_vector(player.position) * self.cam_dist
        }
    }

    pub fn get_matrix(
        &self,
        player: &Player,
        physics: &Physics,
        width: f32,
        height: f32,
        render_cfg: &vv_config::RenderConfig,
    ) -> Mat4 {
        let fov = if self.first_person {
            render_cfg.fov_first_person_deg
        } else {
            render_cfg.fov_orbit_deg
        };
        let proj = Mat4::perspective_rh(
            fov.to_radians(),
            width / height,
            render_cfg.near_plane,
            render_cfg.far_plane,
        );
        let view = if self.first_person {
            player.get_view_matrix(physics)
        } else {
            let up = Physics::get_up_vector(player.position);
            let cam_pos = player.position + up * self.cam_dist;
            let fwd = player.rotation * Vec3::NEG_Z;
            Mat4::look_at_rh(cam_pos, player.position, fwd)
        };
        proj * view
    }

    // --- Ray cast -----------------------------------------------------------

    /// Cast a ray from the camera centre into the world.
    /// In `place_mode` returns the last empty block before the first hit.
    pub fn raycast(
        &self,
        player: &Player,
        planet: &PlanetData,
        physics: &Physics,
        width: f32,
        height: f32,
        render_cfg: &vv_config::RenderConfig,
        place_mode: bool,
    ) -> Option<(BlockId, f32)> {
        let mvp = self.get_matrix(player, physics, width, height, render_cfg);
        let inv = mvp.inverse();
        let (nx, ny) = if self.first_person {
            (0.0f32, 0.0f32)
        } else {
            (
                (2.0 * self.mouse_pos.x / width) - 1.0,
                1.0 - (2.0 * self.mouse_pos.y / height),
            )
        };

        let start = inv.project_point3(Vec3::new(nx, ny, 0.0));
        let end = inv.project_point3(Vec3::new(nx, ny, 1.0));
        let dir = (end - start).normalize();

        let reach = if self.first_person {
            self.reach_distance
        } else {
            self.cam_dist + 100.0
        };

        let mut dist = 0.0f32;
        let mut last_empty = None;
        let step = 0.25f32;

        while dist < reach {
            let p = start + dir * dist;
            if p.length() < 0.5 {
                break;
            }
            if let Some(id) = CoordSystem::pos_to_id(p, planet.resolution) {
                let exists = planet.exists(id);
                if place_mode {
                    if exists {
                        return last_empty.map(|i| (i, dist));
                    } else {
                        last_empty = Some(id);
                    }
                } else if exists {
                    return Some((id, dist));
                }
            }
            dist += step;
        }
        None
    }

    // --- Event processing ---------------------------------------------------

    pub fn process_mouse_motion(&mut self, delta: (f64, f64)) {
        if self.first_person {
            self.mouse_delta.0 += delta.0 as f32;
            self.mouse_delta.1 += delta.1 as f32;
        }
    }

    /// Returns `true` if the event was consumed.
    pub fn process_events(&mut self, event: &WindowEvent, player: &mut Player) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = Vec2::new(position.x as f32, position.y as f32);
                let d = new_pos - self.mouse_pos;
                self.mouse_pos = new_pos;
                self.mouse_delta = (d.x, d.y);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Middle {
                    self.is_orbiting = *state == ElementState::Pressed;
                }
                if *button == MouseButton::Left {
                    self.mine_held = *state == ElementState::Pressed;
                    return true;
                }
                if *button == MouseButton::Right && *state == ElementState::Pressed {
                    self.place_pressed = true;
                    return true;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 * 0.01,
                };
                if self.first_person {
                    if y.abs() > 0.0 {
                        self.hotbar_delta += if y > 0.0 { -1 } else { 1 };
                    }
                    return true;
                } else {
                    self.cam_dist = (self.cam_dist - y * 50.0).clamp(10.0, 10_000.0);
                    return true;
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.keys[0] = pressed,
                    PhysicalKey::Code(KeyCode::KeyA) => self.keys[1] = pressed,
                    PhysicalKey::Code(KeyCode::KeyS) => self.keys[2] = pressed,
                    PhysicalKey::Code(KeyCode::KeyD) => self.keys[3] = pressed,
                    PhysicalKey::Code(KeyCode::Space) => self.keys[4] = pressed,
                    PhysicalKey::Code(KeyCode::ControlLeft) => self.sprint = pressed,
                    PhysicalKey::Code(KeyCode::KeyE) if pressed => {
                        self.toggle_inventory = true;
                        return true;
                    }
                    PhysicalKey::Code(KeyCode::Digit1) if pressed => self.hotbar_slot = Some(0),
                    PhysicalKey::Code(KeyCode::Digit2) if pressed => self.hotbar_slot = Some(1),
                    PhysicalKey::Code(KeyCode::Digit3) if pressed => self.hotbar_slot = Some(2),
                    PhysicalKey::Code(KeyCode::Digit4) if pressed => self.hotbar_slot = Some(3),
                    PhysicalKey::Code(KeyCode::Digit5) if pressed => self.hotbar_slot = Some(4),
                    PhysicalKey::Code(KeyCode::Digit6) if pressed => self.hotbar_slot = Some(5),
                    PhysicalKey::Code(KeyCode::Digit7) if pressed => self.hotbar_slot = Some(6),
                    PhysicalKey::Code(KeyCode::Digit8) if pressed => self.hotbar_slot = Some(7),
                    PhysicalKey::Code(KeyCode::Digit9) if pressed => self.hotbar_slot = Some(8),
                    PhysicalKey::Code(KeyCode::KeyP) if pressed => {
                        if player.debug_mode {
                            self.is_wireframe = !self.is_wireframe;
                        }
                        return true;
                    }
                    PhysicalKey::Code(KeyCode::KeyO) if pressed => {
                        if player.debug_mode {
                            self.show_collisions = !self.show_collisions;
                            println!("Show collisions: {}", self.show_collisions);
                        }
                        return true;
                    }
                    PhysicalKey::Code(KeyCode::Quote) if pressed => {
                        if player.debug_mode {
                            self.freeze_culling = !self.freeze_culling;
                        }
                        return true;
                    }
                    PhysicalKey::Code(KeyCode::KeyK) if pressed => {
                        self.first_person = !self.first_person;
                        self.cam_dist = if self.first_person { 40.0 } else { 100.0 };
                        return true;
                    }
                    PhysicalKey::Code(KeyCode::KeyF) if pressed => {
                        if self.first_person {
                            self.fly_mode = !self.fly_mode;
                            println!("Fly mode: {}", self.fly_mode);
                        }
                        return true;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        false
    }
}
