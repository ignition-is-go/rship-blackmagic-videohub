use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Action data for setting a video route
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetRouteAction {
    // Output port number (0-indexed)
    pub output: u32,
    // Input port number (0-indexed)
    pub input: u32,
}

// Action data for setting an input label
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetInputLabelAction {
    // Input port number (0-indexed)
    pub input: u32,
    // New label for the input
    pub label: String,
}

// Action data for setting an output label
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetOutputLabelAction {
    // Output port number (0-indexed)
    pub output: u32,
    // New label for the output
    pub label: String,
}
