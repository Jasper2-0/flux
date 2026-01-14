//! Example 28: State Machine with Flow Control
//!
//! This example demonstrates building a finite state machine using Flux's
//! flow control and state operators. State machines are fundamental for:
//! - Game mechanics (player states, AI behaviors)
//! - UI state management
//! - Animation state control
//! - Protocol implementations
//!
//! Run with: cargo run --example 28_state_machine

use flux_core::{EvalContext, Operator, Value};
use flux_operators::{
    CounterOp, TriggerOp, SwitchOp, ChangedOp,
    IntModuloOp,
};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 28: State Machine Patterns        ║");
    println!("╚════════════════════════════════════════╝\n");

    demo_counter_state();
    demo_edge_detection();
    demo_cyclic_states();

    println!("\n=== Summary ===\n");
    println!("State machine building blocks in Flux:");
    println!();
    println!("  State Storage:");
    println!("    - Counter: Track state index (increments on trigger)");
    println!("    - IntModulo: Wrap state index for cycling");
    println!();
    println!("  Transitions:");
    println!("    - Trigger: Rising edge detection for state changes");
    println!("    - Changed: Detect when any value changes");
    println!("    - Compare: Check if in specific state");
    println!();
    println!("  Output Selection:");
    println!("    - Switch: Binary state selection (if/else)");
    println!("    - Select: Multi-state selection (switch/case)");
    println!();
    println!("  Pattern:");
    println!("    1. Counter holds current state index");
    println!("    2. Compare operators check current state");
    println!("    3. Trigger/Changed detect transition conditions");
    println!("    4. Switch/Select route to state-specific outputs");
}

/// Demo counter-based state tracking
fn demo_counter_state() {
    println!("=== Part 1: Counter-Based State Tracking ===\n");

    let mut counter = CounterOp::new();
    let ctx = EvalContext::new();

    // State names for display
    let states = ["IDLE", "LOADING", "READY", "ACTIVE"];

    println!("State machine with {} states: {:?}\n", states.len(), states);
    println!("Simulating trigger events to advance state:\n");
    println!("  {:>6}  {:>8}  {:>10}", "Event", "Counter", "State");
    println!("  {:->6}  {:->8}  {:->10}", "", "", "");

    // Initial state
    counter.inputs_mut()[0].default = Value::Bool(false); // Trigger
    counter.inputs_mut()[1].default = Value::Bool(false); // Reset
    counter.compute(&ctx, &|_, _| Value::Float(0.0));
    let count = counter.outputs()[0].value.as_int().unwrap_or(0);
    let state = states[(count as usize) % states.len()];
    println!("  {:>6}  {:>8}  {:>10}", "init", count, state);

    // Simulate trigger pulses (rising edge)
    for event in 1..=6 {
        // Pulse: true then false
        counter.inputs_mut()[0].default = Value::Bool(true);
        counter.compute(&ctx, &|_, _| Value::Float(0.0));

        counter.inputs_mut()[0].default = Value::Bool(false);
        counter.compute(&ctx, &|_, _| Value::Float(0.0));

        let count = counter.outputs()[0].value.as_int().unwrap_or(0);
        let state = states[(count as usize) % states.len()];
        println!("  {:>6}  {:>8}  {:>10}", event, count, state);
    }

    // Reset demonstration
    println!("\n  Sending RESET signal...\n");
    counter.inputs_mut()[1].default = Value::Bool(true);
    counter.compute(&ctx, &|_, _| Value::Float(0.0));
    counter.inputs_mut()[1].default = Value::Bool(false);

    let count = counter.outputs()[0].value.as_int().unwrap_or(0);
    let state = states[(count as usize) % states.len()];
    println!("  {:>6}  {:>8}  {:>10}", "reset", count, state);
}

/// Demo edge detection for state transitions
fn demo_edge_detection() {
    println!("\n=== Part 2: Edge Detection for Transitions ===\n");

    let mut trigger = TriggerOp::new();
    let mut changed = ChangedOp::new();
    let ctx = EvalContext::new();

    println!("TriggerOp detects rising edges (false→true transitions)");
    println!("ChangedOp detects any value change\n");

    // Test sequence for Trigger
    let sequence = [false, false, true, true, false, true, false, true];

    println!("  {:>6}  {:>8}  {:>10}", "Input", "Trigger", "Description");
    println!("  {:->6}  {:->8}  {:->10}", "", "", "");

    for &input in sequence.iter() {
        trigger.inputs_mut()[0].default = Value::Bool(input);
        trigger.compute(&ctx, &|_, _| Value::Float(0.0));
        let triggered = trigger.outputs()[0].value.as_bool().unwrap_or(false);

        let desc = if triggered {
            "RISING EDGE!"
        } else if input {
            "(held high)"
        } else {
            "(low)"
        };

        println!("  {:>6}  {:>8}  {:>10}", input, triggered, desc);
    }

    // Test ChangedOp with numeric values
    println!("\n  ChangedOp with numeric values:");
    println!("  {:>8}  {:>8}", "Value", "Changed");
    println!("  {:->8}  {:->8}", "", "");

    let values = [1.0, 1.0, 2.0, 2.0, 2.0, 3.0, 1.0];

    for &val in &values {
        changed.inputs_mut()[0].default = Value::Float(val);
        changed.compute(&ctx, &|_, _| Value::Float(0.0));
        let did_change = changed.outputs()[0].value.as_bool().unwrap_or(false);
        println!("  {:>8.1}  {:>8}", val, did_change);
    }
}

/// Demo cyclic state machine with modulo
fn demo_cyclic_states() {
    println!("\n=== Part 3: Cyclic State Machine ===\n");

    let mut counter = CounterOp::new();
    let mut modulo = IntModuloOp::new();
    let mut switch = SwitchOp::new();

    let ctx = EvalContext::new();
    let num_states = 3;

    println!("Traffic light state machine (3 states):");
    println!("  0 = RED, 1 = YELLOW, 2 = GREEN\n");

    // Configure modulo for 3 states
    modulo.inputs_mut()[1].default = Value::Int(num_states);

    // Configure switch outputs (used for binary demo)
    switch.inputs_mut()[1].default = Value::Float(1.0);  // True branch
    switch.inputs_mut()[2].default = Value::Float(0.0);  // False branch

    println!("  {:>8}  {:>8}  {:>10}  {:>8}", "Tick", "Raw", "State", "Output");
    println!("  {:->8}  {:->8}  {:->10}  {:->8}", "", "", "", "");

    for tick in 0..10 {
        // Pulse the counter
        if tick > 0 {
            counter.inputs_mut()[0].default = Value::Bool(true);
            counter.compute(&ctx, &|_, _| Value::Float(0.0));
            counter.inputs_mut()[0].default = Value::Bool(false);
            counter.compute(&ctx, &|_, _| Value::Float(0.0));
        } else {
            counter.compute(&ctx, &|_, _| Value::Float(0.0));
        }

        let raw_count = counter.outputs()[0].value.as_int().unwrap_or(0);

        // Apply modulo to cycle states
        modulo.inputs_mut()[0].default = Value::Int(raw_count);
        modulo.compute(&ctx, &|_, _| Value::Float(0.0));
        let state_idx = modulo.outputs()[0].value.as_int().unwrap_or(0);

        let state_name = match state_idx {
            0 => "RED",
            1 => "YELLOW",
            2 => "GREEN",
            _ => "???",
        };

        // Binary output: is the light "GO" (green)?
        let is_green = state_idx == 2;
        switch.inputs_mut()[0].default = Value::Bool(is_green);
        switch.compute(&ctx, &|_, _| Value::Float(0.0));
        let output = switch.outputs()[0].value.as_float().unwrap_or(0.0);

        println!("  {:>8}  {:>8}  {:>10}  {:>8.0}", tick, raw_count, state_name, output);
    }

    // State-specific behavior visualization
    println!("\n  State Machine Visualization:");
    println!();
    println!("    ┌────────┐");
    println!("    │  RED   │ ─── tick ───┐");
    println!("    └────────┘             │");
    println!("         ▲                 ▼");
    println!("         │           ┌──────────┐");
    println!("       tick          │  YELLOW  │");
    println!("         │           └──────────┘");
    println!("         │                 │");
    println!("    ┌────────┐             │");
    println!("    │ GREEN  │ ◀── tick ───┘");
    println!("    └────────┘");
}
