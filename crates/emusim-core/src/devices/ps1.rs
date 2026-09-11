use crate::media::DiscMedia;
use crate::sockets::{SocketKind, SocketPort};
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayStation1Console {
    pub id: String,
    pub power_button_latched: bool,
    pub lid_open: bool,
    pub spindle_disc: Option<DiscMedia>,
    pub sockets: Vec<SocketPort>,
}

impl PlayStation1Console {
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            id: id_str.clone(),
            power_button_latched: false,
            lid_open: false,
            spindle_disc: None,
            sockets: vec![
                // Rear AC power in (IEC C7)
                SocketPort {
                    id: format!("{}_power_in", id_str),
                    kind: SocketKind::IecC7Jack,
                    local_pos: Vec3::new(-0.08, 0.02, -0.095),
                    normal: Vec3::new(0.0, 0.0, -1.0),
                    up: Vec3::Y,
                    label: "AC IN".into(),
                },
                // Rear PlayStation AV Multi-Out
                SocketPort {
                    id: format!("{}_multi_out", id_str),
                    kind: SocketKind::PlayStationMultiOutJack,
                    local_pos: Vec3::new(0.06, 0.02, -0.095),
                    normal: Vec3::new(0.0, 0.0, -1.0),
                    up: Vec3::Y,
                    label: "AV MULTI OUT".into(),
                },
                // Front Controller Port 1
                SocketPort {
                    id: format!("{}_ctrl_1", id_str),
                    kind: SocketKind::PlayStationControllerJack,
                    local_pos: Vec3::new(-0.06, 0.015, 0.095),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    up: Vec3::Y,
                    label: "CONTROLLER 1".into(),
                },
                // Front Controller Port 2
                SocketPort {
                    id: format!("{}_ctrl_2", id_str),
                    kind: SocketKind::PlayStationControllerJack,
                    local_pos: Vec3::new(0.06, 0.015, 0.095),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    up: Vec3::Y,
                    label: "CONTROLLER 2".into(),
                },
            ],
        }
    }

    pub fn press_power_button(&mut self) {
        self.power_button_latched = !self.power_button_latched;
    }

    pub fn press_open_button(&mut self) {
        self.lid_open = true;
    }

    pub fn close_lid(&mut self) {
        self.lid_open = false;
    }

    pub fn insert_disc(&mut self, disc: DiscMedia) -> Option<DiscMedia> {
        if self.lid_open {
            self.spindle_disc.replace(disc)
        } else {
            None
        }
    }

    pub fn remove_disc(&mut self) -> Option<DiscMedia> {
        if self.lid_open {
            self.spindle_disc.take()
        } else {
            None
        }
    }
}
