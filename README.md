# rship-blackmagic-videohub

[![CI](https://github.com/ignition-is-go/rship-blackmagic-videohub/actions/workflows/ci.yml/badge.svg)](https://github.com/ignition-is-go/rship-blackmagic-videohub/actions)
[![Crates.io](https://img.shields.io/crates/v/rship-blackmagic-videohub)](https://crates.io/crates/rship-blackmagic-videohub)
[![Documentation](https://docs.rs/rship-blackmagic-videohub/badge.svg)](https://docs.rs/rship-blackmagic-videohub)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[rship](https://docs.rship.io) executor for controlling [Blackmagic Videohub](https://www.blackmagicdesign.com/products/smartvideohub) devices. Bridges rship with Videohub routers for remote video routing control.

## Quickstart

```bash
git clone https://github.com/ignition-is-go/rship-blackmagic-videohub
cd rship-blackmagic-videohub
cp .env.example .env
cargo run
```

## Features

- Real-time video routing and label management
- Device status monitoring  
- Full rship integration with async architecture

## rship Integration

### Actions
- **`set-route`**: Route input to output (`output`, `input`)
- **`set-input-label`**: Update input label (`input`, `label`)
- **`set-output-label`**: Update output label (`output`, `label`)

### Emitters
- **`route-changed`**: Route updates (`output`, `input`, labels)
- **`device-status`**: Connection and device info (`connected`, `model_name`, port counts)
- **`label-changed`**: Label updates (`port_type`, `port`, `label`)

## Development

```bash
cargo fmt --all                              # Format code
cargo clippy --all-targets --all-features    # Lint code (CI runs with -D warnings)
cargo test                                   # Run tests
cargo run                                    # Run the service
```

## Dependencies

- **[rship-sdk](https://crates.io/crates/rship-sdk)**: rship integration framework
- **[videohub](https://crates.io/crates/videohub)**: Blackmagic Videohub protocol implementation

## License

This project follows the same licensing as the rship ecosystem.
