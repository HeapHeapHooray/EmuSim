use crate::sockets::{RcaColor, SocketKind, SocketPort};
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TvInputSource {
    AntennaChannel3,
    AntennaChannel4,
    CompositeAv1,
    CompositeAv2,
    ComponentYPbPr,
}

impl TvInputSource {
    pub fn next(&self) -> Self {
        match self {
            Self::AntennaChannel3 => Self::AntennaChannel4,
            Self::AntennaChannel4 => Self::CompositeAv1,
            Self::CompositeAv1 => Self::CompositeAv2,
            Self::CompositeAv2 => Self::ComponentYPbPr,
            Self::ComponentYPbPr => Self::AntennaChannel3,
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            Self::AntennaChannel3 => "CH 03",
            Self::AntennaChannel4 => "CH 04",
            Self::CompositeAv1 => "VIDEO 1",
            Self::CompositeAv2 => "VIDEO 2",
            Self::ComponentYPbPr => "COMPONENT",
        }
    }
}

/// CRT Television device with screen, speaker, and connection ports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrtTelevision {
    pub id: String,
    pub power_on: bool,
    pub input_source: TvInputSource,
    pub volume: u8, // 0..100
    pub muted: bool,
    pub degauss_active_timer: f32, // Seconds remaining for degauss wobble/flash
    pub static_noise_seed: u32,

    // Physical dimensions
    pub screen_diagonal_inches: f32,
    pub screen_aspect_ratio: f32, // typically 4.0 / 3.0

    // Sockets located on the TV chassis
    pub sockets: Vec<SocketPort>,
}

impl CrtTelevision {
    pub fn new_retro_crt(id: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            id: id_str.clone(),
            power_on: false,
            input_source: TvInputSource::CompositeAv1,
            volume: 65,
            muted: false,
            degauss_active_timer: 0.0,
            static_noise_seed: 12345,
            screen_diagonal_inches: 21.0,
            screen_aspect_ratio: 4.0 / 3.0,
            sockets: vec![
                // Rear Power cord or inlet
                SocketPort {
                    id: format!("{}_power_in", id_str),
                    kind: SocketKind::IecC7Jack,
                    local_pos: Vec3::new(0.18, 0.08, -0.22),
                    normal: Vec3::new(0.0, 0.0, -1.0),
                    up: Vec3::Y,
                    label: "AC IN".into(),
                },
                // AV1 Composite in (Rear)
                SocketPort {
                    id: format!("{}_av1_video", id_str),
                    kind: SocketKind::RcaJack(RcaColor::YellowVideo),
                    local_pos: Vec3::new(-0.15, 0.12, -0.22),
                    normal: Vec3::new(0.0, 0.0, -1.0),
                    up: Vec3::Y,
                    label: "AV1 VIDEO".into(),
                },
                SocketPort {
                    id: format!("{}_av1_audio_l", id_str),
                    kind: SocketKind::RcaJack(RcaColor::WhiteAudioLeft),
                    local_pos: Vec3::new(-0.15, 0.09, -0.22),
                    normal: Vec3::new(0.0, 0.0, -1.0),
                    up: Vec3::Y,
                    label: "AV1 AUDIO L".into(),
                },
                SocketPort {
                    id: format!("{}_av1_audio_r", id_str),
                    kind: SocketKind::RcaJack(RcaColor::RedAudioRight),
                    local_pos: Vec3::new(-0.15, 0.06, -0.22),
                    normal: Vec3::new(0.0, 0.0, -1.0),
                    up: Vec3::Y,
                    label: "AV1 AUDIO R".into(),
                },
                // AV2 Composite in (Front panel or side)
                SocketPort {
                    id: format!("{}_av2_video", id_str),
                    kind: SocketKind::RcaJack(RcaColor::YellowVideo),
                    local_pos: Vec3::new(-0.22, 0.05, 0.18),
                    normal: Vec3::new(-1.0, 0.0, 0.0),
                    up: Vec3::Y,
                    label: "AV2 VIDEO".into(),
                },
                SocketPort {
                    id: format!("{}_av2_audio_l", id_str),
                    kind: SocketKind::RcaJack(RcaColor::WhiteAudioLeft),
                    local_pos: Vec3::new(-0.22, 0.05, 0.15),
                    normal: Vec3::new(-1.0, 0.0, 0.0),
                    up: Vec3::Y,
                    label: "AV2 AUDIO L".into(),
                },
                SocketPort {
                    id: format!("{}_av2_audio_r", id_str),
                    kind: SocketKind::RcaJack(RcaColor::RedAudioRight),
                    local_pos: Vec3::new(-0.22, 0.05, 0.12),
                    normal: Vec3::new(-1.0, 0.0, 0.0),
                    up: Vec3::Y,
                    label: "AV2 AUDIO R".into(),
                },
            ],
        }
    }

    pub fn toggle_power(&mut self) {
        self.power_on = !self.power_on;
        if self.power_on {
            self.degauss_active_timer = 0.8; // Iconic CRT coil degauss click & picture wobble
        }
    }

    pub fn cycle_input(&mut self) {
        self.input_source = self.input_source.next();
    }
}
