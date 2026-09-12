use crate::desktop_controller::{DesktopFirstPersonController, DesktopPlayMode};
use crate::world::RetroRoomScene;
use emusim_audio::{AudioOutputEngine, CrtStaticAudioGenerator, VrListener};
use emusim_core::graph::TvScreenFeed;
use emusim_render::wgpu_renderer::WgpuCrtRenderer;
use emusim_xr::input::{ControllerPose, QuestControllerInput, XrFrameInput};
use glam::Vec3;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub fn run_desktop_app(
    initial_console: crate::world::SelectedConsole,
    screenshot_target: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting EmuSim Desktop Window with active console: {}", initial_console.display_name());

    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("EmuSim - Virtual Retro Room & CRT Experience")
            .with_inner_size(LogicalSize::new(1280, 960))
            .build(&event_loop)?,
    );

    // Initialize cross-platform GPU renderer
    let mut renderer = pollster::block_on(WgpuCrtRenderer::new(window.clone()))
        .map_err(|e| format!("Failed to create wgpu renderer: {e}"))?;

    // Initialize cross-platform audio engine
    let audio_engine = match AudioOutputEngine::new() {
        Ok(engine) => {
            info!("Cross-platform audio output engine started successfully");
            Some(engine)
        }
        Err(e) => {
            error!("Audio engine initialization skipped: {e}");
            None
        }
    };

    let mut scene = RetroRoomScene::new();
    let mut controller = DesktopFirstPersonController::new();
    let mut crt_noise_gen = CrtStaticAudioGenerator::new(48000.0);

    // Wire up the chosen default console
    scene.wire_console_to_tv(initial_console);

    let mut last_frame_time = Instant::now();
    let mut is_direct_fullscreen = false;
    let mut active_console = initial_console;

    let update_title = |win: &winit::window::Window, mode: DesktopPlayMode, direct_fs: bool, console: crate::world::SelectedConsole| {
        let console_label = console.display_name();
        if direct_fs {
            win.set_title(&format!("EmuSim [DIRECT 2D FULLSCREEN | {}] - F11: 3D Room | F12: Screenshot | 1/2/3: Consoles | 0: Static", console_label));
        } else {
            match mode {
                DesktopPlayMode::GameFocus => {
                    win.set_title(&format!("EmuSim [GAME FOCUS | {}] - Gamepad Active | Tab: Stand Up | F12: Screenshot | 1/2/3: Consoles | 0: Static | F11: 2D Fullscreen", console_label));
                }
                DesktopPlayMode::RoomExploration => {
                    win.set_title(&format!("EmuSim [3D RETRO ROOM | {}] - WASD: Walk | Mouse: Look | F12: Screenshot | Tab: Sit at TV | 1/2/3: Consoles | 0: Static", console_label));
                }
            }
        }
    };
    update_title(&window, controller.mode, is_direct_fullscreen, active_console);

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::Resized(new_size) => {
                        renderer.resize(new_size.width, new_size.height);
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        let size = window.inner_size();
                        renderer.resize(size.width, size.height);
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == winit::event::ElementState::Pressed {
                            match event.physical_key {
                                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F11) => {
                                    is_direct_fullscreen = !is_direct_fullscreen;
                                    update_title(&window, controller.mode, is_direct_fullscreen, active_console);
                                }
                                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F12) => {
                                    let dest = match screenshot_target {
                                        Some(ref path) => path.clone(),
                                        None => {
                                            std::fs::create_dir_all("screenshots").ok();
                                            let now = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_secs();
                                            format!("screenshots/screenshot_{}.png", now)
                                        }
                                    };
                                    if let Some(parent) = std::path::Path::new(&dest).parent() {
                                        std::fs::create_dir_all(parent).ok();
                                    }
                                    let output = std::process::Command::new("spectacle")
                                        .args(["-b", "-n", "-e", "-S", "-a", "-o", &dest])
                                        .output();
                                    match output {
                                        Ok(_) => info!("Screenshot successfully captured to {}", dest),
                                        Err(e) => error!("Failed to execute spectacle for screenshot: {e}"),
                                    }
                                }
                                _ => {}
                            }
                        }

                        let prev_mode = controller.mode;
                        controller.handle_keyboard_input(&event);
                        if controller.mode != prev_mode {
                            update_title(&window, controller.mode, is_direct_fullscreen, active_console);
                        }
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        controller.handle_mouse_button(button, state);
                    }
                    WindowEvent::RedrawRequested => {
                        let now = Instant::now();
                        let dt = (now - last_frame_time).as_secs_f32().min(0.05);
                        last_frame_time = now;

                        // Update desktop controller
                        controller.update(dt);

                        // Raycast interactions when in RoomExploration
                        if controller.mode == DesktopPlayMode::RoomExploration && controller.is_interact_pressed {
                            // Pointing at TV toggles TV power or switches input
                            let forward = controller.forward_ray();
                            let to_tv = Vec3::new(0.0, 0.75, -1.8) - controller.camera_pos;
                            if forward.dot(to_tv.normalize_or_zero()) > 0.85 {
                                if let Some(tv) = scene.graph.televisions.get_mut("crt_tv_1") {
                                    tv.cycle_input();
                                    info!("Toggled TV input: {:?}", tv.input_source.display_label());
                                }
                            }
                            controller.is_interact_pressed = false;
                        }

                        // Handle console hotkey switching (1 = N64, 2 = PS1, 3 = PS2, 0 = None)
                        if let Some(console) = controller.switch_to_console.take() {
                            scene.wire_console_to_tv(console);
                            active_console = console;
                            update_title(&window, controller.mode, is_direct_fullscreen, active_console);
                        }

                        // Always forward retro gamepad input to emulator
                        scene.emulator_worker.send_input(controller.retro_gamepad);

                        // Feed inputs into scene
                        let xr_input = XrFrameInput {
                            head_pose: ControllerPose {
                                position: controller.camera_pos,
                                rotation: controller.camera_rotation(),
                                ..Default::default()
                            },
                            left_controller: QuestControllerInput::default(),
                            right_controller: QuestControllerInput::default(),
                            delta_time: dt,
                        };

                        scene.update(dt, &xr_input);

                        // Stream 3D spatial audio
                        let tv_feed = scene.graph.evaluate_tv_screen("crt_tv_1");
                        if let Some(ref audio) = audio_engine {
                            let listener = VrListener {
                                position: controller.camera_pos,
                                rotation: controller.camera_rotation(),
                            };

                            match tv_feed {
                                TvScreenFeed::StaticNoise => {
                                    // Discard console audio when TV is showing static
                                    while scene.emulator_worker.audio_receiver.try_recv().is_ok() {}

                                    if let Some(tv) = scene.graph.televisions.get("crt_tv_1") {
                                        if tv.power_on && !tv.muted {
                                            let vol = (tv.volume as f32 / 100.0) * 0.35;
                                            let frames = ((dt * 48000.0) as usize).clamp(128, 1024);
                                            let mut noise_samples = crt_noise_gen.generate_stereo_batch(frames, vol);
                                            audio.push_spatial_samples(&mut noise_samples, &scene.tv_spatial_audio, &listener);
                                        }
                                    }
                                }
                                TvScreenFeed::ActiveVideo { .. } => {
                                    let mut got_samples = false;
                                    let tv_opt = scene.graph.televisions.get("crt_tv_1");
                                    let is_audible = tv_opt.map(|tv| tv.power_on && !tv.muted).unwrap_or(true);
                                    let tv_vol = tv_opt.map(|tv| tv.volume as f32 / 100.0).unwrap_or(1.0);
                                    scene.tv_spatial_audio.gain = tv_vol;

                                    while let Ok(mut samples) = scene.emulator_worker.audio_receiver.try_recv() {
                                        if is_audible {
                                            audio.push_spatial_samples(&mut samples, &scene.tv_spatial_audio, &listener);
                                        }
                                        got_samples = true;
                                    }
                                    // If console is on standby screen (core not loaded), emit subtle CRT speaker hum
                                    if !got_samples && scene.active_platform.is_none() && is_audible {
                                        let vol = tv_vol * 0.05;
                                        let frames = ((dt * 48000.0) as usize).clamp(128, 512);
                                        let mut hum_samples = crt_noise_gen.generate_stereo_batch(frames, vol);
                                        audio.push_spatial_samples(&mut hum_samples, &scene.tv_spatial_audio, &listener);
                                    }
                                }
                                TvScreenFeed::PoweredOff => {
                                    while scene.emulator_worker.audio_receiver.try_recv().is_ok() {}
                                }
                            }
                        } else {
                            while scene.emulator_worker.audio_receiver.try_recv().is_ok() {}
                        }

                        // Build dynamic cable mesh from scene.cable_physics
                        let mut dynamic_cables = emusim_render::CpuMesh::default();
                        for (cable_id, strand) in &scene.cable_physics {
                            let color_rgb = if cable_id.contains("yellow") || cable_id.contains("video") {
                                [235, 205, 30] // Yellow RCA
                            } else if cable_id.contains("white") || cable_id.contains("audio_l") {
                                [230, 230, 235] // White RCA
                            } else if cable_id.contains("red") || cable_id.contains("audio_r") {
                                [220, 35, 35] // Red RCA
                            } else {
                                [32, 32, 35] // Black power cord
                            };
                            let cable_mesh = emusim_render::CpuMesh::from_cable_strand(strand, 6, color_rgb);
                            dynamic_cables.append(&cable_mesh);
                        }

                        let crt_glow_color = match tv_feed {
                            TvScreenFeed::ActiveVideo { .. } => {
                                let fade = scene.crt_uniforms.power_fade;
                                [0.40 * fade, 0.50 * fade, 0.75 * fade, 1.0]
                            }
                            TvScreenFeed::StaticNoise => {
                                let fade = scene.crt_uniforms.power_fade;
                                [0.45 * fade, 0.45 * fade, 0.48 * fade, 1.0]
                            }
                            TvScreenFeed::PoweredOff => [0.0, 0.0, 0.0, 0.0],
                        };

                        // Render 3D Retro Room and CRT TV
                        let frame = scene.emulator_worker.video_buffer.read_frame();
                        let render_ctx = emusim_render::Render3dContext {
                            video_frame: &frame,
                            crt_uniforms: &scene.crt_uniforms,
                            camera_pos: controller.camera_pos,
                            camera_rot: controller.camera_rotation(),
                            fov_y_degrees: 60.0,
                            is_direct_fullscreen,
                            time_seconds: scene.elapsed_time,
                            crt_glow_color,
                            dynamic_cables: Some(&dynamic_cables),
                        };

                        if let Err(e) = renderer.render_3d(&render_ctx) {
                            match e {
                                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                                    let size = window.inner_size();
                                    renderer.resize(size.width, size.height);
                                }
                                wgpu::SurfaceError::OutOfMemory => elwt.exit(),
                                _ => {}
                            }
                        }

                        window.request_redraw();
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent {
                event: winit::event::DeviceEvent::MouseMotion { delta },
                ..
            } => {
                controller.handle_mouse_motion(delta.0, delta.1);
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}
