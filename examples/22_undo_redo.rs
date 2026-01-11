//! Example 22: Undo/Redo System
//!
//! This example demonstrates the command pattern implementation for undo/redo:
//! - Individual commands (AddNode, RemoveNode, Connect, SetInputDefault)
//! - MacroCommand for grouping operations atomically
//! - UndoRedoStack for history management
//! - Dirty state tracking for "unsaved changes" detection
//!
//! The command pattern is essential for editor integration, allowing users
//! to undo/redo any graph modification.
//!
//! Run with: cargo run --example 22_undo_redo

use flux_core::{EvalContext, Id, InputPort, Operator, OutputPort, Value, ValueType};
use flux_graph::commands::{
    AddNodeCommand, ConnectCommand, MacroCommand, SetInputDefaultCommand,
};
use flux_graph::{Graph, UndoRedoStack};

// =============================================================================
// Test Operators
// =============================================================================

/// Simple constant operator for testing
struct ConstOp {
    id: Id,
    outputs: Vec<OutputPort>,
    value: f32,
}

impl ConstOp {
    fn new(value: f32) -> Self {
        let mut output = OutputPort::new("Value", ValueType::Float);
        output.set(Value::Float(value));
        Self {
            id: Id::new(),
            outputs: vec![output],
            value,
        }
    }
}

impl Operator for ConstOp {
    fn id(&self) -> Id {
        self.id
    }
    fn name(&self) -> &'static str {
        "Const"
    }
    fn inputs(&self) -> &[InputPort] {
        &[]
    }
    fn inputs_mut(&mut self) -> &mut [InputPort] {
        &mut []
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

/// Add operator that sums inputs
struct AddOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl AddOp {
    fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![
                InputPort::new("A", Value::Float(0.0)),
                InputPort::new("B", Value::Float(0.0)),
            ],
            outputs: vec![OutputPort::new("Sum", ValueType::Float)],
        }
    }
}

impl Operator for AddOp {
    fn id(&self) -> Id {
        self.id
    }
    fn name(&self) -> &'static str {
        "Add"
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
    fn compute(&mut self, _ctx: &EvalContext, get_input: &dyn Fn(Id, usize) -> Value) {
        // get_input takes the SOURCE node's ID and output index, not our ID
        // We must check input.connection to find what we're connected to
        let a = match &self.inputs[0].connection {
            Some((node_id, output_idx)) => get_input(*node_id, *output_idx).as_float().unwrap_or(0.0),
            None => self.inputs[0].default.as_float().unwrap_or(0.0),
        };
        let b = match &self.inputs[1].connection {
            Some((node_id, output_idx)) => get_input(*node_id, *output_idx).as_float().unwrap_or(0.0),
            None => self.inputs[1].default.as_float().unwrap_or(0.0),
        };
        self.outputs[0].set(Value::Float(a + b));
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// =============================================================================
// Demonstrations
// =============================================================================

fn demo_basic_undo_redo() {
    println!("=== Basic Undo/Redo ===\n");

    let mut graph = Graph::new();
    let mut history = UndoRedoStack::new();

    println!("Initial state:");
    println!("  Nodes: {}", graph.node_count());
    println!("  Can undo: {}, Can redo: {}", history.can_undo(), history.can_redo());

    // Add a node via command
    let const_op = ConstOp::new(42.0);
    let const_id = const_op.id;
    let cmd = AddNodeCommand::new(const_op);
    history.execute(&mut graph, cmd);

    println!("\nAfter adding node:");
    println!("  Nodes: {}", graph.node_count());
    println!("  Can undo: {}, Can redo: {}", history.can_undo(), history.can_redo());
    println!("  Undo name: {:?}", history.undo_name());

    // Undo it
    history.undo(&mut graph);

    println!("\nAfter undo:");
    println!("  Nodes: {}", graph.node_count());
    println!("  Can undo: {}, Can redo: {}", history.can_undo(), history.can_redo());
    println!("  Redo name: {:?}", history.redo_name());

    // Redo it
    history.redo(&mut graph);

    println!("\nAfter redo:");
    println!("  Nodes: {}", graph.node_count());
    println!("  Node exists: {}", graph.get(const_id).is_some());
}

fn demo_connect_disconnect() {
    println!("\n=== Connect/Disconnect Commands ===\n");

    let mut graph = Graph::new();
    let mut history = UndoRedoStack::new();

    // Add two nodes
    let const_op = ConstOp::new(10.0);
    let const_id = const_op.id;
    history.execute(&mut graph, AddNodeCommand::new(const_op));

    let add_op = AddOp::new();
    let add_id = add_op.id;
    history.execute(&mut graph, AddNodeCommand::new(add_op));

    println!("Created Const(10) and Add nodes");

    // Connect them
    let connect_cmd = ConnectCommand::new(const_id, 0, add_id, 0);
    history.execute(&mut graph, connect_cmd);

    // Clear cache and evaluate to verify connection
    let ctx = EvalContext::new();
    graph.clear_cache();
    let result = graph.evaluate(add_id, 0, &ctx);
    println!("After connect: Add output = {:?}", result);

    // Undo the connection
    history.undo(&mut graph);

    graph.clear_cache();
    let result = graph.evaluate(add_id, 0, &ctx);
    println!("After undo connect: Add output = {:?} (uses default 0.0)", result);

    // Redo the connection
    history.redo(&mut graph);

    graph.clear_cache();
    let result = graph.evaluate(add_id, 0, &ctx);
    println!("After redo connect: Add output = {:?}", result);
}

fn demo_set_input_default() {
    println!("\n=== Set Input Default Command ===\n");

    let mut graph = Graph::new();
    let mut history = UndoRedoStack::new();

    // Add an add node
    let add_op = AddOp::new();
    let add_id = add_op.id;
    history.execute(&mut graph, AddNodeCommand::new(add_op));

    let ctx = EvalContext::new();
    graph.clear_cache();
    let result = graph.evaluate(add_id, 0, &ctx);
    println!("Initial: A=0, B=0, Sum={:?}", result);

    // Change input A's default to 5.0
    let cmd = SetInputDefaultCommand::new(add_id, 0, Value::Float(5.0));
    history.execute(&mut graph, cmd);

    graph.clear_cache();
    let result = graph.evaluate(add_id, 0, &ctx);
    println!("After set A=5: Sum={:?}", result);

    // Change input B's default to 3.0
    let cmd = SetInputDefaultCommand::new(add_id, 1, Value::Float(3.0));
    history.execute(&mut graph, cmd);

    graph.clear_cache();
    let result = graph.evaluate(add_id, 0, &ctx);
    println!("After set B=3: Sum={:?}", result);

    // Undo B change
    history.undo(&mut graph);
    graph.clear_cache();
    let result = graph.evaluate(add_id, 0, &ctx);
    println!("After undo B: Sum={:?}", result);

    // Undo A change
    history.undo(&mut graph);
    graph.clear_cache();
    let result = graph.evaluate(add_id, 0, &ctx);
    println!("After undo A: Sum={:?}", result);
}

fn demo_macro_command() {
    println!("\n=== MacroCommand (Atomic Grouping) ===\n");

    let mut graph = Graph::new();
    let mut history = UndoRedoStack::new();

    // Create a macro that adds two nodes and connects them
    // This is like "Insert Node on Wire" - one undo step for multiple operations
    let mut macro_cmd = MacroCommand::new("Create Connected Pair");

    let const_op = ConstOp::new(100.0);
    let const_id = const_op.id;
    macro_cmd.push(AddNodeCommand::new(const_op));

    let add_op = AddOp::new();
    let add_id = add_op.id;
    macro_cmd.push(AddNodeCommand::new(add_op));

    // Note: We need to set up the connection command with known IDs
    macro_cmd.push(ConnectCommand::new(const_id, 0, add_id, 0));
    macro_cmd.push(SetInputDefaultCommand::new(add_id, 1, Value::Float(50.0)));

    println!("Executing macro with {} commands", macro_cmd.len());
    history.execute(&mut graph, macro_cmd);

    println!("After macro execute:");
    println!("  Nodes: {}", graph.node_count());
    println!("  History length: {}", history.history_len());

    let ctx = EvalContext::new();
    graph.clear_cache();
    let result = graph.evaluate(add_id, 0, &ctx);
    println!("  Result (100 + 50): {:?}", result);

    // Single undo undoes the entire macro
    history.undo(&mut graph);

    println!("\nAfter single undo (undoes all 4 operations):");
    println!("  Nodes: {}", graph.node_count());
    println!("  Can redo: {}", history.can_redo());
    println!("  Redo name: {:?}", history.redo_name());

    // Single redo restores everything
    history.redo(&mut graph);

    println!("\nAfter single redo:");
    println!("  Nodes: {}", graph.node_count());
    graph.clear_cache();
    let result = graph.evaluate(add_id, 0, &ctx);
    println!("  Result: {:?}", result);
}

fn demo_dirty_state() {
    println!("\n=== Dirty State Tracking ===\n");

    let mut graph = Graph::new();
    let mut history = UndoRedoStack::new();

    println!("Initial: dirty = {}", history.is_dirty());

    history.execute(&mut graph, AddNodeCommand::new(ConstOp::new(1.0)));
    println!("After add: dirty = {}", history.is_dirty());

    history.mark_saved();
    println!("After save: dirty = {}", history.is_dirty());

    history.execute(&mut graph, AddNodeCommand::new(ConstOp::new(2.0)));
    println!("After another add: dirty = {}", history.is_dirty());

    history.undo(&mut graph);
    println!("After undo (back to saved state): dirty = {}", history.is_dirty());

    history.undo(&mut graph);
    println!("After another undo (before saved): dirty = {}", history.is_dirty());
}

fn demo_history_menu() {
    println!("\n=== History Menu (Undo/Redo Lists) ===\n");

    let mut graph = Graph::new();
    let mut history = UndoRedoStack::new();

    // Build up some history
    history.execute(&mut graph, AddNodeCommand::new(ConstOp::new(1.0)));
    history.execute(&mut graph, AddNodeCommand::new(ConstOp::new(2.0)));
    history.execute(&mut graph, AddNodeCommand::new(ConstOp::new(3.0)));

    let add_op = AddOp::new();
    let add_id = add_op.id;
    history.execute(&mut graph, AddNodeCommand::new(add_op));

    history.execute(&mut graph, SetInputDefaultCommand::new(add_id, 0, Value::Float(10.0)));

    // Undo a couple
    history.undo(&mut graph);
    history.undo(&mut graph);

    let (past, future) = history.command_names();

    println!("Undo history (most recent first):");
    for (i, name) in past.iter().enumerate() {
        println!("  {}: {}", i + 1, name);
    }

    println!("\nRedo history:");
    for (i, name) in future.iter().enumerate() {
        println!("  {}: {}", i + 1, name);
    }

    println!("\nCurrent position: {} / {}", history.position(), history.history_len());
}

fn demo_max_history_size() {
    println!("\n=== Max History Size ===\n");

    let mut graph = Graph::new();
    let mut history = UndoRedoStack::with_max_size(3);

    println!("Creating history with max size = 3");

    for i in 1..=5 {
        history.execute(&mut graph, AddNodeCommand::new(ConstOp::new(i as f32)));
        println!("After adding node {}: history_len = {}", i, history.history_len());
    }

    println!("\nNote: Oldest commands are discarded when limit exceeded");
    println!("Graph still has all {} nodes (history limits commands, not graph state)", graph.node_count());
}

fn main() {
    println!("Flux Undo/Redo System Example\n");
    println!("This demonstrates the command pattern for reversible graph mutations.\n");

    demo_basic_undo_redo();
    demo_connect_disconnect();
    demo_set_input_default();
    demo_macro_command();
    demo_dirty_state();
    demo_history_menu();
    demo_max_history_size();

    println!("\n=== Summary ===\n");
    println!("Key concepts demonstrated:");
    println!("  1. Command trait - execute() and undo() for reversible operations");
    println!("  2. Individual commands - AddNode, RemoveNode, Connect, SetInputDefault");
    println!("  3. MacroCommand - Group operations for atomic undo");
    println!("  4. UndoRedoStack - History with position tracking and dirty state");
    println!("  5. History limits - Optional max size to bound memory usage");
}
