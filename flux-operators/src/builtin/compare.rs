//! Compare operator - compares two values

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::port::{InputPort, OutputPort};

use flux_core::{category_colors, InputResolver, Operator, OperatorMeta, PinShape, PortMeta};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompareMode {
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

impl CompareMode {
    /// Convert a mode index (from UI) to a CompareMode.
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(CompareMode::Equal),
            1 => Some(CompareMode::NotEqual),
            2 => Some(CompareMode::LessThan),
            3 => Some(CompareMode::LessOrEqual),
            4 => Some(CompareMode::GreaterThan),
            5 => Some(CompareMode::GreaterOrEqual),
            _ => None,
        }
    }

    /// Convert to a mode index (for UI).
    pub fn to_index(self) -> usize {
        match self {
            CompareMode::Equal => 0,
            CompareMode::NotEqual => 1,
            CompareMode::LessThan => 2,
            CompareMode::LessOrEqual => 3,
            CompareMode::GreaterThan => 4,
            CompareMode::GreaterOrEqual => 5,
        }
    }
}

pub struct CompareOp {
    id: Id,
    pub mode: CompareMode,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl CompareOp {
    pub fn new(mode: CompareMode) -> Self {
        Self {
            id: Id::new(),
            mode,
            inputs: [InputPort::float("A", 0.0), InputPort::float("B", 0.0)],
            outputs: [OutputPort::bool("Result")],
        }
    }

    pub fn less_than() -> Self {
        Self::new(CompareMode::LessThan)
    }

    pub fn greater_than() -> Self {
        Self::new(CompareMode::GreaterThan)
    }

    pub fn equal() -> Self {
        Self::new(CompareMode::Equal)
    }
}

impl Operator for CompareOp {
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
        "Compare"
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

        let result = match self.mode {
            CompareMode::Equal => a == b,
            CompareMode::NotEqual => a != b,
            CompareMode::LessThan => a < b,
            CompareMode::LessOrEqual => a <= b,
            CompareMode::GreaterThan => a > b,
            CompareMode::GreaterOrEqual => a >= b,
        };

        self.outputs[0].set_bool(result);
    }
}

impl OperatorMeta for CompareOp {
    fn category(&self) -> &'static str {
        "Logic"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::LOGIC
    }

    fn description(&self) -> &'static str {
        match self.mode {
            CompareMode::Equal => "Returns true if A equals B",
            CompareMode::NotEqual => "Returns true if A does not equal B",
            CompareMode::LessThan => "Returns true if A is less than B",
            CompareMode::LessOrEqual => "Returns true if A is less than or equal to B",
            CompareMode::GreaterThan => "Returns true if A is greater than B",
            CompareMode::GreaterOrEqual => "Returns true if A is greater than or equal to B",
        }
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

    fn compute_compare(op: &mut CompareOp) -> bool {
        let ctx = EvalContext::new();
        op.compute(&ctx, &|_, _| Value::Float(0.0));
        op.outputs()[0].value.as_bool().unwrap()
    }

    #[test]
    fn test_equal_true() {
        let mut op = CompareOp::new(CompareMode::Equal);
        op.inputs_mut()[0].default = Value::Float(5.0);
        op.inputs_mut()[1].default = Value::Float(5.0);
        assert!(compute_compare(&mut op));
    }

    #[test]
    fn test_equal_false() {
        let mut op = CompareOp::new(CompareMode::Equal);
        op.inputs_mut()[0].default = Value::Float(5.0);
        op.inputs_mut()[1].default = Value::Float(3.0);
        assert!(!compute_compare(&mut op));
    }

    #[test]
    fn test_not_equal_true() {
        let mut op = CompareOp::new(CompareMode::NotEqual);
        op.inputs_mut()[0].default = Value::Float(5.0);
        op.inputs_mut()[1].default = Value::Float(3.0);
        assert!(compute_compare(&mut op));
    }

    #[test]
    fn test_not_equal_false() {
        let mut op = CompareOp::new(CompareMode::NotEqual);
        op.inputs_mut()[0].default = Value::Float(5.0);
        op.inputs_mut()[1].default = Value::Float(5.0);
        assert!(!compute_compare(&mut op));
    }

    #[test]
    fn test_less_than() {
        let mut op = CompareOp::new(CompareMode::LessThan);

        op.inputs_mut()[0].default = Value::Float(3.0);
        op.inputs_mut()[1].default = Value::Float(5.0);
        assert!(compute_compare(&mut op)); // 3 < 5

        op.inputs_mut()[0].default = Value::Float(5.0);
        op.inputs_mut()[1].default = Value::Float(3.0);
        assert!(!compute_compare(&mut op)); // 5 < 3 is false

        op.inputs_mut()[0].default = Value::Float(5.0);
        op.inputs_mut()[1].default = Value::Float(5.0);
        assert!(!compute_compare(&mut op)); // 5 < 5 is false
    }

    #[test]
    fn test_less_or_equal() {
        let mut op = CompareOp::new(CompareMode::LessOrEqual);

        op.inputs_mut()[0].default = Value::Float(3.0);
        op.inputs_mut()[1].default = Value::Float(5.0);
        assert!(compute_compare(&mut op)); // 3 <= 5

        op.inputs_mut()[0].default = Value::Float(5.0);
        op.inputs_mut()[1].default = Value::Float(5.0);
        assert!(compute_compare(&mut op)); // 5 <= 5

        op.inputs_mut()[0].default = Value::Float(6.0);
        op.inputs_mut()[1].default = Value::Float(5.0);
        assert!(!compute_compare(&mut op)); // 6 <= 5 is false
    }

    #[test]
    fn test_greater_than() {
        let mut op = CompareOp::new(CompareMode::GreaterThan);

        op.inputs_mut()[0].default = Value::Float(5.0);
        op.inputs_mut()[1].default = Value::Float(3.0);
        assert!(compute_compare(&mut op)); // 5 > 3

        op.inputs_mut()[0].default = Value::Float(3.0);
        op.inputs_mut()[1].default = Value::Float(5.0);
        assert!(!compute_compare(&mut op)); // 3 > 5 is false
    }

    #[test]
    fn test_greater_or_equal() {
        let mut op = CompareOp::new(CompareMode::GreaterOrEqual);

        op.inputs_mut()[0].default = Value::Float(5.0);
        op.inputs_mut()[1].default = Value::Float(3.0);
        assert!(compute_compare(&mut op)); // 5 >= 3

        op.inputs_mut()[0].default = Value::Float(5.0);
        op.inputs_mut()[1].default = Value::Float(5.0);
        assert!(compute_compare(&mut op)); // 5 >= 5

        op.inputs_mut()[0].default = Value::Float(3.0);
        op.inputs_mut()[1].default = Value::Float(5.0);
        assert!(!compute_compare(&mut op)); // 3 >= 5 is false
    }

    #[test]
    fn test_mode_index_conversion() {
        for i in 0..6 {
            let mode = CompareMode::from_index(i).unwrap();
            assert_eq!(mode.to_index(), i);
        }
        assert!(CompareMode::from_index(6).is_none());
    }
}
