#!/usr/bin/env bash

set -e

if [ "$(id -u)" -ne 0 ]; then
    echo "This script must be run as root (use sudo)"
    exit 1
fi

REPO="ikornaselur/fan-controller"
BINARY="fan-controller"
INSTALL_DIR="/opt"
SERVICE_FILE="/lib/systemd/system/fan.service"

# Get latest release tag
LATEST=$(curl -sf "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)

if [ -z "$LATEST" ]; then
    echo "Failed to fetch latest release"
    exit 1
fi

echo "Installing fan-controller ${LATEST}"

curl -Lfo "${INSTALL_DIR}/fan-controller" \
    "https://github.com/${REPO}/releases/download/${LATEST}/${BINARY}"
chmod +x "${INSTALL_DIR}/fan-controller"

curl -Lfo "${SERVICE_FILE}" \
    "https://raw.githubusercontent.com/${REPO}/main/fan.service"

systemctl daemon-reload
systemctl enable fan
systemctl restart fan

echo "Done. fan-controller ${LATEST} installed and running."
