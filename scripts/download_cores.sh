#!/usr/bin/env bash
set -euo pipefail

# EmuSim Libretro Core Downloader
# Automatically fetches precompiled Libretro cores from the official Libretro Buildbot.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_PLATFORM="${1:-desktop}"

CORES=(
    "parallel_n64_libretro"        # Nintendo 64 (Software rasterized, high compatibility)
    "mupen64plus_next_libretro"    # Nintendo 64
    "swanstation_libretro"         # PlayStation 1
    "pcsx_rearmed_libretro"        # PlayStation 1 (Fast ARM/Desktop alternative)
    "play_libretro"                # PlayStation 2
    "snes9x_libretro"              # Super Nintendo
    "nestopia_libretro"            # Nintendo Entertainment System
    "genesis_plus_gx_libretro"     # Sega Genesis / Mega Drive
)

case "$TARGET_PLATFORM" in
    desktop)
        OS_NAME="$(uname -s)"
        case "$OS_NAME" in
            Linux*)
                BASE_URL="https://buildbot.libretro.com/nightly/linux/x86_64/latest"
                EXT="so"
                DEST_DIR="$ROOT_DIR/cores"
                ;;
            Darwin*)
                BASE_URL="https://buildbot.libretro.com/nightly/apple/osx/x86_64/latest"
                EXT="dylib"
                DEST_DIR="$ROOT_DIR/cores"
                ;;
            MINGW*|MSYS*|CYGWIN*)
                BASE_URL="https://buildbot.libretro.com/nightly/windows/x86_64/latest"
                EXT="dll"
                DEST_DIR="$ROOT_DIR/cores"
                ;;
            *)
                echo "Unknown OS: $OS_NAME. Defaulting to Linux x86_64."
                BASE_URL="https://buildbot.libretro.com/nightly/linux/x86_64/latest"
                EXT="so"
                DEST_DIR="$ROOT_DIR/cores"
                ;;
        esac
        ;;
    quest|android)
        BASE_URL="https://buildbot.libretro.com/nightly/android/latest/arm64-v8a"
        EXT="so"
        DEST_DIR="$ROOT_DIR/cores/quest_arm64"
        ;;
    *)
        echo "Usage: $0 [desktop | quest]"
        exit 1
        ;;
esac

mkdir -p "$DEST_DIR"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "========================================================"
echo " EmuSim - Downloading Libretro Cores ($TARGET_PLATFORM)"
echo " Destination: $DEST_DIR"
echo " Source:      $BASE_URL"
echo "========================================================"

SUCCESS_COUNT=0
FAIL_COUNT=0

for CORE in "${CORES[@]}"; do
    ZIP_NAME="${CORE}.${EXT}.zip"
    URL="${BASE_URL}/${ZIP_NAME}"
    TMP_ZIP="${TMP_DIR}/${ZIP_NAME}"

    echo -n "[>] Downloading ${CORE}... "

    HTTP_STATUS=$(curl -sL -w "%{http_code}" -o "$TMP_ZIP" "$URL" || true)

    if [ "$HTTP_STATUS" -eq 200 ] && [ -f "$TMP_ZIP" ] && [ -s "$TMP_ZIP" ]; then
        # Verify it's a valid zip before unzipping
        if unzip -tq "$TMP_ZIP" >/dev/null 2>&1; then
            unzip -o -q "$TMP_ZIP" -d "$DEST_DIR"
            echo "DONE"
            SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
        else
            echo "FAILED (corrupted archive)"
            FAIL_COUNT=$((FAIL_COUNT + 1))
        fi
    else
        echo "SKIPPED (HTTP $HTTP_STATUS)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
done

echo ""
echo "========================================================"
echo " Installation Summary:"
echo " Installed: $SUCCESS_COUNT core(s)"
echo " Skipped:   $FAIL_COUNT core(s)"
echo " Files in $DEST_DIR:"
ls -lh "$DEST_DIR"/*."$EXT" 2>/dev/null || echo " (No cores found)"
echo "========================================================"
