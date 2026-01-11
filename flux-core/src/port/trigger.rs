//! Trigger port definitions for push-based execution
//!
//! Trigger ports enable push-based execution alongside pull-based value flow.
//! Unlike value ports that carry data, triggers simply signal "execute now".
//!
//! # Pull vs Push Execution
//!
//! - **Pull (values)**: Evaluation happens on demand, flowing backwards from outputs
//! - **Push (triggers)**: Signals propagate forward immediately when fired
//!
//! # Use Cases
//!
//! - `OnFrame` - Fire every frame for animation
//! - `OnClick` - Fire when user clicks
//! - `OnComplete` - Fire when an operation finishes
//! - `OnChange` - Fire when a value changes
//!
//! # Example
//!
//! ```ignore
//! // An operator with trigger ports
//! struct FrameCounter {
//!     trigger_inputs: Vec<TriggerInput>,   // "OnFrame" trigger
//!     trigger_outputs: Vec<TriggerOutput>, // "Done" trigger
//!     count: u64,
//! }
//!
//! impl FrameCounter {
//!     fn on_triggered(&mut self, trigger_index: usize, ctx: &EvalContext) {
//!         if trigger_index == 0 {  // OnFrame
//!             self.count += 1;
//!             // Optionally fire "Done" trigger
//!         }
//!     }
//! }
//! ```

use crate::id::Id;
use serde::{Deserialize, Serialize};

/// An input port that receives trigger signals.
///
/// Trigger inputs don't carry data - they simply indicate that an event occurred
/// and the operator should execute its triggered behavior.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TriggerInput {
    /// Unique identifier for this trigger input
    pub id: Id,
    /// Display name for UI
    pub name: &'static str,
    /// Source connection: (source_node_id, source_trigger_output_index)
    ///
    /// Unlike value inputs, trigger inputs can only have one connection
    /// (the first signal wins, no need to merge triggers)
    pub connection: Option<(Id, usize)>,
}

impl TriggerInput {
    /// Create a new trigger input
    pub fn new(name: &'static str) -> Self {
        Self {
            id: Id::new(),
            name,
            connection: None,
        }
    }

    /// Connect this trigger input to a trigger output
    pub fn connect(&mut self, source_node: Id, source_output: usize) {
        self.connection = Some((source_node, source_output));
    }

    /// Disconnect this trigger input
    pub fn disconnect(&mut self) {
        self.connection = None;
    }

    /// Check if this trigger input is connected
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }
}

/// An output port that emits trigger signals.
///
/// Trigger outputs can be connected to multiple trigger inputs.
/// When fired, all connected inputs receive the signal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TriggerOutput {
    /// Unique identifier for this trigger output
    pub id: Id,
    /// Display name for UI
    pub name: &'static str,
    /// Target connections: Vec<(target_node_id, target_trigger_input_index)>
    ///
    /// A trigger output can fan out to multiple targets
    pub connections: Vec<(Id, usize)>,
}

impl TriggerOutput {
    /// Create a new trigger output
    pub fn new(name: &'static str) -> Self {
        Self {
            id: Id::new(),
            name,
            connections: Vec::new(),
        }
    }

    /// Add a connection to a trigger input
    pub fn connect(&mut self, target_node: Id, target_input: usize) {
        // Avoid duplicate connections
        let conn = (target_node, target_input);
        if !self.connections.contains(&conn) {
            self.connections.push(conn);
        }
    }

    /// Remove a connection to a trigger input
    pub fn disconnect(&mut self, target_node: Id, target_input: usize) {
        self.connections.retain(|&(id, idx)| id != target_node || idx != target_input);
    }

    /// Remove all connections to a specific node
    pub fn disconnect_node(&mut self, target_node: Id) {
        self.connections.retain(|&(id, _)| id != target_node);
    }

    /// Check if this trigger output has any connections
    pub fn is_connected(&self) -> bool {
        !self.connections.is_empty()
    }

    /// Get the number of connections
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_input_new() {
        let input = TriggerInput::new("OnFrame");
        assert_eq!(input.name, "OnFrame");
        assert!(input.connection.is_none());
        assert!(!input.is_connected());
    }

    #[test]
    fn test_trigger_input_connect() {
        let mut input = TriggerInput::new("OnFrame");
        let source_id = Id::new();

        input.connect(source_id, 0);

        assert!(input.is_connected());
        assert_eq!(input.connection, Some((source_id, 0)));
    }

    #[test]
    fn test_trigger_input_disconnect() {
        let mut input = TriggerInput::new("OnFrame");
        let source_id = Id::new();

        input.connect(source_id, 0);
        input.disconnect();

        assert!(!input.is_connected());
        assert!(input.connection.is_none());
    }

    #[test]
    fn test_trigger_output_new() {
        let output = TriggerOutput::new("Done");
        assert_eq!(output.name, "Done");
        assert!(output.connections.is_empty());
        assert!(!output.is_connected());
    }

    #[test]
    fn test_trigger_output_connect() {
        let mut output = TriggerOutput::new("Done");
        let target1 = Id::new();
        let target2 = Id::new();

        output.connect(target1, 0);
        output.connect(target2, 1);

        assert!(output.is_connected());
        assert_eq!(output.connection_count(), 2);
        assert!(output.connections.contains(&(target1, 0)));
        assert!(output.connections.contains(&(target2, 1)));
    }

    #[test]
    fn test_trigger_output_no_duplicate_connections() {
        let mut output = TriggerOutput::new("Done");
        let target = Id::new();

        output.connect(target, 0);
        output.connect(target, 0); // Same connection again

        assert_eq!(output.connection_count(), 1);
    }

    #[test]
    fn test_trigger_output_disconnect() {
        let mut output = TriggerOutput::new("Done");
        let target1 = Id::new();
        let target2 = Id::new();

        output.connect(target1, 0);
        output.connect(target2, 1);
        output.disconnect(target1, 0);

        assert_eq!(output.connection_count(), 1);
        assert!(!output.connections.contains(&(target1, 0)));
        assert!(output.connections.contains(&(target2, 1)));
    }

    #[test]
    fn test_trigger_output_disconnect_node() {
        let mut output = TriggerOutput::new("Done");
        let target1 = Id::new();
        let target2 = Id::new();

        output.connect(target1, 0);
        output.connect(target1, 1); // Same node, different input
        output.connect(target2, 0);
        output.disconnect_node(target1);

        assert_eq!(output.connection_count(), 1);
        assert!(output.connections.contains(&(target2, 0)));
    }
}
