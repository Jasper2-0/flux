//! Example 27: Diamond Dependency Pattern
//!
//! This example demonstrates the classic "diamond dependency" graph pattern
//! where multiple computation paths diverge from a single source and later
//! reconverge. This is a fundamental pattern in dataflow systems and tests:
//!
//! - Evaluation order and topological sorting
//! - Caching effectiveness with shared ancestors
//! - How dirty flag propagation works through diamond patterns
//!
//! Run with: cargo run --example 27_diamond_dependency

use flux_core::EvalContext;
use flux_graph::Graph;
use flux_operators::{
    AddOp, ConstantOp, MultiplyOp, SinOp, CosOp, SqrtOp,
};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 27: Diamond Dependency Pattern    ║");
    println!("╚════════════════════════════════════════╝\n");

    demo_simple_diamond();
    demo_multi_branch_diamond();
    demo_caching_behavior();

    println!("\n=== Summary ===\n");
    println!("Diamond dependency pattern insights:");
    println!();
    println!("  Graph Structure:");
    println!("    - Single source feeds multiple branches");
    println!("    - Branches process data independently");
    println!("    - Results reconverge at a single sink");
    println!();
    println!("  Evaluation Properties:");
    println!("    - Each node computed exactly once per evaluation");
    println!("    - Topological sort ensures correct order");
    println!("    - Shared ancestors are not re-computed");
    println!();
    println!("  Dirty Flag Propagation:");
    println!("    - Source change invalidates all downstream");
    println!("    - Branch-local changes only affect that path");
    println!("    - Sink is dirty if ANY upstream path is dirty");
}

/// Simple diamond: one source, two transforms, one sink
fn demo_simple_diamond() {
    println!("=== Part 1: Simple Diamond Pattern ===\n");

    let mut graph = Graph::new();

    //           ┌──── sin ────┐
    //  source ──┤              ├──► add ──► result
    //           └──── cos ────┘

    let source = graph.add(ConstantOp::new(1.0)); // ~57 degrees in radians
    let sin_op = graph.add(SinOp::new());
    let cos_op = graph.add(CosOp::new());
    let add = graph.add(AddOp::new());

    // Connect diamond
    graph.connect(source, 0, sin_op, 0).unwrap();
    graph.connect(source, 0, cos_op, 0).unwrap();
    graph.connect(sin_op, 0, add, 0).unwrap();
    graph.connect(cos_op, 0, add, 1).unwrap();

    println!("Graph structure:");
    println!("           ┌──── Sin ────┐");
    println!("  Source ──┤              ├──► Add ──► Result");
    println!("           └──── Cos ────┘");
    println!();

    let ctx = EvalContext::new();

    // Evaluate and show intermediate values
    let source_val = graph.evaluate(source, 0, &ctx).unwrap().as_float().unwrap();
    let sin_val = graph.evaluate(sin_op, 0, &ctx).unwrap().as_float().unwrap();
    let cos_val = graph.evaluate(cos_op, 0, &ctx).unwrap().as_float().unwrap();
    let result = graph.evaluate(add, 0, &ctx).unwrap().as_float().unwrap();

    println!("Evaluation at source = 1.0 radians:");
    println!("  Source:     {:.4}", source_val);
    println!("  Sin(1.0):   {:.4}", sin_val);
    println!("  Cos(1.0):   {:.4}", cos_val);
    println!("  Sin + Cos:  {:.4}", result);
    println!();

    // Mathematical verification
    let expected = 1.0_f32.sin() + 1.0_f32.cos();
    println!("Verification: sin(1) + cos(1) = {:.4}", expected);
    println!("Match: {}", if (result - expected).abs() < 0.001 { "YES" } else { "NO" });

    let stats = graph.stats();
    println!("\nGraph: {} nodes, {} connections", stats.node_count, stats.connection_count);
}

/// Multi-branch diamond with 4 parallel paths
fn demo_multi_branch_diamond() {
    println!("\n=== Part 2: Multi-Branch Diamond ===\n");

    let mut graph = Graph::new();

    //           ┌──── ×2  ────┐
    //           ├──── ×3  ────┤
    //  source ──┤              ├──► sum ──► result
    //           ├──── ×5  ────┤
    //           └──── ×7  ────┘

    let source = graph.add(ConstantOp::new(10.0));

    // Create 4 multiplier branches
    let mult2 = graph.add(ConstantOp::new(2.0));
    let mult3 = graph.add(ConstantOp::new(3.0));
    let mult5 = graph.add(ConstantOp::new(5.0));
    let mult7 = graph.add(ConstantOp::new(7.0));

    let branch1 = graph.add(MultiplyOp::new());
    let branch2 = graph.add(MultiplyOp::new());
    let branch3 = graph.add(MultiplyOp::new());
    let branch4 = graph.add(MultiplyOp::new());

    // Reconvergence through cascaded adds
    let add12 = graph.add(AddOp::new());
    let add34 = graph.add(AddOp::new());
    let final_add = graph.add(AddOp::new());

    // Connect source to all branches
    graph.connect(source, 0, branch1, 0).unwrap();
    graph.connect(source, 0, branch2, 0).unwrap();
    graph.connect(source, 0, branch3, 0).unwrap();
    graph.connect(source, 0, branch4, 0).unwrap();

    // Connect multipliers
    graph.connect(mult2, 0, branch1, 1).unwrap();
    graph.connect(mult3, 0, branch2, 1).unwrap();
    graph.connect(mult5, 0, branch3, 1).unwrap();
    graph.connect(mult7, 0, branch4, 1).unwrap();

    // Connect reconvergence
    graph.connect(branch1, 0, add12, 0).unwrap();
    graph.connect(branch2, 0, add12, 1).unwrap();
    graph.connect(branch3, 0, add34, 0).unwrap();
    graph.connect(branch4, 0, add34, 1).unwrap();
    graph.connect(add12, 0, final_add, 0).unwrap();
    graph.connect(add34, 0, final_add, 1).unwrap();

    println!("Graph structure: 4 parallel multiplication branches");
    println!();
    println!("           ┌──── ×2 ────┐");
    println!("           ├──── ×3 ────┤");
    println!("  Source ──┤             ├──► Sum ──► Result");
    println!("           ├──── ×5 ────┤");
    println!("           └──── ×7 ────┘");
    println!();

    let ctx = EvalContext::new();

    // Evaluate all branches
    let b1 = graph.evaluate(branch1, 0, &ctx).unwrap().as_float().unwrap();
    let b2 = graph.evaluate(branch2, 0, &ctx).unwrap().as_float().unwrap();
    let b3 = graph.evaluate(branch3, 0, &ctx).unwrap().as_float().unwrap();
    let b4 = graph.evaluate(branch4, 0, &ctx).unwrap().as_float().unwrap();
    let result = graph.evaluate(final_add, 0, &ctx).unwrap().as_float().unwrap();

    println!("Branch evaluations (source = 10.0):");
    println!("  Branch 1 (×2): {:.1}", b1);
    println!("  Branch 2 (×3): {:.1}", b2);
    println!("  Branch 3 (×5): {:.1}", b3);
    println!("  Branch 4 (×7): {:.1}", b4);
    println!("  Final sum:     {:.1}", result);
    println!();

    // Verification: 10*(2+3+5+7) = 10*17 = 170
    println!("Expected: 10 × (2+3+5+7) = 10 × 17 = 170");
    println!("Match: {}", if (result - 170.0).abs() < 0.001 { "YES" } else { "NO" });

    let stats = graph.stats();
    println!("\nGraph: {} nodes, {} connections", stats.node_count, stats.connection_count);
}

/// Demonstrate caching and re-evaluation behavior
fn demo_caching_behavior() {
    println!("\n=== Part 3: Caching & Re-evaluation ===\n");

    let mut graph = Graph::new();

    //           ┌──── sqrt ───┐
    //  source ──┤              ├──► multiply ──► result
    //           └──── ×2   ───┘

    let source = graph.add(ConstantOp::new(16.0));
    let sqrt_op = graph.add(SqrtOp::new());
    let mult_const = graph.add(ConstantOp::new(2.0));
    let mult_branch = graph.add(MultiplyOp::new());
    let final_mult = graph.add(MultiplyOp::new());

    // Connect diamond
    graph.connect(source, 0, sqrt_op, 0).unwrap();     // sqrt(16) = 4
    graph.connect(source, 0, mult_branch, 0).unwrap();  // 16 * 2 = 32
    graph.connect(mult_const, 0, mult_branch, 1).unwrap();
    graph.connect(sqrt_op, 0, final_mult, 0).unwrap();  // 4 * 32 = 128
    graph.connect(mult_branch, 0, final_mult, 1).unwrap();

    let mut ctx = EvalContext::new();

    println!("Initial evaluation (source = 16.0):");
    let result1 = graph.evaluate(final_mult, 0, &ctx).unwrap().as_float().unwrap();
    println!("  sqrt(16) × (16×2) = 4 × 32 = {:.1}", result1);

    // Change source and re-evaluate
    println!("\nAfter changing source to 25.0:");
    if let Some(op) = graph.get_mut_as::<ConstantOp>(source) {
        op.set_value(25.0);
    }
    ctx.advance(0.016); // Advance time to trigger re-evaluation

    let result2 = graph.evaluate(final_mult, 0, &ctx).unwrap().as_float().unwrap();
    println!("  sqrt(25) × (25×2) = 5 × 50 = {:.1}", result2);

    // Change just one branch
    println!("\nAfter changing multiplier from 2 to 3:");
    if let Some(op) = graph.get_mut_as::<ConstantOp>(mult_const) {
        op.set_value(3.0);
    }
    ctx.advance(0.016);

    let result3 = graph.evaluate(final_mult, 0, &ctx).unwrap().as_float().unwrap();
    println!("  sqrt(25) × (25×3) = 5 × 75 = {:.1}", result3);

    println!("\n--- Caching Observation ---");
    println!();
    println!("In a well-optimized dataflow system:");
    println!("  - When source changes: ALL downstream nodes recompute");
    println!("  - When multiplier changes: Only mult_branch and final recompute");
    println!("  - sqrt(25) is cached and NOT recomputed for the 3rd evaluation");
    println!();
    println!("This demonstrates the efficiency of dataflow evaluation:");
    println!("  Only nodes with changed inputs are recomputed.");
}
