use std::os::raw::{c_char, c_uint, c_void};

pub const RETRO_API_VERSION: c_uint = 1;

// Environment Callbacks
pub const RETRO_ENVIRONMENT_SET_ROTATION: c_uint = 1;
pub const RETRO_ENVIRONMENT_GET_OVERSCAN: c_uint = 2;
pub const RETRO_ENVIRONMENT_GET_CAN_DUPE: c_uint = 3;
pub const RETRO_ENVIRONMENT_SET_MESSAGE: c_uint = 6;
pub const RETRO_ENVIRONMENT_SHUTDOWN: c_uint = 7;
pub const RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL: c_uint = 8;
pub const RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY: c_uint = 9;
pub const RETRO_ENVIRONMENT_SET_PIXEL_FORMAT: c_uint = 10;
pub const RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS: c_uint = 11;
pub const RETRO_ENVIRONMENT_SET_KEYBOARD_CALLBACK: c_uint = 12;
pub const RETRO_ENVIRONMENT_SET_DISK_CONTROL_INTERFACE: c_uint = 13;
pub const RETRO_ENVIRONMENT_SET_HW_RENDER: c_uint = 14;
pub const RETRO_ENVIRONMENT_GET_VARIABLE: c_uint = 15;
pub const RETRO_ENVIRONMENT_SET_VARIABLES: c_uint = 16;
pub const RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE: c_uint = 17;
pub const RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME: c_uint = 18;
pub const RETRO_ENVIRONMENT_GET_LIBRETRO_PATH: c_uint = 19;
pub const RETRO_ENVIRONMENT_SET_FRAME_TIME_CALLBACK: c_uint = 21;
pub const RETRO_ENVIRONMENT_SET_AUDIO_CALLBACK: c_uint = 22;
pub const RETRO_ENVIRONMENT_GET_RUMBLE_INTERFACE: c_uint = 23;
pub const RETRO_ENVIRONMENT_GET_INPUT_DEVICE_CAPABILITIES: c_uint = 24;
pub const RETRO_ENVIRONMENT_GET_SENSOR_INTERFACE: c_uint = 25;
pub const RETRO_ENVIRONMENT_GET_CAMERA_INTERFACE: c_uint = 26;
pub const RETRO_ENVIRONMENT_GET_LOG_INTERFACE: c_uint = 27;
pub const RETRO_ENVIRONMENT_GET_PERF_INTERFACE: c_uint = 28;
pub const RETRO_ENVIRONMENT_GET_LOCATION_INTERFACE: c_uint = 29;
pub const RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY: c_uint = 30;
pub const RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY: c_uint = 31;
pub const RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO: c_uint = 32;
pub const RETRO_ENVIRONMENT_SET_PROC_ADDRESS_CALLBACK: c_uint = 33;
pub const RETRO_ENVIRONMENT_SET_SUBSYSTEM_INFO: c_uint = 34;
pub const RETRO_ENVIRONMENT_SET_CONTROLLER_INFO: c_uint = 35;
pub const RETRO_ENVIRONMENT_SET_MEMORY_MAPS: c_uint = 36;
pub const RETRO_ENVIRONMENT_SET_GEOMETRY: c_uint = 37;
pub const RETRO_ENVIRONMENT_GET_CURRENT_SOFTWARE_FRAMEBUFFER: c_uint = 39;
pub const RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE: c_uint = 41;
pub const RETRO_ENVIRONMENT_SET_SUPPORT_ACHIEVEMENTS: c_uint = 42;
pub const RETRO_ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE: c_uint = 43;
pub const RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS: c_uint = 44;
pub const RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT: c_uint = 45;
pub const RETRO_ENVIRONMENT_GET_AUDIO_VIDEO_ENABLE: c_uint = 47;
pub const RETRO_ENVIRONMENT_GET_INPUT_BITMASKS: c_uint = 51;
pub const RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION: c_uint = 52;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS: c_uint = 53;

// Pixel Formats
pub const RETRO_PIXEL_FORMAT_0RGB1555: c_uint = 0;
pub const RETRO_PIXEL_FORMAT_XRGB8888: c_uint = 1;
pub const RETRO_PIXEL_FORMAT_RGB565: c_uint = 2;

// Devices
pub const RETRO_DEVICE_NONE: c_uint = 0;
pub const RETRO_DEVICE_JOYPAD: c_uint = 1;
pub const RETRO_DEVICE_MOUSE: c_uint = 2;
pub const RETRO_DEVICE_KEYBOARD: c_uint = 3;
pub const RETRO_DEVICE_LIGHTGUN: c_uint = 4;
pub const RETRO_DEVICE_ANALOG: c_uint = 5;
pub const RETRO_DEVICE_POINTER: c_uint = 6;

// Analog indices
pub const RETRO_DEVICE_INDEX_ANALOG_LEFT: c_uint = 0;
pub const RETRO_DEVICE_INDEX_ANALOG_RIGHT: c_uint = 1;
pub const RETRO_DEVICE_INDEX_ANALOG_BUTTON: c_uint = 2;

pub const RETRO_DEVICE_ID_ANALOG_X: c_uint = 0;
pub const RETRO_DEVICE_ID_ANALOG_Y: c_uint = 1;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RetroGameInfo {
    pub path: *const c_char,
    pub data: *const c_void,
    pub size: usize,
    pub meta: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RetroSystemInfo {
    pub library_name: *const c_char,
    pub library_version: *const c_char,
    pub valid_extensions: *const c_char,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RetroGameGeometry {
    pub base_width: c_uint,
    pub base_height: c_uint,
    pub max_width: c_uint,
    pub max_height: c_uint,
    pub aspect_ratio: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RetroSystemTiming {
    pub fps: f64,
    pub sample_rate: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RetroSystemAvInfo {
    pub geometry: RetroGameGeometry,
    pub timing: RetroSystemTiming,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RetroVariable {
    pub key: *const c_char,
    pub value: *const c_char,
}

// Function pointer signatures defined by Libretro
pub type RetroEnvironmentFn = unsafe extern "C" fn(cmd: c_uint, data: *mut c_void) -> bool;
pub type RetroVideoRefreshFn = unsafe extern "C" fn(
    data: *const c_void,
    width: c_uint,
    height: c_uint,
    pitch: usize,
);
pub type RetroAudioSampleFn = unsafe extern "C" fn(left: i16, right: i16);
pub type RetroAudioSampleBatchFn = unsafe extern "C" fn(data: *const i16, frames: usize) -> usize;
pub type RetroInputPollFn = unsafe extern "C" fn();
pub type RetroInputStateFn = unsafe extern "C" fn(
    port: c_uint,
    device: c_uint,
    index: c_uint,
    id: c_uint,
) -> i16;
