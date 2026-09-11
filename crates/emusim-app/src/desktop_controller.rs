use emusim_core::devices::UnifiedGamepadState;
use glam::{Quat, Vec3};
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopPlayMode {
    /// Walk around room, grab cables, plug devices, press power switches
    RoomExploration,
    /// Sit in front of CRT TV; keyboard and gamepad map directly to retro controller
    GameFocus,
}

pub struct DesktopFirstPersonController {
    pub mode: DesktopPlayMode,
    pub camera_pos: Vec3,
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub is_interact_pressed: bool,
    pub is_drop_pressed: bool,
    pub switch_to_console: Option<crate::world::SelectedConsole>,
    pub retro_gamepad: UnifiedGamepadState,
}

impl Default for DesktopFirstPersonController {
    fn default() -> Self {
        Self {
            mode: DesktopPlayMode::RoomExploration,
            camera_pos: Vec3::new(0.0, 1.4, 0.0), // Standing height ~1.4m
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            is_interact_pressed: false,
            is_drop_pressed: false,
            switch_to_console: None,
            retro_gamepad: UnifiedGamepadState::default(),
        }
    }
}

impl DesktopFirstPersonController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_mouse_motion(&mut self, dx: f64, dy: f64) {
        if self.mode == DesktopPlayMode::RoomExploration {
            let sensitivity = 0.0025;
            self.camera_yaw -= (dx as f32) * sensitivity;
            self.camera_pitch -= (dy as f32) * sensitivity;
            self.camera_pitch = self.camera_pitch.clamp(-1.5, 1.5);
        }
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        let is_pressed = state == ElementState::Pressed;
        match button {
            MouseButton::Left => {
                self.is_interact_pressed = is_pressed;
            }
            MouseButton::Right => {
                self.is_drop_pressed = is_pressed;
            }
            _ => {}
        }
    }

    pub fn handle_keyboard_input(&mut self, event: &KeyEvent) {
        let is_pressed = event.state == ElementState::Pressed;
        let key = match event.physical_key {
            PhysicalKey::Code(k) => k,
            _ => return,
        };

        // Mode toggle: Tab or G toggles Game Focus
        if is_pressed && (key == KeyCode::Tab || key == KeyCode::KeyG) {
            self.mode = match self.mode {
                DesktopPlayMode::RoomExploration => {
                    // Sit in front of CRT TV
                    self.camera_pos = Vec3::new(0.0, 0.85, -1.0);
                    self.camera_pitch = 0.0;
                    self.camera_yaw = 0.0;
                    DesktopPlayMode::GameFocus
                }
                DesktopPlayMode::GameFocus => {
                    // Stand back up into room
                    self.camera_pos = Vec3::new(0.0, 1.4, 0.0);
                    DesktopPlayMode::RoomExploration
                }
            };
            return;
        }

        if is_pressed && key == KeyCode::Escape && self.mode == DesktopPlayMode::GameFocus {
            self.mode = DesktopPlayMode::RoomExploration;
            self.camera_pos = Vec3::new(0.0, 1.4, 0.0);
            return;
        }

        match self.mode {
            DesktopPlayMode::RoomExploration => {
                // Room walking WASD
                match key {
                    KeyCode::KeyW => self.move_forward = is_pressed,
                    KeyCode::KeyS => self.move_backward = is_pressed,
                    KeyCode::KeyA => self.move_left = is_pressed,
                    KeyCode::KeyD => self.move_right = is_pressed,
                    KeyCode::KeyE => self.is_interact_pressed = is_pressed,
                    KeyCode::KeyQ => self.is_drop_pressed = is_pressed,

                    // Console quick-swap shortcuts
                    KeyCode::Digit1 if is_pressed => {
                        self.switch_to_console = Some(crate::world::SelectedConsole::Nintendo64);
                    }
                    KeyCode::Digit2 if is_pressed => {
                        self.switch_to_console = Some(crate::world::SelectedConsole::PlayStation1);
                    }
                    KeyCode::Digit3 if is_pressed => {
                        self.switch_to_console = Some(crate::world::SelectedConsole::PlayStation2);
                    }
                    KeyCode::Digit0 if is_pressed => {
                        self.switch_to_console = Some(crate::world::SelectedConsole::None);
                    }

                    _ => {}
                }
            }
            DesktopPlayMode::GameFocus => {
                // Map keyboard directly to retro gamepad
                let mut set_btn = |btn_bit: u32, val: bool| {
                    if val {
                        self.retro_gamepad.buttons |= 1 << btn_bit;
                    } else {
                        self.retro_gamepad.buttons &= !(1 << btn_bit);
                    }
                };

                match key {
                    // D-Pad / Analog Stick
                    KeyCode::ArrowUp | KeyCode::KeyW => {
                        set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_UP, is_pressed);
                        self.retro_gamepad.left_analog_y = if is_pressed { 32767 } else { 0 };
                    }
                    KeyCode::ArrowDown | KeyCode::KeyS => {
                        set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_DOWN, is_pressed);
                        self.retro_gamepad.left_analog_y = if is_pressed { -32767 } else { 0 };
                    }
                    KeyCode::ArrowLeft | KeyCode::KeyA => {
                        set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_LEFT, is_pressed);
                        self.retro_gamepad.left_analog_x = if is_pressed { -32767 } else { 0 };
                    }
                    KeyCode::ArrowRight | KeyCode::KeyD => {
                        set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_RIGHT, is_pressed);
                        self.retro_gamepad.left_analog_x = if is_pressed { 32767 } else { 0 };
                    }

                    // Action buttons (A / B / Cross / Circle)
                    KeyCode::KeyJ => set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_B, is_pressed),
                    KeyCode::KeyK => set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_A, is_pressed),
                    KeyCode::KeyU => set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_Y, is_pressed),
                    KeyCode::KeyI => set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_X, is_pressed),

                    // Triggers / Shoulders
                    KeyCode::KeyL => set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_R, is_pressed),
                    KeyCode::KeyH => set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_L, is_pressed),
                    KeyCode::Space => set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_L2, is_pressed),

                    // Start / Select
                    KeyCode::Enter => set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_START, is_pressed),
                    KeyCode::ShiftRight => set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_SELECT, is_pressed),

                    _ => {}
                }
            }
        }
    }

    /// Step movement physics forward.
    pub fn update(&mut self, dt: f32) {
        if self.mode == DesktopPlayMode::RoomExploration {
            let speed = 2.0; // 2 meters per second
            let forward = Vec3::new(self.camera_yaw.sin(), 0.0, -self.camera_yaw.cos()).normalize_or_zero();
            let right = Vec3::new(self.camera_yaw.cos(), 0.0, self.camera_yaw.sin()).normalize_or_zero();

            let mut movement = Vec3::ZERO;
            if self.move_forward {
                movement += forward;
            }
            if self.move_backward {
                movement -= forward;
            }
            if self.move_right {
                movement += right;
            }
            if self.move_left {
                movement -= right;
            }

            if movement.length_squared() > 1e-4 {
                self.camera_pos += movement.normalize() * speed * dt;
            }

            // Room boundaries clamp: 4m x 4m room
            self.camera_pos.x = self.camera_pos.x.clamp(-2.0, 2.0);
            self.camera_pos.z = self.camera_pos.z.clamp(-3.0, 1.0);
        }
    }

    /// Current camera rotation quaternion.
    pub fn camera_rotation(&self) -> Quat {
        Quat::from_rotation_y(self.camera_yaw) * Quat::from_rotation_x(self.camera_pitch)
    }

    /// Forward gaze ray direction.
    pub fn forward_ray(&self) -> Vec3 {
        self.camera_rotation() * -Vec3::Z
    }
}
