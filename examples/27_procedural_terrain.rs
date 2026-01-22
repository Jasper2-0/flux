//! Example 24: Procedural Terrain Generator
//!
//! This example demonstrates procedural generation using layered Perlin noise
//! (Fractal Brownian Motion / FBM) to create terrain-like heightmaps.
//!
//! Concepts demonstrated:
//! - Multi-octave noise layering (classic FBM technique)
//! - Fan-out graph patterns (one source feeding multiple branches)
//! - Using PerlinNoise operators at different frequencies
//! - Remapping and clamping values to useful ranges
//!
//! Run with: cargo run --example 24_procedural_terrain

use flux_core::{EvalContext, Operator, Value};
use flux_graph::Graph;
use flux_operators::{
    AddOp, ConstantOp, MultiplyOp, PerlinNoiseOp, RemapOp,
};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 24: Procedural Terrain Generator  ║");
    println!("╚════════════════════════════════════════╝\n");

    demo_single_noise();
    demo_fbm_terrain();
    demo_terrain_grid();

    println!("\n=== Summary ===\n");
    println!("Procedural terrain generation techniques:");
    println!();
    println!("  Single Noise:");
    println!("    - Basic Perlin noise gives smooth, natural variation");
    println!("    - Scale parameter controls feature size");
    println!();
    println!("  Fractal Brownian Motion (FBM):");
    println!("    - Layer multiple noise octaves");
    println!("    - Each octave: 2x frequency, 0.5x amplitude");
    println!("    - Creates natural detail at multiple scales");
    println!();
    println!("  Graph Patterns:");
    println!("    - Fan-out: One coordinate feeds multiple noise samples");
    println!("    - Weighted sum: Combine octaves with decreasing amplitude");
    println!("    - Remap: Normalize final height to useful range");
}

/// Demo basic single-octave Perlin noise
fn demo_single_noise() {
    println!("=== Part 1: Single Perlin Noise ===\n");

    let mut noise = PerlinNoiseOp::new();
    let ctx = EvalContext::new();

    println!("Sampling 2D Perlin noise at various coordinates:\n");
    println!("  {:>8} {:>8}  {:>8}", "X", "Y", "Noise");
    println!("  {:->8} {:->8}  {:->8}", "", "", "");

    for i in 0..8 {
        let x = i as f32 * 0.5;
        let y = 0.0;

        noise.inputs_mut()[0].default = Value::Float(x);
        noise.inputs_mut()[1].default = Value::Float(y);
        noise.inputs_mut()[2].default = Value::Float(1.0); // Scale

        noise.compute(&ctx, &|_, _| Value::Float(0.0));
        let value = noise.outputs()[0].value.as_float().unwrap_or(0.0);

        println!("  {:>8.2} {:>8.2}  {:>8.3}", x, y, value);
    }

    println!("\nNotice how values change smoothly - that's the key property of Perlin noise.");
}

/// Demo multi-octave FBM terrain generation
fn demo_fbm_terrain() {
    println!("\n=== Part 2: Fractal Brownian Motion (FBM) ===\n");

    // Create graph with multiple noise layers
    let mut graph = Graph::new();

    // Input coordinates (would normally be connected to Time or input)
    let x_coord = graph.add(ConstantOp::new(2.5));
    let y_coord = graph.add(ConstantOp::new(1.5));

    // Octave 1: Base frequency (largest features)
    let freq1 = graph.add(ConstantOp::new(1.0));
    let noise1 = graph.add(PerlinNoiseOp::new());
    let amp1 = graph.add(ConstantOp::new(1.0));
    let scaled1 = graph.add(MultiplyOp::new());

    // Octave 2: 2x frequency, 0.5x amplitude
    let freq2 = graph.add(ConstantOp::new(2.0));
    let noise2 = graph.add(PerlinNoiseOp::new());
    let amp2 = graph.add(ConstantOp::new(0.5));
    let scaled2 = graph.add(MultiplyOp::new());

    // Octave 3: 4x frequency, 0.25x amplitude
    let freq3 = graph.add(ConstantOp::new(4.0));
    let noise3 = graph.add(PerlinNoiseOp::new());
    let amp3 = graph.add(ConstantOp::new(0.25));
    let scaled3 = graph.add(MultiplyOp::new());

    // Sum all octaves
    let sum12 = graph.add(AddOp::new());
    let sum_all = graph.add(AddOp::new());

    // Remap to 0-1 range (FBM sums to ~1.75 max)
    let remap = graph.add(RemapOp::new());

    // Connect octave 1
    graph.connect(x_coord, 0, noise1, 0).unwrap();
    graph.connect(y_coord, 0, noise1, 1).unwrap();
    graph.connect(freq1, 0, noise1, 2).unwrap();
    graph.connect(noise1, 0, scaled1, 0).unwrap();
    graph.connect(amp1, 0, scaled1, 1).unwrap();

    // Connect octave 2
    graph.connect(x_coord, 0, noise2, 0).unwrap();
    graph.connect(y_coord, 0, noise2, 1).unwrap();
    graph.connect(freq2, 0, noise2, 2).unwrap();
    graph.connect(noise2, 0, scaled2, 0).unwrap();
    graph.connect(amp2, 0, scaled2, 1).unwrap();

    // Connect octave 3
    graph.connect(x_coord, 0, noise3, 0).unwrap();
    graph.connect(y_coord, 0, noise3, 1).unwrap();
    graph.connect(freq3, 0, noise3, 2).unwrap();
    graph.connect(noise3, 0, scaled3, 0).unwrap();
    graph.connect(amp3, 0, scaled3, 1).unwrap();

    // Sum octaves
    graph.connect(scaled1, 0, sum12, 0).unwrap();
    graph.connect(scaled2, 0, sum12, 1).unwrap();
    graph.connect(sum12, 0, sum_all, 0).unwrap();
    graph.connect(scaled3, 0, sum_all, 1).unwrap();

    // Remap to 0-1 (input range 0-1.75, output 0-1)
    graph.connect(sum_all, 0, remap, 0).unwrap();
    graph.set_input_default(remap, 1, Value::Float(0.0));   // in_min
    graph.set_input_default(remap, 2, Value::Float(1.75));  // in_max
    graph.set_input_default(remap, 3, Value::Float(0.0));   // out_min
    graph.set_input_default(remap, 4, Value::Float(1.0));   // out_max

    let ctx = EvalContext::new();

    println!("FBM combines multiple noise octaves:");
    println!("  Octave 1: freq=1.0, amp=1.00  (large hills)");
    println!("  Octave 2: freq=2.0, amp=0.50  (medium detail)");
    println!("  Octave 3: freq=4.0, amp=0.25  (fine detail)\n");

    // Evaluate individual octaves and sum
    let o1 = graph.evaluate(scaled1, 0, &ctx).unwrap().as_float().unwrap();
    let o2 = graph.evaluate(scaled2, 0, &ctx).unwrap().as_float().unwrap();
    let o3 = graph.evaluate(scaled3, 0, &ctx).unwrap().as_float().unwrap();
    let total = graph.evaluate(sum_all, 0, &ctx).unwrap().as_float().unwrap();
    let final_height = graph.evaluate(remap, 0, &ctx).unwrap().as_float().unwrap();

    println!("At position (2.5, 1.5):");
    println!("  Octave 1 contribution: {:.3}", o1);
    println!("  Octave 2 contribution: {:.3}", o2);
    println!("  Octave 3 contribution: {:.3}", o3);
    println!("  Raw sum:               {:.3}", total);
    println!("  Normalized height:     {:.3}", final_height);

    let stats = graph.stats();
    println!("\nGraph stats: {} nodes, {} connections",
             stats.node_count, stats.connection_count);
}

/// Demo terrain heightmap as ASCII grid
fn demo_terrain_grid() {
    println!("\n=== Part 3: Terrain Heightmap Grid ===\n");

    // Create a simple FBM setup for grid sampling
    let mut noise1 = PerlinNoiseOp::new();
    let mut noise2 = PerlinNoiseOp::new();
    let ctx = EvalContext::new();

    println!("Generating 16x8 terrain heightmap (2-octave FBM):\n");

    let width = 16;
    let height = 8;
    let scale = 0.3;

    // Terrain visualization characters by height
    let terrain_chars = [' ', '.', '-', '~', '+', '*', '#', '@'];

    for y in 0..height {
        print!("  ");
        for x in 0..width {
            let fx = x as f32 * scale;
            let fy = y as f32 * scale;

            // Octave 1
            noise1.inputs_mut()[0].default = Value::Float(fx);
            noise1.inputs_mut()[1].default = Value::Float(fy);
            noise1.inputs_mut()[2].default = Value::Float(1.0);
            noise1.compute(&ctx, &|_, _| Value::Float(0.0));
            let v1 = noise1.outputs()[0].value.as_float().unwrap_or(0.0);

            // Octave 2
            noise2.inputs_mut()[0].default = Value::Float(fx);
            noise2.inputs_mut()[1].default = Value::Float(fy);
            noise2.inputs_mut()[2].default = Value::Float(2.0);
            noise2.compute(&ctx, &|_, _| Value::Float(0.0));
            let v2 = noise2.outputs()[0].value.as_float().unwrap_or(0.0);

            // Combine: octave1 * 1.0 + octave2 * 0.5
            let combined = v1 + v2 * 0.5;
            let normalized = (combined / 1.5).clamp(0.0, 0.999);

            // Map to character
            let idx = (normalized * terrain_chars.len() as f32) as usize;
            print!("{}", terrain_chars[idx.min(terrain_chars.len() - 1)]);
        }
        println!();
    }

    println!("\nLegend: ' '=deep water, '.'=shallow, '-'=beach, '~'=grass,");
    println!("        '+'=forest, '*'=rock, '#'=mountain, '@'=peak");

    // Show height distribution
    println!("\n--- Height Classification ---\n");

    let mut low = 0;
    let mut mid = 0;
    let mut high = 0;

    for y in 0..height {
        for x in 0..width {
            let fx = x as f32 * scale;
            let fy = y as f32 * scale;

            noise1.inputs_mut()[0].default = Value::Float(fx);
            noise1.inputs_mut()[1].default = Value::Float(fy);
            noise1.inputs_mut()[2].default = Value::Float(1.0);
            noise1.compute(&ctx, &|_, _| Value::Float(0.0));
            let v1 = noise1.outputs()[0].value.as_float().unwrap_or(0.0);

            noise2.inputs_mut()[0].default = Value::Float(fx);
            noise2.inputs_mut()[1].default = Value::Float(fy);
            noise2.inputs_mut()[2].default = Value::Float(2.0);
            noise2.compute(&ctx, &|_, _| Value::Float(0.0));
            let v2 = noise2.outputs()[0].value.as_float().unwrap_or(0.0);

            let combined = (v1 + v2 * 0.5) / 1.5;

            if combined < 0.33 {
                low += 1;
            } else if combined < 0.66 {
                mid += 1;
            } else {
                high += 1;
            }
        }
    }

    let total = (width * height) as f32;
    println!("  Low terrain  (water/beach): {:>3} cells ({:.0}%)", low, low as f32 / total * 100.0);
    println!("  Mid terrain  (grass/forest): {:>3} cells ({:.0}%)", mid, mid as f32 / total * 100.0);
    println!("  High terrain (mountain):     {:>3} cells ({:.0}%)", high, high as f32 / total * 100.0);
}
