#!/usr/bin/env bash
set -euo pipefail

# EmuSim Console BIOS Downloader
# Fetches authentic console BIOS files (PlayStation 1, etc.) from Internet Archive.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SYSTEM_DIR="$ROOT_DIR/system"

mkdir -p "$SYSTEM_DIR"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "========================================================"
echo " EmuSim - Downloading Console BIOS Files (Internet Archive)"
echo " Destination: $SYSTEM_DIR"
echo "========================================================"

# 1. Download PS1 SCPH-5501 (NTSC-U standard BIOS)
PS1_DIR="$SYSTEM_DIR/ps1"
mkdir -p "$PS1_DIR"

echo -n "[>] Downloading PlayStation 1 BIOS (SCPH-5501)... "
SCPH5501_URL="https://archive.org/download/scph5501_20260218_2338/scph5501.zip"
curl -sL -o "$TMP_DIR/scph5501.zip" "$SCPH5501_URL"
if unzip -tq "$TMP_DIR/scph5501.zip" >/dev/null 2>&1; then
    unzip -o -q "$TMP_DIR/scph5501.zip" -d "$PS1_DIR"
    echo "DONE"
else
    echo "FAILED"
fi

# 2. Download PS1 BIOS pack (SCPH-1001, SCPH-5500, SCPH-5502, SCPH-7001)
echo -n "[>] Downloading PlayStation 1 BIOS Pack (Multi-Region)... "
PSX_URL="https://archive.org/download/PSXbios/PSX.zip"
curl -sL -o "$TMP_DIR/psx.zip" "$PSX_URL"
if unzip -tq "$TMP_DIR/psx.zip" >/dev/null 2>&1; then
    unzip -o -q "$TMP_DIR/psx.zip" -d "$TMP_DIR/psx_extracted"
    
    # Map extracted files to standard libretro naming conventions
    [ -f "$TMP_DIR/psx_extracted/PSX/PSX - SCPH1001.BIN" ] && cp "$TMP_DIR/psx_extracted/PSX/PSX - SCPH1001.BIN" "$PS1_DIR/scph1001.bin"
    [ -f "$TMP_DIR/psx_extracted/PSX/PSX - SCPH5500.BIN" ] && cp "$TMP_DIR/psx_extracted/PSX/PSX - SCPH5500.BIN" "$PS1_DIR/scph5500.bin"
    [ -f "$TMP_DIR/psx_extracted/PSX/PSX - SCPH5502.BIN" ] && cp "$TMP_DIR/psx_extracted/PSX/PSX - SCPH5502.BIN" "$PS1_DIR/scph5502.bin"
    [ -f "$TMP_DIR/psx_extracted/PSX/PSX - SCPH7001.BIN" ] && cp "$TMP_DIR/psx_extracted/PSX/PSX - SCPH7001.BIN" "$PS1_DIR/scph7001.bin"
    echo "DONE"
else
    echo "FAILED"
fi

# 3. Download PS2 BIOS (SCPH-70012 USA & SCPH-50004 Europe)
PS2_DIR="$SYSTEM_DIR/ps2/pcsx2/bios"
mkdir -p "$PS2_DIR"
echo -n "[>] Downloading PlayStation 2 BIOS (SCPH-70012 Slim USA)... "
curl -sL -o "$PS2_DIR/SCPH-70012.bin" "https://archive.org/download/scph-70012/PS2_bios/SCPH-70012.bin"
curl -sL -o "$PS2_DIR/SCPH-70012.nvm" "https://archive.org/download/scph-70012/PS2_bios/SCPH-70012.nvm"
curl -sL -o "$PS2_DIR/SCPH-70012.mec" "https://archive.org/download/scph-70012/PS2_bios/SCPH-70012.mec"
echo "DONE"

echo -n "[>] Downloading PlayStation 2 BIOS (SCPH-50004 Fat Europe)... "
curl -sL -o "$PS2_DIR/SCPH-50004.bin" "https://archive.org/download/scph-50004/SCPH-50004.bin"
curl -sL -o "$PS2_DIR/SCPH-50004.nvm" "https://archive.org/download/scph-50004/SCPH-50004.nvm"
curl -sL -o "$PS2_DIR/SCPH-50004.rom1" "https://archive.org/download/scph-50004/SCPH-50004.rom1"
echo "DONE"

echo ""
echo "========================================================"
echo " Installed PS1 BIOS files in $PS1_DIR:"
find "$PS1_DIR" -maxdepth 1 -name "*.bin" -exec ls -lh {} + 2>/dev/null || true
echo ""
echo " Installed PS2 BIOS files in $PS2_DIR:"
find "$PS2_DIR" -maxdepth 1 -name "*.bin" -exec ls -lh {} + 2>/dev/null || true
echo "========================================================"

