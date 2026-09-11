use emusim_audio::{SpatialSource, VrListener};
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

pub struct RetroRoomScene {
    pub graph: CircuitGraph,
    pub cable_physics: HashMap<String, VerletCableStrand>,
    pub emulator_worker: EmulatorWorkerHandle,
    pub interaction_manager: VrInteractionManager,
    pub tv_spatial_audio: SpatialSource,
    pub crt_uniforms: CrtShaderUniforms,
    pub elapsed_time: f32,
    pub active_loaded_rom: Option<PathBuf>,
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
        let ps2 = PlayStation2Console::new("ps2_console_1");
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

                // Check if we need to launch the emulator core for this console
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

                if let Some(rom_path) = target_rom {
                    if rom_path.is_file() {
                        if self.active_loaded_rom.as_ref() != Some(&rom_path) {
                            let core_lib = format!("cores/{}_libretro.so", platform.default_core_name());
                            info!("Loading platform {:?} with core {}", platform, core_lib);
                            self.emulator_worker
                                .load_game(PathBuf::from(core_lib), rom_path.clone());
                            self.active_loaded_rom = Some(rom_path);
                        }
                    } else {
                        // Game media is inserted but file is not on disk -> display static noise
                        self.crt_uniforms.static_noise_intensity = 0.8;
                    }
                }

                // Forward VR controllers as gamepad input to emulator
                let gamepad = VrInteractionManager::map_quest_to_gamepad(
                    &xr_input.left_controller,
                    &xr_input.right_controller,
                );
                self.emulator_worker.send_input(gamepad);
            }
            TvScreenFeed::StaticNoise => {
                self.crt_uniforms.static_noise_intensity = 1.0;
                self.crt_uniforms.power_fade = 1.0;
            }
            TvScreenFeed::PoweredOff => {
                self.crt_uniforms.power_fade = 0.0;
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

        // 5. Drain audio samples from emulator worker and spatialise
        let listener = VrListener {
            position: xr_input.head_pose.position,
            rotation: xr_input.head_pose.rotation,
        };

        while let Ok(mut samples) = self.emulator_worker.audio_receiver.try_recv() {
            self.tv_spatial_audio.process_spatial(&mut samples, &listener);
            // In full audio engine, write samples to device audio output ring buffer
        }
    }
}
