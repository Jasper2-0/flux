//! Comparison operators: Min, Max, Clamp, Sign, Step
//!
//! All comparison operators are polymorphic and work with:
//! Float, Int, Vec2, Vec3, Vec4, Color

use std::any::Any;

use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::port::{InputPort, OutputPort};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta, Value};

// =============================================================================
// Helper to get value from input (polymorphic)
// =============================================================================

fn get_value(input: &InputPort, get_input: InputResolver) -> Value {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx),
        None => input.default.clone(),
    }
}

// =============================================================================
// Min Operator (polymorphic)
// =============================================================================

pub struct MinOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl MinOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![
                InputPort::arithmetic("A", Value::Float(0.0)),
                InputPort::arithmetic("B", Value::Float(0.0)),
            ],
            outputs: vec![OutputPort::wider_of_inputs("Result")],
        }
    }
}

impl Default for MinOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for MinOp {
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
        "Min"
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
        let a = get_value(&self.inputs[0], get_input);
        let b = get_value(&self.inputs[1], get_input);

        let result = a.min_value(&b).unwrap_or(Value::Float(0.0));
        self.outputs[0].set(result);
    }
}

impl OperatorMeta for MinOp {
    fn category(&self) -> &'static str {
        "Math"
    }
    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }
    fn description(&self) -> &'static str {
        "Per-component minimum of two values"
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
            0 => Some(PortMeta::new("Min").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// =============================================================================
// Max Operator (polymorphic)
// =============================================================================

pub struct MaxOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl MaxOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![
                InputPort::arithmetic("A", Value::Float(0.0)),
                InputPort::arithmetic("B", Value::Float(0.0)),
            ],
            outputs: vec![OutputPort::wider_of_inputs("Result")],
        }
    }
}

impl Default for MaxOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for MaxOp {
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
        "Max"
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
        let a = get_value(&self.inputs[0], get_input);
        let b = get_value(&self.inputs[1], get_input);

        let result = a.max_value(&b).unwrap_or(Value::Float(0.0));
        self.outputs[0].set(result);
    }
}

impl OperatorMeta for MaxOp {
    fn category(&self) -> &'static str {
        "Math"
    }
    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }
    fn description(&self) -> &'static str {
        "Per-component maximum of two values"
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
            0 => Some(PortMeta::new("Max").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// =============================================================================
// Clamp Operator (polymorphic)
// =============================================================================

pub struct ClampOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl ClampOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![
                InputPort::arithmetic("Value", Value::Float(0.0)),
                InputPort::arithmetic("Min", Value::Float(0.0)),
                InputPort::arithmetic("Max", Value::Float(1.0)),
            ],
            outputs: vec![OutputPort::same_as_first("Result")],
        }
    }
}

impl Default for ClampOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ClampOp {
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
        "Clamp"
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
        let value = get_value(&self.inputs[0], get_input);
        let min_val = get_value(&self.inputs[1], get_input);
        let max_val = get_value(&self.inputs[2], get_input);

        let result = value
            .clamp_value(&min_val, &max_val)
            .unwrap_or(Value::Float(0.0));
        self.outputs[0].set(result);
    }
}

impl OperatorMeta for ClampOp {
    fn category(&self) -> &'static str {
        "Math"
    }
    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }
    fn description(&self) -> &'static str {
        "Clamps value to range [min, max] per-component"
    }
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

// =============================================================================
// Sign Operator (polymorphic)
// =============================================================================

pub struct SignOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl SignOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![InputPort::arithmetic("Value", Value::Float(0.0))],
            outputs: vec![OutputPort::same_as_first("Result")],
        }
    }
}

impl Default for SignOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SignOp {
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
        "Sign"
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
        let value = get_value(&self.inputs[0], get_input);
        let result = value.sign().unwrap_or(Value::Float(0.0));
        self.outputs[0].set(result);
    }
}

impl OperatorMeta for SignOp {
    fn category(&self) -> &'static str {
        "Math"
    }
    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }
    fn description(&self) -> &'static str {
        "Returns -1, 0, or 1 per-component based on sign"
    }
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

// =============================================================================
// Step Operator (polymorphic)
// =============================================================================

pub struct StepOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl StepOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![
                InputPort::arithmetic("Edge", Value::Float(0.0)),
                InputPort::arithmetic("Value", Value::Float(0.0)),
            ],
            outputs: vec![OutputPort::wider_of_inputs("Result")],
        }
    }
}

impl Default for StepOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for StepOp {
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
        "Step"
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
        let edge = get_value(&self.inputs[0], get_input);
        let value = get_value(&self.inputs[1], get_input);

        // GLSL step: 0 if value < edge, else 1
        let result = value.step(&edge).unwrap_or(Value::Float(0.0));
        self.outputs[0].set(result);
    }
}

impl OperatorMeta for StepOp {
    fn category(&self) -> &'static str {
        "Math"
    }
    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }
    fn description(&self) -> &'static str {
        "Returns 0 if value < edge, else 1 (per-component)"
    }
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

// =============================================================================
// Registration
// =============================================================================

pub fn register(registry: &OperatorRegistry) {
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Min",
            category: "Math",
            description: "Per-component minimum of two values",
        },
        || capture_meta(MinOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Max",
            category: "Math",
            description: "Per-component maximum of two values",
        },
        || capture_meta(MaxOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Clamp",
            category: "Math",
            description: "Clamps value to range [min, max] per-component",
        },
        || capture_meta(ClampOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Sign",
            category: "Math",
            description: "Returns -1, 0, or 1 per-component based on sign",
        },
        || capture_meta(SignOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Step",
            category: "Math",
            description: "Returns 0 if value < edge, else 1 (per-component)",
        },
        || capture_meta(StepOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::Color;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Float(0.0)
    }

    // Float tests (backward compatibility)
    #[test]
    fn test_min_float() {
        let mut op = MinOp::new();
        op.inputs[0].default = Value::Float(5.0);
        op.inputs[1].default = Value::Float(3.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(3.0));
    }

    #[test]
    fn test_max_float() {
        let mut op = MaxOp::new();
        op.inputs[0].default = Value::Float(5.0);
        op.inputs[1].default = Value::Float(3.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(5.0));
    }

    #[test]
    fn test_clamp_float() {
        let mut op = ClampOp::new();
        op.inputs[0].default = Value::Float(1.5);
        op.inputs[1].default = Value::Float(0.0);
        op.inputs[2].default = Value::Float(1.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(1.0));
    }

    #[test]
    fn test_sign_float() {
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
    fn test_step_float() {
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

    // Vec3 tests (polymorphic)
    #[test]
    fn test_min_vec3() {
        let mut op = MinOp::new();
        op.inputs[0].default = Value::Vec3([1.0, 5.0, 3.0]);
        op.inputs[1].default = Value::Vec3([2.0, 2.0, 4.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value, Value::Vec3([1.0, 2.0, 3.0]));
    }

    #[test]
    fn test_max_vec3() {
        let mut op = MaxOp::new();
        op.inputs[0].default = Value::Vec3([1.0, 5.0, 3.0]);
        op.inputs[1].default = Value::Vec3([2.0, 2.0, 4.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value, Value::Vec3([2.0, 5.0, 4.0]));
    }

    #[test]
    fn test_clamp_vec3() {
        let mut op = ClampOp::new();
        op.inputs[0].default = Value::Vec3([-0.5, 0.5, 1.5]);
        op.inputs[1].default = Value::Vec3([0.0, 0.0, 0.0]);
        op.inputs[2].default = Value::Vec3([1.0, 1.0, 1.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value, Value::Vec3([0.0, 0.5, 1.0]));
    }

    #[test]
    fn test_clamp_vec3_scalar_bounds() {
        let mut op = ClampOp::new();
        op.inputs[0].default = Value::Vec3([-0.5, 0.5, 1.5]);
        op.inputs[1].default = Value::Float(0.0);
        op.inputs[2].default = Value::Float(1.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value, Value::Vec3([0.0, 0.5, 1.0]));
    }

    #[test]
    fn test_sign_vec3() {
        let mut op = SignOp::new();
        op.inputs[0].default = Value::Vec3([5.0, -3.0, 0.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value, Value::Vec3([1.0, -1.0, 0.0]));
    }

    #[test]
    fn test_step_vec3() {
        let mut op = StepOp::new();
        op.inputs[0].default = Value::Vec3([0.5, 0.5, 0.5]); // edge
        op.inputs[1].default = Value::Vec3([0.3, 0.5, 0.7]); // value
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value, Value::Vec3([0.0, 1.0, 1.0]));
    }

    // Color test
    #[test]
    fn test_min_color() {
        let mut op = MinOp::new();
        op.inputs[0].default = Value::Color(Color::rgba(1.0, 0.0, 0.5, 1.0));
        op.inputs[1].default = Value::Color(Color::rgba(0.5, 0.5, 0.5, 0.5));
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        if let Value::Color(c) = &op.outputs[0].value {
            assert!((c.r - 0.5).abs() < 0.001);
            assert!((c.g - 0.0).abs() < 0.001);
            assert!((c.b - 0.5).abs() < 0.001);
            assert!((c.a - 0.5).abs() < 0.001);
        } else {
            panic!("Expected Color");
        }
    }
}
