//! Demo 13: Bypass System
//!
//! This example demonstrates the bypass system for operators:
//! - Checking bypass capability based on input/output types
//! - Understanding bypass pairs (matching input->output types)
//! - Different bypass types (Pass, Zero, etc.)
//! - BypassState management
//!
//! Run with: `cargo run --example 13_bypass_system`

use flux_core::{InputPort, OutputPort};
use flux_graph::bypass::{check_bypassable, BypassState};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 13: Bypass System                 ║");
    println!("╚════════════════════════════════════════╝\n");

    // Check bypass capability of Add operator (float in, float out)
    let add_inputs = vec![InputPort::float("A", 0.0), InputPort::float("B", 0.0)];
    let add_outputs = vec![OutputPort::float("Result")];

    let add_bypass_info = check_bypassable(&add_inputs, &add_outputs);
    println!("Add operator bypass info:");
    println!("  Can bypass: {}", add_bypass_info.can_bypass);
    println!("  Primary pair: {:?}", add_bypass_info.primary_pair());
    println!("  Bypass type: {:?}", add_bypass_info.bypass_type);
    println!("  All pairs: {} found", add_bypass_info.bypass_pairs.len());

    // Check bypass for Compare operator (float in, bool out)
    let cmp_inputs = vec![InputPort::float("A", 0.0), InputPort::float("B", 0.0)];
    let cmp_outputs = vec![OutputPort::bool("Result")];

    let cmp_bypass_info = check_bypassable(&cmp_inputs, &cmp_outputs);
    println!("\nCompare operator bypass info:");
    println!("  Can bypass: {}", cmp_bypass_info.can_bypass);
    println!("  (Float inputs don't match Bool output)");

    // Check bypass for Vec3 operator
    let vec3_inputs = vec![InputPort::vec3("Position", [0.0, 0.0, 0.0])];
    let vec3_outputs = vec![OutputPort::vec3("Result")];

    let vec3_bypass_info = check_bypassable(&vec3_inputs, &vec3_outputs);
    println!("\nVec3 operator bypass info:");
    println!("  Can bypass: {}", vec3_bypass_info.can_bypass);
    println!("  Bypass type: {:?}", vec3_bypass_info.bypass_type);

    // Demonstrate bypass state
    if let Some(mut state) = BypassState::from_info(&add_bypass_info) {
        println!("\nBypass state for Add operator:");
        println!("  Initial: enabled={}", state.enabled);
        state.enable();
        println!("  After enable(): enabled={}", state.enabled);
        state.toggle();
        println!("  After toggle(): enabled={}", state.enabled);
    }
}
