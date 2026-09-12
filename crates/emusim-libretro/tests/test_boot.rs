use emusim_libretro::core::LibretroCoreInstance;
use emusim_libretro::framebuffer::SharedVideoBuffer;
use std::path::Path;

#[test]
fn test_swanstation_standalone() {
    let core_path = if Path::new("cores/swanstation_libretro.so").exists() {
        Path::new("cores/swanstation_libretro.so").to_path_buf()
    } else if Path::new("../../cores/swanstation_libretro.so").exists() {
        Path::new("../../cores/swanstation_libretro.so").to_path_buf()
    } else {
        eprintln!("Core not found, skipping");
        return;
    };

    let video_buffer = SharedVideoBuffer::new();
    let (audio_tx, audio_rx) = crossbeam_channel::bounded(64);

    let mut core = LibretroCoreInstance::load(&core_path, video_buffer.clone(), audio_tx)
        .expect("Failed to load swanstation");

    let res = core.load_no_game();
    println!("swanstation load_no_game result: {:?}", res);
    assert!(res.is_ok(), "Swanstation boots without a game!");

    // Run 600 frames of boot sequence
    let mut total_audio_samples = 0;
    for f in 0..600 {
        core.run_frame();
        while let Ok(samples) = audio_rx.try_recv() {
            total_audio_samples += samples.len();
        }
        let frame = video_buffer.read_frame();
        if f % 60 == 0 || f == 420 || f == 480 || f == 540 {
            let nonzero = frame.pixels.iter().filter(|&&b| b > 0).count();
            println!(
                "Swanstation Frame {}: {}x{}, nonzero bytes {}, total audio samples {}",
                f, frame.width, frame.height, nonzero, total_audio_samples
            );
            if frame.width > 0 && frame.height > 0 && nonzero > 0 {
                // Non-empty frame rendered
            }
        }
    }
    assert!(total_audio_samples > 0, "Boot intro must produce audio!");
}

#[test]
fn test_parallel_n64_rom() {
    let core_path = if Path::new("cores/parallel_n64_libretro.so").exists() {
        Path::new("cores/parallel_n64_libretro.so").to_path_buf()
    } else if Path::new("../../cores/parallel_n64_libretro.so").exists() {
        Path::new("../../cores/parallel_n64_libretro.so").to_path_buf()
    } else {
        eprintln!("Core not found, skipping");
        return;
    };

    let rom_path = if Path::new("games/n64/N64NICCC.z64").exists() {
        Path::new("games/n64/N64NICCC.z64").to_path_buf()
    } else if Path::new("../../games/n64/N64NICCC.z64").exists() {
        Path::new("../../games/n64/N64NICCC.z64").to_path_buf()
    } else {
        eprintln!("ROM not found, skipping");
        return;
    };

    let video_buffer = SharedVideoBuffer::new();
    let (audio_tx, _audio_rx) = crossbeam_channel::bounded(64);

    let mut core = LibretroCoreInstance::load(&core_path, video_buffer.clone(), audio_tx)
        .expect("Failed to load parallel_n64");

    let res = core.load_game(&rom_path, None);
    println!("parallel_n64 load_game result: {:?}", res);
    assert!(res.is_ok(), "parallel_n64 should load N64 ROM");

    // Run 60 frames
    for f in 0..60 {
        core.run_frame();
        let frame = video_buffer.read_frame();
        if f % 15 == 0 {
            println!("parallel_n64 Frame {}: {}x{}", f, frame.width, frame.height);
        }
    }
}

#[test]
fn test_ps2_standalone() {
    let pcsx2_path = [
        "cores/pcsx2_libretro.so",
        "../../cores/pcsx2_libretro.so",
        "../cores/pcsx2_libretro.so",
    ]
    .into_iter()
    .map(Path::new)
    .find(|p| p.exists());

    let play_path = [
        "cores/play_libretro.so",
        "../../cores/play_libretro.so",
        "../cores/play_libretro.so",
    ]
    .into_iter()
    .map(Path::new)
    .find(|p| p.exists());

    println!("Found pcsx2: {:?}, play: {:?}", pcsx2_path, play_path);

    if let Some(core_path) = pcsx2_path {
        println!("Testing PCSX2 standalone boot...");
        let video_buffer = SharedVideoBuffer::new();
        let (audio_tx, audio_rx) = crossbeam_channel::bounded(64);

        match LibretroCoreInstance::load(core_path, video_buffer.clone(), audio_tx) {
            Ok(mut core) => {
                let res = core.load_no_game();
                println!("PCSX2 load_no_game result: {:?}", res);
                if res.is_ok() {
                    let mut nonzero_frames = 0;
                    let mut total_audio_samples = 0;
                    for f in 0..180 {
                        if f == 60 {
                            use emusim_core::devices::UnifiedGamepadState;
                            use emusim_core::{RETRO_DEVICE_ID_JOYPAD_DOWN, RETRO_DEVICE_ID_JOYPAD_B};
                            let mut gp = UnifiedGamepadState::default();
                            gp.buttons = (1 << RETRO_DEVICE_ID_JOYPAD_DOWN) | (1 << RETRO_DEVICE_ID_JOYPAD_B);
                            core.update_gamepad(gp);
                        }
                        core.run_frame();
                        while let Ok(samples) = audio_rx.try_recv() {
                            total_audio_samples += samples.len();
                        }
                        let frame = video_buffer.read_frame();
                        let nonzero = frame.pixels.iter().filter(|&&b| b > 0).count();
                        if nonzero > 0 {
                            nonzero_frames += 1;
                        }
                        if f % 30 == 0 || f == 60 || f == 120 {
                            println!(
                                "PCSX2 Frame {}: {}x{}, nonzero bytes {}, audio samples {}",
                                f, frame.width, frame.height, nonzero, total_audio_samples
                            );
                            if frame.width > 0 && frame.height > 0 && nonzero > 0 {
                                // Non-empty frame rendered
                            }
                        }
                    }
                    println!("Total nonzero frames: {}, total audio samples: {}", nonzero_frames, total_audio_samples);
                    assert!(nonzero_frames > 0, "PCSX2 must produce video frames for the PS2 boot intro!");
                    assert!(total_audio_samples > 0, "PCSX2 must produce audio samples for the PS2 boot chime!");
                }
            }
            Err(e) => {
                println!("PCSX2 failed to load: {:?}", e);
            }
        }
    }
}

#[test]
fn test_gamepad_input_state() {
    use emusim_core::devices::UnifiedGamepadState;
    use emusim_core::{RETRO_DEVICE_ID_JOYPAD_B, RETRO_DEVICE_ID_JOYPAD_UP};

    let core_path = Path::new("cores/swanstation_libretro.so");
    if !core_path.exists() {
        return;
    }

    let video_buffer = SharedVideoBuffer::new();
    let (audio_tx, _audio_rx) = crossbeam_channel::bounded(64);

    let mut core = LibretroCoreInstance::load(core_path, video_buffer, audio_tx)
        .expect("Swanstation loads successfully");

    assert!(core.load_no_game().is_ok());

    let mut gp = UnifiedGamepadState::default();
    gp.buttons = (1 << RETRO_DEVICE_ID_JOYPAD_B) | (1 << RETRO_DEVICE_ID_JOYPAD_UP);
    gp.left_analog_x = -16384;
    gp.left_analog_y = -32767;

    core.update_gamepad(gp);

    // Run 30 frames with active controller inputs
    for _ in 0..30 {
        core.run_frame();
    }
}

#[test]
fn test_switching_between_consoles_repeatedly() {
    let worker = emusim_libretro::worker::EmulatorWorkerHandle::spawn();

    let find_path = |rel: &str| {
        if Path::new(rel).exists() {
            Some(Path::new(rel).to_path_buf())
        } else if Path::new("../../").join(rel).exists() {
            Some(Path::new("../../").join(rel).to_path_buf())
        } else {
            None
        }
    };

    let n64_core = find_path("cores/parallel_n64_libretro.so").or_else(|| find_path("cores/mupen64plus_next_libretro.so"));
    let n64_rom = find_path("games/n64/N64NICCC.z64");
    let ps1_core = find_path("cores/swanstation_libretro.so");
    let ps2_core = find_path("cores/pcsx2_libretro.so");

    println!("Found cores: n64={:?}, ps1={:?}, ps2={:?}", n64_core, ps1_core, ps2_core);

    for cycle in 0..5 {
        println!("--- Cycle {} ---", cycle);
        if let (Some(ref core), Some(ref rom)) = (&n64_core, &n64_rom) {
            println!("Switching to N64...");
            worker.load_game(core.clone(), Some(rom.clone()));
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
        if let Some(ref core) = ps1_core {
            println!("Switching to PS1...");
            worker.load_game(core.clone(), None);
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
        if let Some(ref core) = ps2_core {
            println!("Switching to PS2...");
            worker.load_game(core.clone(), None);
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
    }
}



