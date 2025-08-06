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

## rship

### Actions

- **`set-route`**: Route input to output (`output`, `input`)
- **`set-input-label`**: Update input label (`input`, `label`)
- **`set-output-label`**: Update output label (`output`, `label`)
- **`set-output-lock`**: Lock/unlock output ports (`output`, `locked`)
- **`set-take-mode`**: Enable/disable take mode per output (`output`, `enabled`)

### Emitters

- **`route-changed`**: Route updates (`output`, `input`, labels)
- **`device-status`**: Connection and device info (`connected`, `model_name`, port counts)
- **`label-changed`**: Label updates (`port_type`, `port`, `label`)
- **`output-lock-changed`**: Output lock state changes (`output`, `locked`, `output_label`)
- **`take-mode-changed`**: Take mode state changes (`output`, `enabled`, `output_label`)
- **`network-interface`**: Network interface information (`interface_id`, `name`, `mac_address`, `current_addresses`, etc.)

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
