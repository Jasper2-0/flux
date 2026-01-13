//! Port definitions for operator inputs and outputs
//!
//! This module provides:
//! - [`InputPort`] - Ports that receive values from other operators
//! - [`OutputPort`] - Ports that produce values for downstream operators
//! - [`TriggerInput`] - Ports that receive trigger signals (push-based)
//! - [`TriggerOutput`] - Ports that emit trigger signals (push-based)
//! - [`TypeConstraint`] - Defines what types an input port accepts
//! - [`OutputTypeRule`] - Defines how an output port's type is determined

mod constraint;
mod input;
mod output;
mod trigger;

pub use constraint::{OutputTypeRule, TypeConstraint};
pub use input::InputPort;
pub use output::OutputPort;
pub use trigger::{TriggerInput, TriggerOutput};
