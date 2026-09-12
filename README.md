# EmuSim (VR Standalone Retro Simulation)

> [!NOTE]
> **AI-Driven Development**: This project is actively being developed with AI agents (specifically **Gemini 3.8 Flash** at the moment) and is at the start of development. Gemini has been doing an excellent job and the developer is very optimistic about the journey ahead!

**EmuSim** is a standalone VR experience built in Rust for the **Meta Quest 3 and Quest 3S** (as well as PCVR & Desktop). Inspired by EmuVR, it recreates the tactile nostalgia of sitting in a retro bedroom, manually plugging power cables, connecting RCA composite/component wires, popping in N64 cartridges, opening PlayStation CD lids, turning on vintage CRT televisions, and playing emulated games in real-time VR.

---

## Supported Systems

| System | Format | Default Libretro Core | Physical Mechanics |
| :--- | :--- | :--- | :--- |
| **Nintendo 64** | `.z64`, `.n64`, `.v64` | `mupen64plus_next` | Spring-loaded top cartridge slot, power slider, reset switch, Jumper/Expansion Pak bay, 4 front controller ports |
| **Sony PlayStation (PS1)** | `.cue`, `.chd`, `.iso`, `.pbp` | `swanstation` / `duckstation` | Push-button power latch, spring lid release button, spindle disc clamping, dual controller ports |
| **Sony PlayStation 2** | `.iso`, `.chd`, `.cso`, `.bin` | `play` / `pcsx2` | Rear master rocker switch, front Standby/Reset button with Red/Green LED, motorized disc tray Eject |
| **Retro 8/16-bit** | `.sfc`, `.nes`, `.md` | `snes9x`, `nestopia`, `genesis_plus_gx` | Cartridge slots & multi-out AV |

---

## Workspace Architecture

The project is structured into modular, reusable Rust crates:

```
EmuSim/
├── crates/
│   ├── emusim-core/       # Electronic signal graph, device states (TV, N64, PS1, PS2), sockets, plugs, ROM metadata
│   ├── emusim-libretro/   # Safe Libretro C FFI loader, triple-buffered video frames, decoupled 60Hz worker thread
│   ├── emusim-physics/    # Verlet integration rope & cable physics, socket magnetic snapping and insertion depth
│   ├── emusim-audio/      # 3D spatial audio panner positioning console stereo sound at the TV speaker mesh
│   ├── emusim-render/     # Procedural dynamic cable mesh extrusion, CRT shader (scanlines, aperture grille, bloom)
│   ├── emusim-xr/         # Meta Quest Touch Plus 6DOF input, haptics, and retro gamepad mapping
│   └── emusim-app/        # App runtime coordinator, desktop test runner, and Android Quest entry point
├── android/               # AndroidManifest.xml configured for Meta Quest 3 / Quest 3S standalone VR
└── scripts/               # Build and APK deployment scripts
```

---

## Key Features

1. **Realistic Circuit & Signal Routing Graph**:
   - Outlets provide live 120V AC mains power.
   - Power strips route power through rocker switches.
   - Consoles only power up if their AC power cords are connected to live outlets.
   - CRT TVs only show game video when the console AV Multi-Out is wired to the selected TV input (`AV1`, `AV2`, `COMPONENT`).
   - If the TV is on with no console connected, it displays analog TV static snow.

2. **Physical Cable & Plug Simulation**:
   - Flexible Verlet integration cable strands simulate physics, sag, and tension.
   - Dynamic 3D tubular mesh extruded along cable particles in real time.
   - Magnetic alignment cone and tactile snap locking when plugs are inserted into matching jacks.

3. **Decoupled Emulation Worker**:
   - The Libretro core runs on a dedicated background thread at original console tick rates (e.g. 60 FPS).
   - Lock-free triple-buffering ensures the Quest OpenXR render loop stays at a rock-solid **72, 90, or 120 FPS** without stutter.

4. **Authentic CRT Post-Processing**:
   - Screen curvature barrel distortion.
   - Horizontal scanline emulation.
   - RGB phosphor triad aperture grille (Sony Trinitron style).
   - Magnetic degauss wobble effect on power up.

5. **3D Positional Spatial Audio**:
   - Stereo audio emitted by games is panned and attenuated in 3D world space, emitting directly from the virtual TV's speaker grille.

---

## Installing Libretro Emulation Cores

EmuSim comes with an automated core download script that fetches the official precompiled cores from the Libretro Buildbot:

```bash
# Download cores for Desktop (Linux / macOS / Windows)
./scripts/download_cores.sh desktop

# Download cores for Meta Quest 3 / Quest 3S (Android ARM64)
./scripts/download_cores.sh quest
```

This installs:
- **Nintendo 64**: `mupen64plus_next_libretro.so` & `parallel_n64_libretro.so`
- **PlayStation 1**: `swanstation_libretro.so` & `pcsx_rearmed_libretro.so`
- **PlayStation 2**: `pcsx2_libretro.so` & `play_libretro.so`
- **SNES / NES / Genesis**: `snes9x`, `nestopia`, `genesis_plus_gx`

### Downloading Console BIOS Files

To enable authentic console boot intros and original hardware menus for PS1 and PS2:

```bash
./scripts/download_bios.sh
```

This installs authentic BIOS files into `system/ps1/` and `system/pcsx2/bios/`.

---

## Quick Start: Desktop Runner

You can test and debug the complete simulation directly on your development machine:

```bash
cargo run --bin emusim
```

This runs the interactive scene sequence:
1. Spawns Wall Outlet, Power Strip, CRT TV, Nintendo 64, PlayStation 1, PlayStation 2, and cables.
2. Plugs Power Strip into Wall Outlet.
3. Plugs TV into Power Strip and turns it on (displays analog static noise).
4. Plugs N64 Power Adapter into Power Strip and N64.
5. Connects Nintendo Multi-Out AV cable to CRT TV AV1 inputs.
6. Switches N64 power slider ON.
7. Signal graph automatically triggers the Libretro worker and displays active video on the CRT TV!

---

## Building for Meta Quest 3 / Quest 3S Standalone

### Prerequisites
- Android NDK (r25c or newer)
- Rust target `aarch64-linux-android`:
  ```bash
  rustup target add aarch64-linux-android
  ```
- `cargo-apk`:
  ```bash
  cargo install cargo-apk
  ```

### Build & Deploy to Quest
1. Connect your Meta Quest 3 or Quest 3S to your computer via USB with Developer Mode enabled.
2. Run the packaging command:
   ```bash
   cargo apk run --package emusim-app --target aarch64-linux-android
   ```
3. Put ROMs and `.so` Libretro cores into `/sdcard/EmuSim/` on the headset.
