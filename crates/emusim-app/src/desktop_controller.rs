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
    pub eye_height: f32,
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
        let eye_height = 1.05;
        Self {
            mode: DesktopPlayMode::RoomExploration,
            camera_pos: Vec3::new(0.0, eye_height, -0.15),
            camera_yaw: 0.0,
            camera_pitch: -0.15,
            eye_height,
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
                    // Sit directly in front of CRT TV
                    self.camera_pos = Vec3::new(0.0, 0.77, -0.96);
                    self.camera_pitch = 0.0;
                    self.camera_yaw = 0.0;
                    DesktopPlayMode::GameFocus
                }
                DesktopPlayMode::GameFocus => {
                    // Stand back up into room
                    self.camera_pos = Vec3::new(0.0, self.eye_height, -0.15);
                    self.camera_pitch = -0.15;
                    self.camera_yaw = 0.0;
                    DesktopPlayMode::RoomExploration
                }
            };
            return;
        }

        if is_pressed && key == KeyCode::Escape && self.mode == DesktopPlayMode::GameFocus {
            self.mode = DesktopPlayMode::RoomExploration;
            self.camera_pos = Vec3::new(0.0, 1.35, 0.1);
            return;
        }

        let mut set_btn = |btn_bit: u32, val: bool| {
            if val {
                self.retro_gamepad.buttons |= 1 << btn_bit;
            } else {
                self.retro_gamepad.buttons &= !(1 << btn_bit);
            }
        };

        // 1. Universal console gamepad controls active in BOTH modes:
        // D-Pad and Left Analog stick
        match key {
            KeyCode::ArrowUp => {
                set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_UP, is_pressed);
                self.retro_gamepad.left_analog_y = if is_pressed { -32767 } else { 0 };
                return;
            }
            KeyCode::ArrowDown => {
                set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_DOWN, is_pressed);
                self.retro_gamepad.left_analog_y = if is_pressed { 32767 } else { 0 };
                return;
            }
            KeyCode::ArrowLeft => {
                set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_LEFT, is_pressed);
                self.retro_gamepad.left_analog_x = if is_pressed { -32767 } else { 0 };
                return;
            }
            KeyCode::ArrowRight => {
                set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_RIGHT, is_pressed);
                self.retro_gamepad.left_analog_x = if is_pressed { 32767 } else { 0 };
                return;
            }

            // Cross ✕ / Nintendo B / Accept (US) / Cancel (JP)
            KeyCode::KeyJ | KeyCode::KeyZ | KeyCode::Space => {
                set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_B, is_pressed);
                return;
            }
            // Circle ◯ / Nintendo A / Accept (JP) / Cancel (US)
            KeyCode::KeyK | KeyCode::KeyX => {
                set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_A, is_pressed);
                return;
            }
            // Square ◻ / Nintendo Y
            KeyCode::KeyU | KeyCode::KeyC => {
                set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_Y, is_pressed);
                return;
            }
            // Triangle △ / Nintendo X / PS2 Version Info
            KeyCode::KeyI | KeyCode::KeyV => {
                set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_X, is_pressed);
                return;
            }

            // Start
            KeyCode::Enter | KeyCode::NumpadEnter => {
                set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_START, is_pressed);
                return;
            }
            // Select
            KeyCode::ShiftRight | KeyCode::ShiftLeft | KeyCode::Backspace => {
                set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_SELECT, is_pressed);
                return;
            }

            // Shoulders and triggers
            KeyCode::KeyH => {
                set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_L, is_pressed);
                return;
            }
            KeyCode::KeyL => {
                set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_R, is_pressed);
                return;
            }
            KeyCode::KeyO => {
                set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_L2, is_pressed);
                return;
            }
            KeyCode::KeyP => {
                set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_R2, is_pressed);
                return;
            }

            // Console quick-swap shortcuts (1 = N64, 2 = PS1, 3 = PS2, 0 = None / Noise)
            KeyCode::Digit1 | KeyCode::Numpad1 if is_pressed => {
                self.switch_to_console = Some(crate::world::SelectedConsole::Nintendo64);
                return;
            }
            KeyCode::Digit2 | KeyCode::Numpad2 if is_pressed => {
                self.switch_to_console = Some(crate::world::SelectedConsole::PlayStation1);
                return;
            }
            KeyCode::Digit3 | KeyCode::Numpad3 if is_pressed => {
                self.switch_to_console = Some(crate::world::SelectedConsole::PlayStation2);
                return;
            }
            KeyCode::Digit0 | KeyCode::Numpad0 if is_pressed => {
                self.switch_to_console = Some(crate::world::SelectedConsole::None);
                return;
            }

            _ => {}
        }

        // 2. Mode-specific keys:
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
                    _ => {}
                }
            }
            DesktopPlayMode::GameFocus => {
                // In Game Focus, WASD also controls the D-Pad / Left Stick
                match key {
                    KeyCode::KeyW => {
                        set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_UP, is_pressed);
                        self.retro_gamepad.left_analog_y = if is_pressed { -32767 } else { 0 };
                    }
                    KeyCode::KeyS => {
                        set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_DOWN, is_pressed);
                        self.retro_gamepad.left_analog_y = if is_pressed { 32767 } else { 0 };
                    }
                    KeyCode::KeyA => {
                        set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_LEFT, is_pressed);
                        self.retro_gamepad.left_analog_x = if is_pressed { -32767 } else { 0 };
                    }
                    KeyCode::KeyD => {
                        set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_RIGHT, is_pressed);
                        self.retro_gamepad.left_analog_x = if is_pressed { 32767 } else { 0 };
                    }
                    KeyCode::KeyQ => {
                        set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_L, is_pressed);
                    }
                    KeyCode::KeyE => {
                        set_btn(emusim_core::RETRO_DEVICE_ID_JOYPAD_R, is_pressed);
                    }
                    _ => {}
                }
            }
        }
    }

    /// Step movement physics forward.
    pub fn update(&mut self, dt: f32) {
        if self.mode == DesktopPlayMode::RoomExploration {
            let speed = 2.0; // 2 meters per second
            let yaw_rot = Quat::from_rotation_y(self.camera_yaw);
            let forward = (yaw_rot * -Vec3::Z).normalize_or_zero();
            let right = (yaw_rot * Vec3::X).normalize_or_zero();

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

            // Room boundaries clamp: 5m x 4m room, front of TV stand
            self.camera_pos.x = self.camera_pos.x.clamp(-2.2, 2.2);
            self.camera_pos.z = self.camera_pos.z.clamp(-1.35, 1.3);
            self.camera_pos.y = self.eye_height;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_movement_follows_camera_orientation() {
        let mut ctrl = DesktopFirstPersonController::new();
        ctrl.camera_pos = Vec3::ZERO;
        ctrl.eye_height = 0.0;

        // 1. Facing default (yaw = 0): forward is -Z, right is +X
        ctrl.camera_yaw = 0.0;
        ctrl.move_forward = true;
        ctrl.update(0.1);
        assert!(ctrl.camera_pos.z < 0.0, "Moving forward should decrease Z");
        assert_eq!(ctrl.camera_pos.x, 0.0);

        // Reset
        ctrl.camera_pos = Vec3::ZERO;
        ctrl.move_forward = false;
        ctrl.move_right = true;
        ctrl.update(0.1);
        assert!(ctrl.camera_pos.x > 0.0, "Moving right should increase X");
        assert_eq!(ctrl.camera_pos.z, 0.0);

        // 2. Turn 90 deg right (yaw = -PI/2): forward should now be +X, right should be +Z
        ctrl.camera_pos = Vec3::ZERO;
        ctrl.camera_yaw = -PI * 0.5;
        ctrl.move_right = false;
        ctrl.move_forward = true;
        ctrl.update(0.1);
        assert!(ctrl.camera_pos.x > 0.0, "Moving forward when turned 90 deg right should increase X");
        assert!(ctrl.camera_pos.z.abs() < 1e-5);

        // 3. Turn 90 deg left (yaw = PI/2): forward should now be -X
        ctrl.camera_pos = Vec3::ZERO;
        ctrl.camera_yaw = PI * 0.5;
        ctrl.move_forward = true;
        ctrl.update(0.1);
        assert!(ctrl.camera_pos.x < 0.0, "Moving forward when turned 90 deg left should decrease X");
        assert!(ctrl.camera_pos.z.abs() < 1e-5);
    }
}
