use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RcaColor {
    YellowVideo,
    WhiteAudioLeft,
    RedAudioRight,
    GreenComponentY,
    BlueComponentPb,
    RedComponentPr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocketKind {
    // Power
    WallAcOutlet,
    WallAcPlug,
    IecC7Jack,     // Figure-8 female
    IecC7Plug,     // Figure-8 male
    IecC13Jack,    // PS2 Fat standard kettle cord female
    IecC13Plug,    // Male cord end
    N64AcBrickJack, // N64 back proprietary socket
    N64AcBrickPlug, // N64 power brick output connector

    // Composite & Component RCA
    RcaJack(RcaColor),
    RcaPlug(RcaColor),

    // Proprietary Console AV
    NintendoMultiOutJack,
    NintendoMultiOutPlug,
    PlayStationMultiOutJack,
    PlayStationMultiOutPlug,

    // Controllers
    N64ControllerJack,
    N64ControllerPlug,
    PlayStationControllerJack,
    PlayStationControllerPlug,

    // Digital
    HdmiJack,
    HdmiPlug,
}

impl SocketKind {
    /// Determines whether two connector interfaces are physically and electrically mating pairs.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Self::WallAcOutlet, Self::WallAcPlug) | (Self::WallAcPlug, Self::WallAcOutlet) => true,
            (Self::IecC7Jack, Self::IecC7Plug) | (Self::IecC7Plug, Self::IecC7Jack) => true,
            (Self::IecC13Jack, Self::IecC13Plug) | (Self::IecC13Plug, Self::IecC13Jack) => true,
            (Self::N64AcBrickJack, Self::N64AcBrickPlug) | (Self::N64AcBrickPlug, Self::N64AcBrickJack) => true,

            // RCA jacks can accept RCA plugs of matching or generic RCA types.
            // In physical reality, you can plug a red RCA plug into a yellow jack,
            // which results in audio hum on the screen!
            (Self::RcaJack(_), Self::RcaPlug(_)) | (Self::RcaPlug(_), Self::RcaJack(_)) => true,

            (Self::NintendoMultiOutJack, Self::NintendoMultiOutPlug)
            | (Self::NintendoMultiOutPlug, Self::NintendoMultiOutJack) => true,

            (Self::PlayStationMultiOutJack, Self::PlayStationMultiOutPlug)
            | (Self::PlayStationMultiOutPlug, Self::PlayStationMultiOutJack) => true,

            (Self::N64ControllerJack, Self::N64ControllerPlug)
            | (Self::N64ControllerPlug, Self::N64ControllerJack) => true,

            (Self::PlayStationControllerJack, Self::PlayStationControllerPlug)
            | (Self::PlayStationControllerPlug, Self::PlayStationControllerJack) => true,

            (Self::HdmiJack, Self::HdmiPlug) | (Self::HdmiPlug, Self::HdmiJack) => true,

            _ => false,
        }
    }

    pub fn is_plug(&self) -> bool {
        matches!(
            self,
            Self::WallAcPlug
                | Self::IecC7Plug
                | Self::IecC13Plug
                | Self::N64AcBrickPlug
                | Self::RcaPlug(_)
                | Self::NintendoMultiOutPlug
                | Self::PlayStationMultiOutPlug
                | Self::N64ControllerPlug
                | Self::PlayStationControllerPlug
                | Self::HdmiPlug
        )
    }

    pub fn is_jack(&self) -> bool {
        !self.is_plug()
    }
}

/// A physical socket located on a device or cable end.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SocketPort {
    pub id: String,
    pub kind: SocketKind,
    /// Relative position in device local space
    pub local_pos: Vec3,
    /// Normal vector pointing outward from the socket
    pub normal: Vec3,
    /// Up vector defining rotational orientation
    pub up: Vec3,
    /// Optional label (e.g. "VIDEO IN", "AUDIO L", "PORT 1")
    pub label: String,
}
