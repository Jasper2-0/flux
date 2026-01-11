//! Port definitions for operator inputs and outputs
//!
//! This module provides:
//! - [`InputPort`] - Ports that receive values from other operators
//! - [`OutputPort`] - Ports that produce values for downstream operators
//! - [`TriggerInput`] - Ports that receive trigger signals (push-based)
//! - [`TriggerOutput`] - Ports that emit trigger signals (push-based)

mod input;
mod output;
mod trigger;

pub use input::InputPort;
pub use output::OutputPort;
pub use trigger::{TriggerInput, TriggerOutput};
