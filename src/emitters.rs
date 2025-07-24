use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
