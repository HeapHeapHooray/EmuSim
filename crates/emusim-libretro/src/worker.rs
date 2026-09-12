use crate::core::LibretroCoreInstance;
use crate::framebuffer::SharedVideoBuffer;
use crossbeam_channel::{Receiver, Sender};
use emusim_core::devices::UnifiedGamepadState;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tracing::{error, info};

pub enum EmulatorCommand {
    LoadCore {
        core_path: PathBuf,
        rom_path: Option<PathBuf>,
    },
    UpdateGamepad(UnifiedGamepadState),
    Reset,
    Pause,
    Resume,
    Stop,
}

pub struct EmulatorWorkerHandle {
    command_sender: Sender<EmulatorCommand>,
    pub video_buffer: SharedVideoBuffer,
    pub audio_receiver: Receiver<Vec<i16>>,
    is_running: Arc<AtomicBool>,
    worker_thread: Option<JoinHandle<()>>,
}

impl EmulatorWorkerHandle {
    pub fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded::<EmulatorCommand>();
        let (audio_tx, audio_rx) = crossbeam_channel::bounded::<Vec<i16>>(512);
        let video_buffer = SharedVideoBuffer::new();
        let is_running = Arc::new(AtomicBool::new(true));

        let worker_video = video_buffer.clone();
        let worker_running = is_running.clone();

        let handle = thread::Builder::new()
            .name("emusim-worker".into())
            .spawn(move || {
                run_emulator_loop(cmd_rx, audio_tx, worker_video, worker_running);
            })
            .expect("Failed to spawn emulator worker thread");

        Self {
            command_sender: cmd_tx,
            video_buffer,
            audio_receiver: audio_rx,
            is_running,
            worker_thread: Some(handle),
        }
    }

    pub fn load_game(&self, core_path: PathBuf, rom_path: Option<PathBuf>) {
        let _ = self
            .command_sender
            .send(EmulatorCommand::LoadCore { core_path, rom_path });
    }

    pub fn send_input(&self, gamepad: UnifiedGamepadState) {
        let _ = self
            .command_sender
            .send(EmulatorCommand::UpdateGamepad(gamepad));
    }

    pub fn reset(&self) {
        let _ = self.command_sender.send(EmulatorCommand::Reset);
    }

    pub fn pause(&self) {
        let _ = self.command_sender.send(EmulatorCommand::Pause);
    }

    pub fn resume(&self) {
        let _ = self.command_sender.send(EmulatorCommand::Resume);
    }

    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        let _ = self.command_sender.send(EmulatorCommand::Stop);
        if let Some(h) = self.worker_thread.take() {
            let _ = h.join();
        }
    }
}

impl Drop for EmulatorWorkerHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run_emulator_loop(
    cmd_rx: Receiver<EmulatorCommand>,
    audio_tx: Sender<Vec<i16>>,
    video_buffer: SharedVideoBuffer,
    is_running: Arc<AtomicBool>,
) {
    let mut current_core: Option<LibretroCoreInstance> = None;
    let mut paused = false;
    let target_frame_duration = Duration::from_nanos(16_666_667); // 60.0 FPS

    while is_running.load(Ordering::Relaxed) {
        let frame_start = Instant::now();

        // Process incoming commands
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                EmulatorCommand::LoadCore { core_path, rom_path } => {
                    info!("Loading core {:?} with ROM {:?}", core_path, rom_path);
                    current_core = None; // drop old core first

                    match LibretroCoreInstance::load(&core_path, video_buffer.clone(), audio_tx.clone()) {
                        Ok(mut core) => {
                            let result = match rom_path {
                                Some(ref p) => core.load_game(p, None),
                                None => core.load_no_game(),
                            };
                            if let Err(e) = result {
                                error!("Failed to load game/bios in core: {e}");
                            } else {
                                info!("Core initialized and emulation started successfully");
                                current_core = Some(core);
                            }
                        }
                        Err(e) => {
                            error!("Failed to load Libretro core library: {e}");
                        }
                    }
                }
                EmulatorCommand::UpdateGamepad(gp) => {
                    if let Some(core) = &current_core {
                        core.update_gamepad(gp);
                    }
                }
                EmulatorCommand::Reset => {
                    if let Some(core) = &current_core {
                        core.reset();
                    }
                }
                EmulatorCommand::Pause => {
                    paused = true;
                }
                EmulatorCommand::Resume => {
                    paused = false;
                }
                EmulatorCommand::Stop => {
                    return;
                }
            }
        }

        // Run one frame of emulation
        if !paused {
            if let Some(core) = &current_core {
                core.run_frame();
            }
        }

        // Frame pacing (60 FPS)
        let elapsed = frame_start.elapsed();
        if elapsed < target_frame_duration {
            thread::sleep(target_frame_duration - elapsed);
        }
    }
}
