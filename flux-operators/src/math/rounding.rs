//! Rounding operators: Floor, Ceil, Round, Truncate

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
// Floor Operator
// ============================================================================

pub struct FloorOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl FloorOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Value", 0.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for FloorOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for FloorOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Floor" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_float(&self.inputs[0], get_input);
        self.outputs[0].set_float(value.floor());
    }
}

impl OperatorMeta for FloorOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Rounds down to nearest integer" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
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
// Ceil Operator
// ============================================================================

pub struct CeilOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl CeilOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Value", 0.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for CeilOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for CeilOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Ceil" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_float(&self.inputs[0], get_input);
        self.outputs[0].set_float(value.ceil());
    }
}

impl OperatorMeta for CeilOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Rounds up to nearest integer" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
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
// Round Operator
// ============================================================================

pub struct RoundOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl RoundOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Value", 0.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for RoundOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for RoundOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Round" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_float(&self.inputs[0], get_input);
        self.outputs[0].set_float(value.round());
    }
}

impl OperatorMeta for RoundOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Rounds to nearest integer" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
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
// Truncate Operator
// ============================================================================

pub struct TruncateOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl TruncateOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Value", 0.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for TruncateOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for TruncateOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Truncate" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_float(&self.inputs[0], get_input);
        self.outputs[0].set_float(value.trunc());
    }
}

impl OperatorMeta for TruncateOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Truncates toward zero" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
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
            name: "Floor",
            category: "Math",
            description: "Rounds down to nearest integer",
        },
        || capture_meta(FloorOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Ceil",
            category: "Math",
            description: "Rounds up to nearest integer",
        },
        || capture_meta(CeilOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Round",
            category: "Math",
            description: "Rounds to nearest integer",
        },
        || capture_meta(RoundOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Truncate",
            category: "Math",
            description: "Truncates toward zero",
        },
        || capture_meta(TruncateOp::new()),
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
    fn test_floor() {
        let mut op = FloorOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Float(2.7);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(2.0));

        op.inputs[0].default = Value::Float(-2.7);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(-3.0));
    }

    #[test]
    fn test_ceil() {
        let mut op = CeilOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Float(2.3);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(3.0));

        op.inputs[0].default = Value::Float(-2.3);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(-2.0));
    }

    #[test]
    fn test_round() {
        let mut op = RoundOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Float(2.4);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(2.0));

        op.inputs[0].default = Value::Float(2.6);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(3.0));
    }

    #[test]
    fn test_truncate() {
        let mut op = TruncateOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Float(2.9);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(2.0));

        op.inputs[0].default = Value::Float(-2.9);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(-2.0));
    }
}
