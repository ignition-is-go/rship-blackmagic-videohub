use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// DEVICE-LEVEL EMITTERS (for main device target - include output fields)

// Emitter data for route changes
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RouteChangedEmitter {
    // Output port number
    pub output: u32,
    // Input port number
    pub input: u32,
    // Optional output label
    pub output_label: Option<String>,
    // Optional input label
    pub input_label: Option<String>,
}

// Emitter data for device status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeviceStatusEmitter {
    // Whether the device is connected
    pub connected: bool,
    // Device model name (if available)
    pub model_name: Option<String>,
    // Number of video inputs
    pub video_inputs: Option<u32>,
    // Number of video outputs
    pub video_outputs: Option<u32>,
}

// Emitter data for label changes
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LabelChangedEmitter {
    // Port type ("input" or "output")
    pub port_type: String,
    // Port number
    pub port: u32,
    // New label
    pub label: String,
}

// Emitter data for output lock changes
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OutputLockChangedEmitter {
    // Output port number
    pub output: u32,
    // Whether the output is locked
    pub locked: bool,
    // Optional output label
    pub output_label: Option<String>,
}

// Emitter data for take mode changes
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TakeModeChangedEmitter {
    // Output port number
    pub output: u32,
    // Whether take mode is enabled
    pub enabled: bool,
    // Optional output label
    pub output_label: Option<String>,
}

// Emitter data for network interface status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NetworkInterfaceEmitter {
    // Interface ID
    pub interface_id: u32,
    // Interface name
    pub name: String,
    // MAC address
    pub mac_address: Option<String>,
    // Current IP addresses
    pub current_addresses: Option<String>,
    // Current gateway
    pub current_gateway: Option<String>,
    // Whether using dynamic IP
    pub dynamic_ip: Option<bool>,
}

// OUTPUT-LEVEL EMITTERS (for output subtargets - NO output fields, output is implicit)

// Emitter data for input changes on this output (output is implicit from target)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InputChangedEmitter {
    // Input port number
    pub input: u32,
    // Optional input label
    pub input_label: Option<String>,
}

// Emitter data for lock changes on this output (output is implicit from target)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LockChangedEmitter {
    // Whether the output is locked
    pub locked: bool,
}

// Emitter data for take mode changes on this output (output is implicit from target)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TakeModeOnThisOutputEmitter {
    // Whether take mode is enabled
    pub enabled: bool,
}
