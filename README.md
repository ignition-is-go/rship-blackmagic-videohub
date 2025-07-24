# rship-blackmagic-videohub

A lightweight rship executor for controlling Blackmagic Videohub devices. This executor provides a bridge between the [rship](https://rship.io) ecosystem and Blackmagic Videohub routers, enabling remote control and monitoring of video routing operations.

## Features

- **Real-time Video Routing**: Set video routes between inputs and outputs
- **Label Management**: Update input and output labels
- **Status Monitoring**: Real-time device status and routing state monitoring
- **rship Integration**: Full integration with the rship ecosystem for remote control
- **Rust Best Practices**: Built with modern Rust practices, async/await, and robust error handling

## Architecture

The executor consists of two main components:

### VideohubClient
- Manages TCP connection to Blackmagic Videohub device
- Handles protocol parsing using the `videohub` crate
- Maintains device state (routing, labels, device info)
- Provides async interface for device operations

### RshipExecutor
- Integrates with rship via the `rship-sdk`
- Exposes videohub operations as rship actions
- Emits status updates and route changes via rship emitters
- Handles bidirectional communication between rship and videohub

## Usage

### Environment Variables

Configure the executor using environment variables:

```bash
# Videohub device connection
VIDEOHUB_HOST=192.168.1.100    # Default: 192.168.1.100
VIDEOHUB_PORT=9990             # Default: 9990

# rship connection
RSHIP_ADDRESS=dev.rship.io     # Default: dev.rship.io  
RSHIP_PORT=5155                # Default: 5155

# Logging
RUST_LOG=debug                 # Optional: Set log level
```

### Running the Executor

```bash
# Development
cargo run

# Production
cargo build --release
./target/release/rship-blackmagic-videohub
```

## rship Actions

The executor exposes the following actions:

### Set Video Route
- **Action ID**: `set-route`
- **Description**: Routes a specific input to an output
- **Parameters**:
  - `output`: Output port number (0-indexed)
  - `input`: Input port number (0-indexed)

### Set Input Label
- **Action ID**: `set-input-label`
- **Description**: Updates the label for an input port
- **Parameters**:
  - `input`: Input port number (0-indexed)
  - `label`: New label text

### Set Output Label
- **Action ID**: `set-output-label`
- **Description**: Updates the label for an output port
- **Parameters**:
  - `output`: Output port number (0-indexed)
  - `label`: New label text

## rship Emitters

The executor provides real-time status updates via emitters:

### Route Changed
- **Emitter ID**: `route-changed`
- **Description**: Emitted when a video route changes
- **Data**:
  - `output`: Output port number
  - `input`: Input port number
  - `output_label`: Output label (if available)
  - `input_label`: Input label (if available)

### Device Status
- **Emitter ID**: `device-status`
- **Description**: Emitted for device connectivity and info updates
- **Data**:
  - `connected`: Connection status
  - `model_name`: Device model (if available)
  - `video_inputs`: Number of video inputs
  - `video_outputs`: Number of video outputs

### Label Changed
- **Emitter ID**: `label-changed`
- **Description**: Emitted when port labels are updated
- **Data**:
  - `port_type`: "input" or "output"
  - `port`: Port number
  - `label`: New label

## Protocol Support

This executor uses the `videohub` Rust crate which implements the Blackmagic Videohub Ethernet Protocol. The protocol supports:

- Device information queries
- Video routing (inputs to outputs) 
- Port label management
- Status monitoring
- Real-time updates

## Development

### Dependencies

- **rship-sdk**: rship integration
- **videohub**: Blackmagic Videohub protocol implementation
- **tokio**: Async runtime
- **anyhow**: Error handling
- **serde**: Serialization for rship actions/emitters

### Project Structure

```
src/
├── main.rs              # Application entry point
├── videohub_client.rs   # Videohub device client
└── rship_executor.rs    # rship integration
```

### Building

```bash
# Check compilation
cargo check

# Run tests
cargo test

# Build for production
cargo build --release
```

## Examples

See the `examples/` directory for rship SDK usage patterns that inform this implementation.

## Contributing

1. Follow Rust best practices
2. Maintain async/await patterns
3. Add tests for new functionality
4. Update documentation

## License

This project follows the same licensing as the rship ecosystem.

## Blackmagic Videohub Protocol

The Blackmagic Videohub Ethernet Protocol is a text-based protocol that runs over TCP port 9990. Key message types include:

- `VIDEOHUB DEVICE:` - Device information
- `INPUT LABELS:` - Input port labels  
- `OUTPUT LABELS:` - Output port labels
- `VIDEO OUTPUT ROUTING:` - Current routing state
- Acknowledgment messages (`ACK`/`NAK`)

For full protocol details, refer to the Blackmagic Videohub documentation.