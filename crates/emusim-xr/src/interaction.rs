use crate::input::QuestControllerInput;
use emusim_core::devices::{N64ControllerState, UnifiedGamepadState};

#[derive(Debug, Clone, PartialEq)]
pub enum HeldObject {
    CablePlug { cable_id: String, plug_id: String },
    Cartridge { cartridge_id: String },
    Disc { disc_id: String },
    Controller { controller_id: String },
}

#[derive(Debug, Default)]
pub struct HandInteractionState {
    pub held_object: Option<HeldObject>,
    pub grip_latched: bool,
}

#[derive(Debug, Default)]
pub struct VrInteractionManager {
    pub left_hand: HandInteractionState,
    pub right_hand: HandInteractionState,
    pub active_gameplay_controller: Option<String>,
}

impl VrInteractionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process hand inputs and map to gameplay when holding a retro controller.
    pub fn map_quest_to_gamepad(
        left: &QuestControllerInput,
        right: &QuestControllerInput,
    ) -> UnifiedGamepadState {
        // Map Quest Touch Plus buttons to unified retro gamepad
        let n64_equivalent = N64ControllerState {
            stick: left.thumbstick,
            dpad_up: left.thumbstick.y > 0.6,
            dpad_down: left.thumbstick.y < -0.6,
            dpad_left: left.thumbstick.x < -0.6,
            dpad_right: left.thumbstick.x > 0.6,
            button_a: right.button_primary,    // A button
            button_b: right.button_secondary,  // B button
            trigger_z: left.trigger_value > 0.5,
            button_l: left.grip_value > 0.5,
            button_r: right.grip_value > 0.5,
            button_start: right.menu_button || left.menu_button,
            c_up: right.thumbstick.y > 0.5,
            c_down: right.thumbstick.y < -0.5,
            c_left: right.thumbstick.x < -0.5,
            c_right: right.thumbstick.x > 0.5,
        };

        UnifiedGamepadState::from(&n64_equivalent)
    }
}
