//! Undo/Redo Stack for Command History
//!
//! This module provides an [`UndoRedoStack`] that manages command history,
//! enabling undo and redo operations on a graph.
//!
//! # Example
//!
//! ```ignore
//! use flux_graph::{Graph, UndoRedoStack};
//! use flux_graph::commands::AddNodeCommand;
//!
//! let mut graph = Graph::new();
//! let mut history = UndoRedoStack::new();
//!
//! // Execute a command
//! history.execute(&mut graph, AddNodeCommand::new(MyOp::new()));
//!
//! // Undo it
//! history.undo(&mut graph);
//!
//! // Redo it
//! history.redo(&mut graph);
//!
//! // Check state
//! assert!(history.can_undo());
//! assert!(!history.can_redo());
//! ```

use crate::commands::Command;
use crate::graph::Graph;

/// A stack-based undo/redo system for graph commands.
///
/// Commands are stored in a linear history. When a new command is executed,
/// any "future" commands (from previous undos) are discarded.
///
/// # Memory Management
///
/// The stack has an optional maximum size. When exceeded, the oldest
/// commands are discarded. Set to `None` for unlimited history.
#[derive(Debug)]
pub struct UndoRedoStack {
    /// Command history (oldest first)
    history: Vec<Box<dyn Command>>,
    /// Current position in history (next command to undo)
    /// When at history.len(), we're at the present (nothing to redo)
    position: usize,
    /// Maximum history size (None = unlimited)
    max_size: Option<usize>,
    /// Whether the graph has unsaved changes
    dirty: bool,
    /// Position at which the graph was last saved
    saved_position: Option<usize>,
}

impl Default for UndoRedoStack {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoRedoStack {
    /// Create a new empty UndoRedoStack with unlimited history.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            position: 0,
            max_size: None,
            dirty: false,
            saved_position: Some(0),
        }
    }

    /// Create a new UndoRedoStack with a maximum history size.
    ///
    /// When the history exceeds this size, the oldest commands are discarded.
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            history: Vec::new(),
            position: 0,
            max_size: Some(max_size),
            dirty: false,
            saved_position: Some(0),
        }
    }

    /// Execute a command and add it to the history.
    ///
    /// If there are commands after the current position (from previous undos),
    /// they are discarded before adding the new command.
    pub fn execute<C: Command + 'static>(&mut self, graph: &mut Graph, mut command: C) {
        // Execute the command
        command.execute(graph);

        // If we're not at the end of history, truncate future commands
        if self.position < self.history.len() {
            self.history.truncate(self.position);
            // Saved position might now be invalid
            if let Some(saved_pos) = self.saved_position {
                if saved_pos > self.position {
                    self.saved_position = None;
                }
            }
        }

        // Add to history
        self.history.push(Box::new(command));
        self.position = self.history.len();

        // Mark as dirty
        self.dirty = self.saved_position != Some(self.position);

        // Enforce max size
        if let Some(max) = self.max_size {
            while self.history.len() > max {
                self.history.remove(0);
                self.position = self.position.saturating_sub(1);
                // Adjust saved position
                if let Some(saved_pos) = self.saved_position {
                    if saved_pos == 0 {
                        self.saved_position = None;
                    } else {
                        self.saved_position = Some(saved_pos - 1);
                    }
                }
            }
        }
    }

    /// Execute a boxed command and add it to the history.
    pub fn execute_boxed(&mut self, graph: &mut Graph, mut command: Box<dyn Command>) {
        command.execute(graph);

        if self.position < self.history.len() {
            self.history.truncate(self.position);
            if let Some(saved_pos) = self.saved_position {
                if saved_pos > self.position {
                    self.saved_position = None;
                }
            }
        }

        self.history.push(command);
        self.position = self.history.len();
        self.dirty = self.saved_position != Some(self.position);

        if let Some(max) = self.max_size {
            while self.history.len() > max {
                self.history.remove(0);
                self.position = self.position.saturating_sub(1);
                if let Some(saved_pos) = self.saved_position {
                    if saved_pos == 0 {
                        self.saved_position = None;
                    } else {
                        self.saved_position = Some(saved_pos - 1);
                    }
                }
            }
        }
    }

    /// Undo the last command.
    ///
    /// Returns `true` if a command was undone, `false` if there's nothing to undo.
    pub fn undo(&mut self, graph: &mut Graph) -> bool {
        if self.position == 0 {
            return false;
        }

        self.position -= 1;
        self.history[self.position].undo(graph);
        self.dirty = self.saved_position != Some(self.position);

        true
    }

    /// Redo the last undone command.
    ///
    /// Returns `true` if a command was redone, `false` if there's nothing to redo.
    pub fn redo(&mut self, graph: &mut Graph) -> bool {
        if self.position >= self.history.len() {
            return false;
        }

        self.history[self.position].execute(graph);
        self.position += 1;
        self.dirty = self.saved_position != Some(self.position);

        true
    }

    /// Check if there are commands that can be undone.
    pub fn can_undo(&self) -> bool {
        self.position > 0
    }

    /// Check if there are commands that can be redone.
    pub fn can_redo(&self) -> bool {
        self.position < self.history.len()
    }

    /// Get the name of the command that would be undone.
    pub fn undo_name(&self) -> Option<&str> {
        if self.position > 0 {
            Some(self.history[self.position - 1].name())
        } else {
            None
        }
    }

    /// Get the name of the command that would be redone.
    pub fn redo_name(&self) -> Option<&str> {
        if self.position < self.history.len() {
            Some(self.history[self.position].name())
        } else {
            None
        }
    }

    /// Get the number of commands in history.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Get the current position in history.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Check if the graph has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark the current state as saved.
    ///
    /// After calling this, `is_dirty()` will return `false` until
    /// new commands are executed or undo/redo changes the position.
    pub fn mark_saved(&mut self) {
        self.saved_position = Some(self.position);
        self.dirty = false;
    }

    /// Clear all history.
    ///
    /// This is useful when loading a new document.
    pub fn clear(&mut self) {
        self.history.clear();
        self.position = 0;
        self.dirty = false;
        self.saved_position = Some(0);
    }

    /// Get a list of command names for display in an undo history menu.
    ///
    /// Returns (past_commands, future_commands) where each is a slice of names.
    /// Past commands are in reverse order (most recent first).
    pub fn command_names(&self) -> (Vec<&str>, Vec<&str>) {
        let past: Vec<&str> = self.history[..self.position]
            .iter()
            .rev()
            .map(|cmd| cmd.name())
            .collect();

        let future: Vec<&str> = self.history[self.position..]
            .iter()
            .map(|cmd| cmd.name())
            .collect();

        (past, future)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::tests::TestOp;
    use crate::commands::{AddNodeCommand, MacroCommand};

    #[test]
    fn test_undo_redo_basic() {
        let mut graph = Graph::new();
        let mut history = UndoRedoStack::new();

        let op = TestOp::source(1.0);
        let id = op.id;

        assert!(!history.can_undo());
        assert!(!history.can_redo());

        history.execute(&mut graph, AddNodeCommand::new(op));

        assert_eq!(graph.node_count(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        history.undo(&mut graph);

        assert_eq!(graph.node_count(), 0);
        assert!(!history.can_undo());
        assert!(history.can_redo());

        history.redo(&mut graph);

        assert_eq!(graph.node_count(), 1);
        assert!(graph.get(id).is_some());
    }

    #[test]
    fn test_undo_multiple() {
        let mut graph = Graph::new();
        let mut history = UndoRedoStack::new();

        let op1 = TestOp::source(1.0);
        let id1 = op1.id;
        let op2 = TestOp::source(2.0);
        let id2 = op2.id;

        history.execute(&mut graph, AddNodeCommand::new(op1));
        history.execute(&mut graph, AddNodeCommand::new(op2));

        assert_eq!(graph.node_count(), 2);
        assert_eq!(history.history_len(), 2);

        history.undo(&mut graph);
        assert_eq!(graph.node_count(), 1);
        assert!(graph.get(id1).is_some());
        assert!(graph.get(id2).is_none());

        history.undo(&mut graph);
        assert_eq!(graph.node_count(), 0);
    }

    #[test]
    fn test_new_command_clears_redo() {
        let mut graph = Graph::new();
        let mut history = UndoRedoStack::new();

        let op1 = TestOp::source(1.0);
        let op2 = TestOp::source(2.0);
        let op3 = TestOp::source(3.0);

        history.execute(&mut graph, AddNodeCommand::new(op1));
        history.execute(&mut graph, AddNodeCommand::new(op2));

        history.undo(&mut graph);
        assert!(history.can_redo());

        // New command should clear redo stack
        history.execute(&mut graph, AddNodeCommand::new(op3));
        assert!(!history.can_redo());
        assert_eq!(history.history_len(), 2); // op1 + op3
    }

    #[test]
    fn test_max_size() {
        let mut graph = Graph::new();
        let mut history = UndoRedoStack::with_max_size(3);

        for i in 0..5 {
            let op = TestOp::source(i as f32);
            history.execute(&mut graph, AddNodeCommand::new(op));
        }

        assert_eq!(history.history_len(), 3);
        // Should still have all 5 nodes (max size doesn't remove nodes, just history)
        assert_eq!(graph.node_count(), 5);
    }

    #[test]
    fn test_dirty_state() {
        let mut graph = Graph::new();
        let mut history = UndoRedoStack::new();

        assert!(!history.is_dirty());

        history.execute(&mut graph, AddNodeCommand::new(TestOp::source(1.0)));
        assert!(history.is_dirty());

        history.mark_saved();
        assert!(!history.is_dirty());

        history.execute(&mut graph, AddNodeCommand::new(TestOp::source(2.0)));
        assert!(history.is_dirty());

        history.undo(&mut graph);
        assert!(!history.is_dirty()); // Back to saved state

        history.undo(&mut graph);
        assert!(history.is_dirty()); // Before saved state
    }

    #[test]
    fn test_command_names() {
        let mut graph = Graph::new();
        let mut history = UndoRedoStack::new();

        history.execute(&mut graph, AddNodeCommand::new(TestOp::source(1.0)));
        history.execute(&mut graph, AddNodeCommand::new(TestOp::source(2.0)));

        let (past, future) = history.command_names();
        assert_eq!(past, vec!["Add Node", "Add Node"]);
        assert!(future.is_empty());

        history.undo(&mut graph);

        let (past, future) = history.command_names();
        assert_eq!(past, vec!["Add Node"]);
        assert_eq!(future, vec!["Add Node"]);
    }

    #[test]
    fn test_undo_redo_name() {
        let mut graph = Graph::new();
        let mut history = UndoRedoStack::new();

        assert!(history.undo_name().is_none());
        assert!(history.redo_name().is_none());

        history.execute(&mut graph, AddNodeCommand::new(TestOp::source(1.0)));

        assert_eq!(history.undo_name(), Some("Add Node"));
        assert!(history.redo_name().is_none());

        history.undo(&mut graph);

        assert!(history.undo_name().is_none());
        assert_eq!(history.redo_name(), Some("Add Node"));
    }

    #[test]
    fn test_clear() {
        let mut graph = Graph::new();
        let mut history = UndoRedoStack::new();

        history.execute(&mut graph, AddNodeCommand::new(TestOp::source(1.0)));
        history.execute(&mut graph, AddNodeCommand::new(TestOp::source(2.0)));

        history.clear();

        assert_eq!(history.history_len(), 0);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert!(!history.is_dirty());
    }

    #[test]
    fn test_macro_command_undo() {
        let mut graph = Graph::new();
        let mut history = UndoRedoStack::new();

        let op1 = TestOp::source(1.0);
        let op2 = TestOp::source(2.0);
        let id1 = op1.id;
        let id2 = op2.id;

        let mut macro_cmd = MacroCommand::new("Add Two Nodes");
        macro_cmd.push(AddNodeCommand::new(op1));
        macro_cmd.push(AddNodeCommand::new(op2));

        history.execute(&mut graph, macro_cmd);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(history.history_len(), 1); // One macro = one undo step

        history.undo(&mut graph);

        assert_eq!(graph.node_count(), 0);
        assert!(graph.get(id1).is_none());
        assert!(graph.get(id2).is_none());

        history.redo(&mut graph);

        assert_eq!(graph.node_count(), 2);
    }
}
