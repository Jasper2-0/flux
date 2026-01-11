//! SetInputDefaultCommand - Change an input's default value

use flux_core::{Id, Value};

use super::Command;
use crate::graph::Graph;

/// Command to change an input port's default value.
///
/// On execute, the default is changed to the new value.
/// On undo, the default is restored to the previous value.
#[derive(Debug, Clone)]
pub struct SetInputDefaultCommand {
    /// Node ID
    node_id: Id,
    /// Input port index
    input_index: usize,
    /// New default value
    new_value: Value,
    /// Previous default value (for undo)
    previous_value: Option<Value>,
    /// Whether the command was successfully executed
    executed: bool,
}

impl SetInputDefaultCommand {
    /// Create a new SetInputDefaultCommand.
    pub fn new(node_id: Id, input_index: usize, new_value: Value) -> Self {
        Self {
            node_id,
            input_index,
            new_value,
            previous_value: None,
            executed: false,
        }
    }

    /// Get the previous value (available after execute).
    pub fn previous_value(&self) -> Option<&Value> {
        self.previous_value.as_ref()
    }
}

impl Command for SetInputDefaultCommand {
    fn name(&self) -> &str {
        "Set Value"
    }

    fn execute(&mut self, graph: &mut Graph) {
        // Access node directly to avoid lifetime issues with dyn Operator
        if let Some(node) = graph.nodes.get_mut(&self.node_id) {
            if let Some(input) = node.operator.inputs_mut().get_mut(self.input_index) {
                // Store previous value for undo
                self.previous_value = Some(input.default.clone());
                // Set new value
                input.default = self.new_value.clone();
                self.executed = true;
            }
        }
    }

    fn undo(&mut self, graph: &mut Graph) {
        if !self.executed {
            return;
        }

        // Restore previous value
        if let Some(ref prev) = self.previous_value {
            if let Some(node) = graph.nodes.get_mut(&self.node_id) {
                if let Some(input) = node.operator.inputs_mut().get_mut(self.input_index) {
                    input.default = prev.clone();
                }
            }
        }

        self.executed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::tests::TestOp;

    #[test]
    fn test_set_default_execute() {
        let mut graph = Graph::new();

        let op = TestOp::new(0.0);
        let id = op.id;
        graph.add(op);

        let mut cmd = SetInputDefaultCommand::new(id, 0, Value::Float(42.0));
        cmd.execute(&mut graph);

        let node = graph.get(id).unwrap();
        assert_eq!(node.inputs()[0].default, Value::Float(42.0));
        assert_eq!(cmd.previous_value(), Some(&Value::Float(0.0)));
    }

    #[test]
    fn test_set_default_undo() {
        let mut graph = Graph::new();

        let op = TestOp::new(0.0);
        let id = op.id;
        graph.add(op);

        let mut cmd = SetInputDefaultCommand::new(id, 0, Value::Float(42.0));
        cmd.execute(&mut graph);
        cmd.undo(&mut graph);

        let node = graph.get(id).unwrap();
        assert_eq!(node.inputs()[0].default, Value::Float(0.0));
    }

    #[test]
    fn test_set_default_redo() {
        let mut graph = Graph::new();

        let op = TestOp::new(0.0);
        let id = op.id;
        graph.add(op);

        let mut cmd = SetInputDefaultCommand::new(id, 0, Value::Float(42.0));
        cmd.execute(&mut graph);
        cmd.undo(&mut graph);
        cmd.execute(&mut graph);

        let node = graph.get(id).unwrap();
        assert_eq!(node.inputs()[0].default, Value::Float(42.0));
    }
}
