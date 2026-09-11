use crate::sockets::{SocketKind, SocketPort};
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Fixed wall outlet offering 2 AC power sockets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallOutlet {
    pub id: String,
    pub sockets: Vec<SocketPort>,
}

impl WallOutlet {
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            id: id_str.clone(),
            sockets: vec![
                SocketPort {
                    id: format!("{}_top", id_str),
                    kind: SocketKind::WallAcOutlet,
                    local_pos: Vec3::new(0.0, 0.03, 0.0),
                    normal: Vec3::Z,
                    up: Vec3::Y,
                    label: "AC 120V TOP".into(),
                },
                SocketPort {
                    id: format!("{}_bottom", id_str),
                    kind: SocketKind::WallAcOutlet,
                    local_pos: Vec3::new(0.0, -0.03, 0.0),
                    normal: Vec3::Z,
                    up: Vec3::Y,
                    label: "AC 120V BOTTOM".into(),
                },
            ],
        }
    }
}

/// Power strip surge protector with master rocker switch and 6 AC outlets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerStrip {
    pub id: String,
    pub switch_on: bool,
    pub plug_id: String, // The cord plug connected to wall outlet
    pub sockets: Vec<SocketPort>,
}

impl PowerStrip {
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        let mut sockets = Vec::with_capacity(6);
        for i in 0..6 {
            sockets.push(SocketPort {
                id: format!("{}_outlet_{}", id_str, i + 1),
                kind: SocketKind::WallAcOutlet,
                local_pos: Vec3::new(0.0, 0.015, -0.15 + (i as f32) * 0.06),
                normal: Vec3::Y,
                up: Vec3::Z,
                label: format!("OUTLET {}", i + 1),
            });
        }

        Self {
            id: id_str.clone(),
            switch_on: true,
            plug_id: format!("{}_cord_plug", id_str),
            sockets,
        }
    }

    pub fn toggle_switch(&mut self) {
        self.switch_on = !self.switch_on;
    }
}
