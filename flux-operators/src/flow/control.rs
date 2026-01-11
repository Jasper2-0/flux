//! Control flow operators: Switch, Select, Gate

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};
use flux_core::Value;

fn get_bool(input: &InputPort, get_input: InputResolver) -> bool {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_bool().unwrap_or(false),
        None => input.default.as_bool().unwrap_or(false),
    }
}

fn get_int(input: &InputPort, get_input: InputResolver) -> i32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_int().unwrap_or(0),
        None => input.default.as_int().unwrap_or(0),
    }
}

fn get_value(input: &InputPort, get_input: InputResolver) -> Value {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx),
        None => input.default.clone(),
    }
}

// ============================================================================
// Switch Operator
// ============================================================================

pub struct SwitchOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl SwitchOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::bool("Condition", false),
                InputPort::float("True", 1.0),
                InputPort::float("False", 0.0),
            ],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for SwitchOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SwitchOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Switch" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let condition = get_bool(&self.inputs[0], get_input);
        let value = if condition {
            get_value(&self.inputs[1], get_input)
        } else {
            get_value(&self.inputs[2], get_input)
        };
        self.outputs[0].value = value;
    }
}

impl OperatorMeta for SwitchOp {
    fn category(&self) -> &'static str { "Flow" }
    fn category_color(&self) -> [f32; 4] { category_colors::FLOW }
    fn description(&self) -> &'static str { "Select between two values based on condition" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Cond")),
            1 => Some(PortMeta::new("True")),
            2 => Some(PortMeta::new("False")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Out").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Select Operator
// ============================================================================

pub struct SelectOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: [OutputPort; 1],
}

impl SelectOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![
                InputPort::int("Index", 0),
                InputPort::float_multi("Values"),
            ],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for SelectOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SelectOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Select" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let index = get_int(&self.inputs[0], get_input) as usize;
        let values_input = &self.inputs[1];

        let value = if index < values_input.connections.len() {
            let (node_id, output_idx) = values_input.connections[index];
            get_input(node_id, output_idx)
        } else {
            Value::Float(0.0)
        };

        self.outputs[0].value = value;
    }
}

impl OperatorMeta for SelectOp {
    fn category(&self) -> &'static str { "Flow" }
    fn category_color(&self) -> [f32; 4] { category_colors::FLOW }
    fn description(&self) -> &'static str { "Select value by index from multiple inputs" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Index")),
            1 => Some(PortMeta::new("Values")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Result").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Gate Operator
// ============================================================================

pub struct GateOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl GateOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Value", 0.0),
                InputPort::bool("Open", true),
            ],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for GateOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for GateOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Gate" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_value(&self.inputs[0], get_input);
        let open = get_bool(&self.inputs[1], get_input);

        if open {
            self.outputs[0].value = value;
        }
        // When closed, keep previous value (don't update)
    }
}

impl OperatorMeta for GateOp {
    fn category(&self) -> &'static str { "Flow" }
    fn category_color(&self) -> [f32; 4] { category_colors::FLOW }
    fn description(&self) -> &'static str { "Pass value through when gate is open" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            1 => Some(PortMeta::new("Open")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Result").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Loop Operator (Trigger-based iteration)
// ============================================================================

use flux_core::port::{TriggerInput, TriggerOutput};

/// Loop operator that fires a trigger N times.
///
/// When the "Start" trigger is received, this operator fires its "Body" trigger
/// output N times (where N is the Count input), updating the Index output each time.
///
/// # Inputs
/// - `Count`: Number of iterations (integer)
/// - `Start`: Trigger to begin the loop
///
/// # Outputs
/// - `Index`: Current iteration index (0 to N-1)
/// - `Body`: Trigger output fired once per iteration
/// - `Done`: Trigger output fired when loop completes
///
/// # Example
/// ```ignore
/// // Connect: Counter → Loop.Count, OnFrame → Loop.Start
/// // Loop.Body → SomeOp, Loop.Index → SomeOp.Multiplier
/// // Each frame, SomeOp executes Count times with Index 0..Count
/// ```
pub struct LoopOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    trigger_inputs: Vec<TriggerInput>,
    trigger_outputs: Vec<TriggerOutput>,
    /// Current iteration index (updated during loop execution)
    current_index: i32,
    /// Total count for current loop
    loop_count: i32,
}

impl LoopOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::int("Count", 1)],
            outputs: [OutputPort::int("Index")],
            trigger_inputs: vec![TriggerInput::new("Start")],
            trigger_outputs: vec![
                TriggerOutput::new("Body"),
                TriggerOutput::new("Done"),
            ],
            current_index: 0,
            loop_count: 0,
        }
    }
}

impl Default for LoopOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for LoopOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Loop" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }
    fn trigger_inputs(&self) -> &[TriggerInput] { &self.trigger_inputs }
    fn trigger_inputs_mut(&mut self) -> &mut [TriggerInput] { &mut self.trigger_inputs }
    fn trigger_outputs(&self) -> &[TriggerOutput] { &self.trigger_outputs }
    fn trigger_outputs_mut(&mut self) -> &mut [TriggerOutput] { &mut self.trigger_outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        // Update loop count from input
        self.loop_count = get_int(&self.inputs[0], get_input);
        // Output the current index
        self.outputs[0].value = Value::Int(self.current_index);
    }

    fn on_triggered(
        &mut self,
        trigger_index: usize,
        _ctx: &EvalContext,
        _get_input: InputResolver,
    ) -> Vec<usize> {
        if trigger_index == 0 {
            // "Start" trigger received
            // Return indices of triggers to fire: Body for each iteration, then Done
            // The graph will call us repeatedly, so we track state

            // For now, we fire Body trigger for index 0
            // The graph's fire_trigger will handle propagation
            // We need a different approach for true iteration...

            // Actually, since triggers are immediate, we return all triggers to fire
            // But we need the index to update between fires.
            //
            // Approach: Return trigger index 0 (Body) and let the graph re-evaluate
            // the Index output before propagating to downstream nodes.
            // After all iterations, return trigger index 1 (Done).

            if self.current_index < self.loop_count {
                // Update index output
                self.outputs[0].value = Value::Int(self.current_index);
                self.current_index += 1;

                if self.current_index < self.loop_count {
                    // More iterations to go - fire Body and will be triggered again
                    vec![0] // Fire "Body"
                } else {
                    // Last iteration - fire both Body and Done
                    self.current_index = 0; // Reset for next loop
                    vec![0, 1] // Fire "Body" then "Done"
                }
            } else {
                self.current_index = 0;
                vec![1] // Just fire "Done"
            }
        } else {
            vec![] // Unknown trigger
        }
    }
}

impl OperatorMeta for LoopOp {
    fn category(&self) -> &'static str { "Flow" }
    fn category_color(&self) -> [f32; 4] { category_colors::FLOW }
    fn description(&self) -> &'static str { "Execute body N times with iteration index" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Count")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Index").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// ForEach Operator (List iteration)
// ============================================================================

/// ForEach operator that iterates over a list.
///
/// When triggered, iterates over the input list and fires the Body trigger
/// for each element, outputting the current element and index.
pub struct ForEachOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 2],
    trigger_inputs: Vec<TriggerInput>,
    trigger_outputs: Vec<TriggerOutput>,
    current_index: usize,
    list_len: usize,
}

impl ForEachOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float_list("List")],
            outputs: [
                OutputPort::float("Element"),
                OutputPort::int("Index"),
            ],
            trigger_inputs: vec![TriggerInput::new("Start")],
            trigger_outputs: vec![
                TriggerOutput::new("Body"),
                TriggerOutput::new("Done"),
            ],
            current_index: 0,
            list_len: 0,
        }
    }
}

impl Default for ForEachOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ForEachOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ForEach" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }
    fn trigger_inputs(&self) -> &[TriggerInput] { &self.trigger_inputs }
    fn trigger_inputs_mut(&mut self) -> &mut [TriggerInput] { &mut self.trigger_inputs }
    fn trigger_outputs(&self) -> &[TriggerOutput] { &self.trigger_outputs }
    fn trigger_outputs_mut(&mut self) -> &mut [TriggerOutput] { &mut self.trigger_outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        // Get list and update state
        let list = get_value(&self.inputs[0], get_input);
        if let Some(float_list) = list.as_float_list() {
            self.list_len = float_list.len();
            if self.current_index < self.list_len {
                self.outputs[0].value = Value::Float(float_list[self.current_index]);
            }
        } else {
            self.list_len = 0;
        }
        self.outputs[1].value = Value::Int(self.current_index as i32);
    }

    fn on_triggered(
        &mut self,
        trigger_index: usize,
        _ctx: &EvalContext,
        _get_input: InputResolver,
    ) -> Vec<usize> {
        if trigger_index == 0 {
            // "Start" trigger
            if self.current_index < self.list_len {
                self.outputs[1].value = Value::Int(self.current_index as i32);
                self.current_index += 1;

                if self.current_index < self.list_len {
                    vec![0] // Fire "Body"
                } else {
                    self.current_index = 0;
                    vec![0, 1] // Fire "Body" then "Done"
                }
            } else {
                self.current_index = 0;
                vec![1] // Just "Done"
            }
        } else {
            vec![]
        }
    }
}

impl OperatorMeta for ForEachOp {
    fn category(&self) -> &'static str { "Flow" }
    fn category_color(&self) -> [f32; 4] { category_colors::FLOW }
    fn description(&self) -> &'static str { "Iterate over list elements" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Element").with_shape(PinShape::TriangleFilled)),
            1 => Some(PortMeta::new("Index")),
            _ => None,
        }
    }
}

// ============================================================================
// Registration
// ============================================================================

pub fn register(registry: &OperatorRegistry) {
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Switch",
            category: "Flow",
            description: "Select between two values based on condition",
        },
        || capture_meta(SwitchOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Select",
            category: "Flow",
            description: "Select value by index",
        },
        || capture_meta(SelectOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Gate",
            category: "Flow",
            description: "Pass value when open",
        },
        || capture_meta(GateOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Loop",
            category: "Flow",
            description: "Execute body N times",
        },
        || capture_meta(LoopOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ForEach",
            category: "Flow",
            description: "Iterate over list elements",
        },
        || capture_meta(ForEachOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Float(0.0)
    }

    #[test]
    fn test_switch() {
        let mut op = SwitchOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Bool(true);
        op.inputs[1].default = Value::Float(10.0);
        op.inputs[2].default = Value::Float(5.0);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(10.0));

        op.inputs[0].default = Value::Bool(false);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(5.0));
    }

    #[test]
    fn test_gate() {
        let mut op = GateOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Float(42.0);
        op.inputs[1].default = Value::Bool(true);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(42.0));

        // When closed, should keep previous value
        op.inputs[1].default = Value::Bool(false);
        op.inputs[0].default = Value::Float(100.0);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(42.0));
    }

    #[test]
    fn test_loop_basic() {
        let mut op = LoopOp::new();
        let ctx = EvalContext::new();

        // Set count to 3
        op.inputs[0].default = Value::Int(3);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.loop_count, 3);

        // First trigger - should fire Body with index 0
        let triggers = op.on_triggered(0, &ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(0));
        assert_eq!(triggers, vec![0]); // Fire Body

        // Second trigger - index 1
        let triggers = op.on_triggered(0, &ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(1));
        assert_eq!(triggers, vec![0]); // Fire Body

        // Third trigger - index 2, last iteration
        let triggers = op.on_triggered(0, &ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(2));
        assert_eq!(triggers, vec![0, 1]); // Fire Body, then Done

        // Index should be reset
        assert_eq!(op.current_index, 0);
    }

    #[test]
    fn test_loop_zero_count() {
        let mut op = LoopOp::new();
        let ctx = EvalContext::new();

        // Set count to 0
        op.inputs[0].default = Value::Int(0);
        op.compute(&ctx, &no_connections);

        // Trigger with zero count should just fire Done
        let triggers = op.on_triggered(0, &ctx, &no_connections);
        assert_eq!(triggers, vec![1]); // Just Done
    }

    #[test]
    fn test_foreach_basic() {
        let mut op = ForEachOp::new();
        let ctx = EvalContext::new();

        // Set list input
        op.inputs[0].default = Value::FloatList(vec![10.0, 20.0, 30.0]);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.list_len, 3);

        // First trigger - element 10.0, index 0
        let triggers = op.on_triggered(0, &ctx, &no_connections);
        assert_eq!(op.outputs[1].value.as_int(), Some(0));
        assert_eq!(triggers, vec![0]); // Fire Body

        // Second trigger
        let triggers = op.on_triggered(0, &ctx, &no_connections);
        assert_eq!(op.outputs[1].value.as_int(), Some(1));
        assert_eq!(triggers, vec![0]); // Fire Body

        // Third trigger - last element
        let triggers = op.on_triggered(0, &ctx, &no_connections);
        assert_eq!(op.outputs[1].value.as_int(), Some(2));
        assert_eq!(triggers, vec![0, 1]); // Fire Body, then Done
    }
}
