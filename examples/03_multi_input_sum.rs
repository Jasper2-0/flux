//! Demo 3: Multi-Input Sum (Variadic)
//!
//! This example demonstrates variadic inputs:
//! - SumOp with multiple connections to the same input
//! - Multi-input port behavior
//!
//! Run with: `cargo run --example 03_multi_input_sum`

use flux_core::EvalContext;
use flux_graph::Graph;
use flux_operators::{ConstantOp, SumOp};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 3: Multi-Input Sum (Variadic)     ║");
    println!("╚════════════════════════════════════════╝\n");

    let mut graph = Graph::new();

    // Create multiple constants and sum them all
    let v1 = graph.add(ConstantOp::new(10.0));
    let v2 = graph.add(ConstantOp::new(20.0));
    let v3 = graph.add(ConstantOp::new(30.0));
    let v4 = graph.add(ConstantOp::new(40.0));
    let sum = graph.add(SumOp::new());

    // Connect all to the Sum's multi-input slot (index 0)
    graph.connect(v1, 0, sum, 0).expect("v1 -> sum");
    graph.connect(v2, 0, sum, 0).expect("v2 -> sum");
    graph.connect(v3, 0, sum, 0).expect("v3 -> sum");
    graph.connect(v4, 0, sum, 0).expect("v4 -> sum");

    let ctx = EvalContext::new();
    let result = graph.evaluate(sum, 0, &ctx);
    println!("Sum of 10 + 20 + 30 + 40 = {:?}\n", result);

    println!("Graph stats: {:?}", graph.stats());
}
