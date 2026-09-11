use emusim_core::media::Platform;
use emusim_libretro::framebuffer::VideoFrame;

/// Basic 5x7 ASCII bitmap font (characters 32 ' ' to 126 '~').
/// Each character is 5 bytes (columns). Bit 0 is top pixel, Bit 6 is bottom pixel.
const FONT_5X7: [[u8; 5]; 95] = [
    [0x00, 0x00, 0x00, 0x00, 0x00], // ' '
    [0x00, 0x00, 0x5f, 0x00, 0x00], // '!'
    [0x00, 0x07, 0x00, 0x07, 0x00], // '"'
    [0x14, 0x7f, 0x14, 0x7f, 0x14], // '#'
    [0x24, 0x2a, 0x7f, 0x2a, 0x12], // '$'
    [0x23, 0x13, 0x08, 0x64, 0x62], // '%'
    [0x36, 0x49, 0x55, 0x22, 0x50], // '&'
    [0x00, 0x05, 0x03, 0x00, 0x00], // '\''
    [0x00, 0x1c, 0x22, 0x41, 0x00], // '('
    [0x00, 0x41, 0x22, 0x1c, 0x00], // ')'
    [0x14, 0x08, 0x3e, 0x08, 0x14], // '*'
    [0x08, 0x08, 0x3e, 0x08, 0x08], // '+'
    [0x00, 0x50, 0x30, 0x00, 0x00], // ','
    [0x08, 0x08, 0x08, 0x08, 0x08], // '-'
    [0x00, 0x60, 0x60, 0x00, 0x00], // '.'
    [0x20, 0x10, 0x08, 0x04, 0x02], // '/'
    [0x3e, 0x51, 0x49, 0x45, 0x3e], // '0'
    [0x00, 0x42, 0x7f, 0x40, 0x00], // '1'
    [0x42, 0x61, 0x51, 0x49, 0x46], // '2'
    [0x21, 0x41, 0x45, 0x4b, 0x31], // '3'
    [0x18, 0x14, 0x12, 0x7f, 0x10], // '4'
    [0x27, 0x45, 0x45, 0x45, 0x39], // '5'
    [0x3c, 0x4a, 0x49, 0x49, 0x30], // '6'
    [0x01, 0x71, 0x09, 0x05, 0x03], // '7'
    [0x36, 0x49, 0x49, 0x49, 0x36], // '8'
    [0x06, 0x49, 0x49, 0x29, 0x1e], // '9'
    [0x00, 0x36, 0x36, 0x00, 0x00], // ':'
    [0x00, 0x56, 0x36, 0x00, 0x00], // ';'
    [0x08, 0x14, 0x22, 0x41, 0x00], // '<'
    [0x14, 0x14, 0x14, 0x14, 0x14], // '='
    [0x00, 0x41, 0x22, 0x14, 0x08], // '>'
    [0x02, 0x01, 0x51, 0x09, 0x06], // '?'
    [0x32, 0x49, 0x79, 0x41, 0x3e], // '@'
    [0x7e, 0x11, 0x11, 0x11, 0x7e], // 'A'
    [0x7f, 0x49, 0x49, 0x49, 0x36], // 'B'
    [0x3e, 0x41, 0x41, 0x41, 0x22], // 'C'
    [0x7f, 0x41, 0x41, 0x22, 0x1c], // 'D'
    [0x7f, 0x49, 0x49, 0x49, 0x41], // 'E'
    [0x7f, 0x09, 0x09, 0x09, 0x01], // 'F'
    [0x3e, 0x41, 0x49, 0x49, 0x7a], // 'G'
    [0x7f, 0x08, 0x08, 0x08, 0x7f], // 'H'
    [0x00, 0x41, 0x7f, 0x41, 0x00], // 'I'
    [0x20, 0x40, 0x41, 0x3f, 0x01], // 'J'
    [0x7f, 0x08, 0x14, 0x22, 0x41], // 'K'
    [0x7f, 0x40, 0x40, 0x40, 0x40], // 'L'
    [0x7f, 0x02, 0x0c, 0x02, 0x7f], // 'M'
    [0x7f, 0x04, 0x08, 0x10, 0x7f], // 'N'
    [0x3e, 0x41, 0x41, 0x41, 0x3e], // 'O'
    [0x7f, 0x09, 0x09, 0x09, 0x06], // 'P'
    [0x3e, 0x41, 0x51, 0x21, 0x5e], // 'Q'
    [0x7f, 0x09, 0x19, 0x29, 0x46], // 'R'
    [0x46, 0x49, 0x49, 0x49, 0x31], // 'S'
    [0x01, 0x01, 0x7f, 0x01, 0x01], // 'T'
    [0x3f, 0x40, 0x40, 0x40, 0x3f], // 'U'
    [0x1f, 0x20, 0x40, 0x20, 0x1f], // 'V'
    [0x7f, 0x20, 0x18, 0x20, 0x7f], // 'W'
    [0x63, 0x14, 0x08, 0x14, 0x63], // 'X'
    [0x07, 0x08, 0x70, 0x08, 0x07], // 'Y'
    [0x61, 0x51, 0x49, 0x45, 0x43], // 'Z'
    [0x00, 0x7f, 0x41, 0x41, 0x00], // '['
    [0x02, 0x04, 0x08, 0x10, 0x20], // '\'
    [0x00, 0x41, 0x41, 0x7f, 0x00], // ']'
    [0x04, 0x02, 0x01, 0x02, 0x04], // '^'
    [0x40, 0x40, 0x40, 0x40, 0x40], // '_'
    [0x00, 0x01, 0x02, 0x04, 0x00], // '`'
    [0x20, 0x54, 0x54, 0x54, 0x78], // 'a'
    [0x7f, 0x48, 0x44, 0x44, 0x38], // 'b'
    [0x38, 0x44, 0x44, 0x44, 0x20], // 'c'
    [0x38, 0x44, 0x44, 0x48, 0x7f], // 'd'
    [0x38, 0x54, 0x54, 0x54, 0x18], // 'e'
    [0x08, 0x7e, 0x09, 0x01, 0x02], // 'f'
    [0x0c, 0x52, 0x52, 0x52, 0x3e], // 'g'
    [0x7f, 0x08, 0x04, 0x04, 0x78], // 'h'
    [0x00, 0x44, 0x7d, 0x40, 0x00], // 'i'
    [0x20, 0x40, 0x44, 0x3d, 0x00], // 'j'
    [0x7f, 0x10, 0x28, 0x44, 0x00], // 'k'
    [0x00, 0x41, 0x7f, 0x40, 0x00], // 'l'
    [0x7c, 0x04, 0x18, 0x04, 0x78], // 'm'
    [0x7c, 0x08, 0x04, 0x04, 0x78], // 'n'
    [0x38, 0x44, 0x44, 0x44, 0x38], // 'o'
    [0x7c, 0x14, 0x14, 0x14, 0x08], // 'p'
    [0x08, 0x14, 0x14, 0x18, 0x7c], // 'q'
    [0x7c, 0x08, 0x04, 0x04, 0x08], // 'r'
    [0x48, 0x54, 0x54, 0x54, 0x20], // 's'
    [0x04, 0x3f, 0x44, 0x40, 0x20], // 't'
    [0x3c, 0x40, 0x40, 0x20, 0x7c], // 'u'
    [0x1c, 0x20, 0x40, 0x20, 0x1c], // 'v'
    [0x3c, 0x40, 0x30, 0x40, 0x3c], // 'w'
    [0x44, 0x28, 0x10, 0x28, 0x44], // 'x'
    [0x0c, 0x50, 0x50, 0x50, 0x3c], // 'y'
    [0x44, 0x64, 0x54, 0x4c, 0x44], // 'z'
    [0x00, 0x08, 0x36, 0x41, 0x00], // '{'
    [0x00, 0x00, 0x7f, 0x00, 0x00], // '|'
    [0x00, 0x41, 0x36, 0x08, 0x00], // '}'
    [0x08, 0x08, 0x2a, 0x1c, 0x08], // '~'
];

/// Draw a single character into an RGBA framebuffer.
pub fn draw_char(
    dst: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    scale: i32,
    c: char,
    color: [u8; 4],
) {
    let idx = (c as usize).saturating_sub(32);
    if idx >= FONT_5X7.len() {
        return;
    }

    let glyph = &FONT_5X7[idx];

    for col in 0..5 {
        let bits = glyph[col];
        for row in 0..7 {
            if (bits & (1 << row)) != 0 {
                for sx in 0..scale {
                    for sy in 0..scale {
                        let px = x + col as i32 * scale + sx;
                        let py = y + row as i32 * scale + sy;
                        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                            let pixel_idx = ((py as u32 * width + px as u32) * 4) as usize;
                            dst[pixel_idx] = color[0];
                            dst[pixel_idx + 1] = color[1];
                            dst[pixel_idx + 2] = color[2];
                            dst[pixel_idx + 3] = color[3];
                        }
                    }
                }
            }
        }
    }
}

/// Draw a string of text into an RGBA framebuffer.
pub fn draw_text(
    dst: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    scale: i32,
    text: &str,
    color: [u8; 4],
) {
    let mut cur_x = x;
    for c in text.chars() {
        draw_char(dst, width, height, cur_x, y, scale, c, color);
        cur_x += 6 * scale; // 5 width + 1 spacing
    }
}

/// Render an authentic retro console standby screen when powered on with no game.
pub fn render_standby_screen(frame: &mut VideoFrame, platform: Platform, elapsed: f32) {
    let width = 320u32;
    let height = 240u32;
    let total_bytes = (width * height * 4) as usize;

    if frame.pixels.len() != total_bytes {
        frame.width = width;
        frame.height = height;
        frame.pixels.resize(total_bytes, 0);
    }

    let dst = frame.pixels.as_mut_slice();

    match platform {
        Platform::PlayStation1 => {
            // Iconic PlayStation 1 navy blue gradient
            for y in 0..height {
                let grad = (y as f32 / height as f32) * 20.0;
                let b = (25.0 + grad) as u8;
                let r = (2.0 + grad * 0.2) as u8;
                let g = (5.0 + grad * 0.3) as u8;

                for x in 0..width {
                    let idx = ((y * width + x) * 4) as usize;
                    dst[idx] = r;
                    dst[idx + 1] = g;
                    dst[idx + 2] = b;
                    dst[idx + 3] = 255;
                }
            }

            // Draw header bar
            let gold = [218, 165, 32, 255];
            let white = [240, 240, 255, 255];
            let yellow = [255, 220, 50, 255];
            let gray = [160, 160, 180, 255];

            draw_text(dst, width, height, 48, 45, 2, "Sony PlayStation", gold);
            draw_text(dst, width, height, 48, 65, 1, "---------------------------------", gold);

            // Subtle pulsing cursor for "NO DISC INSERTED"
            let pulse = ((elapsed * 3.0).sin() * 0.5 + 0.5) > 0.3;
            if pulse {
                draw_text(dst, width, height, 66, 105, 2, "[ NO DISC INSERTED ]", yellow);
            } else {
                draw_text(dst, width, height, 66, 105, 2, "  NO DISC INSERTED  ", white);
            }

            draw_text(dst, width, height, 38, 155, 1, "Please insert CD-ROM to boot game", white);
            draw_text(dst, width, height, 22, 175, 1, "Place .cue / .chd / .iso into games/ps1/", gray);
            draw_text(dst, width, height, 76, 205, 1, "BIOS: Built-in HLE / system/", gray);
        }
        Platform::PlayStation2 => {
            // Deep PS2 ocean dark blue
            for y in 0..height {
                let b = (35.0 + (y as f32 / height as f32) * 25.0) as u8;
                for x in 0..width {
                    let idx = ((y * width + x) * 4) as usize;
                    dst[idx] = 4;
                    dst[idx + 1] = 10;
                    dst[idx + 2] = b;
                    dst[idx + 3] = 255;
                }
            }

            let cyan = [0, 180, 255, 255];
            let white = [240, 240, 255, 255];
            let gray = [150, 170, 200, 255];

            draw_text(dst, width, height, 70, 45, 2, "PlayStation 2", cyan);
            draw_text(dst, width, height, 70, 65, 1, "-------------------------", cyan);

            draw_text(dst, width, height, 60, 105, 2, "[ NO DISC IN TRAY ]", white);
            draw_text(dst, width, height, 30, 155, 1, "Press Eject or place .iso into games/ps2/", gray);
            draw_text(dst, width, height, 55, 180, 1, "Supported: .iso .chd .bin .cso", gray);
        }
        Platform::Nintendo64 => {
            // N64 dark charcoal screen
            for y in 0..height {
                for x in 0..width {
                    let idx = ((y * width + x) * 4) as usize;
                    dst[idx] = 12;
                    dst[idx + 1] = 12;
                    dst[idx + 2] = 16;
                    dst[idx + 3] = 255;
                }
            }

            let red = [230, 50, 50, 255];
            let white = [240, 240, 240, 255];
            let yellow = [240, 200, 40, 255];

            draw_text(dst, width, height, 68, 50, 2, "Nintendo 64", red);
            draw_text(dst, width, height, 68, 70, 1, "-----------------------", red);

            draw_text(dst, width, height, 42, 110, 2, "NO CARTRIDGE INSERTED", yellow);
            draw_text(dst, width, height, 36, 160, 1, "Place .z64 / .n64 file into games/n64/", white);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps1_standby_screen_rendered() {
        let mut frame = VideoFrame::default();
        render_standby_screen(&mut frame, Platform::PlayStation1, 0.0);
        assert_eq!(frame.width, 320);
        assert_eq!(frame.height, 240);
        // Ensure screen is not completely blank black
        assert!(frame.pixels.iter().any(|&p| p > 0));
    }
}
