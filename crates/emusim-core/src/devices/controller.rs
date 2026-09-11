use glam::Vec2;
use serde::{Deserialize, Serialize};

/// State of an N64 Trident controller.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct N64ControllerState {
    pub stick: Vec2, // (-1.0 to 1.0)
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub button_a: bool,
    pub button_b: bool,
    pub trigger_z: bool,
    pub button_l: bool,
    pub button_r: bool,
    pub button_start: bool,
    pub c_up: bool,
    pub c_down: bool,
    pub c_left: bool,
    pub c_right: bool,
}

/// State of a PlayStation DualShock controller.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DualShockControllerState {
    pub left_stick: Vec2,
    pub right_stick: Vec2,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub cross: bool,
    pub circle: bool,
    pub square: bool,
    pub triangle: bool,
    pub l1: bool,
    pub l2: bool,
    pub l3: bool,
    pub r1: bool,
    pub r2: bool,
    pub r3: bool,
    pub select: bool,
    pub start: bool,
    pub analog_mode: bool,
}

/// Generic gamepad state unified for Libretro core consumption.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct UnifiedGamepadState {
    pub buttons: u32,
    pub left_analog_x: i16,
    pub left_analog_y: i16,
    pub right_analog_x: i16,
    pub right_analog_y: i16,
}

pub const RETRO_DEVICE_ID_JOYPAD_B: u32 = 0;
pub const RETRO_DEVICE_ID_JOYPAD_Y: u32 = 1;
pub const RETRO_DEVICE_ID_JOYPAD_SELECT: u32 = 2;
pub const RETRO_DEVICE_ID_JOYPAD_START: u32 = 3;
pub const RETRO_DEVICE_ID_JOYPAD_UP: u32 = 4;
pub const RETRO_DEVICE_ID_JOYPAD_DOWN: u32 = 5;
pub const RETRO_DEVICE_ID_JOYPAD_LEFT: u32 = 6;
pub const RETRO_DEVICE_ID_JOYPAD_RIGHT: u32 = 7;
pub const RETRO_DEVICE_ID_JOYPAD_A: u32 = 8;
pub const RETRO_DEVICE_ID_JOYPAD_X: u32 = 9;
pub const RETRO_DEVICE_ID_JOYPAD_L: u32 = 10;
pub const RETRO_DEVICE_ID_JOYPAD_R: u32 = 11;
pub const RETRO_DEVICE_ID_JOYPAD_L2: u32 = 12;
pub const RETRO_DEVICE_ID_JOYPAD_R2: u32 = 13;
pub const RETRO_DEVICE_ID_JOYPAD_L3: u32 = 14;
pub const RETRO_DEVICE_ID_JOYPAD_R3: u32 = 15;

impl From<&N64ControllerState> for UnifiedGamepadState {
    fn from(n64: &N64ControllerState) -> Self {
        let mut buttons = 0u32;
        let mut set = |id: u32, pressed: bool| {
            if pressed {
                buttons |= 1 << id;
            }
        };

        // N64 mapping onto Libretro RetroPad / Mupen64plus default conventions:
        // A -> A, B -> B, Z -> L2, L -> L, R -> R, Start -> Start
        // C-buttons can map to right analog or secondary buttons
        set(RETRO_DEVICE_ID_JOYPAD_A, n64.button_a);
        set(RETRO_DEVICE_ID_JOYPAD_B, n64.button_b);
        set(RETRO_DEVICE_ID_JOYPAD_L2, n64.trigger_z);
        set(RETRO_DEVICE_ID_JOYPAD_L, n64.button_l);
        set(RETRO_DEVICE_ID_JOYPAD_R, n64.button_r);
        set(RETRO_DEVICE_ID_JOYPAD_START, n64.button_start);
        set(RETRO_DEVICE_ID_JOYPAD_UP, n64.dpad_up);
        set(RETRO_DEVICE_ID_JOYPAD_DOWN, n64.dpad_down);
        set(RETRO_DEVICE_ID_JOYPAD_LEFT, n64.dpad_left);
        set(RETRO_DEVICE_ID_JOYPAD_RIGHT, n64.dpad_right);

        let lx = (n64.stick.x.clamp(-1.0, 1.0) * 32767.0) as i16;
        let ly = (n64.stick.y.clamp(-1.0, 1.0) * 32767.0) as i16;

        let mut rx = 0i16;
        let mut ry = 0i16;
        if n64.c_left {
            rx -= 32767;
        }
        if n64.c_right {
            rx += 32767;
        }
        if n64.c_up {
            ry += 32767;
        }
        if n64.c_down {
            ry -= 32767;
        }

        Self {
            buttons,
            left_analog_x: lx,
            left_analog_y: ly,
            right_analog_x: rx,
            right_analog_y: ry,
        }
    }
}

impl From<&DualShockControllerState> for UnifiedGamepadState {
    fn from(ps: &DualShockControllerState) -> Self {
        let mut buttons = 0u32;
        let mut set = |id: u32, pressed: bool| {
            if pressed {
                buttons |= 1 << id;
            }
        };

        // Standard PlayStation to RetroPad mapping:
        // Cross -> B, Circle -> A, Square -> Y, Triangle -> X
        set(RETRO_DEVICE_ID_JOYPAD_B, ps.cross);
        set(RETRO_DEVICE_ID_JOYPAD_A, ps.circle);
        set(RETRO_DEVICE_ID_JOYPAD_Y, ps.square);
        set(RETRO_DEVICE_ID_JOYPAD_X, ps.triangle);
        set(RETRO_DEVICE_ID_JOYPAD_L, ps.l1);
        set(RETRO_DEVICE_ID_JOYPAD_R, ps.r1);
        set(RETRO_DEVICE_ID_JOYPAD_L2, ps.l2);
        set(RETRO_DEVICE_ID_JOYPAD_R2, ps.r2);
        set(RETRO_DEVICE_ID_JOYPAD_L3, ps.l3);
        set(RETRO_DEVICE_ID_JOYPAD_R3, ps.r3);
        set(RETRO_DEVICE_ID_JOYPAD_SELECT, ps.select);
        set(RETRO_DEVICE_ID_JOYPAD_START, ps.start);
        set(RETRO_DEVICE_ID_JOYPAD_UP, ps.dpad_up);
        set(RETRO_DEVICE_ID_JOYPAD_DOWN, ps.dpad_down);
        set(RETRO_DEVICE_ID_JOYPAD_LEFT, ps.dpad_left);
        set(RETRO_DEVICE_ID_JOYPAD_RIGHT, ps.dpad_right);

        let lx = (ps.left_stick.x.clamp(-1.0, 1.0) * 32767.0) as i16;
        let ly = (ps.left_stick.y.clamp(-1.0, 1.0) * 32767.0) as i16;
        let rx = (ps.right_stick.x.clamp(-1.0, 1.0) * 32767.0) as i16;
        let ry = (ps.right_stick.y.clamp(-1.0, 1.0) * 32767.0) as i16;

        Self {
            buttons,
            left_analog_x: lx,
            left_analog_y: ly,
            right_analog_x: rx,
            right_analog_y: ry,
        }
    }
}
