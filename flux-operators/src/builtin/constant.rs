//! Constant operator - produces a fixed value
//!
//! This operator uses the "Identity" pattern: it has an input pin with a default value.
//! - If the input is not connected, it outputs the default value (acting as a constant)
//! - If the input is connected, it passes through the connected value (acting as identity)
//!
//! This design allows the constant value to be edited via pin value drag on the input pin.

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::port::{InputPort, OutputPort};

use flux_core::{category_colors, InputResolver, Operator, OperatorMeta, PinShape, PortMeta};

pub struct ConstantOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl ConstantOp {
    pub fn new(value: f32) -> Self {
        let mut output = OutputPort::float("Value");
        output.set_float(value);
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Value", value)],
            outputs: [output],
        }
    }

    /// Change the constant value (marks output as dirty)
    pub fn set_value(&mut self, value: f32) {
        self.inputs[0].default = flux_core::Value::Float(value);
        self.outputs[0].mark_dirty();
    }

    /// Get the current constant value (the input's default)
    pub fn value(&self) -> f32 {
        self.inputs[0].default.as_float().unwrap_or(0.0)
    }
}

impl Operator for ConstantOp {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &'static str {
        "Constant"
    }

    fn inputs(&self) -> &[InputPort] {
        &self.inputs
    }

    fn inputs_mut(&mut self) -> &mut [InputPort] {
        &mut self.inputs
    }

    fn outputs(&self) -> &[OutputPort] {
        &self.outputs
    }

    fn outputs_mut(&mut self) -> &mut [OutputPort] {
        &mut self.outputs
    }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        // Identity pattern: use connected value if available, otherwise use default
        let value = match self.inputs[0].connection {
            Some((node_id, output_idx)) => {
                get_input(node_id, output_idx).as_float().unwrap_or(0.0)
            }
            None => self.inputs[0].default.as_float().unwrap_or(0.0),
        };
        self.outputs[0].set_float(value);
    }
}

impl OperatorMeta for ConstantOp {
    fn category(&self) -> &'static str {
        "Sources"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::SOURCES
    }

    fn description(&self) -> &'static str {
        "Outputs a constant float value, or passes through a connected input"
    }

    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value").with_shape(PinShape::CircleFilled)),
            _ => None,
        }
    }

    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::Value;

    fn compute_op(op: &mut ConstantOp) -> f32 {
        let ctx = EvalContext::new();
        op.compute(&ctx, &|_, _| Value::Float(0.0));
        op.outputs()[0].value.as_float().unwrap()
    }

    #[test]
    fn test_constant_initial_value() {
        let op = ConstantOp::new(42.0);
        assert_eq!(op.value(), 42.0);
    }

    #[test]
    fn test_constant_compute() {
        let mut op = ConstantOp::new(3.14);
        let result = compute_op(&mut op);
        assert!((result - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_constant_set_value() {
        let mut op = ConstantOp::new(1.0);
        assert_eq!(op.value(), 1.0);

        op.set_value(99.0);
        assert_eq!(op.value(), 99.0);

        let result = compute_op(&mut op);
        assert_eq!(result, 99.0);
    }

    #[test]
    fn test_constant_zero() {
        let mut op = ConstantOp::new(0.0);
        let result = compute_op(&mut op);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_constant_negative() {
        let mut op = ConstantOp::new(-123.456);
        let result = compute_op(&mut op);
        assert!((result - (-123.456)).abs() < 0.001);
    }

    #[test]
    fn test_constant_output_dirty_on_set() {
        let mut op = ConstantOp::new(1.0);

        // Setting value should mark output as dirty
        op.set_value(2.0);
        assert!(op.outputs()[0].is_dirty());
    }
}
