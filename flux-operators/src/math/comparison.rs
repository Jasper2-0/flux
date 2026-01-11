//! Comparison operators: Min, Max, Clamp, Sign, Step
//! Note: Compare is already implemented in the legacy operator.rs

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};

fn get_float(input: &InputPort, get_input: InputResolver) -> f32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
        None => input.default.as_float().unwrap_or(0.0),
    }
}

// ============================================================================
// Min Operator
// ============================================================================

pub struct MinOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl MinOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("A", 0.0), InputPort::float("B", 0.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for MinOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for MinOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Min" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_float(&self.inputs[0], get_input);
        let b = get_float(&self.inputs[1], get_input);
        self.outputs[0].set_float(a.min(b));
    }
}

impl OperatorMeta for MinOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Returns the smaller of two values" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Min").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Max Operator
// ============================================================================

pub struct MaxOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl MaxOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("A", 0.0), InputPort::float("B", 0.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for MaxOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for MaxOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Max" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_float(&self.inputs[0], get_input);
        let b = get_float(&self.inputs[1], get_input);
        self.outputs[0].set_float(a.max(b));
    }
}

impl OperatorMeta for MaxOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Returns the larger of two values" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Max").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Clamp Operator
// ============================================================================

pub struct ClampOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl ClampOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Value", 0.0),
                InputPort::float("Min", 0.0),
                InputPort::float("Max", 1.0),
            ],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for ClampOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ClampOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Clamp" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_float(&self.inputs[0], get_input);
        let min = get_float(&self.inputs[1], get_input);
        let max = get_float(&self.inputs[2], get_input);
        self.outputs[0].set_float(value.clamp(min, max));
    }
}

impl OperatorMeta for ClampOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Clamps value to range [min, max]" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            1 => Some(PortMeta::new("Min")),
            2 => Some(PortMeta::new("Max")),
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
// Sign Operator
// ============================================================================

pub struct SignOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl SignOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Value", 0.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for SignOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SignOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Sign" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_float(&self.inputs[0], get_input);
        let sign = if value > 0.0 {
            1.0
        } else if value < 0.0 {
            -1.0
        } else {
            0.0
        };
        self.outputs[0].set_float(sign);
    }
}

impl OperatorMeta for SignOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Returns -1, 0, or 1 based on sign" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Sign").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Step Operator
// ============================================================================

pub struct StepOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl StepOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Edge", 0.0),
                InputPort::float("Value", 0.0),
            ],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for StepOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for StepOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Step" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let edge = get_float(&self.inputs[0], get_input);
        let value = get_float(&self.inputs[1], get_input);
        // GLSL step: 0 if value < edge, else 1
        self.outputs[0].set_float(if value < edge { 0.0 } else { 1.0 });
    }
}

impl OperatorMeta for StepOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Returns 0 if value < edge, else 1" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Edge")),
            1 => Some(PortMeta::new("Value")),
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
// Registration
// ============================================================================

pub fn register(registry: &OperatorRegistry) {
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Min",
            category: "Math",
            description: "Returns the smaller of two values",
        },
        || capture_meta(MinOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Max",
            category: "Math",
            description: "Returns the larger of two values",
        },
        || capture_meta(MaxOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Clamp",
            category: "Math",
            description: "Clamps value to range [min, max]",
        },
        || capture_meta(ClampOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Sign",
            category: "Math",
            description: "Returns -1, 0, or 1 based on sign",
        },
        || capture_meta(SignOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Step",
            category: "Math",
            description: "Returns 0 if value < edge, else 1",
        },
        || capture_meta(StepOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::Value;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Float(0.0)
    }

    #[test]
    fn test_min() {
        let mut op = MinOp::new();
        op.inputs[0].default = Value::Float(5.0);
        op.inputs[1].default = Value::Float(3.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(3.0));
    }

    #[test]
    fn test_max() {
        let mut op = MaxOp::new();
        op.inputs[0].default = Value::Float(5.0);
        op.inputs[1].default = Value::Float(3.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(5.0));
    }

    #[test]
    fn test_clamp() {
        let mut op = ClampOp::new();
        op.inputs[0].default = Value::Float(1.5);
        op.inputs[1].default = Value::Float(0.0);
        op.inputs[2].default = Value::Float(1.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(1.0));
    }

    #[test]
    fn test_sign() {
        let mut op = SignOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Float(5.0);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(1.0));

        op.inputs[0].default = Value::Float(-5.0);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(-1.0));

        op.inputs[0].default = Value::Float(0.0);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(0.0));
    }

    #[test]
    fn test_step() {
        let mut op = StepOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Float(0.5);
        op.inputs[1].default = Value::Float(0.3);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(0.0));

        op.inputs[1].default = Value::Float(0.7);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(1.0));
    }
}
