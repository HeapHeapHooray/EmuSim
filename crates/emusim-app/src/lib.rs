pub mod desktop_controller;
pub mod desktop_runner;
pub mod world;

pub use desktop_controller::*;
pub use desktop_runner::*;
pub use world::{RetroRoomScene, SelectedConsole};

/// Android entry point for Meta Quest standalone APK.
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn android_main(_app: *mut std::os::raw::c_void) {
    use tracing::info;
    tracing_subscriber::fmt::init();
    info!("Starting EmuSim on Meta Quest standalone (Android Horizon OS)...");

    let mut scene = RetroRoomScene::new();
    let xr_input = emusim_xr::input::XrFrameInput::default();
    scene.update(1.0 / 72.0, &xr_input);
}

#[cfg(test)]
mod tests {
    use super::*;
    use emusim_audio::{AudioOutputEngine, VrListener};
    use emusim_xr::input::{QuestControllerInput, XrFrameInput};
    use glam::{Quat, Vec3};

    #[test]
    fn test_ps1_audio_streaming() {
        let mut scene = RetroRoomScene::new();
        scene.wire_console_to_tv(SelectedConsole::PlayStation1);

        let xr_input = XrFrameInput {
            head_pose: emusim_xr::input::ControllerPose {
                position: Vec3::new(0.0, 1.4, 0.0),
                rotation: Quat::IDENTITY,
                ..Default::default()
            },
            left_controller: QuestControllerInput::default(),
            right_controller: QuestControllerInput::default(),
            delta_time: 1.0 / 60.0,
        };

        let listener = VrListener {
            position: Vec3::new(0.0, 1.4, 0.0),
            rotation: Quat::IDENTITY,
        };

        let audio_engine = AudioOutputEngine::new().ok();

        // Step scene to trigger core loading
        scene.update(1.0 / 60.0, &xr_input);

        // Let the worker thread run several frames of PS1 emulation
        let mut received_samples = 0;
        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            scene.update(1.0 / 60.0, &xr_input);
            while let Ok(mut samples) = scene.emulator_worker.audio_receiver.try_recv() {
                received_samples += samples.len();
                if let Some(ref audio) = audio_engine {
                    audio.push_spatial_samples(&mut samples, &scene.tv_spatial_audio, &listener);
                }
            }
            if received_samples > 0 {
                break;
            }
        }

        println!("PS1 audio test successfully captured {} audio samples from worker!", received_samples);
        assert!(received_samples > 0, "Emulator worker must produce audio samples that are preserved for the audio engine!");
    }
}

