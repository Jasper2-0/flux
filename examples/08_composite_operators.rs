//! Demo 8: Composite Operators (Subgraphs)
//!
//! This example demonstrates:
//! - Creating composite operators that encapsulate subgraphs
//! - Exposing internal inputs and outputs
//! - Using composites as single operators in larger graphs
//!
//! Run with: `cargo run --example 08_composite_operators`

use flux_core::{EvalContext, Operator};
use flux_graph::{CompositeOp, Graph};
use flux_operators::{AddOp, ConstantOp, MultiplyOp};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 8: Composite Operators (Subgraph) ║");
    println!("╚════════════════════════════════════════╝\n");

    // Create a composite operator that computes (A + B) * 2
    // This encapsulates a subgraph with exposed inputs/outputs
    let mut add_and_double = CompositeOp::new("AddAndDouble");

    // Add internal operators
    let const_two = add_and_double.add(ConstantOp::new(2.0));
    let add_internal = add_and_double.add(AddOp::new());
    let mul_internal = add_and_double.add(MultiplyOp::new());

    // Connect internal graph: Add.Result -> Multiply.A, Const(2) -> Multiply.B
    add_and_double
        .connect_internal(add_internal, 0, mul_internal, 0)
        .expect("connect add to multiply");
    add_and_double
        .connect_internal(const_two, 0, mul_internal, 1)
        .expect("connect const to multiply");

    // Expose inputs and outputs
    add_and_double
        .expose_input("A", add_internal, 0)
        .expect("expose input A");
    add_and_double
        .expose_input("B", add_internal, 1)
        .expect("expose input B");
    add_and_double
        .expose_output("Result", mul_internal, 0)
        .expect("expose output");

    println!(
        "Created composite '{}' with {} inputs and {} outputs",
        add_and_double.name(),
        add_and_double.inputs().len(),
        add_and_double.outputs().len()
    );

    // Now use the composite in a larger graph
    let mut graph = Graph::new();
    let input_a = graph.add(ConstantOp::new(7.0));
    let input_b = graph.add(ConstantOp::new(3.0));
    let composite_id = graph.add(add_and_double);

    // Connect inputs to the composite
    graph
        .connect(input_a, 0, composite_id, 0)
        .expect("A -> composite");
    graph
        .connect(input_b, 0, composite_id, 1)
        .expect("B -> composite");

    let ctx = EvalContext::new();
    let result = graph.evaluate(composite_id, 0, &ctx);
    println!("\nResult of (7 + 3) * 2 using composite = {:?}", result);

    println!("\nGraph stats: {:?}", graph.stats());
}
