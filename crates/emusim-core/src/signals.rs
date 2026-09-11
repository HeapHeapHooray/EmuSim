use serde::{Deserialize, Serialize};

/// Discrete electric or data signal carried over a wire/pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalType {
    /// AC Mains power (Wall voltage, nominal 120V or 230V)
    PowerAc,
    /// DC Low voltage regulated power (e.g. 5V, 9V, 12V)
    PowerDc,
    /// Analog Composite Video (Yellow RCA / CVBS)
    CompositeVideo,
    /// Analog S-Video Luma (Y)
    SVideoLuma,
    /// Analog S-Video Chroma (C)
    SVideoChroma,
    /// Analog Component Y (Luminance + Sync)
    ComponentY,
    /// Analog Component Pb (Blue difference)
    ComponentPb,
    /// Analog Component Pr (Red difference)
    ComponentPr,
    /// Analog Audio Left Channel (White RCA)
    AudioLeft,
    /// Analog Audio Right Channel (Red RCA)
    AudioRight,
    /// RF modulated signal (Coaxial / antenna)
    RfAntenna,
    /// Digital HDMI
    HdmiDigital,
    /// Digital Optical SPDIF / Toslink
    OpticalSpdif,
    /// Controller bidirectional input/output protocol
    ControllerData,
}

/// Power status across an electrical node.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PowerStatus {
    pub has_power: bool,
    pub voltage: f32,
}

impl PowerStatus {
    pub const OFF: Self = Self {
        has_power: false,
        voltage: 0.0,
    };

    pub const AC_MAINS: Self = Self {
        has_power: true,
        voltage: 120.0,
    };
}
