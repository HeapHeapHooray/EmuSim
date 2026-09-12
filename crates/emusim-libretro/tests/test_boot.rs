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

    // Run 120 frames of boot sequence
    let mut total_audio_samples = 0;
    for f in 0..120 {
        core.run_frame();
        while let Ok(samples) = audio_rx.try_recv() {
            total_audio_samples += samples.len();
        }
        let frame = video_buffer.read_frame();
        if f % 30 == 0 {
            let nonzero = frame.pixels.iter().filter(|&&b| b > 0).count();
            println!(
                "Swanstation Frame {}: {}x{}, nonzero bytes {}, total audio samples {}",
                f, frame.width, frame.height, nonzero, total_audio_samples
            );
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

