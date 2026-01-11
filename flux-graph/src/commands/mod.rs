//! Command Pattern for Graph Mutations
//!
//! This module provides an undo/redo system for graph operations, inspired by tixl.
//! Every mutation to the graph can be wrapped in a [`Command`] that knows how to
//! execute and undo itself.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     UndoRedoStack                           │
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
//! │  │ Command │  │ Command │  │ Command │  │ Command │  ...   │
//! │  └─────────┘  └─────────┘  └─────────┘  └─────────┘        │
//! │       ↑                         ↑                           │
//! │    oldest                    current                        │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Available Commands
//!
//! - [`AddNodeCommand`] - Add a new operator to the graph
//! - [`RemoveNodeCommand`] - Remove an operator from the graph
//! - [`ConnectCommand`] - Connect two ports
//! - [`DisconnectCommand`] - Disconnect a port
//! - [`SetInputDefaultCommand`] - Change an input's default value
//! - [`MacroCommand`] - Group multiple commands for atomic undo
//!
//! # Example
//!
//! ```ignore
//! use flux_graph::{Graph, UndoRedoStack};
//! use flux_graph::commands::{AddNodeCommand, ConnectCommand};
//!
//! let mut graph = Graph::new();
//! let mut history = UndoRedoStack::new();
//!
//! // Execute commands through the history
//! let add_cmd = AddNodeCommand::new(MyOperator::new());
//! let node_id = history.execute(&mut graph, add_cmd);
//!
//! // Undo the last command
//! history.undo(&mut graph);
//!
//! // Redo it
//! history.redo(&mut graph);
//! ```

mod add_node;
mod connect;
mod disconnect;
mod macro_command;
mod remove_node;
mod set_default;

pub use add_node::AddNodeCommand;
pub use connect::ConnectCommand;
pub use disconnect::DisconnectCommand;
pub use macro_command::MacroCommand;
pub use remove_node::RemoveNodeCommand;
pub use set_default::SetInputDefaultCommand;

use crate::graph::Graph;

/// A reversible operation on a graph.
///
/// Commands encapsulate graph mutations in a way that supports undo/redo.
/// Each command must be able to execute its operation and reverse it.
///
/// # Implementation Notes
///
/// - Commands should store any state needed to undo the operation
/// - `execute()` may be called multiple times (after undo/redo cycles)
/// - Commands should be serializable for session persistence (future)
pub trait Command: std::fmt::Debug {
    /// Human-readable name for this command (shown in undo menu).
    fn name(&self) -> &str;

    /// Execute the command, mutating the graph.
    ///
    /// This may be called multiple times if the command is undone and redone.
    fn execute(&mut self, graph: &mut Graph);

    /// Undo the command, reversing the mutation.
    ///
    /// After calling `undo()`, the graph should be in the same state as
    /// before `execute()` was called.
    fn undo(&mut self, graph: &mut Graph);

    /// Check if this command can be merged with another command.
    ///
    /// Some commands (like typing text) can be merged to reduce undo steps.
    /// Default implementation returns false.
    fn can_merge_with(&self, _other: &dyn Command) -> bool {
        false
    }

    /// Merge another command into this one.
    ///
    /// Only called if `can_merge_with()` returns true.
    /// Default implementation does nothing.
    fn merge(&mut self, _other: Box<dyn Command>) {
        // Default: no merging
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use flux_core::{EvalContext, Id, InputPort, Operator, OutputPort, Value, ValueType};

    /// Simple test operator
    pub(crate) struct TestOp {
        pub id: Id,
        pub inputs: Vec<InputPort>,
        pub outputs: Vec<OutputPort>,
        pub value: f32,
    }

    impl TestOp {
        pub fn new(value: f32) -> Self {
            let mut output = OutputPort::new("Out", ValueType::Float);
            output.set(Value::Float(value));
            Self {
                id: Id::new(),
                inputs: vec![InputPort::new("In", Value::Float(0.0))],
                outputs: vec![output],
                value,
            }
        }

        pub fn source(value: f32) -> Self {
            let mut output = OutputPort::new("Out", ValueType::Float);
            output.set(Value::Float(value));
            Self {
                id: Id::new(),
                inputs: vec![],
                outputs: vec![output],
                value,
            }
        }
    }

    impl Operator for TestOp {
        fn id(&self) -> Id {
            self.id
        }
        fn name(&self) -> &'static str {
            "TestOp"
        }
        fn inputs(&self) -> &[InputPort] {
            &self.inputs
        }
        fn inputs_mut(&mut self) -> &mut [InputPort] {
            &mut self.inputs
        }
        fn outputs(&self) -> &[OutputPort] {
            &self.outputs
        }
        fn outputs_mut(&mut self) -> &mut [OutputPort] {
            &mut self.outputs
        }
        fn compute(&mut self, _ctx: &EvalContext, _get_input: &dyn Fn(Id, usize) -> Value) {
            self.outputs[0].set(Value::Float(self.value));
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
}
