//! Example 25: Spring Physics Playground
//!
//! This example demonstrates chained spring physics operators to create
//! wave-like propagation effects. Springs are fundamental to smooth
//! animation and physical simulation.
//!
//! Concepts demonstrated:
//! - Spring operator physics (stiffness, damping)
//! - Chaining stateful operators (each maintains velocity/position)
//! - Wave propagation through connected systems
//! - Using oscillators to drive physical systems
//!
//! Run with: cargo run --example 25_spring_physics

use flux_core::{EvalContext, Operator, Value};
use flux_operators::{SineWaveOp, SpringOp};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 25: Spring Physics Playground     ║");
    println!("╚════════════════════════════════════════╝\n");

    demo_single_spring();
    demo_spring_chain();
    demo_spring_parameters();

    println!("\n=== Summary ===\n");
    println!("Spring physics key concepts:");
    println!();
    println!("  Spring Equation: F = -k*x - d*v");
    println!("    k = stiffness (higher = snappier response)");
    println!("    d = damping (higher = less oscillation)");
    println!("    x = displacement from target");
    println!("    v = current velocity");
    println!();
    println!("  Stiffness Effects:");
    println!("    Low (10-50):   Slow, lazy movement");
    println!("    Medium (100):  Responsive, some bounce");
    println!("    High (200+):   Snappy, minimal delay");
    println!();
    println!("  Damping Effects:");
    println!("    Low (1-5):     Lots of oscillation/ringing");
    println!("    Critical (~2*sqrt(k)): No overshoot");
    println!("    High (20+):    Sluggish, over-damped");
    println!();
    println!("  Chain Behavior:");
    println!("    - Each spring follows its predecessor");
    println!("    - Creates wave-like propagation delay");
    println!("    - Great for tentacles, chains, hair physics");
}

/// Demo a single spring tracking a step change
fn demo_single_spring() {
    println!("=== Part 1: Single Spring Response ===\n");

    let mut spring = SpringOp::new();

    // Configure spring
    spring.inputs_mut()[0].default = Value::Float(0.0);   // Target (will change)
    spring.inputs_mut()[1].default = Value::Float(100.0); // Stiffness
    spring.inputs_mut()[2].default = Value::Float(10.0);  // Damping

    println!("Spring parameters: stiffness=100, damping=10");
    println!("Initial: target=0, position=0");
    println!("\nStep change: target jumps to 1.0\n");

    // Simulate step response
    spring.inputs_mut()[0].default = Value::Float(1.0); // Target = 1.0

    println!("  {:>6}  {:>10}  Graph", "Time", "Position");
    println!("  {:->6}  {:->10}  {:->30}", "", "", "");

    let mut ctx = EvalContext::new();
    let dt = 0.033; // ~30fps

    for frame in 0..20 {
        ctx.time = frame as f64 * dt;
        spring.compute(&ctx, &|_, _| Value::Float(0.0));
        let pos = spring.outputs()[0].value.as_float().unwrap_or(0.0);

        // ASCII graph
        let bar_pos = (pos * 25.0).clamp(0.0, 30.0) as usize;
        let bar: String = (0..30).map(|i| {
            if i == 25 { '|' } // Target marker
            else if i == bar_pos { '#' }
            else if i < bar_pos { '=' }
            else { ' ' }
        }).collect();

        println!("  {:>6.2}  {:>10.3}  [{}]", ctx.time, pos, bar);
    }

    println!("\nSpring converges to target (1.0) with slight overshoot.");
}

/// Demo a chain of springs for wave propagation
fn demo_spring_chain() {
    println!("\n=== Part 2: Spring Chain (Wave Propagation) ===\n");

    // Create 4 springs in a chain
    let mut springs = [
        SpringOp::new(),
        SpringOp::new(),
        SpringOp::new(),
        SpringOp::new(),
    ];

    // Configure all springs - higher damping to prevent runaway oscillation
    for spring in &mut springs {
        spring.inputs_mut()[1].default = Value::Float(80.0);  // Stiffness
        spring.inputs_mut()[2].default = Value::Float(15.0);  // Damping (more stable)
    }

    // Drive the chain with a sine wave (SineWaveOp has 3 inputs: Freq, Amp, Phase)
    let mut driver = SineWaveOp::new();
    driver.inputs_mut()[0].default = Value::Float(0.5);  // 0.5 Hz
    driver.inputs_mut()[1].default = Value::Float(1.0);  // Amplitude
    driver.inputs_mut()[2].default = Value::Float(0.0);  // Phase

    println!("Chain of 4 springs, each following the previous.");
    println!("Driver: sine wave at 0.5 Hz\n");
    println!("  {:>6}  {:>8}  {:>8}  {:>8}  {:>8}  {:>8}",
             "Time", "Driver", "S1", "S2", "S3", "S4");
    println!("  {:->6}  {:->8}  {:->8}  {:->8}  {:->8}  {:->8}",
             "", "", "", "", "", "");

    let mut ctx = EvalContext::new();
    let dt = 0.1;

    for frame in 0..25 {
        ctx.time = frame as f64 * dt;

        // Update driver
        driver.compute(&ctx, &|_, _| Value::Float(0.0));
        let driver_val = driver.outputs()[0].value.as_float().unwrap_or(0.0);

        // Chain: spring 1 follows driver, spring 2 follows spring 1, etc.
        springs[0].inputs_mut()[0].default = Value::Float(driver_val);
        springs[0].compute(&ctx, &|_, _| Value::Float(0.0));
        let s1 = springs[0].outputs()[0].value.as_float().unwrap_or(0.0);

        springs[1].inputs_mut()[0].default = Value::Float(s1);
        springs[1].compute(&ctx, &|_, _| Value::Float(0.0));
        let s2 = springs[1].outputs()[0].value.as_float().unwrap_or(0.0);

        springs[2].inputs_mut()[0].default = Value::Float(s2);
        springs[2].compute(&ctx, &|_, _| Value::Float(0.0));
        let s3 = springs[2].outputs()[0].value.as_float().unwrap_or(0.0);

        springs[3].inputs_mut()[0].default = Value::Float(s3);
        springs[3].compute(&ctx, &|_, _| Value::Float(0.0));
        let s4 = springs[3].outputs()[0].value.as_float().unwrap_or(0.0);

        // Safety check - array has 4 elements (indices 0-3)
        let _ = &springs[0..4]; // Compile-time guarantee

        // Visual representation
        let visual = format_chain_visual(driver_val, s1, s2, s3, s4);

        println!("  {:>6.1}  {:>+8.3}  {:>+8.3}  {:>+8.3}  {:>+8.3}  {:>+8.3}  {}",
                 ctx.time, driver_val, s1, s2, s3, s4, visual);
    }

    println!("\nNotice how the wave propagates through the chain with delay.");
}

/// Demo different spring parameter combinations
fn demo_spring_parameters() {
    println!("\n=== Part 3: Spring Parameter Comparison ===\n");

    println!("Comparing spring responses with different parameters.\n");
    println!("All springs start at 0, target jumps to 1.0 at t=0.\n");

    let configs = [
        ("Underdamped (bouncy)", 100.0, 5.0),
        ("Critically damped", 100.0, 20.0),
        ("Overdamped (sluggish)", 100.0, 40.0),
        ("Stiff + light damping", 300.0, 10.0),
    ];

    for (name, stiffness, damping) in configs {
        println!("--- {} (k={}, d={}) ---", name, stiffness, damping);

        let mut spring = SpringOp::new();
        spring.inputs_mut()[0].default = Value::Float(1.0);
        spring.inputs_mut()[1].default = Value::Float(stiffness);
        spring.inputs_mut()[2].default = Value::Float(damping);

        let mut ctx = EvalContext::new();
        let dt = 0.05;

        print!("  ");
        for frame in 0..15 {
            ctx.time = frame as f64 * dt;
            spring.compute(&ctx, &|_, _| Value::Float(0.0));
            let pos = spring.outputs()[0].value.as_float().unwrap_or(0.0);

            // Simple character visualization
            let c = if pos < 0.2 { '_' }
                   else if pos < 0.5 { '.' }
                   else if pos < 0.8 { '-' }
                   else if pos < 1.1 { '=' }
                   else if pos < 1.3 { '+' }
                   else { '*' };
            print!("{}", c);
        }
        println!(" (target: =)");
    }

    println!("\nLegend: _=0-0.2, .=0.2-0.5, -=0.5-0.8, ==0.8-1.1, +=1.1-1.3, *=>1.3");
}

/// Format a visual representation of chain positions
fn format_chain_visual(d: f32, s1: f32, s2: f32, s3: f32, s4: f32) -> String {
    fn pos_char(v: f32) -> char {
        if v < -0.5 { 'v' }
        else if v < 0.0 { '-' }
        else if v < 0.5 { '+' }
        else { '^' }
    }

    format!("[D:{} 1:{} 2:{} 3:{} 4:{}]",
            pos_char(d), pos_char(s1), pos_char(s2), pos_char(s3), pos_char(s4))
}
