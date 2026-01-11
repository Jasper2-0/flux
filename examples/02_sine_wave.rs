//! Demo 2: Time-Based SineWave Operator
//!
//! This example demonstrates time-varying operators:
//! - Using SineWaveOp for animated values
//! - Time progression with EvalContext::advance()
//! - Frequency and amplitude modulation
//!
//! Run with: `cargo run --example 02_sine_wave`

use flux_core::EvalContext;
use flux_graph::Graph;
use flux_operators::{AddOp, ConstantOp, SineWaveOp};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 2: Time-Based SineWave Operator   ║");
    println!("╚════════════════════════════════════════╝\n");

    let mut graph = Graph::new();

    // Create a modulated sine wave: sin(t) * 0.5 + 0.5 (normalized to 0-1)
    let freq = graph.add(ConstantOp::new(1.0)); // 1 Hz
    let amp = graph.add(ConstantOp::new(0.5)); // Half amplitude
    let sine = graph.add(SineWaveOp::new());
    let offset = graph.add(ConstantOp::new(0.5)); // DC offset
    let add_offset = graph.add(AddOp::new());

    // Connect: sine(freq, amp) + offset
    graph
        .connect(freq, 0, sine, 0)
        .expect("freq -> sine.Frequency");
    graph
        .connect(amp, 0, sine, 1)
        .expect("amp -> sine.Amplitude");
    graph
        .connect(sine, 0, add_offset, 0)
        .expect("sine -> add.A");
    graph
        .connect(offset, 0, add_offset, 1)
        .expect("offset -> add.B");

    let mut ctx = EvalContext::new();

    println!("Sine wave over time (normalized 0-1):");
    for _ in 0..5 {
        let result = graph.evaluate(add_offset, 0, &ctx);
        println!("  t={:.2}s: {:?}", ctx.time, result);
        ctx.advance(0.25); // Advance 0.25 seconds
    }

    println!("\nGraph stats: {:?}", graph.stats());
}
