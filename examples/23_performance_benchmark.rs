//! Example 29: Performance Benchmark
//!
//! This example demonstrates Flux's scaling characteristics with large graphs
//! and compares interpreted vs compiled execution performance.
//!
//! Benchmarks include:
//! - Wide graphs (many parallel branches)
//! - Deep graphs (long computation chains)
//! - Diamond patterns (fan-out/fan-in)
//! - Time-varying sources
//!
//! Run with: cargo run --example 29_performance_benchmark --release

use flux_core::{EvalContext, Value};
use flux_graph::Graph;
use flux_operators::{AddOp, ConstantOp, MultiplyOp, SinOp, SineWaveOp};
use std::time::Instant;

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 29: Performance Benchmark         ║");
    println!("╚════════════════════════════════════════╝\n");

    println!("NOTE: Run with --release for accurate benchmarks!\n");

    bench_wide_graph();
    bench_deep_graph();
    bench_diamond_graph();
    bench_time_varying();

    println!("\n=== Summary ===\n");
    println!("Performance characteristics:");
    println!();
    println!("  Graph Shape Impact:");
    println!("    - Wide graphs: Parallel evaluation, good cache behavior");
    println!("    - Deep graphs: Sequential dependencies, linear scaling");
    println!("    - Diamond graphs: Shared computation, caching benefits");
    println!();
    println!("  Execution Mode Trade-offs:");
    println!("    - Interpreted: Flexible, supports runtime changes");
    println!("    - Compiled: Faster iteration, optimized memory access");
    println!("    - Optimized: Dead code elimination, smaller footprint");
    println!();
    println!("  Best Practices:");
    println!("    - Use compile() for render loops");
    println!("    - Use compile_optimized() for production");
    println!("    - Use evaluate() during graph editing");
}

/// Benchmark a wide graph with many parallel branches
fn bench_wide_graph() {
    println!("=== Part 1: Wide Graph (Parallel Branches) ===\n");

    let branch_counts = [10, 50, 100];

    for &branches in &branch_counts {
        let mut graph = Graph::new();

        // Create source
        let source = graph.add(ConstantOp::new(1.0));

        // Create many parallel branches: source -> sin -> accumulator
        let mut add_ids = Vec::new();

        for i in 0..branches {
            // Each branch: source -> sin -> partial sum
            let sin_op = graph.add(SinOp::new());
            graph.connect(source, 0, sin_op, 0).unwrap();

            // Chain additions to accumulate
            if i == 0 {
                // First branch just uses the sin output
                let add = graph.add(AddOp::new());
                graph.connect(sin_op, 0, add, 0).unwrap();
                graph.set_input_default(add, 1, Value::Float(0.0));
                add_ids.push(add);
            } else {
                // Subsequent branches add to previous result
                let add = graph.add(AddOp::new());
                graph.connect(sin_op, 0, add, 0).unwrap();
                graph.connect(add_ids[i - 1], 0, add, 1).unwrap();
                add_ids.push(add);
            }
        }

        let output = *add_ids.last().unwrap();
        let ctx = EvalContext::new();

        // Warm up
        let _ = graph.evaluate(output, 0, &ctx);

        // Benchmark interpreted
        let iterations = 1000;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = graph.evaluate(output, 0, &ctx);
        }
        let interpreted_us = start.elapsed().as_micros() as f64 / iterations as f64;

        // Compile and benchmark
        let compiled = graph.compile(output, 0).unwrap();
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = compiled.execute(&mut graph, &ctx);
        }
        let compiled_us = start.elapsed().as_micros() as f64 / iterations as f64;

        let speedup = interpreted_us / compiled_us;
        let stats = graph.stats();

        println!(
            "  {:>3} branches: {:>4} nodes | Interpreted: {:>7.1}µs | Compiled: {:>7.1}µs | Speedup: {:>5.2}x",
            branches, stats.node_count, interpreted_us, compiled_us, speedup
        );
    }
    println!();
}

/// Benchmark a deep graph with long chains
fn bench_deep_graph() {
    println!("=== Part 2: Deep Graph (Long Chains) ===\n");

    let chain_lengths = [10, 50, 100];

    for &depth in &chain_lengths {
        let mut graph = Graph::new();

        // Create source
        let source = graph.add(ConstantOp::new(0.5));

        // Create a long chain: source -> mul -> mul -> mul -> ...
        let mut current = source;
        for _ in 0..depth {
            let mul = graph.add(MultiplyOp::new());
            graph.connect(current, 0, mul, 0).unwrap();
            // Multiply by 1.01 each step (slight growth)
            graph.set_input_default(mul, 1, Value::Float(1.01));
            current = mul;
        }

        let output = current;
        let ctx = EvalContext::new();

        // Warm up
        let _ = graph.evaluate(output, 0, &ctx);

        // Benchmark interpreted
        let iterations = 1000;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = graph.evaluate(output, 0, &ctx);
        }
        let interpreted_us = start.elapsed().as_micros() as f64 / iterations as f64;

        // Compile and benchmark
        let compiled = graph.compile(output, 0).unwrap();
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = compiled.execute(&mut graph, &ctx);
        }
        let compiled_us = start.elapsed().as_micros() as f64 / iterations as f64;

        let speedup = interpreted_us / compiled_us;
        let result = graph.evaluate(output, 0, &ctx).unwrap();

        println!(
            "  Depth {:>3}: Interpreted: {:>7.1}µs | Compiled: {:>7.1}µs | Speedup: {:>5.2}x | Result: {:.4}",
            depth, interpreted_us, compiled_us, speedup, result.as_float().unwrap_or(0.0)
        );
    }
    println!();
}

/// Benchmark diamond pattern (fan-out/fan-in)
fn bench_diamond_graph() {
    println!("=== Part 3: Diamond Pattern (Fan-out/Fan-in) ===\n");

    let fan_sizes = [4, 8, 16];

    for &fan in &fan_sizes {
        let mut graph = Graph::new();

        // Single source
        let source = graph.add(ConstantOp::new(1.0));

        // Fan out: source feeds N sin operations
        let mut sin_ids = Vec::new();
        for _ in 0..fan {
            let sin = graph.add(SinOp::new());
            graph.connect(source, 0, sin, 0).unwrap();
            sin_ids.push(sin);
        }

        // Fan in: accumulate all sin outputs
        let mut current_sum = sin_ids[0];
        for i in 1..fan {
            let add = graph.add(AddOp::new());
            graph.connect(current_sum, 0, add, 0).unwrap();
            graph.connect(sin_ids[i], 0, add, 1).unwrap();
            current_sum = add;
        }

        let output = current_sum;
        let ctx = EvalContext::new();

        // Warm up
        let _ = graph.evaluate(output, 0, &ctx);

        // Benchmark interpreted
        let iterations = 1000;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = graph.evaluate(output, 0, &ctx);
        }
        let interpreted_us = start.elapsed().as_micros() as f64 / iterations as f64;

        // Compile both ways
        let compiled = graph.compile(output, 0).unwrap();
        let compiled_opt = graph.compile_optimized(output, 0).unwrap();

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = compiled.execute(&mut graph, &ctx);
        }
        let compiled_us = start.elapsed().as_micros() as f64 / iterations as f64;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = compiled_opt.execute(&mut graph, &ctx);
        }
        let optimized_us = start.elapsed().as_micros() as f64 / iterations as f64;

        println!(
            "  Fan {:>2}: Interpreted: {:>6.1}µs | Compiled: {:>6.1}µs | Optimized: {:>6.1}µs | Commands: {}/{}",
            fan, interpreted_us, compiled_us, optimized_us,
            compiled.command_count(), compiled_opt.command_count()
        );
    }
    println!();
}

/// Benchmark with time-varying sources
fn bench_time_varying() {
    println!("=== Part 4: Time-Varying Sources ===\n");

    let mut graph = Graph::new();

    // Create oscillating sources at different frequencies
    let osc1 = graph.add(SineWaveOp::new());
    let osc2 = graph.add(SineWaveOp::new());
    let osc3 = graph.add(SineWaveOp::new());

    // Configure frequencies: 1Hz, 2Hz, 3Hz
    graph.set_input_default(osc1, 0, Value::Float(1.0));
    graph.set_input_default(osc2, 0, Value::Float(2.0));
    graph.set_input_default(osc3, 0, Value::Float(3.0));

    // Combine: (osc1 + osc2) * osc3
    let add = graph.add(AddOp::new());
    graph.connect(osc1, 0, add, 0).unwrap();
    graph.connect(osc2, 0, add, 1).unwrap();

    let mul = graph.add(MultiplyOp::new());
    graph.connect(add, 0, mul, 0).unwrap();
    graph.connect(osc3, 0, mul, 1).unwrap();

    let output = mul;

    // Simulate 60fps for 1 second (60 frames)
    let frames = 60;
    let dt = 1.0 / 60.0;

    println!("  Simulating {} frames at 60fps with time-varying oscillators\n", frames);

    // Benchmark interpreted
    let mut ctx = EvalContext::new();
    let start = Instant::now();
    for frame in 0..frames {
        ctx.time = frame as f64 * dt;
        let _ = graph.evaluate(output, 0, &ctx);
    }
    let interpreted_total = start.elapsed();

    // Benchmark compiled
    let compiled = graph.compile(output, 0).unwrap();
    ctx.time = 0.0;
    let start = Instant::now();
    for frame in 0..frames {
        ctx.time = frame as f64 * dt;
        let _ = compiled.execute(&mut graph, &ctx);
    }
    let compiled_total = start.elapsed();

    let interpreted_per_frame = interpreted_total.as_micros() as f64 / frames as f64;
    let compiled_per_frame = compiled_total.as_micros() as f64 / frames as f64;
    let speedup = interpreted_per_frame / compiled_per_frame;

    println!("  Per-frame times:");
    println!("    Interpreted: {:>7.1}µs/frame ({:.1}ms total)", interpreted_per_frame, interpreted_total.as_millis());
    println!("    Compiled:    {:>7.1}µs/frame ({:.1}ms total)", compiled_per_frame, compiled_total.as_millis());
    println!("    Speedup:     {:>5.2}x", speedup);

    // Show some sample output values
    println!("\n  Sample output values:");
    println!("  {:>8}  {:>10}", "Time", "Output");
    println!("  {:->8}  {:->10}", "", "");

    ctx.time = 0.0;
    for i in 0..6 {
        ctx.time = i as f64 * 0.1;
        let result = compiled.execute(&mut graph, &ctx);
        println!("  {:>8.2}  {:>+10.4}", ctx.time, result.as_float().unwrap_or(0.0));
    }
}
