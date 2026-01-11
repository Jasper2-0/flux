//! Demo 4: Compare Operator (Boolean Output)
//!
//! This example demonstrates:
//! - CompareOp with different comparison modes
//! - Boolean output type from float inputs
//! - Value mutation and re-evaluation
//!
//! Run with: `cargo run --example 04_compare_operator`

use flux_core::EvalContext;
use flux_graph::Graph;
use flux_operators::{CompareMode, CompareOp, ConstantOp};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 4: Compare Operator (Bool Output) ║");
    println!("╚════════════════════════════════════════╝\n");

    let mut graph = Graph::new();

    let threshold = graph.add(ConstantOp::new(50.0));
    let value = graph.add(ConstantOp::new(75.0));
    let compare = graph.add(CompareOp::new(CompareMode::GreaterThan));

    // value > threshold?
    graph
        .connect(value, 0, compare, 0)
        .expect("value -> compare.A");
    graph
        .connect(threshold, 0, compare, 1)
        .expect("threshold -> compare.B");

    let ctx = EvalContext::new();
    let result = graph.evaluate(compare, 0, &ctx);
    println!("Is 75 > 50? {:?}\n", result);

    // Change value to 25 and re-evaluate
    if let Some(v) = graph.get_mut_as::<ConstantOp>(value) {
        v.set_value(25.0);
    }
    let result = graph.evaluate(compare, 0, &ctx);
    println!("Is 25 > 50? {:?}\n", result);

    println!("Graph stats: {:?}", graph.stats());
}
