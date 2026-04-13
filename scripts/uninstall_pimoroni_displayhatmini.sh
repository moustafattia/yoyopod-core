#!/bin/bash
# Uninstall Pimoroni Display HAT Mini support on Radxa Cubie A7Z.
#
# This script reverses everything done by install_pimoroni.sh:
# 1. Restores the stock SPI1 overlay (with MISO assigned to SPI).
# 2. Disables and removes the I2S0 unbind service (re-enables WM8960 audio).
#
# Requires: sudo access.
# Run once, then reboot.
#
# See docs/CUBIE_A7Z_PIMORONI_SETUP.md for full details.

set -euo pipefail

OVERLAY_PATH="/boot/dtbo/sun60iw2p1-spi1-spidev.dtbo"
OVERLAY_BACKUP="${OVERLAY_PATH}.bak"
SERVICE_NAME="disable-i2s0"
SERVICE_PATH="/etc/systemd/system/${SERVICE_NAME}.service"

echo "=== Pimoroni Display HAT Mini uninstaller for Cubie A7Z ==="
echo

# --- Step 1: Restore stock SPI1 overlay ---

echo "[1/2] Restoring stock SPI1 overlay..."

if [ -f "$OVERLAY_BACKUP" ]; then
    sudo cp "$OVERLAY_BACKUP" "$OVERLAY_PATH"
    sudo rm "$OVERLAY_BACKUP"
    echo "  Stock overlay restored from backup."
else
    echo "  No backup found at $OVERLAY_BACKUP — skipping."
    echo "  If SPI1 is misconfigured, re-enable it via rsetup."
fi

# --- Step 2: Remove I2S0 unbind service ---

echo "[2/2] Removing I2S0 unbind service..."

if [ -f "$SERVICE_PATH" ]; then
    sudo systemctl stop "$SERVICE_NAME" 2>/dev/null || true
    sudo systemctl disable "$SERVICE_NAME" --quiet 2>/dev/null || true
    sudo rm "$SERVICE_PATH"
    sudo systemctl daemon-reload
    echo "  Service removed. I2S0 audio will be available after reboot."
else
    echo "  Service not found — skipping."
fi

# --- Done ---

echo
echo "Done. Reboot to restore defaults:"
echo "  sudo reboot"
echo
echo "After reboot:"
echo "  - SPI1 MISO (PIN_21) returns to SPI function"
echo "  - I2S0 audio (WM8960) is re-enabled"
echo "  - Pimoroni display will no longer work"
echo "  - Set YOYOPOD_DISPLAY=whisplay to use Whisplay HAT"
