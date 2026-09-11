use crate::media::CartridgeMedia;
use crate::sockets::{SocketKind, SocketPort};
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum N64RamPak {
    JumperPak4MB,
    ExpansionPak8MB, // Required for games like Donkey Kong 64, Majora's Mask, Perfect Dark
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nintendo64Console {
    pub id: String,
    pub power_switch_on: bool,
    pub reset_button_pressed: bool,
    pub ram_pak: N64RamPak,
    pub inserted_cartridge: Option<CartridgeMedia>,
    pub sockets: Vec<SocketPort>,
}

impl Nintendo64Console {
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            id: id_str.clone(),
            power_switch_on: false,
            reset_button_pressed: false,
            ram_pak: N64RamPak::ExpansionPak8MB,
            inserted_cartridge: None,
            sockets: vec![
                // Rear Power Adapter port
                SocketPort {
                    id: format!("{}_power_in", id_str),
                    kind: SocketKind::N64AcBrickJack,
                    local_pos: Vec3::new(-0.06, 0.02, -0.09),
                    normal: Vec3::new(0.0, 0.0, -1.0),
                    up: Vec3::Y,
                    label: "POWER IN".into(),
                },
                // Rear Nintendo Multi-Out AV Port
                SocketPort {
                    id: format!("{}_multi_out", id_str),
                    kind: SocketKind::NintendoMultiOutJack,
                    local_pos: Vec3::new(0.05, 0.02, -0.09),
                    normal: Vec3::new(0.0, 0.0, -1.0),
                    up: Vec3::Y,
                    label: "MULTI OUT".into(),
                },
                // Front 4 Controller Ports
                SocketPort {
                    id: format!("{}_ctrl_1", id_str),
                    kind: SocketKind::N64ControllerJack,
                    local_pos: Vec3::new(-0.075, 0.015, 0.09),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    up: Vec3::Y,
                    label: "PORT 1".into(),
                },
                SocketPort {
                    id: format!("{}_ctrl_2", id_str),
                    kind: SocketKind::N64ControllerJack,
                    local_pos: Vec3::new(-0.025, 0.015, 0.09),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    up: Vec3::Y,
                    label: "PORT 2".into(),
                },
                SocketPort {
                    id: format!("{}_ctrl_3", id_str),
                    kind: SocketKind::N64ControllerJack,
                    local_pos: Vec3::new(0.025, 0.015, 0.09),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    up: Vec3::Y,
                    label: "PORT 3".into(),
                },
                SocketPort {
                    id: format!("{}_ctrl_4", id_str),
                    kind: SocketKind::N64ControllerJack,
                    local_pos: Vec3::new(0.075, 0.015, 0.09),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    up: Vec3::Y,
                    label: "PORT 4".into(),
                },
            ],
        }
    }

    /// Insert an N64 cartridge into the top slot.
    pub fn insert_cartridge(&mut self, cart: CartridgeMedia) -> Option<CartridgeMedia> {
        self.inserted_cartridge.replace(cart)
    }

    /// Eject / remove the cartridge from the slot.
    pub fn eject_cartridge(&mut self) -> Option<CartridgeMedia> {
        self.inserted_cartridge.take()
    }

    /// Toggle the front-left power slider switch.
    pub fn set_power_switch(&mut self, on: bool) {
        self.power_switch_on = on;
    }
}
