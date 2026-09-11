use crate::framebuffer::SharedVideoBuffer;
use crate::sys::*;
use emusim_core::devices::UnifiedGamepadState;
use libloading::{Library, Symbol};
use parking_lot::Mutex;
use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::{c_uint, c_void};
use std::path::Path;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Failed to dynamically load library: {0}")]
    LoadError(#[from] libloading::Error),
    #[error("Missing symbol '{0}' in Libretro core")]
    MissingSymbol(&'static str),
    #[error("Game load failed in core")]
    GameLoadFailed,
}

pub struct LibretroSymbols {
    pub retro_init: unsafe extern "C" fn(),
    pub retro_deinit: unsafe extern "C" fn(),
    pub retro_api_version: unsafe extern "C" fn() -> c_uint,
    pub retro_get_system_info: unsafe extern "C" fn(info: *mut RetroSystemInfo),
    pub retro_get_system_av_info: unsafe extern "C" fn(info: *mut RetroSystemAvInfo),
    pub retro_set_environment: unsafe extern "C" fn(cb: RetroEnvironmentFn),
    pub retro_set_video_refresh: unsafe extern "C" fn(cb: RetroVideoRefreshFn),
    pub retro_set_audio_sample: unsafe extern "C" fn(cb: RetroAudioSampleFn),
    pub retro_set_audio_sample_batch: unsafe extern "C" fn(cb: RetroAudioSampleBatchFn),
    pub retro_set_input_poll: unsafe extern "C" fn(cb: RetroInputPollFn),
    pub retro_set_input_state: unsafe extern "C" fn(cb: RetroInputStateFn),
    pub retro_load_game: unsafe extern "C" fn(game: *const RetroGameInfo) -> bool,
    pub retro_unload_game: unsafe extern "C" fn(),
    pub retro_run: unsafe extern "C" fn(),
    pub retro_reset: unsafe extern "C" fn(),
}

pub struct ActiveCoreContext {
    pub video_buffer: SharedVideoBuffer,
    pub audio_producer: crossbeam_channel::Sender<Vec<i16>>,
    pub pixel_format: AtomicU32,
    pub frame_counter: AtomicU64,
    pub gamepad: Mutex<UnifiedGamepadState>,
    pub base_width: u32,
    pub base_height: u32,
    pub target_fps: f64,
    pub sample_rate: f64,
}

// Thread-local pointer to active context during `retro_run()` execution
thread_local! {
    static CURRENT_CONTEXT: RefCell<Option<Arc<ActiveCoreContext>>> = const { RefCell::new(None) };
}

pub struct LibretroCoreInstance {
    _lib: Library,
    symbols: LibretroSymbols,
    context: Arc<ActiveCoreContext>,
    game_loaded: bool,
}

impl LibretroCoreInstance {
    pub fn load(
        core_path: &Path,
        video_buffer: SharedVideoBuffer,
        audio_producer: crossbeam_channel::Sender<Vec<i16>>,
    ) -> Result<Self, CoreError> {
        unsafe {
            let lib = Library::new(core_path)?;

            let retro_init: Symbol<unsafe extern "C" fn()> =
                lib.get(b"retro_init\0")?;
            let retro_deinit: Symbol<unsafe extern "C" fn()> =
                lib.get(b"retro_deinit\0")?;
            let retro_api_version: Symbol<unsafe extern "C" fn() -> c_uint> =
                lib.get(b"retro_api_version\0")?;
            let retro_get_system_info: Symbol<unsafe extern "C" fn(*mut RetroSystemInfo)> =
                lib.get(b"retro_get_system_info\0")?;
            let retro_get_system_av_info: Symbol<unsafe extern "C" fn(*mut RetroSystemAvInfo)> =
                lib.get(b"retro_get_system_av_info\0")?;
            let retro_set_environment: Symbol<unsafe extern "C" fn(RetroEnvironmentFn)> =
                lib.get(b"retro_set_environment\0")?;
            let retro_set_video_refresh: Symbol<unsafe extern "C" fn(RetroVideoRefreshFn)> =
                lib.get(b"retro_set_video_refresh\0")?;
            let retro_set_audio_sample: Symbol<unsafe extern "C" fn(RetroAudioSampleFn)> =
                lib.get(b"retro_set_audio_sample\0")?;
            let retro_set_audio_sample_batch: Symbol<unsafe extern "C" fn(RetroAudioSampleBatchFn)> =
                lib.get(b"retro_set_audio_sample_batch\0")?;
            let retro_set_input_poll: Symbol<unsafe extern "C" fn(RetroInputPollFn)> =
                lib.get(b"retro_set_input_poll\0")?;
            let retro_set_input_state: Symbol<unsafe extern "C" fn(RetroInputStateFn)> =
                lib.get(b"retro_set_input_state\0")?;
            let retro_load_game: Symbol<unsafe extern "C" fn(*const RetroGameInfo) -> bool> =
                lib.get(b"retro_load_game\0")?;
            let retro_unload_game: Symbol<unsafe extern "C" fn()> =
                lib.get(b"retro_unload_game\0")?;
            let retro_run: Symbol<unsafe extern "C" fn()> =
                lib.get(b"retro_run\0")?;
            let retro_reset: Symbol<unsafe extern "C" fn()> =
                lib.get(b"retro_reset\0")?;

            let symbols = LibretroSymbols {
                retro_init: *retro_init,
                retro_deinit: *retro_deinit,
                retro_api_version: *retro_api_version,
                retro_get_system_info: *retro_get_system_info,
                retro_get_system_av_info: *retro_get_system_av_info,
                retro_set_environment: *retro_set_environment,
                retro_set_video_refresh: *retro_set_video_refresh,
                retro_set_audio_sample: *retro_set_audio_sample,
                retro_set_audio_sample_batch: *retro_set_audio_sample_batch,
                retro_set_input_poll: *retro_set_input_poll,
                retro_set_input_state: *retro_set_input_state,
                retro_load_game: *retro_load_game,
                retro_unload_game: *retro_unload_game,
                retro_run: *retro_run,
                retro_reset: *retro_reset,
            };

            let context = Arc::new(ActiveCoreContext {
                video_buffer,
                audio_producer,
                pixel_format: AtomicU32::new(RETRO_PIXEL_FORMAT_0RGB1555),
                frame_counter: AtomicU64::new(0),
                gamepad: Mutex::new(UnifiedGamepadState::default()),
                base_width: 320,
                base_height: 240,
                target_fps: 60.0,
                sample_rate: 44100.0,
            });

            // Set callbacks
            CURRENT_CONTEXT.with(|c| *c.borrow_mut() = Some(context.clone()));
            (symbols.retro_set_environment)(core_environment_callback);
            (symbols.retro_set_video_refresh)(core_video_refresh_callback);
            (symbols.retro_set_audio_sample)(core_audio_sample_callback);
            (symbols.retro_set_audio_sample_batch)(core_audio_sample_batch_callback);
            (symbols.retro_set_input_poll)(core_input_poll_callback);
            (symbols.retro_set_input_state)(core_input_state_callback);
            (symbols.retro_init)();

            Ok(Self {
                _lib: lib,
                symbols,
                context,
                game_loaded: false,
            })
        }
    }

    pub fn load_game(&mut self, rom_path: &Path, rom_data: Option<&[u8]>) -> Result<(), CoreError> {
        let path_c = CString::new(rom_path.to_string_lossy().as_bytes()).unwrap();

        let (data_ptr, size) = if let Some(d) = rom_data {
            (d.as_ptr() as *const c_void, d.len())
        } else {
            (std::ptr::null(), 0)
        };

        let game_info = RetroGameInfo {
            path: path_c.as_ptr(),
            data: data_ptr,
            size,
            meta: std::ptr::null(),
        };

        CURRENT_CONTEXT.with(|c| *c.borrow_mut() = Some(self.context.clone()));
        let ok = unsafe { (self.symbols.retro_load_game)(&game_info) };
        if !ok {
            return Err(CoreError::GameLoadFailed);
        }

        self.game_loaded = true;
        Ok(())
    }

    pub fn update_gamepad(&self, gamepad: UnifiedGamepadState) {
        *self.context.gamepad.lock() = gamepad;
    }

    pub fn run_frame(&self) {
        if self.game_loaded {
            CURRENT_CONTEXT.with(|c| *c.borrow_mut() = Some(self.context.clone()));
            unsafe {
                (self.symbols.retro_run)();
            }
        }
    }

    pub fn reset(&self) {
        if self.game_loaded {
            CURRENT_CONTEXT.with(|c| *c.borrow_mut() = Some(self.context.clone()));
            unsafe {
                (self.symbols.retro_reset)();
            }
        }
    }
}

impl Drop for LibretroCoreInstance {
    fn drop(&mut self) {
        CURRENT_CONTEXT.with(|c| *c.borrow_mut() = Some(self.context.clone()));
        unsafe {
            if self.game_loaded {
                (self.symbols.retro_unload_game)();
            }
            (self.symbols.retro_deinit)();
        }
        CURRENT_CONTEXT.with(|c| *c.borrow_mut() = None);
    }
}

// C Callbacks redirected via thread-local context
unsafe extern "C" fn core_environment_callback(cmd: c_uint, data: *mut c_void) -> bool {
    CURRENT_CONTEXT.with(|ctx_cell| {
        let borrowed = ctx_cell.borrow();
        let ctx = match borrowed.as_ref() {
            Some(c) => c,
            None => return false,
        };

        match cmd {
            RETRO_ENVIRONMENT_SET_PIXEL_FORMAT => {
                if !data.is_null() {
                    let fmt = *(data as *const c_uint);
                    ctx.pixel_format.store(fmt, Ordering::Relaxed);
                    return true;
                }
                false
            }
            RETRO_ENVIRONMENT_GET_CAN_DUPE => {
                if !data.is_null() {
                    *(data as *mut bool) = true;
                    return true;
                }
                false
            }
            RETRO_ENVIRONMENT_GET_VARIABLE => {
                // Return default options for cores
                false
            }
            _ => false,
        }
    })
}

unsafe extern "C" fn core_video_refresh_callback(
    data: *const c_void,
    width: c_uint,
    height: c_uint,
    pitch: usize,
) {
    if data.is_null() {
        return;
    }

    CURRENT_CONTEXT.with(|ctx_cell| {
        let borrowed = ctx_cell.borrow();
        if let Some(ctx) = borrowed.as_ref() {
            let fmt = ctx.pixel_format.load(Ordering::Relaxed);
            let frame_idx = ctx.frame_counter.fetch_add(1, Ordering::Relaxed);
            ctx.video_buffer.update_from_raw(
                data as *const u8,
                width,
                height,
                pitch,
                fmt,
                frame_idx,
            );
        }
    });
}

unsafe extern "C" fn core_audio_sample_callback(left: i16, right: i16) {
    CURRENT_CONTEXT.with(|ctx_cell| {
        let borrowed = ctx_cell.borrow();
        if let Some(ctx) = borrowed.as_ref() {
            let _ = ctx.audio_producer.try_send(vec![left, right]);
        }
    });
}

unsafe extern "C" fn core_audio_sample_batch_callback(data: *const i16, frames: usize) -> usize {
    if data.is_null() || frames == 0 {
        return 0;
    }

    CURRENT_CONTEXT.with(|ctx_cell| {
        let borrowed = ctx_cell.borrow();
        if let Some(ctx) = borrowed.as_ref() {
            let slice = std::slice::from_raw_parts(data, frames * 2);
            let _ = ctx.audio_producer.try_send(slice.to_vec());
        }
    });

    frames
}

unsafe extern "C" fn core_input_poll_callback() {
    // Gamepad state is updated asynchronously by the main/VR loop
}

unsafe extern "C" fn core_input_state_callback(
    port: c_uint,
    device: c_uint,
    index: c_uint,
    id: c_uint,
) -> i16 {
    if port != 0 {
        return 0;
    }

    CURRENT_CONTEXT.with(|ctx_cell| {
        let borrowed = ctx_cell.borrow();
        let ctx = match borrowed.as_ref() {
            Some(c) => c,
            None => return 0,
        };

        let gp = ctx.gamepad.lock();
        match device {
            RETRO_DEVICE_JOYPAD => {
                if (gp.buttons & (1 << id)) != 0 {
                    1
                } else {
                    0
                }
            }
            RETRO_DEVICE_ANALOG => {
                match (index, id) {
                    (RETRO_DEVICE_INDEX_ANALOG_LEFT, RETRO_DEVICE_ID_ANALOG_X) => gp.left_analog_x,
                    (RETRO_DEVICE_INDEX_ANALOG_LEFT, RETRO_DEVICE_ID_ANALOG_Y) => gp.left_analog_y,
                    (RETRO_DEVICE_INDEX_ANALOG_RIGHT, RETRO_DEVICE_ID_ANALOG_X) => gp.right_analog_x,
                    (RETRO_DEVICE_INDEX_ANALOG_RIGHT, RETRO_DEVICE_ID_ANALOG_Y) => gp.right_analog_y,
                    _ => 0,
                }
            }
            _ => 0,
        }
    })
}
