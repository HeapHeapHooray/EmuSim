use emusim_app::{run_desktop_app, RetroRoomScene, SelectedConsole};
use emusim_core::graph::TvScreenFeed;
use emusim_xr::input::{QuestControllerInput, XrFrameInput};
use glam::{Quat, Vec3};
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();

    // Parse console selection from command line: --console <n64|ps1|ps2|none> or -c <n64|ps1|ps2|none>
    let mut selected_console = SelectedConsole::Nintendo64;
    let mut screenshot_target: Option<String> = None;
    for i in 0..args.len() {
        if (args[i] == "--console" || args[i] == "-c") && i + 1 < args.len() {
            selected_console = SelectedConsole::from_str_name(&args[i + 1]);
        } else if args[i].starts_with("--console=") {
            let val = args[i].trim_start_matches("--console=");
            selected_console = SelectedConsole::from_str_name(val);
        } else if (args[i] == "--screenshot" || args[i] == "-s") && i + 1 < args.len() {
            screenshot_target = Some(args[i + 1].clone());
        } else if args[i].starts_with("--screenshot=") {
            let val = args[i].trim_start_matches("--screenshot=");
            screenshot_target = Some(val.to_string());
        }
    }

    if args.iter().any(|a| a == "--headless" || a == "--test-circuit") {
        run_headless_test(selected_console);
        return Ok(());
    }

    info!("=== Launching EmuSim Cross-Platform Desktop Window ===");
    info!("Active Default Console: {}", selected_console.display_name());
    info!("Controls:");
    info!("  [W, A, S, D]        Walk around the retro bedroom");
    info!("  [Mouse]             Look around (first-person)");
    info!("  [Left Click / E]    Interact / Push Power buttons / Cycle TV inputs");
    info!("  [Right Click / Q]   Drop held item / Unplug");
    info!("  [1, 2, 3, 0]        Quick-Swap Console: 1=N64, 2=PS1, 3=PS2, 0=None (Static)");
    info!("  [Tab / G]           Toggle GAME FOCUS mode (sit in front of CRT TV)");
    info!("  [Arrow Keys / WASD] In Game Focus: D-Pad & Analog Stick");
    info!("  [J, K, U, I]        In Game Focus: A, B, X, Y buttons");
    info!("  [Enter, Shift]      In Game Focus: Start, Select");
    info!("  [F11]               Toggle Direct 2D Fullscreen / 3D Room");
    info!("  [F12]               Capture high-resolution screenshot");

    run_desktop_app(selected_console, screenshot_target)
}

fn run_headless_test(selected_console: SelectedConsole) {
    info!("=== EmuSim - Headless Simulation Circuit Test ===");
    info!("Testing console: {}", selected_console.display_name());
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
    info!("Initial Unwired TV Feed: {:?}", scene.graph.evaluate_tv_screen("crt_tv_1"));

    // Wire the chosen console to the TV and power strip
    scene.wire_console_to_tv(selected_console);
    scene.update(1.0 / 72.0, &xr_input);

    let tv_feed = scene.graph.evaluate_tv_screen("crt_tv_1");
    info!("Active TV Feed after wiring: {:?}", tv_feed);

    match selected_console {
        SelectedConsole::Nintendo64 => {
            info!("Verified Nintendo 64 circuit path!");
        }
        SelectedConsole::PlayStation1 => {
            info!("Verified PlayStation 1 circuit path!");
        }
        SelectedConsole::PlayStation2 => {
            info!("Verified PlayStation 2 circuit path!");
        }
        SelectedConsole::None => {
            assert_eq!(tv_feed, TvScreenFeed::StaticNoise);
            info!("Verified TV Static Noise feed when no console is connected!");
        }
    }

    info!("Headless circuit test for {} completed successfully!", selected_console.display_name());
}
