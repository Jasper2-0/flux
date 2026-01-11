//! Conversion operator for automatic type coercion
//!
//! This module provides the [`ConversionOp`] operator which is automatically
//! inserted by the graph when connecting ports of different but compatible types.
//!
//! # Auto-Conversion
//!
//! When connecting an output port to an input port with a different but coercible
//! type (e.g., Float to Vec3), the graph automatically inserts a ConversionOp
//! between them. This makes the conversion explicit and visible in the graph.
//!
//! # Example
//!
//! ```ignore
//! // Connecting Float output to Vec3 input automatically inserts:
//! // [Float Output] -> [ConversionOp(Float->Vec3)] -> [Vec3 Input]
//! graph.connect(float_source, 0, vec3_target, 0)?;
//! ```

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::port::{InputPort, OutputPort};
use flux_core::value::ValueType;

/// An operator that converts values from one type to another.
///
/// This operator is automatically inserted by the graph when connecting
/// ports of different but compatible types. It uses the [`Value::coerce_to`]
/// method to perform the actual conversion.
///
/// ConversionOps are marked as "synthetic" nodes - they are auto-generated
/// by the system rather than explicitly created by users. UI layers may
/// choose to hide or style these nodes differently.
pub struct ConversionOp {
    id: Id,
    source_type: ValueType,
    target_type: ValueType,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl ConversionOp {
    /// Create a new conversion operator from source type to target type.
    ///
    /// # Panics
    ///
    /// Panics if the source type cannot be coerced to the target type.
    /// Use [`ValueType::can_coerce_to`] to check compatibility first.
    pub fn new(source_type: ValueType, target_type: ValueType) -> Self {
        assert!(
            source_type.can_coerce_to(target_type),
            "Cannot create ConversionOp: {:?} cannot be coerced to {:?}",
            source_type,
            target_type
        );

        Self {
            id: Id::new(),
            source_type,
            target_type,
            inputs: [InputPort::new("In", source_type.default_value())],
            outputs: [OutputPort::new("Out", target_type)],
        }
    }

    /// Get the source type this operator converts from.
    pub fn source_type(&self) -> ValueType {
        self.source_type
    }

    /// Get the target type this operator converts to.
    pub fn target_type(&self) -> ValueType {
        self.target_type
    }

    /// Check if this is a synthetic (auto-generated) node.
    ///
    /// Conversion operators are always synthetic - they are inserted
    /// automatically by the graph when connecting incompatible types.
    pub fn is_synthetic(&self) -> bool {
        true
    }
}

impl Operator for ConversionOp {
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
        "Convert"
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
        // Get input value (either from connection or default)
        let input_value = match self.inputs[0].connection {
            Some((node_id, output_idx)) => get_input(node_id, output_idx),
            None => self.inputs[0].default.clone(),
        };

        // Coerce to target type
        let output_value = input_value
            .coerce_to(self.target_type)
            .unwrap_or_else(|| self.target_type.default_value());

        self.outputs[0].set(output_value);
    }

    fn can_operate_in_place(&self) -> bool {
        // Conversions don't need to preserve the input
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::Value;

    #[test]
    fn test_conversion_op_float_to_vec3() {
        let mut op = ConversionOp::new(ValueType::Float, ValueType::Vec3);

        assert_eq!(op.name(), "Convert");
        assert_eq!(op.source_type(), ValueType::Float);
        assert_eq!(op.target_type(), ValueType::Vec3);
        assert!(op.is_synthetic());

        // Set input default
        op.inputs_mut()[0].default = Value::Float(2.5);

        // Compute
        let ctx = EvalContext::new();
        let get_input = |_: Id, _: usize| Value::Float(0.0);
        op.compute(&ctx, &get_input);

        // Check output (Float 2.5 should broadcast to Vec3 [2.5, 2.5, 2.5])
        assert_eq!(op.outputs()[0].value, Value::Vec3([2.5, 2.5, 2.5]));
    }

    #[test]
    fn test_conversion_op_int_to_float() {
        let mut op = ConversionOp::new(ValueType::Int, ValueType::Float);

        op.inputs_mut()[0].default = Value::Int(42);

        let ctx = EvalContext::new();
        let get_input = |_: Id, _: usize| Value::Int(0);
        op.compute(&ctx, &get_input);

        assert_eq!(op.outputs()[0].value, Value::Float(42.0));
    }

    #[test]
    fn test_conversion_op_vec4_to_color() {
        let mut op = ConversionOp::new(ValueType::Vec4, ValueType::Color);

        op.inputs_mut()[0].default = Value::Vec4([1.0, 0.5, 0.25, 0.8]);

        let ctx = EvalContext::new();
        let get_input = |_: Id, _: usize| Value::Vec4([0.0; 4]);
        op.compute(&ctx, &get_input);

        if let Value::Color(c) = &op.outputs()[0].value {
            assert_eq!(c.r, 1.0);
            assert_eq!(c.g, 0.5);
            assert_eq!(c.b, 0.25);
            assert_eq!(c.a, 0.8);
        } else {
            panic!("Expected Color output");
        }
    }

    #[test]
    #[should_panic(expected = "Cannot create ConversionOp")]
    fn test_conversion_op_incompatible_types() {
        // String cannot be coerced to Vec3
        ConversionOp::new(ValueType::String, ValueType::Vec3);
    }

    #[test]
    fn test_can_operate_in_place() {
        let op = ConversionOp::new(ValueType::Float, ValueType::Vec3);
        assert!(op.can_operate_in_place());
    }
}
