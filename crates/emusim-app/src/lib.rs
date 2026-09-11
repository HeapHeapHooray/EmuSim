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
