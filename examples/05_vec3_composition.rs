//! Demo 5: Vec3 Composition
//!
//! This example demonstrates vector type operations:
//! - Vec3ComposeOp to create Vec3 from floats
//! - Animated vector components using SineWaveOp
//! - Different frequencies per axis
//!
//! Run with: `cargo run --example 05_vec3_composition`

use flux_core::EvalContext;
use flux_graph::Graph;
use flux_operators::{ConstantOp, SineWaveOp, Vec3ComposeOp};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 5: Vec3 Composition               ║");
    println!("╚════════════════════════════════════════╝\n");

    let mut graph = Graph::new();

    // Create animated position using sine waves
    let x_sine = graph.add(SineWaveOp::new());
    let y_const = graph.add(ConstantOp::new(0.0));
    let z_sine = graph.add(SineWaveOp::new());
    let vec3 = graph.add(Vec3ComposeOp::new());

    // Set up different frequencies for X and Z
    let freq_x = graph.add(ConstantOp::new(1.0));
    let freq_z = graph.add(ConstantOp::new(0.5));

    graph
        .connect(freq_x, 0, x_sine, 0)
        .expect("freq_x -> x_sine");
    graph
        .connect(freq_z, 0, z_sine, 0)
        .expect("freq_z -> z_sine");
    graph.connect(x_sine, 0, vec3, 0).expect("x_sine -> vec3.X");
    graph
        .connect(y_const, 0, vec3, 1)
        .expect("y_const -> vec3.Y");
    graph.connect(z_sine, 0, vec3, 2).expect("z_sine -> vec3.Z");

    let mut ctx = EvalContext::new();

    println!("Animated position vector over time:");
    for _ in 0..4 {
        let result = graph.evaluate(vec3, 0, &ctx);
        println!("  t={:.2}s: {:?}", ctx.time, result);
        ctx.advance(0.5);
    }

    println!("\nGraph stats: {:?}", graph.stats());
}
