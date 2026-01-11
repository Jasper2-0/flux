//! Demo 6: Type Mismatch Error Handling
//!
//! This example demonstrates type safety in the graph:
//! - Type validation during connection
//! - Error handling for incompatible types
//! - Bool output cannot connect to Float input
//!
//! Run with: `cargo run --example 06_type_validation`

use flux_graph::Graph;
use flux_operators::{AddOp, CompareOp};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 6: Type Mismatch Error Handling   ║");
    println!("╚════════════════════════════════════════╝\n");

    let mut graph = Graph::new();

    let bool_op = graph.add(CompareOp::less_than());
    let float_op = graph.add(AddOp::new());

    // Try to connect bool output to float input (should fail)
    println!("Attempting to connect Bool output -> Float input...");
    let result = graph.connect(bool_op, 0, float_op, 0);
    match result {
        Ok(_) => println!("Connection succeeded (unexpected!)"),
        Err(e) => println!("Type mismatch error (expected): {}\n", e),
    }

    println!("Graph stats: {:?}", graph.stats());
}
