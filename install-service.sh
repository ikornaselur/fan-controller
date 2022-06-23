#!/usr/bin/env bash

set -e

wget https://github.com/ikornaselur/fan-controller/releases/download/v0.1.0/fan-controller-aarch64-unknown-linux-gnu -O /opt/fan-controller
chown pi /opt/fan-controller
chmod +x /opt/fan-controller

wget https://raw.githubusercontent.com/ikornaselur/fan-controller/main/fan.service -O /lib/systemd/system/fan.service
systemctl daemon-reload
systemctl enable fan
systemctl start fan
