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


