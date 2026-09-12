use crate::cables::{CableEntity, CableType};
use crate::devices::{
    CrtTelevision, Nintendo64Console, PlayStation1Console, PlayStation2Console, PowerStrip,
    TvInputSource, UnifiedGamepadState, WallOutlet,
};
use crate::media::Platform;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Result of evaluating the signal graph for a TV screen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TvScreenFeed {
    PoweredOff,
    StaticNoise,
    ActiveVideo {
        console_id: String,
        platform: Platform,
    },
}

/// Identifies an active audio feed routed into a TV's speaker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TvAudioFeed {
    pub console_id: String,
    pub volume: f32, // 0.0 to 1.0
}

/// The global circuit and signal routing graph.
#[derive(Debug, Default)]
pub struct CircuitGraph {
    pub outlets: HashMap<String, WallOutlet>,
    pub power_strips: HashMap<String, PowerStrip>,
    pub televisions: HashMap<String, CrtTelevision>,
    pub n64_consoles: HashMap<String, Nintendo64Console>,
    pub ps1_consoles: HashMap<String, PlayStation1Console>,
    pub ps2_consoles: HashMap<String, PlayStation2Console>,
    pub cables: HashMap<String, CableEntity>,

    /// Socket connections: key = SocketPort ID, value = Mated SocketPort ID (bidirectional link)
    pub connections: HashMap<String, String>,

    /// Controller input states mapped by controller ID
    pub controller_states: HashMap<String, UnifiedGamepadState>,
}

impl CircuitGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Plug a connector into a socket.
    pub fn connect(&mut self, socket_a: impl Into<String>, socket_b: impl Into<String>) {
        let a = socket_a.into();
        let b = socket_b.into();

        // Disconnect existing links if any
        self.disconnect(&a);
        self.disconnect(&b);

        self.connections.insert(a.clone(), b.clone());
        self.connections.insert(b, a);
    }

    /// Unplug a connector from its socket.
    pub fn disconnect(&mut self, socket_id: &str) {
        if let Some(mated) = self.connections.remove(socket_id) {
            self.connections.remove(&mated);
        }
    }

    /// Check if two sockets are currently connected.
    pub fn is_connected(&self, socket_a: &str, socket_b: &str) -> bool {
        self.connections.get(socket_a).map(|s| s.as_str()) == Some(socket_b)
    }

    /// Evaluate which sockets currently receive live AC mains power.
    pub fn evaluate_powered_sockets(&self) -> HashSet<String> {
        let mut live_sockets = HashSet::new();

        // 1. All wall outlets are permanently energized
        for outlet in self.outlets.values() {
            for port in &outlet.sockets {
                live_sockets.insert(port.id.clone());
            }
        }

        // 2. Propagate power through power strips & power cables iteratively
        let mut changed = true;
        while changed {
            changed = false;

            // Power strips: if their plug is connected to a live socket and switch is on, their outlets are energized
            for strip in self.power_strips.values() {
                if !strip.switch_on {
                    continue;
                }
                if let Some(mated) = self.connections.get(&strip.plug_id) {
                    if live_sockets.contains(mated) {
                        for socket in &strip.sockets {
                            if live_sockets.insert(socket.id.clone()) {
                                changed = true;
                            }
                        }
                    }
                }
            }

            // Power cables: conduct power from side A to side B
            for cable in self.cables.values() {
                if matches!(
                    cable.cable_type,
                    CableType::PowerCordC7 | CableType::PowerCordC13 | CableType::N64PowerAdapter
                ) {
                    for a in &cable.ends_a {
                        if let Some(mated) = self.connections.get(&a.id) {
                            if live_sockets.contains(mated) {
                                for b in &cable.ends_b {
                                    if live_sockets.insert(b.id.clone()) {
                                        changed = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        live_sockets
    }

    /// Check if a specific device's power input socket is receiving energized power.
    pub fn is_socket_energized(&self, live_sockets: &HashSet<String>, socket_id: &str) -> bool {
        if let Some(mated) = self.connections.get(socket_id) {
            live_sockets.contains(mated)
        } else {
            false
        }
    }

    /// Check whether a console is currently powered and operating.
    pub fn is_n64_running(&self, live_sockets: &HashSet<String>, console_id: &str) -> bool {
        if let Some(n64) = self.n64_consoles.get(console_id) {
            let power_socket_id = format!("{}_power_in", console_id);
            if n64.power_switch_on && self.is_socket_energized(live_sockets, &power_socket_id) {
                return true;
            }
        }
        false
    }

    pub fn is_ps1_running(&self, live_sockets: &HashSet<String>, console_id: &str) -> bool {
        if let Some(ps1) = self.ps1_consoles.get(console_id) {
            let power_socket_id = format!("{}_power_in", console_id);
            if ps1.power_button_latched && self.is_socket_energized(live_sockets, &power_socket_id) {
                // PS1 boots even with no disc (BIOS memory card screen)
                return true;
            }
        }
        false
    }

    pub fn is_ps2_running(&self, live_sockets: &HashSet<String>, console_id: &str) -> bool {
        if let Some(ps2) = self.ps2_consoles.get(console_id) {
            let power_socket_id = format!("{}_power_in", console_id);
            if ps2.rear_rocker_switch_on
                && ps2.is_system_running
                && self.is_socket_energized(live_sockets, &power_socket_id)
            {
                return true;
            }
        }
        false
    }

    /// Trace what is currently displayed on a given TV screen.
    pub fn evaluate_tv_screen(&self, tv_id: &str) -> TvScreenFeed {
        let tv = match self.televisions.get(tv_id) {
            Some(t) => t,
            None => return TvScreenFeed::PoweredOff,
        };

        let live_sockets = self.evaluate_powered_sockets();
        let tv_power_socket_id = format!("{}_power_in", tv_id);
        if !tv.power_on || !self.is_socket_energized(&live_sockets, &tv_power_socket_id) {
            return TvScreenFeed::PoweredOff;
        }

        // Check which video jack corresponds to current TV input channel
        let target_video_jack = match tv.input_source {
            TvInputSource::CompositeAv1 => format!("{}_av1_video", tv_id),
            TvInputSource::CompositeAv2 => format!("{}_av2_video", tv_id),
            _ => return TvScreenFeed::StaticNoise,
        };

        // Find which cable end is plugged into this TV video jack
        if let Some(plug_id) = self.connections.get(&target_video_jack) {
            for cable in self.cables.values() {
                // Is this plug on cable side B (e.g. Yellow RCA)?
                if cable.ends_b.iter().any(|e| &e.id == plug_id) {
                    // Check side A of this cable
                    if let Some(side_a_plug) = cable.ends_a.first() {
                        if let Some(console_multiout) = self.connections.get(&side_a_plug.id) {
                            // Check N64
                            for (n64_id, _) in &self.n64_consoles {
                                if console_multiout == &format!("{}_multi_out", n64_id) {
                                    if self.is_n64_running(&live_sockets, n64_id) {
                                        return TvScreenFeed::ActiveVideo {
                                            console_id: n64_id.clone(),
                                            platform: Platform::Nintendo64,
                                        };
                                    }
                                }
                            }
                            // Check PS1
                            for (ps1_id, _) in &self.ps1_consoles {
                                if console_multiout == &format!("{}_multi_out", ps1_id) {
                                    if self.is_ps1_running(&live_sockets, ps1_id) {
                                        return TvScreenFeed::ActiveVideo {
                                            console_id: ps1_id.clone(),
                                            platform: Platform::PlayStation1,
                                        };
                                    }
                                }
                            }
                            // Check PS2
                            for (ps2_id, _) in &self.ps2_consoles {
                                if console_multiout == &format!("{}_multi_out", ps2_id) {
                                    if self.is_ps2_running(&live_sockets, ps2_id) {
                                        return TvScreenFeed::ActiveVideo {
                                            console_id: ps2_id.clone(),
                                            platform: Platform::PlayStation2,
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // TV is ON, but no video signal received on selected input
        TvScreenFeed::StaticNoise
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::CartridgeMedia;

    #[test]
    fn test_n64_signal_propagation() {
        let mut graph = CircuitGraph::new();

        let outlet = WallOutlet::new("wall");
        let power_strip = PowerStrip::new("strip");
        let mut tv = CrtTelevision::new_retro_crt("tv");
        let mut n64 = Nintendo64Console::new("n64");
        let n64_av = CableEntity::new_nintendo_av_composite("n64_av");
        let n64_power = CableEntity::new_n64_power_adapter("n64_pwr");
        let tv_power = CableEntity::new_power_c7("tv_pwr");

        graph.outlets.insert(outlet.id.clone(), outlet);
        graph.power_strips.insert(power_strip.id.clone(), power_strip);
        graph.televisions.insert(tv.id.clone(), tv.clone());
        graph.n64_consoles.insert(n64.id.clone(), n64.clone());
        graph.cables.insert(n64_av.id.clone(), n64_av);
        graph.cables.insert(n64_power.id.clone(), n64_power);
        graph.cables.insert(tv_power.id.clone(), tv_power);

        // Initially unplugged
        assert_eq!(graph.evaluate_tv_screen("tv"), TvScreenFeed::PoweredOff);

        // Plug power strip into wall
        graph.connect("strip_cord_plug", "wall_top");

        // Plug TV into strip and turn it ON
        graph.connect("tv_pwr_wall", "strip_outlet_1");
        graph.connect("tv_pwr_c7", "tv_power_in");
        tv.toggle_power();
        graph.televisions.insert("tv".into(), tv);

        // Now TV is on, but no input signal -> Static Noise
        assert_eq!(graph.evaluate_tv_screen("tv"), TvScreenFeed::StaticNoise);

        // Plug N64 into power strip and back of console
        graph.connect("n64_pwr_wall", "strip_outlet_2");
        graph.connect("n64_pwr_n64plug", "n64_power_in");

        // Plug Nintendo Multi-Out to TV AV1
        graph.connect("n64_av_multiout", "n64_multi_out");
        graph.connect("n64_av_rca_yellow", "tv_av1_video");
        graph.connect("n64_av_rca_white", "tv_av1_audio_l");
        graph.connect("n64_av_rca_red", "tv_av1_audio_r");

        // Insert game and turn N64 switch ON
        let cart = CartridgeMedia {
            id: "mario64".into(),
            title: "Super Mario 64".into(),
            platform: Platform::Nintendo64,
            rom_path: std::path::PathBuf::from("sm64.z64"),
            label_texture_path: None,
            plastic_color_rgba: [50, 50, 50, 255],
        };
        n64.insert_cartridge(cart);
        n64.set_power_switch(true);
        graph.n64_consoles.insert("n64".into(), n64);

        // Signal graph now resolves active video feed!
        assert_eq!(
            graph.evaluate_tv_screen("tv"),
            TvScreenFeed::ActiveVideo {
                console_id: "n64".into(),
                platform: Platform::Nintendo64,
            }
        );
    }
}
