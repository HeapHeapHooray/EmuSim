use crate::media::DiscMedia;
use crate::sockets::{SocketKind, SocketPort};
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Ps2TrayState {
    Closed,
    Opening(f32), // Progress 0.0 to 1.0
    Open,
    Closing(f32), // Progress 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayStation2Console {
    pub id: String,
    pub rear_rocker_switch_on: bool,
    pub is_system_running: bool, // false = Standby (Red LED), true = Running (Green LED)
    pub tray_state: Ps2TrayState,
    pub tray_disc: Option<DiscMedia>,
    pub sockets: Vec<SocketPort>,
}

impl PlayStation2Console {
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            id: id_str.clone(),
            rear_rocker_switch_on: true,
            is_system_running: false,
            tray_state: Ps2TrayState::Closed,
            tray_disc: None,
            sockets: vec![
                // Rear AC power in (IEC C13/C7)
                SocketPort {
                    id: format!("{}_power_in", id_str),
                    kind: SocketKind::IecC7Jack,
                    local_pos: Vec3::new(-0.11, 0.02, -0.12),
                    normal: Vec3::new(0.0, 0.0, -1.0),
                    up: Vec3::Y,
                    label: "AC IN".into(),
                },
                // Rear PlayStation Multi-Out
                SocketPort {
                    id: format!("{}_multi_out", id_str),
                    kind: SocketKind::PlayStationMultiOutJack,
                    local_pos: Vec3::new(0.08, 0.02, -0.12),
                    normal: Vec3::new(0.0, 0.0, -1.0),
                    up: Vec3::Y,
                    label: "AV MULTI OUT".into(),
                },
                // Front Controller Port 1
                SocketPort {
                    id: format!("{}_ctrl_1", id_str),
                    kind: SocketKind::PlayStationControllerJack,
                    local_pos: Vec3::new(-0.08, 0.035, 0.12),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    up: Vec3::Y,
                    label: "CONTROLLER 1".into(),
                },
                // Front Controller Port 2
                SocketPort {
                    id: format!("{}_ctrl_2", id_str),
                    kind: SocketKind::PlayStationControllerJack,
                    local_pos: Vec3::new(-0.03, 0.035, 0.12),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    up: Vec3::Y,
                    label: "CONTROLLER 2".into(),
                },
            ],
        }
    }

    /// Press the front Reset/Standby button.
    pub fn press_reset_standby_button(&mut self) {
        if self.rear_rocker_switch_on {
            self.is_system_running = !self.is_system_running;
        }
    }

    /// Press the front motorized tray Eject button.
    pub fn press_eject_button(&mut self) {
        match self.tray_state {
            Ps2TrayState::Closed | Ps2TrayState::Closing(_) => {
                self.tray_state = Ps2TrayState::Opening(0.0);
            }
            Ps2TrayState::Open | Ps2TrayState::Opening(_) => {
                self.tray_state = Ps2TrayState::Closing(0.0);
            }
        }
    }

    pub fn insert_disc(&mut self, disc: DiscMedia) -> Option<DiscMedia> {
        if matches!(self.tray_state, Ps2TrayState::Open) {
            self.tray_disc.replace(disc)
        } else {
            None
        }
    }

    pub fn remove_disc(&mut self) -> Option<DiscMedia> {
        if matches!(self.tray_state, Ps2TrayState::Open) {
            self.tray_disc.take()
        } else {
            None
        }
    }

    /// Advance motorized tray movement physics.
    pub fn update_tray(&mut self, dt: f32) {
        let tray_speed = 1.2; // takes ~0.83s to slide open/close
        match &mut self.tray_state {
            Ps2TrayState::Opening(ref mut progress) => {
                *progress += dt * tray_speed;
                if *progress >= 1.0 {
                    self.tray_state = Ps2TrayState::Open;
                }
            }
            Ps2TrayState::Closing(ref mut progress) => {
                *progress += dt * tray_speed;
                if *progress >= 1.0 {
                    self.tray_state = Ps2TrayState::Closed;
                }
            }
            _ => {}
        }
    }
}
