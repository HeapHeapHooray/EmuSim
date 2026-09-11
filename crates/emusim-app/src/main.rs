use emusim_app::{run_desktop_app, RetroRoomScene};
use emusim_core::graph::TvScreenFeed;
use emusim_xr::input::{QuestControllerInput, XrFrameInput};
use glam::{Quat, Vec3};
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--headless" || a == "--test-circuit") {
        run_headless_test();
        return Ok(());
    }

    info!("=== Launching EmuSim Cross-Platform Desktop Window ===");
    info!("Controls:");
    info!("  [W, A, S, D]        Walk around the retro bedroom");
    info!("  [Mouse]             Look around (first-person)");
    info!("  [Left Click / E]    Interact / Push Power buttons / Cycle TV inputs");
    info!("  [Right Click / Q]   Drop held item / Unplug");
    info!("  [Tab / G]           Toggle GAME FOCUS mode (sit in front of CRT TV)");
    info!("  [Arrow Keys / WASD] In Game Focus: D-Pad & Analog Stick");
    info!("  [J, K, U, I]        In Game Focus: A, B, X, Y buttons");
    info!("  [Enter, Shift]      In Game Focus: Start, Select");

    run_desktop_app()
}

fn run_headless_test() {
    info!("=== EmuSim - Headless Simulation Circuit Test ===");
    let mut scene = RetroRoomScene::new();

    let xr_input = XrFrameInput {
        head_pose: emusim_xr::input::ControllerPose {
            position: Vec3::new(0.0, 1.6, 0.0),
            rotation: Quat::IDENTITY,
            ..Default::default()
        },
        left_controller: QuestControllerInput::default(),
        right_controller: QuestControllerInput::default(),
        delta_time: 1.0 / 72.0,
    };

    scene.update(1.0 / 72.0, &xr_input);
    info!("Initial TV Feed: {:?}", scene.graph.evaluate_tv_screen("crt_tv_1"));

    // Connect Power Strip
    scene.graph.connect("power_strip_1_cord_plug", "wall_outlet_1_top");
    // Connect CRT TV
    scene.graph.connect("tv_power_1_wall", "power_strip_1_outlet_1");
    scene.graph.connect("tv_power_1_c7", "crt_tv_1_power_in");

    if let Some(tv) = scene.graph.televisions.get_mut("crt_tv_1") {
        tv.toggle_power();
    }
    scene.update(1.0 / 72.0, &xr_input);
    info!("TV Power ON feed: {:?}", scene.graph.evaluate_tv_screen("crt_tv_1"));

    // Connect N64
    scene.graph.connect("n64_power_1_wall", "power_strip_1_outlet_2");
    scene.graph.connect("n64_power_1_n64plug", "n64_console_1_power_in");
    scene.graph.connect("n64_av_cable_1_multiout", "n64_console_1_multi_out");
    scene.graph.connect("n64_av_cable_1_rca_yellow", "crt_tv_1_av1_video");
    scene.graph.connect("n64_av_cable_1_rca_white", "crt_tv_1_av1_audio_l");
    scene.graph.connect("n64_av_cable_1_rca_red", "crt_tv_1_av1_audio_r");

    if let Some(n64) = scene.graph.n64_consoles.get_mut("n64_console_1") {
        n64.set_power_switch(true);
    }
    scene.update(1.0 / 72.0, &xr_input);

    let tv_feed = scene.graph.evaluate_tv_screen("crt_tv_1");
    info!("N64 Live Video Feed: {:?}", tv_feed);
    assert!(matches!(tv_feed, TvScreenFeed::ActiveVideo { .. }));
    info!("Headless circuit test completed successfully!");
}
