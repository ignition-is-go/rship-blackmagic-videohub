# rship-blackmagic-videohub

[![CI](https://github.com/ignition-is-go/rship-blackmagic-videohub/actions/workflows/ci.yml/badge.svg)](https://github.com/ignition-is-go/rship-blackmagic-videohub/actions)
[![Crates.io](https://img.shields.io/crates/v/rship-blackmagic-videohub)](https://crates.io/crates/rship-blackmagic-videohub)
[![Documentation](https://docs.rs/rship-blackmagic-videohub/badge.svg)](https://docs.rs/rship-blackmagic-videohub)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[rship](https://docs.rship.io) executor for controlling [Blackmagic Videohub](https://www.blackmagicdesign.com/products/smartvideohub) devices. Bridges rship with Videohub routers for remote video routing control.

## Quickstart

- Setup and run the [VideoHub Simulator](https://github.com/peschuster/VideoHub-Simulator)
- Setup and run rship-blackmagic-videohub

```bash
git clone https://github.com/ignition-is-go/rship-blackmagic-videohub
cd rship-blackmagic-videohub
cp .env.example .env
cargo run
```

## Development

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings -A unused-variables -A dead-code -D warnings
cargo test
cargo build --release
```

## rship

### Device-Level Actions

- **`set-route`**: Route input to output (`output`, `input`)
- **`set-input-label`**: Update input label (`input`, `label`) - global device setting
- **`set-output-label`**: Update output label (`output`, `label`)
- **`set-output-lock`**: Lock/unlock output ports (`output`, `locked`)
- **`set-take-mode`**: Enable/disable take mode per output (`output`, `enabled`)

### Output Subtarget Actions

Each output port has actions:

- **`set-input`**: Set input for this output (`input`)
- **`set-label`**: Update this output's label (`label`)
- **`set-lock`**: Lock/unlock this output (`locked`)
- **`set-take-mode`**: Enable/disable take mode for this output (`enabled`)

### Device-Level Emitters

- **`device-status`**: Connection and device info (`connected`, `model_name`, `video_inputs`, `video_outputs`)
- **`network-interface`**: Network interface information (`interface_id`, `name`, `mac_address`, `current_addresses`, `current_gateway`, `dynamic_ip`)

### Output Subtarget Emitters

Each output subtarget provides individual event notifications:

- **`input-changed`**: Input routing updates (`input`, `input_label`)
- **`label-changed`**: Label updates (`port_type`, `port`, `label`)
- **`lock-changed`**: Lock state changes (`locked`)
- **`take-mode-changed`**: Take mode state changes (`enabled`)

## Dependencies

- **[rship-sdk](https://crates.io/crates/rship-sdk)**: rship integration framework
- **[videohub](https://crates.io/crates/videohub)**: Blackmagic Videohub protocol implementation

## License

This project follows the same licensing as the rship ecosystem.
