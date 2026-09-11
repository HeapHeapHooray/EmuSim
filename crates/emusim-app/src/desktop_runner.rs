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

pub fn run_desktop_app() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting EmuSim Desktop Window...");

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

    // Default wiring setup for immediate desktop play convenience
    info!("Wiring default circuit: Power Strip -> TV & N64, Multi-Out AV -> TV AV1");
    scene
        .graph
        .connect("power_strip_1_cord_plug", "wall_outlet_1_top");
    scene.graph.connect("tv_power_1_wall", "power_strip_1_outlet_1");
    scene.graph.connect("tv_power_1_c7", "crt_tv_1_power_in");
    scene
        .graph
        .connect("n64_power_1_wall", "power_strip_1_outlet_2");
    scene
        .graph
        .connect("n64_power_1_n64plug", "n64_console_1_power_in");
    scene
        .graph
        .connect("n64_av_cable_1_multiout", "n64_console_1_multi_out");
    scene
        .graph
        .connect("n64_av_cable_1_rca_yellow", "crt_tv_1_av1_video");
    scene
        .graph
        .connect("n64_av_cable_1_rca_white", "crt_tv_1_av1_audio_l");
    scene
        .graph
        .connect("n64_av_cable_1_rca_red", "crt_tv_1_av1_audio_r");

    // Turn ON CRT TV and N64
    if let Some(tv) = scene.graph.televisions.get_mut("crt_tv_1") {
        tv.toggle_power();
    }
    if let Some(n64) = scene.graph.n64_consoles.get_mut("n64_console_1") {
        n64.set_power_switch(true);
    }

    let mut last_frame_time = Instant::now();

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
                        controller.handle_keyboard_input(&event);
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

                        if controller.mode == DesktopPlayMode::GameFocus {
                            scene.emulator_worker.send_input(controller.retro_gamepad);
                        }

                        scene.update(dt, &xr_input);

                        // Stream 3D spatial audio
                        if let Some(ref audio) = audio_engine {
                            let listener = VrListener {
                                position: controller.camera_pos,
                                rotation: controller.camera_rotation(),
                            };

                            let tv_feed = scene.graph.evaluate_tv_screen("crt_tv_1");
                            match tv_feed {
                                TvScreenFeed::StaticNoise => {
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
                                    while let Ok(mut samples) = scene.emulator_worker.audio_receiver.try_recv() {
                                        audio.push_spatial_samples(&mut samples, &scene.tv_spatial_audio, &listener);
                                    }
                                }
                                TvScreenFeed::PoweredOff => {}
                            }
                        }

                        // Render CRT screen quad
                        let frame = scene.emulator_worker.video_buffer.read_frame();
                        if let Err(e) = renderer.render(&frame, &scene.crt_uniforms) {
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
