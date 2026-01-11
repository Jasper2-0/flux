//! State operators: Delay, Previous, Changed, Trigger, Once, Counter

use std::any::Any;
use std::collections::VecDeque;

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
// Delay Operator
// ============================================================================

pub struct DelayOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    buffer: VecDeque<Value>,
}

impl DelayOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Value", 0.0),
                InputPort::int("Frames", 1),
            ],
            outputs: [OutputPort::float("Result")],
            buffer: VecDeque::new(),
        }
    }
}

impl Default for DelayOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for DelayOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Delay" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_value(&self.inputs[0], get_input);
        let frames = get_int(&self.inputs[1], get_input).max(0) as usize;

        self.buffer.push_back(value);

        while self.buffer.len() > frames + 1 {
            self.buffer.pop_front();
        }

        let output = if self.buffer.len() > frames {
            self.buffer.front().cloned().unwrap_or(Value::Float(0.0))
        } else {
            Value::Float(0.0)
        };

        self.outputs[0].value = output;
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for DelayOp {
    fn category(&self) -> &'static str { "Flow" }
    fn category_color(&self) -> [f32; 4] { category_colors::STATE }
    fn description(&self) -> &'static str { "Delay value by frames" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            1 => Some(PortMeta::new("Frames").with_range(0.0, 60.0)),
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
// Previous Operator
// ============================================================================

pub struct PreviousOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    previous: Value,
}

impl PreviousOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Value", 0.0)],
            outputs: [OutputPort::float("Previous")],
            previous: Value::Float(0.0),
        }
    }
}

impl Default for PreviousOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for PreviousOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Previous" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let current = get_value(&self.inputs[0], get_input);
        self.outputs[0].value = self.previous.clone();
        self.previous = current;
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for PreviousOp {
    fn category(&self) -> &'static str { "Flow" }
    fn category_color(&self) -> [f32; 4] { category_colors::STATE }
    fn description(&self) -> &'static str { "Previous frame value" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Previous").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Changed Operator
// ============================================================================

pub struct ChangedOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    previous: Option<Value>,
}

impl ChangedOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Value", 0.0)],
            outputs: [OutputPort::bool("Changed")],
            previous: None,
        }
    }
}

impl Default for ChangedOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ChangedOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Changed" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let current = get_value(&self.inputs[0], get_input);
        let changed = match &self.previous {
            Some(prev) => prev != &current,
            None => true, // First frame is considered a change
        };
        self.outputs[0].set_bool(changed);
        self.previous = Some(current);
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for ChangedOp {
    fn category(&self) -> &'static str { "Flow" }
    fn category_color(&self) -> [f32; 4] { category_colors::STATE }
    fn description(&self) -> &'static str { "Detect value changes" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Changed").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Trigger Operator (Rising Edge)
// ============================================================================

pub struct TriggerOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    previous: bool,
}

impl TriggerOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::bool("Value", false)],
            outputs: [OutputPort::bool("Triggered")],
            previous: false,
        }
    }
}

impl Default for TriggerOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for TriggerOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Trigger" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let current = get_bool(&self.inputs[0], get_input);
        let triggered = current && !self.previous; // Rising edge
        self.outputs[0].set_bool(triggered);
        self.previous = current;
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for TriggerOp {
    fn category(&self) -> &'static str { "Flow" }
    fn category_color(&self) -> [f32; 4] { category_colors::STATE }
    fn description(&self) -> &'static str { "Rising edge detection" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Triggered").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Once Operator
// ============================================================================

pub struct OnceOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    executed: bool,
    stored_value: Value,
}

impl OnceOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Value", 0.0),
                InputPort::bool("Reset", false),
            ],
            outputs: [OutputPort::float("Result")],
            executed: false,
            stored_value: Value::Float(0.0),
        }
    }
}

impl Default for OnceOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for OnceOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Once" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let reset = get_bool(&self.inputs[1], get_input);

        if reset {
            self.executed = false;
        }

        if !self.executed {
            self.stored_value = get_value(&self.inputs[0], get_input);
            self.executed = true;
        }

        self.outputs[0].value = self.stored_value.clone();
    }
}

impl OperatorMeta for OnceOp {
    fn category(&self) -> &'static str { "Flow" }
    fn category_color(&self) -> [f32; 4] { category_colors::STATE }
    fn description(&self) -> &'static str { "Execute once until reset" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            1 => Some(PortMeta::new("Reset")),
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
// Counter Operator
// ============================================================================

pub struct CounterOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    count: i32,
    previous_trigger: bool,
}

impl CounterOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::bool("Trigger", false),
                InputPort::bool("Reset", false),
            ],
            outputs: [OutputPort::int("Count")],
            count: 0,
            previous_trigger: false,
        }
    }
}

impl Default for CounterOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for CounterOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Counter" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let trigger = get_bool(&self.inputs[0], get_input);
        let reset = get_bool(&self.inputs[1], get_input);

        if reset {
            self.count = 0;
        } else if trigger && !self.previous_trigger {
            self.count += 1;
        }

        self.previous_trigger = trigger;
        self.outputs[0].set_int(self.count);
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for CounterOp {
    fn category(&self) -> &'static str { "Flow" }
    fn category_color(&self) -> [f32; 4] { category_colors::STATE }
    fn description(&self) -> &'static str { "Count trigger events" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Trigger")),
            1 => Some(PortMeta::new("Reset")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Count").with_shape(PinShape::TriangleFilled)),
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
            name: "Delay",
            category: "Flow",
            description: "Delay value by frames",
        },
        || capture_meta(DelayOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Previous",
            category: "Flow",
            description: "Previous frame value",
        },
        || capture_meta(PreviousOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Changed",
            category: "Flow",
            description: "Detect value changes",
        },
        || capture_meta(ChangedOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Trigger",
            category: "Flow",
            description: "Rising edge detection",
        },
        || capture_meta(TriggerOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Once",
            category: "Flow",
            description: "Execute once until reset",
        },
        || capture_meta(OnceOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Counter",
            category: "Flow",
            description: "Count trigger events",
        },
        || capture_meta(CounterOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Float(0.0)
    }

    #[test]
    fn test_trigger() {
        let mut op = TriggerOp::new();
        let ctx = EvalContext::new();

        // False -> True = Trigger
        op.inputs[0].default = Value::Bool(true);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(true));

        // True -> True = No trigger
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(false));

        // True -> False = No trigger
        op.inputs[0].default = Value::Bool(false);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(false));

        // False -> True = Trigger again
        op.inputs[0].default = Value::Bool(true);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(true));
    }

    #[test]
    fn test_counter() {
        let mut op = CounterOp::new();
        let ctx = EvalContext::new();

        // Initial count
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(0));

        // Trigger
        op.inputs[0].default = Value::Bool(true);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(1));

        // Stay high - no increment
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(1));

        // Trigger again
        op.inputs[0].default = Value::Bool(false);
        op.compute(&ctx, &no_connections);
        op.inputs[0].default = Value::Bool(true);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(2));

        // Reset
        op.inputs[1].default = Value::Bool(true);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(0));
    }
}
