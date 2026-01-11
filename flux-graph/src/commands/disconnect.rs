//! DisconnectCommand - Disconnect a port in the graph

use flux_core::Id;

use super::Command;
use crate::graph::Graph;

/// Command to disconnect an input port.
///
/// On execute, the connection is removed.
/// On undo, the connection is restored.
#[derive(Debug)]
pub struct DisconnectCommand {
    /// Target node ID
    target_node: Id,
    /// Target input port index
    target_input: usize,
    /// The connection that was removed (for undo)
    previous_connection: Option<(Id, usize)>,
    /// Whether the command was successfully executed
    executed: bool,
}

impl DisconnectCommand {
    /// Create a new DisconnectCommand.
    pub fn new(target_node: Id, target_input: usize) -> Self {
        Self {
            target_node,
            target_input,
            previous_connection: None,
            executed: false,
        }
    }

    /// Get the previous connection that was removed.
    pub fn previous_connection(&self) -> Option<(Id, usize)> {
        self.previous_connection
    }
}

impl Command for DisconnectCommand {
    fn name(&self) -> &str {
        "Disconnect"
    }

    fn execute(&mut self, graph: &mut Graph) {
        // Store previous connection for undo
        if let Some(node) = graph.get(self.target_node) {
            if let Some(input) = node.inputs().get(self.target_input) {
                self.previous_connection = input.connection;
            }
        }

        // Only proceed if there was actually a connection
        if self.previous_connection.is_some() {
            match graph.disconnect(self.target_node, self.target_input) {
                Ok(()) => {
                    self.executed = true;
                }
                Err(e) => {
                    eprintln!("DisconnectCommand failed: {}", e);
                    self.executed = false;
                }
            }
        }
    }

    fn undo(&mut self, graph: &mut Graph) {
        if !self.executed {
            return;
        }

        // Restore the previous connection
        if let Some((source_node, source_output)) = self.previous_connection {
            let _ = graph.connect(source_node, source_output, self.target_node, self.target_input);
        }

        self.executed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::tests::TestOp;

    #[test]
    fn test_disconnect_execute() {
        let mut graph = Graph::new();

        let src = TestOp::source(1.0);
        let src_id = src.id;
        graph.add(src);

        let sink = TestOp::new(0.0);
        let sink_id = sink.id;
        graph.add(sink);

        graph.connect(src_id, 0, sink_id, 0).unwrap();

        let mut cmd = DisconnectCommand::new(sink_id, 0);
        cmd.execute(&mut graph);

        // Check connection was removed
        let sink_node = graph.get(sink_id).unwrap();
        assert!(sink_node.inputs()[0].connection.is_none());
        assert_eq!(cmd.previous_connection(), Some((src_id, 0)));
    }

    #[test]
    fn test_disconnect_undo() {
        let mut graph = Graph::new();

        let src = TestOp::source(1.0);
        let src_id = src.id;
        graph.add(src);

        let sink = TestOp::new(0.0);
        let sink_id = sink.id;
        graph.add(sink);

        graph.connect(src_id, 0, sink_id, 0).unwrap();

        let mut cmd = DisconnectCommand::new(sink_id, 0);
        cmd.execute(&mut graph);
        cmd.undo(&mut graph);

        // Check connection was restored
        let sink_node = graph.get(sink_id).unwrap();
        assert_eq!(sink_node.inputs()[0].connection, Some((src_id, 0)));
    }

    #[test]
    fn test_disconnect_no_connection() {
        let mut graph = Graph::new();

        let sink = TestOp::new(0.0);
        let sink_id = sink.id;
        graph.add(sink);

        // No connection to disconnect
        let mut cmd = DisconnectCommand::new(sink_id, 0);
        cmd.execute(&mut graph);

        // Should not have executed since there was no connection
        assert!(!cmd.executed);
        assert!(cmd.previous_connection().is_none());
    }
}
