use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// DEVICE-LEVEL ACTIONS (for main device target - include output fields)

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

// Action data for setting output lock state
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetOutputLockAction {
    // Output port number (0-indexed)
    pub output: u32,
    // Whether to lock the output
    pub locked: bool,
}

// Action data for setting take mode on an output
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetTakeModeAction {
    // Output port number (0-indexed)
    pub output: u32,
    // Whether to enable take mode
    pub enabled: bool,
}

// OUTPUT-LEVEL ACTIONS (for output subtargets - NO output fields, output is implicit)

// Action data for setting input on this output (output is implicit from target)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetInputAction {
    // Input port number (0-indexed)
    pub input: u32,
}

// Action data for setting label on this output (output is implicit from target)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetLabelAction {
    // New label for the output
    pub label: String,
}

// Action data for setting lock state on this output (output is implicit from target)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetLockAction {
    // Whether to lock the output
    pub locked: bool,
}

// Action data for setting take mode on this output (output is implicit from target)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetTakeModeOnThisOutputAction {
    // Whether to enable take mode
    pub enabled: bool,
}
