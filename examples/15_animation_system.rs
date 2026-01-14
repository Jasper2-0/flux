//! Demo 10: Animation System
//!
//! This example demonstrates the animation system:
//! - Creating animation curves with keyframes
//! - CurveBuilder for fluent curve creation
//! - Animator with loop modes and time control
//! - Binding curves to operator inputs
//!
//! Run with: `cargo run --example 10_animation_system`

use flux_core::EvalContext;
use flux_graph::animation::{Animator, CurveBuilder, LoopMode};
use flux_graph::Graph;
use flux_operators::{ConstantOp, MultiplyOp};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 10: Animation System              ║");
    println!("╚════════════════════════════════════════╝\n");

    // Create a graph with operators we want to animate
    let mut graph = Graph::new();
    let const_value = graph.add(ConstantOp::new(0.0));
    let multiplier = graph.add(ConstantOp::new(10.0));
    let mul = graph.add(MultiplyOp::new());

    graph
        .connect(const_value, 0, mul, 0)
        .expect("value -> mul.A");
    graph
        .connect(multiplier, 0, mul, 1)
        .expect("mult -> mul.B");

    // Create animation curves
    let opacity_curve = CurveBuilder::named("opacity")
        .keyframe(0.0, 0.0) // Start at 0
        .keyframe(0.5, 1.0) // Ramp up to 1 at halfway
        .keyframe(1.0, 0.5) // Ease down to 0.5 at end
        .build();

    let scale_curve = CurveBuilder::named("scale")
        .keyframe(0.0, 1.0) // Start at 1x
        .keyframe(0.25, 1.5) // Expand
        .keyframe(0.5, 0.8) // Contract
        .keyframe(0.75, 1.2) // Bounce
        .keyframe(1.0, 1.0) // Back to 1x
        .build();

    // Create animator and bind curves to operator inputs
    let mut animator = Animator::with_range(0.0, 1.0);
    animator.set_loop_mode(LoopMode::PingPong);
    animator.add_curve(opacity_curve, const_value, 0); // Animate const_value's input 0
    animator.add_curve(scale_curve, multiplier, 0); // Animate multiplier's input 0

    println!(
        "Animation curves bound to {} operators",
        animator.binding_count()
    );
    println!("Playback range: {:?}", animator.range());
    println!("Loop mode: {:?}\n", animator.loop_mode());

    // Sample animation at different times
    println!("Sampling animation curves:");
    for step in 0..=8 {
        let t = step as f64 * 0.125; // 0.0, 0.125, 0.25, ...
        animator.set_time(t);

        let opacity = animator.sample(const_value, 0).unwrap_or(0.0);
        let scale = animator.sample(multiplier, 0).unwrap_or(1.0);

        println!("  t={:.3}: opacity={:.3}, scale={:.3}", t, opacity, scale);
    }

    // Demonstrate playback simulation
    println!("\nSimulating playback with advance():");
    animator.set_time(0.0);
    animator.play();

    for frame in 0..6 {
        let values = animator.sample_all();
        println!(
            "  Frame {}: time={:.3}, {} animated values",
            frame,
            animator.current_time(),
            values.len()
        );
        animator.advance(0.2); // Advance 0.2 units per frame
    }

    // Evaluate the graph with animated values
    println!("\nEvaluating graph with animation:");
    animator.set_time(0.5);
    let ctx = EvalContext::new();
    let result = graph.evaluate(mul, 0, &ctx);
    println!("At t=0.5, result = {:?}", result);
}
