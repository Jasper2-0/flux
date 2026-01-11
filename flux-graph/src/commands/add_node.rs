//! AddNodeCommand - Add an operator to the graph

use flux_core::{Id, Operator};

use super::Command;
use crate::graph::Graph;

/// Command to add a new operator to the graph.
///
/// On execute, the operator is added and its ID is stored.
/// On undo, the operator is removed and stored for potential redo.
pub struct AddNodeCommand {
    /// The operator to add (None after execute, restored on undo)
    operator: Option<Box<dyn Operator>>,
    /// The ID assigned to the node (set after first execute)
    node_id: Option<Id>,
    /// Human-readable name for the command
    op_name: &'static str,
}

impl std::fmt::Debug for AddNodeCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AddNodeCommand")
            .field("node_id", &self.node_id)
            .field("op_name", &self.op_name)
            .field("has_operator", &self.operator.is_some())
            .finish()
    }
}

impl AddNodeCommand {
    /// Create a new AddNodeCommand.
    ///
    /// The operator will be added to the graph when `execute()` is called.
    pub fn new<O: Operator + 'static>(operator: O) -> Self {
        let op_name = operator.name();
        Self {
            operator: Some(Box::new(operator)),
            node_id: None,
            op_name,
        }
    }

    /// Create from a boxed operator.
    pub fn from_boxed(operator: Box<dyn Operator>) -> Self {
        let op_name = operator.name();
        Self {
            operator: Some(operator),
            node_id: None,
            op_name,
        }
    }

    /// Get the ID of the added node (available after execute).
    pub fn node_id(&self) -> Option<Id> {
        self.node_id
    }
}

impl Command for AddNodeCommand {
    fn name(&self) -> &str {
        // Return a static string since we can't format dynamically
        "Add Node"
    }

    fn execute(&mut self, graph: &mut Graph) {
        if let Some(operator) = self.operator.take() {
            // First execution or redo - add the operator
            let id = operator.id();
            graph.add_boxed(operator);
            self.node_id = Some(id);
        } else if let Some(id) = self.node_id {
            // This shouldn't happen in normal usage, but handle it gracefully
            // The operator was already added and not undone
            eprintln!("Warning: AddNodeCommand executed without operator (id: {:?})", id);
        }
    }

    fn undo(&mut self, graph: &mut Graph) {
        if let Some(id) = self.node_id {
            // Remove the node and store it for redo
            if let Some(operator) = graph.remove(id) {
                self.operator = Some(operator);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::tests::TestOp;

    #[test]
    fn test_add_node_execute() {
        let mut graph = Graph::new();
        let mut cmd = AddNodeCommand::new(TestOp::source(42.0));

        assert_eq!(graph.node_count(), 0);
        assert!(cmd.node_id().is_none());

        cmd.execute(&mut graph);

        assert_eq!(graph.node_count(), 1);
        assert!(cmd.node_id().is_some());
        assert!(graph.get(cmd.node_id().unwrap()).is_some());
    }

    #[test]
    fn test_add_node_undo() {
        let mut graph = Graph::new();
        let mut cmd = AddNodeCommand::new(TestOp::source(42.0));

        cmd.execute(&mut graph);
        let id = cmd.node_id().unwrap();
        assert_eq!(graph.node_count(), 1);

        cmd.undo(&mut graph);
        assert_eq!(graph.node_count(), 0);
        assert!(graph.get(id).is_none());
    }

    #[test]
    fn test_add_node_redo() {
        let mut graph = Graph::new();
        let mut cmd = AddNodeCommand::new(TestOp::source(42.0));

        cmd.execute(&mut graph);
        let id = cmd.node_id().unwrap();

        cmd.undo(&mut graph);
        assert_eq!(graph.node_count(), 0);

        cmd.execute(&mut graph);
        assert_eq!(graph.node_count(), 1);
        // After redo, the node should have the same ID
        assert!(graph.get(id).is_some());
    }
}
