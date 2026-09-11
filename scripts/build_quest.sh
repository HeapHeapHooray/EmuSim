#!/usr/bin/env bash
set -euo pipefail

TARGET="aarch64-linux-android"
API_LEVEL="29"

echo "=== EmuSim - Meta Quest 3/3S Build Script ==="

# Check Android NDK
if [ -z "${ANDROID_NDK_HOME:-}" ] && [ -z "${NDK_HOME:-}" ]; then
    echo "Warning: ANDROID_NDK_HOME or NDK_HOME not set."
    echo "Please export ANDROID_NDK_HOME=/path/to/android-ndk"
fi

# Ensure rust target is installed
echo "[1/3] Verifying Rust target $TARGET..."
rustup target add "$TARGET" || true

# Build cdylib for Quest (aarch64-linux-android)
echo "[2/3] Compiling emusim-app for Quest 3 / 3S ($TARGET)..."
cargo build --package emusim-app --target "$TARGET" --release

echo "[3/3] Build finished successfully."
echo "Native library located at: target/$TARGET/release/libemusim_app.so"
echo ""
echo "To package into Quest APK and run:"
echo "  1. Install cargo-apk: cargo install cargo-apk"
echo "  2. Connect Quest 3 via USB with Developer Mode enabled"
echo "  3. cargo apk run --package emusim-app --target aarch64-linux-android"
