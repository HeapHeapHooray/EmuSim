use crate::sockets::{RcaColor, SocketKind};
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CableType {
    /// Standard 2-prong wall plug to IEC C7 figure-8 (Used by PS1, PS2 Slim, modern devices)
    PowerCordC7,
    /// 3-prong wall plug to IEC C13 kettle lead (Used by fat PS2, PC, high-power TVs)
    PowerCordC13,
    /// N64 AC Adapter with built-in wall cord and brick-to-console latching plug
    N64PowerAdapter,
    /// Proprietary Nintendo Multi-Out to 3x RCA (Yellow Video, White Audio L, Red Audio R)
    NintendoMultiOutToComposite,
    /// Proprietary PlayStation AV Multi-Out to 3x RCA (Yellow, White, Red)
    PlayStationMultiOutToComposite,
    /// Component AV Cable (5x RCA: Y, Pb, Pr, Audio L, Audio R)
    Component5Rca,
    /// Generic RCA Composite Cable (3x RCA male to 3x RCA male)
    RcaComposite3x3,
    /// N64 Controller with wired cord
    N64ControllerCord,
    /// PS1/PS2 Controller with wired cord
    PlayStationControllerCord,
}

/// A plug connector at an extremity of a cable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CableEnd {
    pub id: String,
    pub kind: SocketKind,
    pub world_pos: Vec3,
    pub connected_to_socket: Option<String>, // ID of the mated socket port
}

/// A physical cable holding one or more plugs at its ends.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CableEntity {
    pub id: String,
    pub cable_type: CableType,
    /// Plugs on side A (e.g. wall plug or console multi-out)
    pub ends_a: Vec<CableEnd>,
    /// Plugs on side B (e.g. TV RCA plugs or console connector)
    pub ends_b: Vec<CableEnd>,
    /// Cable thickness in meters
    pub thickness: f32,
    /// Total length in meters
    pub length: f32,
    /// Cable jacket color [R, G, B]
    pub color_rgb: [u8; 3],
}

impl CableEntity {
    pub fn new_nintendo_av_composite(id: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            id: id_str.clone(),
            cable_type: CableType::NintendoMultiOutToComposite,
            ends_a: vec![CableEnd {
                id: format!("{}_multiout", id_str),
                kind: SocketKind::NintendoMultiOutPlug,
                world_pos: Vec3::ZERO,
                connected_to_socket: None,
            }],
            ends_b: vec![
                CableEnd {
                    id: format!("{}_rca_yellow", id_str),
                    kind: SocketKind::RcaPlug(RcaColor::YellowVideo),
                    world_pos: Vec3::ZERO,
                    connected_to_socket: None,
                },
                CableEnd {
                    id: format!("{}_rca_white", id_str),
                    kind: SocketKind::RcaPlug(RcaColor::WhiteAudioLeft),
                    world_pos: Vec3::ZERO,
                    connected_to_socket: None,
                },
                CableEnd {
                    id: format!("{}_rca_red", id_str),
                    kind: SocketKind::RcaPlug(RcaColor::RedAudioRight),
                    world_pos: Vec3::ZERO,
                    connected_to_socket: None,
                },
            ],
            thickness: 0.006,
            length: 1.8,
            color_rgb: [30, 30, 30],
        }
    }

    pub fn new_playstation_av_composite(id: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            id: id_str.clone(),
            cable_type: CableType::PlayStationMultiOutToComposite,
            ends_a: vec![CableEnd {
                id: format!("{}_ps_multiout", id_str),
                kind: SocketKind::PlayStationMultiOutPlug,
                world_pos: Vec3::ZERO,
                connected_to_socket: None,
            }],
            ends_b: vec![
                CableEnd {
                    id: format!("{}_rca_yellow", id_str),
                    kind: SocketKind::RcaPlug(RcaColor::YellowVideo),
                    world_pos: Vec3::ZERO,
                    connected_to_socket: None,
                },
                CableEnd {
                    id: format!("{}_rca_white", id_str),
                    kind: SocketKind::RcaPlug(RcaColor::WhiteAudioLeft),
                    world_pos: Vec3::ZERO,
                    connected_to_socket: None,
                },
                CableEnd {
                    id: format!("{}_rca_red", id_str),
                    kind: SocketKind::RcaPlug(RcaColor::RedAudioRight),
                    world_pos: Vec3::ZERO,
                    connected_to_socket: None,
                },
            ],
            thickness: 0.006,
            length: 2.0,
            color_rgb: [25, 25, 25],
        }
    }

    pub fn new_power_c7(id: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            id: id_str.clone(),
            cable_type: CableType::PowerCordC7,
            ends_a: vec![CableEnd {
                id: format!("{}_wall", id_str),
                kind: SocketKind::WallAcPlug,
                world_pos: Vec3::ZERO,
                connected_to_socket: None,
            }],
            ends_b: vec![CableEnd {
                id: format!("{}_c7", id_str),
                kind: SocketKind::IecC7Plug,
                world_pos: Vec3::ZERO,
                connected_to_socket: None,
            }],
            thickness: 0.007,
            length: 1.5,
            color_rgb: [20, 20, 20],
        }
    }

    pub fn new_n64_power_adapter(id: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            id: id_str.clone(),
            cable_type: CableType::N64PowerAdapter,
            ends_a: vec![CableEnd {
                id: format!("{}_wall", id_str),
                kind: SocketKind::WallAcPlug,
                world_pos: Vec3::ZERO,
                connected_to_socket: None,
            }],
            ends_b: vec![CableEnd {
                id: format!("{}_n64plug", id_str),
                kind: SocketKind::N64AcBrickPlug,
                world_pos: Vec3::ZERO,
                connected_to_socket: None,
            }],
            thickness: 0.008,
            length: 2.2,
            color_rgb: [35, 35, 38],
        }
    }
}
