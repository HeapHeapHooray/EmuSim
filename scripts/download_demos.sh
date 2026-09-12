#!/usr/bin/env bash
set -euo pipefail

# EmuSim Demo / Homebrew ROM Downloader
# Fetches open-source, public domain, and permissive homebrew test ROMs.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

mkdir -p "$ROOT_DIR/games/n64" "$ROOT_DIR/games/ps1" "$ROOT_DIR/games/ps2"

echo "========================================================"
echo " EmuSim - Downloading Open-Source Demos & Test ROMs"
echo "========================================================"

# 1. Nintendo 64: N64NICCC (Peter Lemon / Krom 3D polygon rotating bare-metal demo)
N64_DEMO="$ROOT_DIR/games/n64/N64NICCC.z64"
if [ ! -f "$N64_DEMO" ]; then
    echo -n "[>] Downloading N64 3D rotating demo (N64NICCC)... "
    curl -sL "https://raw.githubusercontent.com/PeterLemon/N64/master/N64NICCC/N64NICCC.N64" -o "$N64_DEMO"
    echo "DONE ($(du -h "$N64_DEMO" | cut -f1))"
else
    echo "[i] N64 demo already present: $N64_DEMO"
fi

echo ""
echo "========================================================"
echo " Demos & Test Media Ready!"
echo " - N64:  games/n64/N64NICCC.z64"
echo " - PS1:  Full standalone Sony PlayStation BIOS boot intro & chime"
echo "         (Place any .cue / .chd / .iso in games/ps1/ to boot game directly)"
echo "========================================================"
