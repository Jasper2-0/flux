//! Arithmetic operators - Add and Multiply

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::port::{InputPort, OutputPort};

use flux_core::{category_colors, InputResolver, Operator, OperatorMeta, PinShape, PortMeta};

// ============================================================
// Add Operator - adds two values
// ============================================================

pub struct AddOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl AddOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("A", 0.0), InputPort::float("B", 0.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for AddOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for AddOp {
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
        "Add"
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
        let a = match self.inputs[0].connection {
            Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
            None => self.inputs[0].default.as_float().unwrap_or(0.0),
        };
        let b = match self.inputs[1].connection {
            Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
            None => self.inputs[1].default.as_float().unwrap_or(0.0),
        };

        let result = a + b;
        self.outputs[0].set_float(result);
    }
}

impl OperatorMeta for AddOp {
    fn category(&self) -> &'static str {
        "Math"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }

    fn description(&self) -> &'static str {
        "Adds two values together"
    }

    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
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

// ============================================================
// Multiply Operator - multiplies two values
// ============================================================

pub struct MultiplyOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl MultiplyOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("A", 0.0), InputPort::float("B", 1.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for MultiplyOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for MultiplyOp {
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
        "Multiply"
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
        let a = match self.inputs[0].connection {
            Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
            None => self.inputs[0].default.as_float().unwrap_or(0.0),
        };
        let b = match self.inputs[1].connection {
            Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
            None => self.inputs[1].default.as_float().unwrap_or(0.0),
        };

        let result = a * b;
        self.outputs[0].set_float(result);
    }
}

impl OperatorMeta for MultiplyOp {
    fn category(&self) -> &'static str {
        "Math"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }

    fn description(&self) -> &'static str {
        "Multiplies two values together"
    }

    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
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

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::Value;

    fn compute_op(op: &mut impl Operator) {
        let ctx = EvalContext::new();
        op.compute(&ctx, &|_, _| Value::Float(0.0));
    }

    #[test]
    fn test_add_defaults() {
        let mut op = AddOp::new();
        compute_op(&mut op);
        let result = op.outputs()[0].value.as_float().unwrap();
        assert_eq!(result, 0.0); // 0 + 0 = 0
    }

    #[test]
    fn test_add_with_values() {
        let mut op = AddOp::new();
        op.inputs_mut()[0].default = Value::Float(3.0);
        op.inputs_mut()[1].default = Value::Float(4.0);
        compute_op(&mut op);
        let result = op.outputs()[0].value.as_float().unwrap();
        assert_eq!(result, 7.0); // 3 + 4 = 7
    }

    #[test]
    fn test_add_negative() {
        let mut op = AddOp::new();
        op.inputs_mut()[0].default = Value::Float(5.0);
        op.inputs_mut()[1].default = Value::Float(-3.0);
        compute_op(&mut op);
        let result = op.outputs()[0].value.as_float().unwrap();
        assert_eq!(result, 2.0); // 5 + (-3) = 2
    }

    #[test]
    fn test_multiply_defaults() {
        let mut op = MultiplyOp::new();
        compute_op(&mut op);
        let result = op.outputs()[0].value.as_float().unwrap();
        assert_eq!(result, 0.0); // 0 * 1 = 0
    }

    #[test]
    fn test_multiply_with_values() {
        let mut op = MultiplyOp::new();
        op.inputs_mut()[0].default = Value::Float(3.0);
        op.inputs_mut()[1].default = Value::Float(4.0);
        compute_op(&mut op);
        let result = op.outputs()[0].value.as_float().unwrap();
        assert_eq!(result, 12.0); // 3 * 4 = 12
    }

    #[test]
    fn test_multiply_by_zero() {
        let mut op = MultiplyOp::new();
        op.inputs_mut()[0].default = Value::Float(100.0);
        op.inputs_mut()[1].default = Value::Float(0.0);
        compute_op(&mut op);
        let result = op.outputs()[0].value.as_float().unwrap();
        assert_eq!(result, 0.0); // 100 * 0 = 0
    }

    #[test]
    fn test_multiply_negative() {
        let mut op = MultiplyOp::new();
        op.inputs_mut()[0].default = Value::Float(-2.0);
        op.inputs_mut()[1].default = Value::Float(3.0);
        compute_op(&mut op);
        let result = op.outputs()[0].value.as_float().unwrap();
        assert_eq!(result, -6.0); // -2 * 3 = -6
    }
}
