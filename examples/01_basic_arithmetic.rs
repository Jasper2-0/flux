//! Demo 1: Basic Arithmetic with Lazy Evaluation
//!
//! This example demonstrates the core graph evaluation system:
//! - Creating operators (ConstantOp, AddOp, MultiplyOp)
//! - Connecting operators to form a computation graph
//! - Lazy evaluation with dirty flag tracking
//! - Modifying operator values and re-evaluating
//!
//! Run with: `cargo run --example 01_basic_arithmetic`

use flux_core::EvalContext;
use flux_graph::Graph;
use flux_operators::{AddOp, ConstantOp, MultiplyOp};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 1: Basic Arithmetic + Lazy Eval   ║");
    println!("╚════════════════════════════════════════╝\n");

    let mut graph = Graph::new();

    // Create operators
    let const_a = graph.add(ConstantOp::new(5.0));
    let const_b = graph.add(ConstantOp::new(3.0));
    let const_c = graph.add(ConstantOp::new(2.0));
    let add = graph.add(AddOp::new());
    let multiply = graph.add(MultiplyOp::new());

    // Connect: (A + B) * C
    //
    //  const_a (5) ──┐
    //                ├──▶ add ──┐
    //  const_b (3) ──┘          ├──▶ multiply ──▶ output
    //  const_c (2) ─────────────┘
    //
    graph
        .connect(const_a, 0, add, 0)
        .expect("connect A -> Add.A");
    graph
        .connect(const_b, 0, add, 1)
        .expect("connect B -> Add.B");
    graph
        .connect(add, 0, multiply, 0)
        .expect("connect Add -> Multiply.A");
    graph
        .connect(const_c, 0, multiply, 1)
        .expect("connect C -> Multiply.B");

    let mut ctx = EvalContext::new();

    println!("--- First Evaluation ---");
    let result = graph.evaluate(multiply, 0, &ctx);
    println!("Result: (5 + 3) * 2 = {:?}\n", result);

    println!("--- Second Evaluation (no changes, should skip) ---");
    ctx.advance(0.016);
    let result = graph.evaluate(multiply, 0, &ctx);
    println!("Result: {:?}\n", result);

    println!("--- Changing A from 5 to 10 ---\n");
    if let Some(constant) = graph.get_mut_as::<ConstantOp>(const_a) {
        constant.set_value(10.0);
    }

    println!("--- Third Evaluation (A changed, should recompute) ---");
    ctx.advance(0.016);
    let result = graph.evaluate(multiply, 0, &ctx);
    println!("Result: (10 + 3) * 2 = {:?}\n", result);

    println!("Graph stats: {:?}", graph.stats());
}
