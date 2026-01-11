//! RemoveNodeCommand - Remove an operator from the graph

use flux_core::{Id, Operator};

use super::Command;
use crate::graph::Graph;

/// Command to remove an operator from the graph.
///
/// On execute, the operator is removed and stored for undo.
/// On undo, the operator is re-added to the graph.
///
/// Note: This command does NOT restore connections that were made
/// TO this node from other nodes. Those connections are broken permanently.
/// For full connection restoration, use a MacroCommand that includes
/// disconnect commands for each affected connection.
pub struct RemoveNodeCommand {
    /// The ID of the node to remove
    node_id: Id,
    /// The removed operator (stored after execute for undo)
    operator: Option<Box<dyn Operator>>,
}

impl std::fmt::Debug for RemoveNodeCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoveNodeCommand")
            .field("node_id", &self.node_id)
            .field("has_operator", &self.operator.is_some())
            .finish()
    }
}

impl RemoveNodeCommand {
    /// Create a new RemoveNodeCommand.
    ///
    /// The node with the given ID will be removed when `execute()` is called.
    pub fn new(node_id: Id) -> Self {
        Self {
            node_id,
            operator: None,
        }
    }

    /// Get the ID of the node being removed.
    pub fn node_id(&self) -> Id {
        self.node_id
    }
}

impl Command for RemoveNodeCommand {
    fn name(&self) -> &str {
        "Remove Node"
    }

    fn execute(&mut self, graph: &mut Graph) {
        // Remove the node and store it for undo
        if let Some(operator) = graph.remove(self.node_id) {
            self.operator = Some(operator);
        }
    }

    fn undo(&mut self, graph: &mut Graph) {
        // Re-add the operator
        if let Some(operator) = self.operator.take() {
            graph.add_boxed(operator);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::tests::TestOp;

    #[test]
    fn test_remove_node_execute() {
        let mut graph = Graph::new();
        let op = TestOp::source(42.0);
        let id = op.id;
        graph.add(op);

        assert_eq!(graph.node_count(), 1);

        let mut cmd = RemoveNodeCommand::new(id);
        cmd.execute(&mut graph);

        assert_eq!(graph.node_count(), 0);
        assert!(graph.get(id).is_none());
    }

    #[test]
    fn test_remove_node_undo() {
        let mut graph = Graph::new();
        let op = TestOp::source(42.0);
        let id = op.id;
        graph.add(op);

        let mut cmd = RemoveNodeCommand::new(id);
        cmd.execute(&mut graph);
        assert_eq!(graph.node_count(), 0);

        cmd.undo(&mut graph);
        assert_eq!(graph.node_count(), 1);
        assert!(graph.get(id).is_some());
    }

    #[test]
    fn test_remove_node_redo() {
        let mut graph = Graph::new();
        let op = TestOp::source(42.0);
        let id = op.id;
        graph.add(op);

        let mut cmd = RemoveNodeCommand::new(id);
        cmd.execute(&mut graph);
        cmd.undo(&mut graph);
        assert_eq!(graph.node_count(), 1);

        cmd.execute(&mut graph);
        assert_eq!(graph.node_count(), 0);
        assert!(graph.get(id).is_none());
    }

    #[test]
    fn test_remove_nonexistent_node() {
        let mut graph = Graph::new();
        let fake_id = Id::new();

        let mut cmd = RemoveNodeCommand::new(fake_id);
        cmd.execute(&mut graph); // Should not panic

        // Undo should also be safe
        cmd.undo(&mut graph);
    }
}
