//! Example 23: Flow Control Operators
//!
//! This example demonstrates the flow control operators in flux:
//! - SwitchOp: Select between two values based on condition (if/else)
//! - SelectOp: Select value by index from multiple inputs (switch/case)
//! - GateOp: Conditionally pass through values
//! - LoopOp: Trigger-based iteration with index output
//! - ForEachOp: Iterate over list elements with triggers
//!
//! Run with: cargo run --example 23_flow_control

use flux_core::{EvalContext, Id, InputPort, Operator, OutputPort, Value, ValueType};
use flux_core::port::TriggerInput;

// =============================================================================
// Import flow operators
// =============================================================================

use flux_operators::flow::{SwitchOp, SelectOp, GateOp, LoopOp, ForEachOp};

// =============================================================================
// Test Operators
// =============================================================================

/// Accumulator that sums values when triggered
struct AccumulatorOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
    trigger_inputs: Vec<TriggerInput>,
    sum: f32,
}

impl AccumulatorOp {
    fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![InputPort::new("Value", Value::Float(0.0))],
            outputs: vec![OutputPort::new("Sum", ValueType::Float)],
            trigger_inputs: vec![TriggerInput::new("Add")],
            sum: 0.0,
        }
    }
}

impl Operator for AccumulatorOp {
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Accumulator" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }
    fn trigger_inputs(&self) -> &[TriggerInput] { &self.trigger_inputs }
    fn trigger_inputs_mut(&mut self) -> &mut [TriggerInput] { &mut self.trigger_inputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: &dyn Fn(Id, usize) -> Value) {
        let value = match &self.inputs[0].connection {
            Some((node_id, output_idx)) => get_input(*node_id, *output_idx).as_float().unwrap_or(0.0),
            None => self.inputs[0].default.as_float().unwrap_or(0.0),
        };
        self.inputs[0].default = Value::Float(value);
        self.outputs[0].set(Value::Float(self.sum));
    }

    fn on_triggered(
        &mut self,
        trigger_index: usize,
        _ctx: &EvalContext,
        _get_input: &dyn Fn(Id, usize) -> Value,
    ) -> Vec<usize> {
        if trigger_index == 0 {
            let value = self.inputs[0].default.as_float().unwrap_or(0.0);
            self.sum += value;
            self.outputs[0].set(Value::Float(self.sum));
        }
        vec![]
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// =============================================================================
// Demonstrations (using operators directly, not through graph)
// =============================================================================

fn demo_switch() {
    println!("=== SwitchOp (If/Else) ===\n");

    let mut switch = SwitchOp::new();
    let ctx = EvalContext::new();

    // Set up: True=100.0, False=0.0
    switch.inputs_mut()[1].default = Value::Float(100.0);
    switch.inputs_mut()[2].default = Value::Float(0.0);

    // Test with condition = true
    switch.inputs_mut()[0].default = Value::Bool(true);
    switch.compute(&ctx, &|_, _| Value::Float(0.0));
    let result = switch.outputs()[0].value.as_float().unwrap_or(0.0);
    println!("Condition=true,  True=100, False=0 -> Result: {}", result);

    // Test with condition = false
    switch.inputs_mut()[0].default = Value::Bool(false);
    switch.compute(&ctx, &|_, _| Value::Float(0.0));
    let result = switch.outputs()[0].value.as_float().unwrap_or(0.0);
    println!("Condition=false, True=100, False=0 -> Result: {}", result);

    println!("\nSwitchOp selects between two values based on a boolean condition.");
    println!("This is equivalent to: result = condition ? true_val : false_val");
}

fn demo_select() {
    println!("\n=== SelectOp (Switch/Case) ===\n");

    let mut select = SelectOp::new();

    // Create value sources and connect them to the multi-input
    let values = [10.0, 20.0, 30.0, 40.0];
    println!("Values: {:?}", values);

    // For SelectOp, we simulate connections by setting up the connections vector
    // In a real graph, these would be actual connections to other nodes
    for (i, &v) in values.iter().enumerate() {
        // Create a fake connection - in practice these would be real node IDs
        let fake_id = Id::new();
        select.inputs_mut()[1].connections.push((fake_id, 0));
        println!("  [{}] = {}", i, v);
    }

    println!("\nNote: SelectOp requires actual graph connections for multi-input.");
    println!("In isolation, demonstrating index-based selection concept:");

    // Demonstrate the concept
    for i in 0i32..4 {
        println!("  Index={} would select value at position {}", i, i);
    }
}

fn demo_gate() {
    println!("\n=== GateOp (Conditional Pass-through) ===\n");

    let mut gate = GateOp::new();
    let ctx = EvalContext::new();

    // Initial value
    gate.inputs_mut()[0].default = Value::Float(42.0);

    // Gate open - value passes through
    gate.inputs_mut()[1].default = Value::Bool(true);
    gate.compute(&ctx, &|_, _| Value::Float(0.0));
    let result = gate.outputs()[0].value.as_float().unwrap_or(0.0);
    println!("Value=42, Gate=OPEN  -> Output: {}", result);

    // Gate closed - previous value is held
    gate.inputs_mut()[1].default = Value::Bool(false);
    gate.inputs_mut()[0].default = Value::Float(100.0); // New input value
    gate.compute(&ctx, &|_, _| Value::Float(0.0));
    let result = gate.outputs()[0].value.as_float().unwrap_or(0.0);
    println!("Value=100, Gate=CLOSED -> Output: {} (previous value held)", result);

    // Gate opens again
    gate.inputs_mut()[1].default = Value::Bool(true);
    gate.compute(&ctx, &|_, _| Value::Float(0.0));
    let result = gate.outputs()[0].value.as_float().unwrap_or(0.0);
    println!("Value=100, Gate=OPEN  -> Output: {} (new value passes)", result);

    println!("\nGateOp only updates its output when the gate is open.");
    println!("This is useful for sample-and-hold behavior.");
}

fn demo_loop() {
    println!("\n=== LoopOp (Trigger-based Iteration) ===\n");

    let mut loop_op = LoopOp::new();
    let ctx = EvalContext::new();

    // Set count to 5
    loop_op.inputs_mut()[0].default = Value::Int(5);
    loop_op.compute(&ctx, &|_, _| Value::Float(0.0));

    println!("Loop count: 5");
    println!("Simulating trigger sequence:\n");

    for iteration in 0..6 {
        let triggers = loop_op.on_triggered(0, &ctx, &|_, _| Value::Float(0.0));
        let index = loop_op.outputs()[0].value.as_int().unwrap_or(-1);

        let trigger_names: Vec<&str> = triggers
            .iter()
            .map(|&t| match t {
                0 => "Body",
                1 => "Done",
                _ => "?",
            })
            .collect();

        println!(
            "  Trigger {}: Index output = {}, Fires: {:?}",
            iteration, index, trigger_names
        );

        if triggers.contains(&1) {
            println!("\n  Loop completed! Index reset for next cycle.");
            break;
        }
    }

    println!("\nLoopOp fires the 'Body' trigger N times, then 'Done' once.");
    println!("Downstream nodes receive the current Index via the output port.");
}

fn demo_foreach() {
    println!("\n=== ForEachOp (List Iteration) ===\n");

    let mut foreach_op = ForEachOp::new();
    let ctx = EvalContext::new();

    // Set list input
    let list = vec![1.5, 2.5, 3.5];
    foreach_op.inputs_mut()[0].default = Value::FloatList(list.clone());
    foreach_op.compute(&ctx, &|_, _| Value::Float(0.0));

    println!("Input list: {:?}", list);
    println!("Simulating iteration:\n");

    for iteration in 0..4 {
        foreach_op.compute(&ctx, &|_, _| Value::Float(0.0));
        let triggers = foreach_op.on_triggered(0, &ctx, &|_, _| Value::Float(0.0));

        let element = foreach_op.outputs()[0].value.as_float().unwrap_or(0.0);
        let index = foreach_op.outputs()[1].value.as_int().unwrap_or(-1);

        let trigger_names: Vec<&str> = triggers
            .iter()
            .map(|&t| match t {
                0 => "Body",
                1 => "Done",
                _ => "?",
            })
            .collect();

        println!(
            "  Iteration {}: Element = {:.1}, Index = {}, Fires: {:?}",
            iteration, element, index, trigger_names
        );

        if triggers.contains(&1) {
            println!("\n  ForEach completed!");
            break;
        }
    }

    println!("\nForEachOp provides both the current element and index as outputs.");
}

fn demo_loop_with_accumulator() {
    println!("\n=== Loop + Accumulator (Sum 1..N) ===\n");

    let mut loop_op = LoopOp::new();
    let mut acc = AccumulatorOp::new();
    let ctx = EvalContext::new();

    let n = 5;
    loop_op.inputs_mut()[0].default = Value::Int(n);
    loop_op.compute(&ctx, &|_, _| Value::Float(0.0));

    println!("Computing: 1 + 2 + 3 + 4 + 5");
    println!("Using Loop(count=5) driving Accumulator:\n");

    for i in 0..n {
        // Get current loop index (1-based for sum)
        let index = loop_op.outputs()[0].value.as_int().unwrap_or(0) + 1;

        // Set accumulator input to current index
        acc.inputs_mut()[0].default = Value::Float(index as f32);

        // Trigger accumulator to add
        acc.on_triggered(0, &ctx, &|_, _| Value::Float(0.0));

        let sum = acc.outputs()[0].value.as_float().unwrap_or(0.0);
        println!("  Loop index {} (value {}): Running sum = {}", i, index, sum);

        // Advance loop
        loop_op.on_triggered(0, &ctx, &|_, _| Value::Float(0.0));
    }

    let final_sum = acc.outputs()[0].value.as_float().unwrap_or(0.0);
    let expected = (n * (n + 1) / 2) as f32;
    println!("\nFinal sum: {} (expected: {})", final_sum, expected);
    println!("\nThis pattern enables iterative algorithms in the dataflow graph.");
}

fn main() {
    println!("Flux Flow Control Operators Example\n");
    println!("Demonstrates conditional and iterative flow control.\n");

    demo_switch();
    demo_select();
    demo_gate();
    demo_loop();
    demo_foreach();
    demo_loop_with_accumulator();

    println!("\n=== Summary ===\n");
    println!("Flow control operators enable complex logic in dataflow graphs:");
    println!();
    println!("  Conditional:");
    println!("    SwitchOp  - Binary selection (if/else)");
    println!("    SelectOp  - Multi-way selection (switch/case)");
    println!("    GateOp    - Conditional pass-through with value holding");
    println!();
    println!("  Iterative:");
    println!("    LoopOp    - Fire body trigger N times with index output");
    println!("    ForEachOp - Iterate list with element + index outputs");
    println!();
    println!("Loop operators use the trigger system (Phase 3.1) for push-based");
    println!("iteration, allowing downstream nodes to execute multiple times");
    println!("per evaluation cycle.");
}
