use crate::sys::{
    RETRO_PIXEL_FORMAT_0RGB1555, RETRO_PIXEL_FORMAT_RGB565, RETRO_PIXEL_FORMAT_XRGB8888,
};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    /// Linear RGBA8888 pixels
    pub pixels: Vec<u8>,
    pub frame_index: u64,
}

impl Default for VideoFrame {
    fn default() -> Self {
        Self {
            width: 320,
            height: 240,
            pixels: vec![0; 320 * 240 * 4],
            frame_index: 0,
        }
    }
}

/// Thread-safe shared video frame buffer.
#[derive(Clone)]
pub struct SharedVideoBuffer {
    inner: Arc<RwLock<VideoFrame>>,
}

impl Default for SharedVideoBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedVideoBuffer {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(VideoFrame::default())),
        }
    }

    /// Read current frame for rendering.
    pub fn read_frame(&self) -> parking_lot::RwLockReadGuard<'_, VideoFrame> {
        self.inner.read()
    }

    /// Mutably access current frame for rendering OSD or standby screens.
    pub fn write_frame(&self) -> parking_lot::RwLockWriteGuard<'_, VideoFrame> {
        self.inner.write()
    }

    /// Update frame from raw Libretro callback buffer.
    ///
    /// # Safety
    /// `raw_data` must either be null or point to a readable memory buffer containing at
    /// least `height * pitch` bytes.
    pub unsafe fn update_from_raw(
        &self,
        raw_data: *const u8,
        width: u32,
        height: u32,
        pitch: usize,
        pixel_format: u32,
        frame_index: u64,
    ) {
        if raw_data.is_null() || width == 0 || height == 0 {
            return;
        }

        let total_pixels = (width * height) as usize;
        let mut frame = self.inner.write();

        frame.width = width;
        frame.height = height;
        frame.frame_index = frame_index;

        let required_bytes = total_pixels * 4;
        if frame.pixels.len() != required_bytes {
            frame.pixels.resize(required_bytes, 0);
        }

        let dst = frame.pixels.as_mut_slice();

        unsafe {
            match pixel_format {
                RETRO_PIXEL_FORMAT_XRGB8888 => {
                    // 32-bit: B, G, R, X in memory on little-endian
                    for y in 0..height {
                        let src_row = raw_data.add(y as usize * pitch) as *const u32;
                        let dst_row = &mut dst[(y * width * 4) as usize..((y + 1) * width * 4) as usize];
                        for x in 0..width {
                            let pixel = *src_row.add(x as usize);
                            let r = ((pixel >> 16) & 0xFF) as u8;
                            let g = ((pixel >> 8) & 0xFF) as u8;
                            let b = (pixel & 0xFF) as u8;
                            let idx = (x * 4) as usize;
                            dst_row[idx] = r;
                            dst_row[idx + 1] = g;
                            dst_row[idx + 2] = b;
                            dst_row[idx + 3] = 255;
                        }
                    }
                }
                RETRO_PIXEL_FORMAT_RGB565 => {
                    // 16-bit: RRRRRGGG GGGBBBBB
                    for y in 0..height {
                        let src_row = raw_data.add(y as usize * pitch) as *const u16;
                        let dst_row = &mut dst[(y * width * 4) as usize..((y + 1) * width * 4) as usize];
                        for x in 0..width {
                            let pixel = *src_row.add(x as usize);
                            let r = (((pixel >> 11) & 0x1F) * 255 / 31) as u8;
                            let g = (((pixel >> 5) & 0x3F) * 255 / 63) as u8;
                            let b = ((pixel & 0x1F) * 255 / 31) as u8;
                            let idx = (x * 4) as usize;
                            dst_row[idx] = r;
                            dst_row[idx + 1] = g;
                            dst_row[idx + 2] = b;
                            dst_row[idx + 3] = 255;
                        }
                    }
                }
                RETRO_PIXEL_FORMAT_0RGB1555 => {
                    // 16-bit: 0RRRRRGG GGGBBBBB
                    for y in 0..height {
                        let src_row = raw_data.add(y as usize * pitch) as *const u16;
                        let dst_row = &mut dst[(y * width * 4) as usize..((y + 1) * width * 4) as usize];
                        for x in 0..width {
                            let pixel = *src_row.add(x as usize);
                            let r = (((pixel >> 10) & 0x1F) * 255 / 31) as u8;
                            let g = (((pixel >> 5) & 0x1F) * 255 / 31) as u8;
                            let b = ((pixel & 0x1F) * 255 / 31) as u8;
                            let idx = (x * 4) as usize;
                            dst_row[idx] = r;
                            dst_row[idx + 1] = g;
                            dst_row[idx + 2] = b;
                            dst_row[idx + 3] = 255;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
