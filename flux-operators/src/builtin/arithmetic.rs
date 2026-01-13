//! Arithmetic operators - Add and Multiply (polymorphic)
//!
//! These operators are now polymorphic and work with Float, Int, Vec2, Vec3, Vec4, Color.

// Re-export polymorphic versions from math module
pub use crate::math::{AddOp, MultiplyOp};

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::context::EvalContext;
    use flux_core::operator::Operator;
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
