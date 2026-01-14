//! Demo 20: Trigger System - Push-Based Execution
//!
//! This example demonstrates:
//! - Creating operators with trigger ports (TriggerInput/TriggerOutput)
//! - Connecting trigger ports for push-based signal flow
//! - Firing triggers and observing cascading propagation
//! - Combining trigger-based execution with value outputs
//!
//! # Pull vs Push Execution
//!
//! - **Pull (values)**: Evaluation flows backwards from outputs on demand
//! - **Push (triggers)**: Signals propagate forward immediately when fired
//!
//! Triggers are useful for:
//! - Animation loops (OnFrame fires every frame)
//! - User interactions (OnClick, OnKeyPress)
//! - Completion signals (OnComplete, Done)
//! - State transitions (OnChange)
//!
//! Run with: `cargo run --example 20_trigger_system`

use flux_core::{
    EvalContext, Id, InputPort, InputResolver, Operator, OutputPort, TriggerInput, TriggerOutput,
    Value, ValueType,
};
use flux_graph::{Graph, GraphEvent};
use std::cell::Cell;

// =============================================================================
// Custom operators with trigger ports
// =============================================================================

/// A "MainLoop" operator that emits an OnFrame trigger.
/// In a real application, this would fire every frame from the render loop.
struct MainLoop {
    id: Id,
    outputs: Vec<OutputPort>,
    trigger_outputs: Vec<TriggerOutput>,
}

impl MainLoop {
    fn new() -> Self {
        Self {
            id: Id::new(),
            outputs: vec![],
            trigger_outputs: vec![TriggerOutput::new("OnFrame")],
        }
    }
}

impl Operator for MainLoop {
    fn id(&self) -> Id {
        self.id
    }
    fn name(&self) -> &'static str {
        "MainLoop"
    }
    fn inputs(&self) -> &[InputPort] {
        &[]
    }
    fn inputs_mut(&mut self) -> &mut [InputPort] {
        &mut []
    }
    fn outputs(&self) -> &[OutputPort] {
        &self.outputs
    }
    fn outputs_mut(&mut self) -> &mut [OutputPort] {
        &mut self.outputs
    }
    fn trigger_outputs(&self) -> &[TriggerOutput] {
        &self.trigger_outputs
    }
    fn trigger_outputs_mut(&mut self) -> &mut [TriggerOutput] {
        &mut self.trigger_outputs
    }
    fn compute(&mut self, _ctx: &EvalContext, _get_input: &dyn Fn(Id, usize) -> Value) {}
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// A counter that increments on each trigger and outputs the count.
struct FrameCounter {
    id: Id,
    outputs: Vec<OutputPort>,
    trigger_inputs: Vec<TriggerInput>,
    trigger_outputs: Vec<TriggerOutput>,
    count: Cell<u64>,
}

impl FrameCounter {
    fn new() -> Self {
        Self {
            id: Id::new(),
            outputs: vec![OutputPort::new("Count", ValueType::Int)],
            trigger_inputs: vec![TriggerInput::new("OnFrame")],
            trigger_outputs: vec![TriggerOutput::new("OnCount")],
            count: Cell::new(0),
        }
    }

    fn count(&self) -> u64 {
        self.count.get()
    }
}

impl Operator for FrameCounter {
    fn id(&self) -> Id {
        self.id
    }
    fn name(&self) -> &'static str {
        "FrameCounter"
    }
    fn inputs(&self) -> &[InputPort] {
        &[]
    }
    fn inputs_mut(&mut self) -> &mut [InputPort] {
        &mut []
    }
    fn outputs(&self) -> &[OutputPort] {
        &self.outputs
    }
    fn outputs_mut(&mut self) -> &mut [OutputPort] {
        &mut self.outputs
    }
    fn trigger_inputs(&self) -> &[TriggerInput] {
        &self.trigger_inputs
    }
    fn trigger_inputs_mut(&mut self) -> &mut [TriggerInput] {
        &mut self.trigger_inputs
    }
    fn trigger_outputs(&self) -> &[TriggerOutput] {
        &self.trigger_outputs
    }
    fn trigger_outputs_mut(&mut self) -> &mut [TriggerOutput] {
        &mut self.trigger_outputs
    }
    fn compute(&mut self, _ctx: &EvalContext, _get_input: &dyn Fn(Id, usize) -> Value) {
        self.outputs[0].set(Value::Int(self.count.get() as i32));
    }
    fn on_triggered(
        &mut self,
        trigger_index: usize,
        _ctx: &EvalContext,
        _get_input: InputResolver,
    ) -> Vec<usize> {
        if trigger_index == 0 {
            // Increment count
            let new_count = self.count.get() + 1;
            self.count.set(new_count);
            // Update output
            self.outputs[0].set(Value::Int(new_count as i32));
            // Fire OnCount trigger
            vec![0]
        } else {
            vec![]
        }
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// A logger that prints when triggered.
struct Logger {
    id: Id,
    inputs: Vec<InputPort>,
    trigger_inputs: Vec<TriggerInput>,
    name_prefix: &'static str,
    log_count: Cell<usize>,
}

impl Logger {
    fn new(name_prefix: &'static str) -> Self {
        Self {
            id: Id::new(),
            inputs: vec![InputPort::new("Message", Value::String("".into()))],
            trigger_inputs: vec![TriggerInput::new("Log")],
            name_prefix,
            log_count: Cell::new(0),
        }
    }

    fn log_count(&self) -> usize {
        self.log_count.get()
    }
}

impl Operator for Logger {
    fn id(&self) -> Id {
        self.id
    }
    fn name(&self) -> &'static str {
        "Logger"
    }
    fn inputs(&self) -> &[InputPort] {
        &self.inputs
    }
    fn inputs_mut(&mut self) -> &mut [InputPort] {
        &mut self.inputs
    }
    fn outputs(&self) -> &[OutputPort] {
        &[]
    }
    fn outputs_mut(&mut self) -> &mut [OutputPort] {
        &mut []
    }
    fn trigger_inputs(&self) -> &[TriggerInput] {
        &self.trigger_inputs
    }
    fn trigger_inputs_mut(&mut self) -> &mut [TriggerInput] {
        &mut self.trigger_inputs
    }
    fn compute(&mut self, _ctx: &EvalContext, _get_input: &dyn Fn(Id, usize) -> Value) {}
    fn on_triggered(
        &mut self,
        trigger_index: usize,
        _ctx: &EvalContext,
        _get_input: InputResolver,
    ) -> Vec<usize> {
        if trigger_index == 0 {
            self.log_count.set(self.log_count.get() + 1);
            println!("  [{}] Triggered! (total: {})", self.name_prefix, self.log_count.get());
        }
        vec![] // Logger doesn't emit triggers
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 20: Trigger System                ║");
    println!("╚════════════════════════════════════════╝\n");

    // =========================================================================
    // Part 1: Basic Trigger Connection
    // =========================================================================
    println!("═══ Part 1: Basic Trigger Connection ═══\n");

    let mut graph = Graph::new();

    // Create MainLoop (trigger source)
    let main_loop = graph.add(MainLoop::new());
    println!("Created MainLoop (emits OnFrame trigger)");

    // Create FrameCounter (receives OnFrame, emits OnCount)
    let counter_id = {
        let counter = FrameCounter::new();
        let id = counter.id;
        graph.add(counter);
        id
    };
    println!("Created FrameCounter (receives OnFrame, emits OnCount)");

    // Clear events from additions
    graph.clear_events();

    // Connect MainLoop.OnFrame -> FrameCounter.OnFrame
    println!("\nConnecting MainLoop.OnFrame -> FrameCounter.OnFrame");
    graph.connect_trigger(main_loop, 0, counter_id, 0).unwrap();

    // Check events
    for event in graph.drain_events() {
        if let GraphEvent::TriggerConnected {
            source,
            source_output,
            target,
            target_input,
        } = event
        {
            println!(
                "  Event: TriggerConnected {:?}[{}] -> {:?}[{}]",
                source, source_output, target, target_input
            );
        }
    }

    // =========================================================================
    // Part 2: Firing Triggers
    // =========================================================================
    println!("\n═══ Part 2: Firing Triggers ═══\n");

    let ctx = EvalContext::new();

    // Simulate 5 frames
    println!("Simulating 5 frames...");
    for frame in 1..=5 {
        print!("  Frame {}: ", frame);
        graph.fire_trigger(main_loop, 0, &ctx);

        // Check the counter value
        let counter = graph.get(counter_id).unwrap();
        let fc = counter.as_any().downcast_ref::<FrameCounter>().unwrap();
        println!("Counter = {}", fc.count());
    }

    // =========================================================================
    // Part 3: Trigger Fan-Out (One to Many)
    // =========================================================================
    println!("\n═══ Part 3: Trigger Fan-Out ═══\n");

    // Create two loggers that both receive the counter's OnCount trigger
    let logger1_id = {
        let logger = Logger::new("Logger1");
        let id = logger.id;
        graph.add(logger);
        id
    };

    let logger2_id = {
        let logger = Logger::new("Logger2");
        let id = logger.id;
        graph.add(logger);
        id
    };

    // Connect FrameCounter.OnCount -> Logger1.Log
    graph.connect_trigger(counter_id, 0, logger1_id, 0).unwrap();
    // Connect FrameCounter.OnCount -> Logger2.Log
    graph.connect_trigger(counter_id, 0, logger2_id, 0).unwrap();

    println!("Connected FrameCounter.OnCount to both Logger1 and Logger2");
    println!("\nSimulating 3 more frames (triggers cascade from MainLoop -> Counter -> Loggers):");

    for frame in 6..=8 {
        println!("Frame {}:", frame);
        graph.fire_trigger(main_loop, 0, &ctx);
    }

    // Check final counts
    let counter = graph.get(counter_id).unwrap();
    let fc = counter.as_any().downcast_ref::<FrameCounter>().unwrap();
    println!("\nFinal counter value: {}", fc.count());

    let logger1 = graph.get(logger1_id).unwrap();
    let l1 = logger1.as_any().downcast_ref::<Logger>().unwrap();
    println!("Logger1 triggered {} times", l1.log_count());

    let logger2 = graph.get(logger2_id).unwrap();
    let l2 = logger2.as_any().downcast_ref::<Logger>().unwrap();
    println!("Logger2 triggered {} times", l2.log_count());

    // =========================================================================
    // Part 4: Trigger Disconnection
    // =========================================================================
    println!("\n═══ Part 4: Trigger Disconnection ═══\n");

    // Disconnect Logger2
    println!("Disconnecting Logger2...");
    let prev = graph.disconnect_trigger(logger2_id, 0).unwrap();
    println!("  Previous connection: {:?}", prev);

    // Check events
    for event in graph.drain_events() {
        if let GraphEvent::TriggerDisconnected { target, .. } = event {
            println!("  Event: TriggerDisconnected from {:?}", target);
        }
    }

    println!("\nSimulating 2 more frames (only Logger1 should trigger):");
    for frame in 9..=10 {
        println!("Frame {}:", frame);
        graph.fire_trigger(main_loop, 0, &ctx);
    }

    // Check final logger counts
    let logger1 = graph.get(logger1_id).unwrap();
    let l1 = logger1.as_any().downcast_ref::<Logger>().unwrap();
    let logger2 = graph.get(logger2_id).unwrap();
    let l2 = logger2.as_any().downcast_ref::<Logger>().unwrap();

    println!("\nFinal trigger counts:");
    println!("  Logger1: {} (received all triggers)", l1.log_count());
    println!("  Logger2: {} (stopped after disconnection)", l2.log_count());

    // =========================================================================
    // Part 5: Value Output from Triggered Operator
    // =========================================================================
    println!("\n═══ Part 5: Value Output from Triggered Operator ═══\n");

    // The FrameCounter updates its output value when triggered
    // We can read it using regular value evaluation
    let result = graph.evaluate(counter_id, 0, &ctx).unwrap();
    println!("Counter's current output value: {:?}", result);
    println!("  (This combines push-based trigger counting with pull-based value access)");

    println!();
    println!("Graph stats: {:?}", graph.stats());
}
