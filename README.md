# Fan Controller

Hardware PWM fan controller for Raspberry Pi 4. Uses a PID algorithm to smoothly regulate fan speed based on CPU temperature. Optionally publishes metrics to Home Assistant via MQTT auto-discovery.

## Pi setup

Enable hardware PWM and disable the default audio overlay in `/boot/firmware/config.txt`:

```
dtparam=audio=off
dtoverlay=pwm,pin=18,func=2
```

Reboot after changing.

## Usage

```
fan-controller <COMMAND>

Commands:
  run        Run the fan control loop
  install    Install and enable the systemd service
  uninstall  Stop, disable, and remove the systemd service
  update     Self-update from the latest GitHub release
```

### Run options

| Option | Default | Description |
|---|---|---|
| `-c, --channel` | `0` | PWM channel (0 or 1) |
| `-f, --frequency` | `25000` | PWM frequency in Hz |
| `-d, --duty-cycle` | `1.0` | Initial duty cycle (0.0-1.0) |
| `-t, --temp-path` | `/sys/class/thermal/thermal_zone0/temp` | Temperature source |
| `-s, --sleep` | `1` | Seconds between readings |
| `-l, --log-level` | `DEBUG` | TRACE/DEBUG/INFO/WARN/ERROR/OFF |
| `--target-temp` | `45` | Target temperature in C |
| `--kp` | `0.02` | PID proportional gain |
| `--ki` | `0.001` | PID integral gain |
| `--kd` | `0.01` | PID derivative gain |
| `--temp-samples` | `1` | Number of readings to average (smooths sensor noise) |
| `--mqtt-broker` | | MQTT broker address (enables MQTT when set) |
| `--mqtt-port` | `1883` | MQTT broker port |
| `--mqtt-prefix` | `fan_controller_{hostname}` | MQTT topic prefix |

MQTT credentials are read from `MQTT_USERNAME` and `MQTT_PASSWORD` environment variables.

## Install as a service

```bash
export MQTT_USERNAME=<username>
export MQTT_PASSWORD=<password>

sudo -E ./fan-controller install -- -l INFO --temp-samples 5 --mqtt-broker <broker-ip>
```

This will:
1. Point the systemd service at the binary's current location
2. Write the service file with the provided run args
3. Bake `MQTT_USERNAME`/`MQTT_PASSWORD` into the service as `Environment=` lines
4. Enable and start the service

To remove:

```bash
sudo ./fan-controller uninstall
```

## Self-update

```bash
sudo ./fan-controller update
```

Downloads the latest release from GitHub, replaces the binary in-place, and restarts the service if installed.

## Build

Requires [just](https://github.com/casey/just) and [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild).

```
just build     # debug build for aarch64
just release   # release build for aarch64
just check     # type-check only
```
