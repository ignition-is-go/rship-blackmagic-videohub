# rship-blackmagic-videohub

A lightweight rship executor for controlling Blackmagic Videohub devices. This service provides a bridge between the [rship](https://docs.rship.io) ecosystem and Blackmagic Videohub routers, enabling remote control and monitoring of video routing operations.

## Quickstart

Clone the repo, copy the example `.env` and run:

```bash
git clone 
cp .env.example .env
cargo run
```

## Features

- **Real-time Video Routing**: Set video routes between inputs and outputs
- **Label Management**: Update input and output labels
- **Status Monitoring**: Real-time device status and routing state monitoring
- **rship Integration**: Full integration with the rship ecosystem for remote control
- **Async Architecture**: Built with tokio and mpsc channels for high performance

## Architecture

The service uses a single-process, dual-client architecture:

### VideohubService
- **Main orchestrator** managing both videohub and rship connections
- **mpsc channels** for efficient communication between clients
- **Event-driven** real-time routing and status updates

### VideohubClient
- Manages TCP connection to Blackmagic Videohub device
- Handles protocol parsing using the `videohub` crate
- Maintains device state and provides async operations

### rship Integration
- Actions for controlling videohub operations
- Emitters for real-time status and routing updates

## rship Actions

### Set Video Route (`set-route`)
Routes a specific input to an output
- `output`: Output port number (0-indexed)
- `input`: Input port number (0-indexed)

### Set Input Label (`set-input-label`)
Updates the label for an input port
- `input`: Input port number (0-indexed)
- `label`: New label text

### Set Output Label (`set-output-label`) 
Updates the label for an output port
- `output`: Output port number (0-indexed)
- `label`: New label text

## rship Emitters

### Route Changed (`route-changed`)
Emitted when a video route changes
- `output`, `input`: Port numbers
- `output_label`, `input_label`: Port labels (optional)

### Device Status (`device-status`)
Emitted for device connectivity and info updates
- `connected`: Connection status
- `model_name`: Device model (optional)
- `video_inputs`, `video_outputs`: Port counts (optional)

### Label Changed (`label-changed`)
Emitted when port labels are updated
- `port_type`: "input" or "output"
- `port`: Port number
- `label`: New label

## Project Structure

```
src/
├── main.rs           # Application entry point
├── service.rs        # VideohubService orchestrator
├── client.rs         # Videohub protocol client
├── actions.rs        # rship action schemas
└── emitters.rs       # rship emitter schemas
```

## Development

```bash
# Check compilation
cargo check

# Run tests  
cargo test

# Build for production
cargo build --release
```

## Dependencies

- **rship-sdk**: rship integration framework
- **videohub**: Blackmagic Videohub protocol implementation
- **tokio**: Async runtime with mpsc channels
- **anyhow**: Error handling
- **serde**: Serialization for rship schemas

## License

This project follows the same licensing as the rship ecosystem.