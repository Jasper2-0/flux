//! Symbol/Instance separation for operator definitions and runtime state.
//!
//! This module implements the pattern of separating:
//! - **Symbol**: The definition/blueprint of an operator, including its inputs,
//!   outputs, child operators, connections, and animation data.
//! - **Instance**: The runtime state of a symbol, created when the operator is
//!   actually used in a graph.
//!
//! This separation allows:
//! - Multiple instances of the same operator definition
//! - Efficient memory usage (shared definitions)
//! - Clear separation between design-time and runtime state
//!
//! # Example
//!
//! ```ignore
//! use flux_graph::symbol::{Symbol, SymbolChild, InputDefinition, OutputDefinition};
//! use flux_core::{Id, ValueType, Value};
//!
//! // Create a symbol definition
//! let mut symbol = Symbol::new("MyOperator");
//! symbol.add_input(InputDefinition::new("A", ValueType::Float, Value::Float(0.0)));
//! symbol.add_output(OutputDefinition::new("Result", ValueType::Float));
//!
//! // Add child operators
//! let child_id = Id::new();
//! let child = SymbolChild::new(child_id, symbol.id); // Reference to another symbol
//! symbol.add_child(child);
//!
//! // Create an instance from the symbol
//! let instance = symbol.create_instance();
//! ```

mod child;
mod core;
mod definition;
mod instance;
mod registry;

pub use child::{ChildInput, ChildOutput, SymbolChild};
pub use core::{Symbol, SymbolError};
pub use definition::{InputDefinition, OutputDefinition};
pub use instance::{Instance, InstanceChildren, InstanceStatus};
pub use registry::SymbolRegistry;
