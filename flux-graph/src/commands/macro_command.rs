//! MacroCommand - Group multiple commands for atomic undo/redo

use super::Command;
use crate::graph::Graph;

/// A command that groups multiple commands together.
///
/// All sub-commands are executed in order, and undone in reverse order.
/// This ensures complex operations (like "insert node on wire") can be
/// undone/redone atomically.
///
/// # Example
///
/// ```ignore
/// // "Insert node on wire" = disconnect + add node + connect x2
/// let mut macro_cmd = MacroCommand::new("Insert Node");
/// macro_cmd.push(DisconnectCommand::new(target, 0));
/// macro_cmd.push(AddNodeCommand::new(new_op));
/// macro_cmd.push(ConnectCommand::new(source, 0, new_node, 0));
/// macro_cmd.push(ConnectCommand::new(new_node, 0, target, 0));
///
/// history.execute(&mut graph, macro_cmd);
/// // All 4 operations undo together
/// history.undo(&mut graph);
/// ```
#[derive(Debug)]
pub struct MacroCommand {
    /// Human-readable name for this macro
    name: String,
    /// Sub-commands to execute in order
    commands: Vec<Box<dyn Command>>,
}

impl MacroCommand {
    /// Create a new empty MacroCommand.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            commands: Vec::new(),
        }
    }

    /// Add a command to the macro.
    ///
    /// Commands are executed in the order they're added.
    pub fn push<C: Command + 'static>(&mut self, command: C) {
        self.commands.push(Box::new(command));
    }

    /// Add a boxed command to the macro.
    pub fn push_boxed(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }

    /// Get the number of sub-commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the macro is empty.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Create a MacroCommand from a vector of commands.
    pub fn from_commands(name: impl Into<String>, commands: Vec<Box<dyn Command>>) -> Self {
        Self {
            name: name.into(),
            commands,
        }
    }
}

impl Command for MacroCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&mut self, graph: &mut Graph) {
        // Execute all sub-commands in order
        for cmd in &mut self.commands {
            cmd.execute(graph);
        }
    }

    fn undo(&mut self, graph: &mut Graph) {
        // Undo all sub-commands in reverse order
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(graph);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::tests::TestOp;
    use crate::commands::{AddNodeCommand, ConnectCommand, DisconnectCommand};

    #[test]
    fn test_macro_execute() {
        let mut graph = Graph::new();

        // Add two nodes via macro
        let op1 = TestOp::source(1.0);
        let id1 = op1.id;
        let op2 = TestOp::new(0.0);
        let id2 = op2.id;

        let mut macro_cmd = MacroCommand::new("Add Two Nodes");
        macro_cmd.push(AddNodeCommand::new(op1));
        macro_cmd.push(AddNodeCommand::new(op2));

        assert_eq!(graph.node_count(), 0);

        macro_cmd.execute(&mut graph);

        assert_eq!(graph.node_count(), 2);
        assert!(graph.get(id1).is_some());
        assert!(graph.get(id2).is_some());
    }

    #[test]
    fn test_macro_undo() {
        let mut graph = Graph::new();

        let op1 = TestOp::source(1.0);
        let id1 = op1.id;
        let op2 = TestOp::new(0.0);
        let id2 = op2.id;

        let mut macro_cmd = MacroCommand::new("Add Two Nodes");
        macro_cmd.push(AddNodeCommand::new(op1));
        macro_cmd.push(AddNodeCommand::new(op2));

        macro_cmd.execute(&mut graph);
        assert_eq!(graph.node_count(), 2);

        macro_cmd.undo(&mut graph);
        assert_eq!(graph.node_count(), 0);
        assert!(graph.get(id1).is_none());
        assert!(graph.get(id2).is_none());
    }

    #[test]
    fn test_macro_insert_on_wire() {
        // Simulates "insert node on wire" operation
        let mut graph = Graph::new();

        // Create initial graph: src -> sink
        let src = TestOp::source(1.0);
        let src_id = src.id;
        graph.add(src);

        let sink = TestOp::new(0.0);
        let sink_id = sink.id;
        graph.add(sink);

        graph.connect(src_id, 0, sink_id, 0).unwrap();

        // Now insert a new node in between
        let middle = TestOp::new(0.0);
        let middle_id = middle.id;

        let mut macro_cmd = MacroCommand::new("Insert Node");
        macro_cmd.push(DisconnectCommand::new(sink_id, 0));
        macro_cmd.push(AddNodeCommand::new(middle));
        macro_cmd.push(ConnectCommand::new(src_id, 0, middle_id, 0));
        macro_cmd.push(ConnectCommand::new(middle_id, 0, sink_id, 0));

        macro_cmd.execute(&mut graph);

        // Verify: src -> middle -> sink
        assert_eq!(graph.node_count(), 3);
        let middle_node = graph.get(middle_id).unwrap();
        assert_eq!(middle_node.inputs()[0].connection, Some((src_id, 0)));
        let sink_node = graph.get(sink_id).unwrap();
        assert_eq!(sink_node.inputs()[0].connection, Some((middle_id, 0)));

        // Undo should restore: src -> sink
        macro_cmd.undo(&mut graph);

        assert_eq!(graph.node_count(), 2);
        assert!(graph.get(middle_id).is_none());
        let sink_node = graph.get(sink_id).unwrap();
        assert_eq!(sink_node.inputs()[0].connection, Some((src_id, 0)));
    }

    #[test]
    fn test_macro_redo() {
        let mut graph = Graph::new();

        let op = TestOp::source(1.0);
        let id = op.id;

        let mut macro_cmd = MacroCommand::new("Add Node");
        macro_cmd.push(AddNodeCommand::new(op));

        macro_cmd.execute(&mut graph);
        macro_cmd.undo(&mut graph);
        assert_eq!(graph.node_count(), 0);

        macro_cmd.execute(&mut graph);
        assert_eq!(graph.node_count(), 1);
        assert!(graph.get(id).is_some());
    }

    #[test]
    fn test_macro_empty() {
        let mut graph = Graph::new();

        let mut macro_cmd = MacroCommand::new("Empty");
        assert!(macro_cmd.is_empty());

        // Should be safe to execute/undo empty macro
        macro_cmd.execute(&mut graph);
        macro_cmd.undo(&mut graph);
    }
}
