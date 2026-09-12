use emusim_audio::SpatialSource;
use emusim_core::cables::CableEntity;
use emusim_core::devices::{
    CrtTelevision, Nintendo64Console, PlayStation1Console, PlayStation2Console, PowerStrip,
    WallOutlet,
};
use emusim_core::graph::{CircuitGraph, TvScreenFeed};
use emusim_core::media::{CartridgeMedia, DiscMedia, Platform};
use emusim_libretro::worker::EmulatorWorkerHandle;
use emusim_physics::cable_verlet::VerletCableStrand;
use emusim_render::crt_shader::CrtShaderUniforms;
use emusim_xr::input::XrFrameInput;
use emusim_xr::interaction::VrInteractionManager;
use glam::Vec3;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedConsole {
    Nintendo64,
    PlayStation1,
    PlayStation2,
    None,
}

impl SelectedConsole {
    pub fn from_str_name(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "n64" | "nintendo64" => Self::Nintendo64,
            "ps1" | "psx" | "playstation" | "playstation1" => Self::PlayStation1,
            "ps2" | "playstation2" => Self::PlayStation2,
            "none" | "static" | "tv" => Self::None,
            _ => Self::Nintendo64,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Nintendo64 => "Nintendo 64",
            Self::PlayStation1 => "Sony PlayStation 1",
            Self::PlayStation2 => "Sony PlayStation 2",
            Self::None => "None (Unwired)",
        }
    }
}

pub struct RetroRoomScene {
    pub graph: CircuitGraph,
    pub cable_physics: HashMap<String, VerletCableStrand>,
    pub emulator_worker: EmulatorWorkerHandle,
    pub interaction_manager: VrInteractionManager,
    pub tv_spatial_audio: SpatialSource,
    pub crt_uniforms: CrtShaderUniforms,
    pub elapsed_time: f32,
    pub active_loaded_rom: Option<PathBuf>,
    pub active_platform: Option<Platform>,
}

impl Default for RetroRoomScene {
    fn default() -> Self {
        Self::new()
    }
}

impl RetroRoomScene {
    pub fn new() -> Self {
        let mut graph = CircuitGraph::new();

        // 1. Fixed Wall Outlet on the room wall
        let wall_outlet = WallOutlet::new("wall_outlet_1");
        graph.outlets.insert(wall_outlet.id.clone(), wall_outlet);

        // 2. Power Strip surge protector on the floor
        let power_strip = PowerStrip::new("power_strip_1");
        graph.power_strips.insert(power_strip.id.clone(), power_strip);

        // 3. Retro 21-inch CRT Television on the TV stand
        let crt_tv = CrtTelevision::new_retro_crt("crt_tv_1");
        let tv_pos = Vec3::new(0.0, 0.75, -1.8);
        graph.televisions.insert(crt_tv.id.clone(), crt_tv);

        // 4. Nintendo 64 Console
        let mut n64 = Nintendo64Console::new("n64_console_1");
        if let Ok(entries) = std::fs::read_dir("games/n64") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if Platform::Nintendo64.matches_extension(ext) {
                        let title = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                        info!("Discovered N64 game on disk: {}", title);
                        n64.insert_cartridge(CartridgeMedia {
                            id: format!("cart_{}", title),
                            title,
                            platform: Platform::Nintendo64,
                            rom_path: path,
                            label_texture_path: None,
                            plastic_color_rgba: [50, 50, 50, 255],
                        });
                        break;
                    }
                }
            }
        }
        graph.n64_consoles.insert(n64.id.clone(), n64);

        // 5. PlayStation 1 Console
        let mut ps1 = PlayStation1Console::new("ps1_console_1");
        if let Ok(entries) = std::fs::read_dir("games/ps1") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if Platform::PlayStation1.matches_extension(ext) {
                        let title = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                        info!("Discovered PS1 game on disk: {}", title);
                        ps1.insert_disc(DiscMedia {
                            id: format!("disc_{}", title),
                            title,
                            platform: Platform::PlayStation1,
                            disc_path: path,
                            label_texture_path: None,
                            is_dvd: false,
                        });
                        break;
                    }
                }
            }
        }
        graph.ps1_consoles.insert(ps1.id.clone(), ps1);

        // 6. PlayStation 2 Console
        let mut ps2 = PlayStation2Console::new("ps2_console_1");
        if let Ok(entries) = std::fs::read_dir("games/ps2") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if Platform::PlayStation2.matches_extension(ext) {
                        let title = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                        info!("Discovered PS2 game on disk: {}", title);
                        ps2.insert_disc(DiscMedia {
                            id: format!("disc_ps2_{}", title),
                            title,
                            platform: Platform::PlayStation2,
                            disc_path: path,
                            label_texture_path: None,
                            is_dvd: true,
                        });
                        break;
                    }
                }
            }
        }
        graph.ps2_consoles.insert(ps2.id.clone(), ps2);

        // 7. Physical Cables
        let n64_av_cable = CableEntity::new_nintendo_av_composite("n64_av_cable_1");
        let ps_av_cable = CableEntity::new_playstation_av_composite("ps_av_cable_1");
        let n64_power = CableEntity::new_n64_power_adapter("n64_power_1");
        let tv_power = CableEntity::new_power_c7("tv_power_1");
        let ps1_power = CableEntity::new_power_c7("ps1_power_1");

        let mut cable_physics = HashMap::new();

        // Initialize Verlet physics strands for each cable
        for cable in [&n64_av_cable, &ps_av_cable, &n64_power, &tv_power, &ps1_power] {
            let start = Vec3::new(0.0, 0.0, -1.0);
            let end = Vec3::new(0.2, 0.0, -1.0);
            let strand = VerletCableStrand::new(start, end, 16, cable.thickness * 0.5);
            cable_physics.insert(cable.id.clone(), strand);
        }

        graph.cables.insert(n64_av_cable.id.clone(), n64_av_cable);
        graph.cables.insert(ps_av_cable.id.clone(), ps_av_cable);
        graph.cables.insert(n64_power.id.clone(), n64_power);
        graph.cables.insert(tv_power.id.clone(), tv_power);
        graph.cables.insert(ps1_power.id.clone(), ps1_power);

        let emulator_worker = EmulatorWorkerHandle::spawn();
        let interaction_manager = VrInteractionManager::new();
        let tv_spatial_audio = SpatialSource::new(tv_pos);

        info!("RetroRoomScene initialized with N64, PS1, PS2, CRT TV, and cables");

        Self {
            graph,
            cable_physics,
            emulator_worker,
            interaction_manager,
            tv_spatial_audio,
            crt_uniforms: CrtShaderUniforms::default(),
            elapsed_time: 0.0,
            active_loaded_rom: None,
            active_platform: None,
        }
    }

    /// Step simulation: physics, circuit evaluation, emulator synchronization, and audio.
    pub fn update(&mut self, dt: f32, xr_input: &XrFrameInput) {
        self.elapsed_time += dt;

        // 1. Step cable physics
        let gravity = Vec3::new(0.0, -9.81, 0.0);
        let floor_y = 0.0;
        for strand in self.cable_physics.values_mut() {
            strand.step(dt, gravity, floor_y);
        }

        // 2. Evaluate circuit & signal routing graph
        let tv_feed = self.graph.evaluate_tv_screen("crt_tv_1");

        // 3. Synchronize emulation state based on TV video feed
        match tv_feed {
            TvScreenFeed::ActiveVideo {
                ref console_id,
                platform,
            } => {
                self.crt_uniforms.static_noise_intensity = 0.0;
                self.crt_uniforms.power_fade = 1.0;

                // Check if we have media inserted for this console
                let target_rom = match platform {
                    Platform::Nintendo64 => self
                        .graph
                        .n64_consoles
                        .get(console_id)
                        .and_then(|n| n.inserted_cartridge.as_ref().map(|c| c.rom_path.clone())),
                    Platform::PlayStation1 => self
                        .graph
                        .ps1_consoles
                        .get(console_id)
                        .and_then(|p| p.spindle_disc.as_ref().map(|d| d.disc_path.clone())),
                    Platform::PlayStation2 => self
                        .graph
                        .ps2_consoles
                        .get(console_id)
                        .and_then(|p| p.tray_disc.as_ref().map(|d| d.disc_path.clone())),
                    _ => None,
                };

                let valid_rom = target_rom.filter(|p| p.is_file());

                if self.active_platform != Some(platform) || self.active_loaded_rom != valid_rom {
                    let core_name = platform.default_core_name();
                    let core_path = [
                        format!("cores/{}_libretro.so", core_name),
                        format!("../../cores/{}_libretro.so", core_name),
                        format!("../cores/{}_libretro.so", core_name),
                    ]
                    .into_iter()
                    .map(PathBuf::from)
                    .find(|p| p.is_file());

                    if let Some(core_path) = core_path {
                        info!(
                            "Booting authentic emulation for platform {:?} with core {:?} (ROM: {:?})",
                            platform, core_path, valid_rom
                        );
                        self.emulator_worker.load_game(core_path, valid_rom.clone());
                        self.active_platform = Some(platform);
                        self.active_loaded_rom = valid_rom;
                    } else {
                        // Core library not downloaded yet -> show standby screen instructing how to download cores
                        let mut frame = self.emulator_worker.video_buffer.write_frame();
                        emusim_render::osd::render_standby_screen(&mut frame, platform, self.elapsed_time);
                    }
                }

                // Forward VR controllers as gamepad input to emulator if VR controllers are tracked
                if xr_input.left_controller.pose.is_tracked || xr_input.right_controller.pose.is_tracked {
                    let gamepad = VrInteractionManager::map_quest_to_gamepad(
                        &xr_input.left_controller,
                        &xr_input.right_controller,
                    );
                    self.emulator_worker.send_input(gamepad);
                }
            }
            TvScreenFeed::StaticNoise => {
                self.crt_uniforms.static_noise_intensity = 1.0;
                self.crt_uniforms.power_fade = 1.0;
                if self.active_platform.is_some() {
                    self.active_platform = None;
                    self.active_loaded_rom = None;
                    self.emulator_worker.pause();
                }
            }
            TvScreenFeed::PoweredOff => {
                self.crt_uniforms.power_fade = 0.0;
                if self.active_platform.is_some() {
                    self.active_platform = None;
                    self.active_loaded_rom = None;
                    self.emulator_worker.pause();
                }
            }
        }

        // 4. Update CRT shader uniforms
        if let Some(tv) = self.graph.televisions.get_mut("crt_tv_1") {
            if tv.degauss_active_timer > 0.0 {
                tv.degauss_active_timer -= dt;
                self.crt_uniforms.degauss_active = tv.degauss_active_timer.max(0.0);
            } else {
                self.crt_uniforms.degauss_active = 0.0;
            }
        }
        self.crt_uniforms.time_seconds = self.elapsed_time;
    }

    /// Wires power strip to wall, CRT TV to power strip, and routes the selected console to the TV's AV1 input.
    pub fn wire_console_to_tv(&mut self, console: SelectedConsole) {
        info!("Setting active console in room to: {}", console.display_name());

        // 1. Ensure Power Strip is plugged into Wall Outlet
        self.graph
            .connect("power_strip_1_cord_plug", "wall_outlet_1_top");

        // 2. Ensure CRT TV is plugged into Power Strip and turned ON
        self.graph
            .connect("tv_power_1_wall", "power_strip_1_outlet_1");
        self.graph.connect("tv_power_1_c7", "crt_tv_1_power_in");
        if let Some(tv) = self.graph.televisions.get_mut("crt_tv_1") {
            if !tv.power_on {
                tv.toggle_power();
            }
        }

        // 3. Disconnect any existing console AV connections to TV AV1
        self.graph.disconnect("crt_tv_1_av1_video");
        self.graph.disconnect("crt_tv_1_av1_audio_l");
        self.graph.disconnect("crt_tv_1_av1_audio_r");

        // Turn off all consoles first
        if let Some(n64) = self.graph.n64_consoles.get_mut("n64_console_1") {
            n64.set_power_switch(false);
        }
        if let Some(ps1) = self.graph.ps1_consoles.get_mut("ps1_console_1") {
            ps1.power_button_latched = false;
        }
        if let Some(ps2) = self.graph.ps2_consoles.get_mut("ps2_console_1") {
            ps2.is_system_running = false;
        }

        // Reset loaded ROM cache to trigger clean core load for new platform
        self.active_loaded_rom = None;
        self.active_platform = None;

        // 4. Wire and power up selected console
        match console {
            SelectedConsole::Nintendo64 => {
                self.graph
                    .connect("n64_power_1_wall", "power_strip_1_outlet_2");
                self.graph
                    .connect("n64_power_1_n64plug", "n64_console_1_power_in");
                self.graph
                    .connect("n64_av_cable_1_multiout", "n64_console_1_multi_out");
                self.graph
                    .connect("n64_av_cable_1_rca_yellow", "crt_tv_1_av1_video");
                self.graph
                    .connect("n64_av_cable_1_rca_white", "crt_tv_1_av1_audio_l");
                self.graph
                    .connect("n64_av_cable_1_rca_red", "crt_tv_1_av1_audio_r");

                if let Some(n64) = self.graph.n64_consoles.get_mut("n64_console_1") {
                    n64.set_power_switch(true);
                }
            }
            SelectedConsole::PlayStation1 => {
                self.graph
                    .connect("ps1_power_1_wall", "power_strip_1_outlet_3");
                self.graph
                    .connect("ps1_power_1_c7", "ps1_console_1_power_in");
                self.graph
                    .connect("ps_av_cable_1_ps_multiout", "ps1_console_1_multi_out");
                self.graph
                    .connect("ps_av_cable_1_rca_yellow", "crt_tv_1_av1_video");
                self.graph
                    .connect("ps_av_cable_1_rca_white", "crt_tv_1_av1_audio_l");
                self.graph
                    .connect("ps_av_cable_1_rca_red", "crt_tv_1_av1_audio_r");

                if let Some(ps1) = self.graph.ps1_consoles.get_mut("ps1_console_1") {
                    ps1.power_button_latched = true;
                }
            }
            SelectedConsole::PlayStation2 => {
                self.graph
                    .connect("ps1_power_1_wall", "power_strip_1_outlet_3");
                self.graph
                    .connect("ps1_power_1_c7", "ps2_console_1_power_in");
                self.graph
                    .connect("ps_av_cable_1_ps_multiout", "ps2_console_1_multi_out");
                self.graph
                    .connect("ps_av_cable_1_rca_yellow", "crt_tv_1_av1_video");
                self.graph
                    .connect("ps_av_cable_1_rca_white", "crt_tv_1_av1_audio_l");
                self.graph
                    .connect("ps_av_cable_1_rca_red", "crt_tv_1_av1_audio_r");

                if let Some(ps2) = self.graph.ps2_consoles.get_mut("ps2_console_1") {
                    ps2.rear_rocker_switch_on = true;
                    ps2.is_system_running = true;
                }
            }
            SelectedConsole::None => {
                info!("CRT TV is on with static noise (no console connected).");
            }
        }
    }
}
