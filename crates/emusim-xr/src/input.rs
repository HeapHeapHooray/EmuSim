use glam::{Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ControllerPose {
    pub position: Vec3,
    pub rotation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub is_tracked: bool,
}

impl Default for ControllerPose {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            is_tracked: false,
        }
    }
}

/// Hardware state of a Meta Quest Touch Plus controller.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct QuestControllerInput {
    pub pose: ControllerPose,
    pub thumbstick: Vec2, // (-1.0 to 1.0)
    pub thumbstick_click: bool,
    pub trigger_value: f32, // (0.0 to 1.0)
    pub grip_value: f32,    // (0.0 to 1.0)
    pub button_primary: bool,   // X (Left) or A (Right)
    pub button_secondary: bool, // Y (Left) or B (Right)
    pub menu_button: bool,
}

/// Combined XR headset and both controllers snapshot.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct XrFrameInput {
    pub head_pose: ControllerPose,
    pub left_controller: QuestControllerInput,
    pub right_controller: QuestControllerInput,
    pub delta_time: f32,
}

/// Haptic vibration request for Quest Touch Plus controllers.
#[derive(Debug, Clone, Copy)]
pub struct HapticPulse {
    pub hand: HandSide,
    pub duration_seconds: f32,
    pub frequency_hz: f32,
    pub amplitude: f32, // 0.0 to 1.0
}
