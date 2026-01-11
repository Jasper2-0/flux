//! ConnectCommand - Connect two ports in the graph

use flux_core::Id;

use super::Command;
use crate::graph::Graph;

/// Command to connect an output port to an input port.
///
/// On execute, the connection is made (possibly inserting a conversion node).
/// On undo, the connection is removed (and any conversion node is removed).
#[derive(Debug)]
pub struct ConnectCommand {
    /// Source node ID
    source_node: Id,
    /// Source output port index
    source_output: usize,
    /// Target node ID
    target_node: Id,
    /// Target input port index
    target_input: usize,
    /// Previous connection on the target input (for undo)
    previous_connection: Option<(Id, usize)>,
    /// Conversion node inserted by auto-conversion (if any)
    conversion_node: Option<Id>,
    /// Whether the command was successfully executed
    executed: bool,
}

impl ConnectCommand {
    /// Create a new ConnectCommand.
    pub fn new(
        source_node: Id,
        source_output: usize,
        target_node: Id,
        target_input: usize,
    ) -> Self {
        Self {
            source_node,
            source_output,
            target_node,
            target_input,
            previous_connection: None,
            conversion_node: None,
            executed: false,
        }
    }

    /// Get the conversion node ID if one was auto-inserted.
    pub fn conversion_node(&self) -> Option<Id> {
        self.conversion_node
    }
}

impl Command for ConnectCommand {
    fn name(&self) -> &str {
        "Connect"
    }

    fn execute(&mut self, graph: &mut Graph) {
        // Store previous connection for undo
        if let Some(node) = graph.get(self.target_node) {
            if let Some(input) = node.inputs().get(self.target_input) {
                self.previous_connection = input.connection;
            }
        }

        // Make the connection
        match graph.connect(
            self.source_node,
            self.source_output,
            self.target_node,
            self.target_input,
        ) {
            Ok(conversion_id) => {
                self.conversion_node = conversion_id;
                self.executed = true;
            }
            Err(e) => {
                eprintln!("ConnectCommand failed: {}", e);
                self.executed = false;
            }
        }
    }

    fn undo(&mut self, graph: &mut Graph) {
        if !self.executed {
            return;
        }

        // Remove the conversion node if one was inserted
        if let Some(conv_id) = self.conversion_node.take() {
            graph.remove(conv_id);
        }

        // Disconnect the target input
        let _ = graph.disconnect(self.target_node, self.target_input);

        // Restore previous connection if there was one
        if let Some((prev_source, prev_output)) = self.previous_connection {
            let _ = graph.connect(prev_source, prev_output, self.target_node, self.target_input);
        }

        self.executed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::tests::TestOp;

    #[test]
    fn test_connect_execute() {
        let mut graph = Graph::new();

        let src = TestOp::source(1.0);
        let src_id = src.id;
        graph.add(src);

        let sink = TestOp::new(0.0);
        let sink_id = sink.id;
        graph.add(sink);

        let mut cmd = ConnectCommand::new(src_id, 0, sink_id, 0);
        cmd.execute(&mut graph);

        // Check connection was made
        let sink_node = graph.get(sink_id).unwrap();
        assert!(sink_node.inputs()[0].connection.is_some());
    }

    #[test]
    fn test_connect_undo() {
        let mut graph = Graph::new();

        let src = TestOp::source(1.0);
        let src_id = src.id;
        graph.add(src);

        let sink = TestOp::new(0.0);
        let sink_id = sink.id;
        graph.add(sink);

        let mut cmd = ConnectCommand::new(src_id, 0, sink_id, 0);
        cmd.execute(&mut graph);
        cmd.undo(&mut graph);

        // Check connection was removed
        let sink_node = graph.get(sink_id).unwrap();
        assert!(sink_node.inputs()[0].connection.is_none());
    }

    #[test]
    fn test_connect_preserves_previous() {
        let mut graph = Graph::new();

        let src1 = TestOp::source(1.0);
        let src1_id = src1.id;
        graph.add(src1);

        let src2 = TestOp::source(2.0);
        let src2_id = src2.id;
        graph.add(src2);

        let sink = TestOp::new(0.0);
        let sink_id = sink.id;
        graph.add(sink);

        // First connection
        graph.connect(src1_id, 0, sink_id, 0).unwrap();

        // Second connection (replaces first)
        let mut cmd = ConnectCommand::new(src2_id, 0, sink_id, 0);
        cmd.execute(&mut graph);

        // Verify new connection
        let sink_node = graph.get(sink_id).unwrap();
        assert_eq!(sink_node.inputs()[0].connection, Some((src2_id, 0)));

        // Undo should restore first connection
        cmd.undo(&mut graph);
        let sink_node = graph.get(sink_id).unwrap();
        assert_eq!(sink_node.inputs()[0].connection, Some((src1_id, 0)));
    }
}
