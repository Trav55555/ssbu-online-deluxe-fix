#!/usr/bin/env bash

# ==============================
# Edit these to match your setup
# ==============================

PLUGIN_NAME="ssbu_online_deluxe"
EMU_PLUGIN_PATH="$HOME/.local/share/eden/sdmc/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/lib$PLUGIN_NAME.nro"
LOCAL_PLUGIN_PATH="./target/aarch64-skyline-switch/release/lib$PLUGIN_NAME.nro"
EMU_EXE_PATH="${EMU_EXE_PATH:-$HOME/.local/share/emulators/eden/Eden-Linux-v0.2.0-rc1-amd64-gcc-standard.AppImage}"
SMASH_NSP_PATH="${SMASH_NSP_PATH:-$HOME/Games/Switch/Super Smash Brothers Ultimate [v0].nsp}"
CLEAN="${CLEAN:-1}"

# ==============================
# Get local IPv4 address
# ==============================

#IP=$(ip route get 1 | awk '{print $7; exit}')
IP="127.0.0.1"

# ==============================
# Build plugin
# ==============================

cargo skyline build --release
if [ $? -ne 0 ]; then
    exit $?
fi

# ==============================
# Copy to plugin to mod directory
# ==============================
if [ "$CLEAN" -eq 1 ]; then
    echo "Cleaning .nro plugins from target directory..."
    find $(dirname "$EMU_PLUGIN_PATH") -type f -name "*.nro" -exec rm -f {} +
fi
cp -f "$LOCAL_PLUGIN_PATH" "$EMU_PLUGIN_PATH"
if [ $? -ne 0 ]; then
    exit $?
fi

# ==============================
# Cleanup handler (kill emulator on exit)
# ==============================

cleanup() {
    if [ -n "$EMU_PID" ]; then
        kill -9 "$EMU_PID" 2>/dev/null
    fi
}

trap cleanup EXIT INT TERM

# ==============================
# Start emlator in background
# ==============================

#"$EMU_EXE_PATH" "$SMASH_NSP_PATH"
"$EMU_EXE_PATH" "$SMASH_NSP_PATH" &
EMU_PID=$!

#tail --follow --retry "$HOME/.local/share/eden/sdmc/ultimate/ssbu_online_deluxe/logs/ssbu_online_deluxe.log"
echo "Starting cargo skyline listen on $IP..."
cargo skyline listen --ip "$IP"
echo "Finishing cargo skyline listen..."
