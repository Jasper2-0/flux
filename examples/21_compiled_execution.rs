//! Demo 21: Compiled Execution - Two-Tier Runtime
//!
//! This example demonstrates:
//! - Compiling a graph to a CompiledGraph for efficient execution
//! - Comparing interpreted vs compiled execution
//! - Dead code elimination with compile_optimized()
//! - When to use compilation (repeated execution scenarios)
//!
//! # Two-Tier Runtime Model
//!
//! Inspired by Werkkzeug4, the flux graph supports two execution modes:
//!
//! 1. **Interpreted** (`graph.evaluate()`): Flexible, supports dynamic changes
//! 2. **Compiled** (`graph.compile().execute()`): Faster, for repeated execution
//!
//! Compilation benefits:
//! - Pre-computed topological order
//! - Index-based lookups instead of HashMap lookups
//! - Linear memory access pattern
//! - Dead code elimination (with `compile_optimized()`)
//!
//! Run with: `cargo run --example 21_compiled_execution`

use flux_core::{EvalContext, Id, InputPort, Operator, OutputPort, Value, ValueType};
use flux_graph::Graph;
use std::time::Instant;

// =============================================================================
// Test operators for benchmarking
// =============================================================================

/// A source that outputs a constant value
struct ConstOp {
    id: Id,
    outputs: Vec<OutputPort>,
    value: f32,
}

impl ConstOp {
    fn new(value: f32) -> Self {
        let mut output = OutputPort::new("Out", ValueType::Float);
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

/// A simple add operator
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
        let a = if let Some((src, idx)) = self.inputs[0].connection {
            get_input(src, idx)
        } else {
            self.inputs[0].default.clone()
        };
        let b = if let Some((src, idx)) = self.inputs[1].connection {
            get_input(src, idx)
        } else {
            self.inputs[1].default.clone()
        };

        let sum = match (a, b) {
            (Value::Float(x), Value::Float(y)) => x + y,
            _ => 0.0,
        };
        self.outputs[0].set(Value::Float(sum));
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// A multiply operator
struct MulOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl MulOp {
    fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![
                InputPort::new("A", Value::Float(1.0)),
                InputPort::new("B", Value::Float(1.0)),
            ],
            outputs: vec![OutputPort::new("Product", ValueType::Float)],
        }
    }
}

impl Operator for MulOp {
    fn id(&self) -> Id {
        self.id
    }
    fn name(&self) -> &'static str {
        "Mul"
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
        let a = if let Some((src, idx)) = self.inputs[0].connection {
            get_input(src, idx)
        } else {
            self.inputs[0].default.clone()
        };
        let b = if let Some((src, idx)) = self.inputs[1].connection {
            get_input(src, idx)
        } else {
            self.inputs[1].default.clone()
        };

        let product = match (a, b) {
            (Value::Float(x), Value::Float(y)) => x * y,
            _ => 0.0,
        };
        self.outputs[0].set(Value::Float(product));
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 21: Compiled Execution            ║");
    println!("╚════════════════════════════════════════╝\n");

    // =========================================================================
    // Part 1: Basic Compilation
    // =========================================================================
    println!("═══ Part 1: Basic Compilation ═══\n");

    let mut graph = Graph::new();

    // Build: (a + b) * c
    let a = graph.add(ConstOp::new(10.0));
    let b = graph.add(ConstOp::new(20.0));
    let c = graph.add(ConstOp::new(3.0));

    let add_id = {
        let add = AddOp::new();
        let id = add.id;
        graph.add(add);
        id
    };

    let mul_id = {
        let mul = MulOp::new();
        let id = mul.id;
        graph.add(mul);
        id
    };

    graph.connect(a, 0, add_id, 0).unwrap(); // a -> Add.A
    graph.connect(b, 0, add_id, 1).unwrap(); // b -> Add.B
    graph.connect(add_id, 0, mul_id, 0).unwrap(); // Add.Sum -> Mul.A
    graph.connect(c, 0, mul_id, 1).unwrap(); // c -> Mul.B

    println!("Graph: (10 + 20) * 3 = 90");
    println!("Nodes: 5 (3 consts, 1 add, 1 mul)");

    // Compile the graph
    let compiled = graph.compile(mul_id, 0).unwrap();

    println!("\nCompiled graph:");
    println!("  Commands: {}", compiled.command_count());
    println!("  Output slots: {}", compiled.output_count());

    // Execute
    let ctx = EvalContext::new();
    let result = compiled.execute(&mut graph, &ctx);

    println!("\nExecution result: {:?}", result);

    // =========================================================================
    // Part 2: Interpreted vs Compiled
    // =========================================================================
    println!("\n═══ Part 2: Interpreted vs Compiled ═══\n");

    let iterations = 10000;

    // Interpreted execution
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = graph.evaluate(mul_id, 0, &ctx).unwrap();
    }
    let interpreted_time = start.elapsed();

    // Compiled execution
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = compiled.execute(&mut graph, &ctx);
    }
    let compiled_time = start.elapsed();

    println!("Executed {} iterations:", iterations);
    println!("  Interpreted: {:?}", interpreted_time);
    println!("  Compiled:    {:?}", compiled_time);

    if compiled_time < interpreted_time {
        let speedup = interpreted_time.as_nanos() as f64 / compiled_time.as_nanos() as f64;
        println!("  Speedup:     {:.2}x", speedup);
    } else {
        println!("  (No speedup in this small example - overhead dominates)");
    }

    // =========================================================================
    // Part 3: Dead Code Elimination
    // =========================================================================
    println!("\n═══ Part 3: Dead Code Elimination ═══\n");

    // Add an unused branch to the graph
    let unused1 = graph.add(ConstOp::new(999.0));
    let unused2 = graph.add(ConstOp::new(888.0));
    let unused_add_id = {
        let add = AddOp::new();
        let id = add.id;
        graph.add(add);
        id
    };
    graph.connect(unused1, 0, unused_add_id, 0).unwrap();
    graph.connect(unused2, 0, unused_add_id, 1).unwrap();

    println!("Added unused branch: 999 + 888 (not connected to output)");
    println!("Graph now has {} nodes", graph.stats().node_count);

    // Compile without optimization
    let compiled_full = graph.compile(mul_id, 0).unwrap();
    println!("\ncompile() (includes all nodes):");
    println!("  Commands: {}", compiled_full.command_count());

    // Compile with optimization
    let compiled_opt = graph.compile_optimized(mul_id, 0).unwrap();
    println!("\ncompile_optimized() (dead code eliminated):");
    println!("  Commands: {}", compiled_opt.command_count());
    println!("  Eliminated: {} dead nodes", compiled_full.command_count() - compiled_opt.command_count());

    // Both should give the same result
    let result_full = compiled_full.execute(&mut graph, &ctx);
    let result_opt = compiled_opt.execute(&mut graph, &ctx);
    println!("\nBoth produce same result:");
    println!("  Full: {:?}", result_full);
    println!("  Optimized: {:?}", result_opt);

    // =========================================================================
    // Part 4: When to Use Compilation
    // =========================================================================
    println!("\n═══ Part 4: When to Use Compilation ═══\n");

    println!("Use graph.compile() when:");
    println!("  - You'll execute the same graph many times (render loop)");
    println!("  - The graph structure doesn't change frequently");
    println!("  - You want maximum performance for real-time applications");
    println!();
    println!("Use graph.evaluate() when:");
    println!("  - The graph changes frequently (editing)");
    println!("  - You're debugging or prototyping");
    println!("  - You only need occasional evaluation");
    println!();
    println!("Use graph.compile_optimized() when:");
    println!("  - You have unused nodes (disabled branches, debugging nodes)");
    println!("  - You want the smallest possible command buffer");
    println!("  - Memory efficiency matters");
}
