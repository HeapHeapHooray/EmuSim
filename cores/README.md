# Libretro Emulation Cores Directory

Place your platform-specific Libretro shared libraries in this directory:

### Linux / Meta Quest (Android)
- **Nintendo 64**: `mupen64plus_next_libretro.so` or `parallel_n64_libretro.so`
- **PlayStation 1**: `swanstation_libretro.so` or `duckstation_libretro.so` or `pcsx_rearmed_libretro.so`
- **PlayStation 2**: `play_libretro.so` or `pcsx2_libretro.so`
- **SNES**: `snes9x_libretro.so`
- **NES**: `nestopia_libretro.so`
- **Genesis**: `genesis_plus_gx_libretro.so`

### Windows
- Rename `.so` to `.dll` (e.g. `mupen64plus_next_libretro.dll`, `swanstation_libretro.dll`)

### macOS
- Rename `.so` to `.dylib` (e.g. `mupen64plus_next_libretro.dylib`)
