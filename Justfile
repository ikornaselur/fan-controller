target := "aarch64-unknown-linux-gnu"

# Type-check for the Pi target
check:
    cargo check --target {{target}}

# Build debug binary for Raspberry Pi
build:
    cargo zigbuild --target {{target}}

# Build release binary for Raspberry Pi
release:
    cargo zigbuild --target {{target}} --release

# Run tests
test:
    cargo test
