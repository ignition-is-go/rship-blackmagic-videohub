//! # rship-blackmagic-videohub
//!
//! rship executor for controlling Blackmagic Videohub devices.
//!
//! This crate provides functionality to connect to and control Blackmagic Videohub video routing devices
//! with [rship](https://docs.rship.io).

pub mod actions;
pub mod client;
pub mod emitters;
pub mod service;

// Re-export the main service and commonly used types
pub use actions::{SetInputLabelAction, SetOutputLabelAction, SetRouteAction};
pub use emitters::{DeviceStatusEmitter, LabelChangedEmitter, RouteChangedEmitter};
pub use service::VideohubService;
