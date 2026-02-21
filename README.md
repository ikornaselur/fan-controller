# Fan Controller

Hardware PWM fan controller for Raspberry Pi 4. Uses a PID algorithm to smoothly regulate fan speed based on CPU temperature.

## Pi setup

Enable hardware PWM and disable the default audio overlay in `/boot/firmware/config.txt`:

```
dtparam=audio=off
dtoverlay=pwm,pin=18,func=2
```

Reboot after changing.

## Usage

```
fan-controller [OPTIONS]
```

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

## Build

Requires [just](https://github.com/casey/just) and [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild).

```
just build     # debug build for aarch64
just release   # release build for aarch64
just check     # type-check only
```
