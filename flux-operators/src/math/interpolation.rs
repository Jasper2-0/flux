//! Interpolation operators: Lerp, SmoothStep, Remap, InverseLerp, MapRange
//!
//! Lerp and SmoothStep are polymorphic and work with:
//! Float, Int, Vec2, Vec3, Vec4, Color

use std::any::Any;

use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::port::{InputPort, OutputPort};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta, Value};
use flux_macros::Operator;


// =============================================================================
// Lerp Operator (polymorphic - NOT migrated)
// =============================================================================

pub struct LerpOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl LerpOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![
                InputPort::arithmetic("A", Value::Float(0.0)),
                InputPort::arithmetic("B", Value::Float(1.0)),
                InputPort::float("T", 0.5), // T is always float
            ],
            outputs: vec![OutputPort::wider_of_inputs("Result")],
        }
    }
}

impl Default for LerpOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for LerpOp {
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
        "Lerp"
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
        let a = self.inputs[0].resolve(get_input);
        let b = self.inputs[1].resolve(get_input);
        let t = self.inputs[2].resolve(get_input);

        let result = a.lerp(&b, &t).unwrap_or(Value::Float(0.0));
        self.outputs[0].set(result);
    }
}

impl OperatorMeta for LerpOp {
    fn category(&self) -> &'static str {
        "Math"
    }
    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }
    fn description(&self) -> &'static str {
        "Linear interpolation between A and B (per-component)"
    }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            2 => Some(PortMeta::new("T").with_range(0.0, 1.0)),
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
// SmoothStep Operator (polymorphic - NOT migrated)
// =============================================================================

pub struct SmoothStepOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl SmoothStepOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![
                InputPort::arithmetic("Edge0", Value::Float(0.0)),
                InputPort::arithmetic("Edge1", Value::Float(1.0)),
                InputPort::arithmetic("X", Value::Float(0.5)),
            ],
            outputs: vec![OutputPort::same_as_input("Result", 2)], // Same as X
        }
    }
}

impl Default for SmoothStepOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SmoothStepOp {
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
        "SmoothStep"
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
        let edge0 = self.inputs[0].resolve(get_input);
        let edge1 = self.inputs[1].resolve(get_input);
        let x = self.inputs[2].resolve(get_input);

        let result = x.smoothstep(&edge0, &edge1).unwrap_or(Value::Float(0.0));
        self.outputs[0].set(result);
    }
}

impl OperatorMeta for SmoothStepOp {
    fn category(&self) -> &'static str {
        "Math"
    }
    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }
    fn description(&self) -> &'static str {
        "Hermite interpolation with smooth edges (per-component)"
    }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Edge0")),
            1 => Some(PortMeta::new("Edge1")),
            2 => Some(PortMeta::new("X")),
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
// Remap Operator (using derive macro)
// =============================================================================

#[derive(Operator)]
#[operator(name = "Remap", category = "Math", description = "Remaps value from one range to another")]
#[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
#[allow(dead_code)]
pub struct RemapOp {
    id: Id,
    inputs: [InputPort; 5],
    outputs: [OutputPort; 1],
    #[input(label = "Value", default = 0.0)]
    value: f32,
    #[input(label = "InMin", default = 0.0)]
    in_min: f32,
    #[input(label = "InMax", default = 1.0)]
    in_max: f32,
    #[input(label = "OutMin", default = 0.0)]
    out_min: f32,
    #[input(label = "OutMax", default = 1.0)]
    out_max: f32,
    #[output(label = "Result")]
    result: f32,
}

impl RemapOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = self.get_value(get_input);
        let in_min = self.get_in_min(get_input);
        let in_max = self.get_in_max(get_input);
        let out_min = self.get_out_min(get_input);
        let out_max = self.get_out_max(get_input);

        let in_range = in_max - in_min;
        if in_range.abs() < f32::EPSILON {
            self.set_result(out_min);
            return;
        }

        let t = (value - in_min) / in_range;
        self.set_result(out_min + t * (out_max - out_min));
    }
}

// =============================================================================
// InverseLerp Operator (using derive macro)
// =============================================================================

#[derive(Operator)]
#[operator(name = "InverseLerp", category = "Math", description = "Gets T from lerp result")]
#[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
#[allow(dead_code)]
pub struct InverseLerpOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
    #[input(label = "A", default = 0.0)]
    a: f32,
    #[input(label = "B", default = 1.0)]
    b: f32,
    #[input(label = "Value", default = 0.5)]
    value: f32,
    #[output(label = "T")]
    t: f32,
}

impl InverseLerpOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_a(get_input);
        let b = self.get_b(get_input);
        let value = self.get_value(get_input);

        let range = b - a;
        if range.abs() < f32::EPSILON {
            self.set_t(0.0);
            return;
        }

        self.set_t((value - a) / range);
    }
}

// =============================================================================
// MapRange Operator (using derive macro)
// =============================================================================

#[derive(Operator)]
#[operator(name = "MapRange", category = "Math", description = "Maps value from one range to another")]
#[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
#[allow(dead_code)]
pub struct MapRangeOp {
    id: Id,
    inputs: [InputPort; 5],
    outputs: [OutputPort; 1],
    #[input(label = "Value", default = 0.0)]
    value: f32,
    #[input(label = "FromMin", default = 0.0)]
    from_min: f32,
    #[input(label = "FromMax", default = 1.0)]
    from_max: f32,
    #[input(label = "ToMin", default = 0.0)]
    to_min: f32,
    #[input(label = "ToMax", default = 1.0)]
    to_max: f32,
    #[output(label = "Result")]
    result: f32,
}

impl MapRangeOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = self.get_value(get_input);
        let from_min = self.get_from_min(get_input);
        let from_max = self.get_from_max(get_input);
        let to_min = self.get_to_min(get_input);
        let to_max = self.get_to_max(get_input);

        let from_range = from_max - from_min;
        if from_range.abs() < f32::EPSILON {
            self.set_result(to_min);
            return;
        }

        let t = (value - from_min) / from_range;
        self.set_result(to_min + t * (to_max - to_min));
    }
}

// =============================================================================
// Registration
// =============================================================================

pub fn register(registry: &OperatorRegistry) {
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Lerp",
            category: "Math",
            description: "Linear interpolation between A and B (per-component)",
        },
        || capture_meta(LerpOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "SmoothStep",
            category: "Math",
            description: "Hermite interpolation with smooth edges (per-component)",
        },
        || capture_meta(SmoothStepOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Remap",
            category: "Math",
            description: "Remaps value from one range to another",
        },
        || capture_meta(RemapOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "InverseLerp",
            category: "Math",
            description: "Gets T from lerp result",
        },
        || capture_meta(InverseLerpOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "MapRange",
            category: "Math",
            description: "Maps value from one range to another",
        },
        || capture_meta(MapRangeOp::new()),
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
    fn test_lerp_float() {
        let mut op = LerpOp::new();
        op.inputs[0].default = Value::Float(0.0);
        op.inputs[1].default = Value::Float(10.0);
        op.inputs[2].default = Value::Float(0.5);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(5.0));
    }

    #[test]
    fn test_lerp_extrapolation() {
        let mut op = LerpOp::new();
        op.inputs[0].default = Value::Float(0.0);
        op.inputs[1].default = Value::Float(10.0);
        op.inputs[2].default = Value::Float(1.5);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(15.0));
    }

    #[test]
    fn test_smoothstep_float() {
        let mut op = SmoothStepOp::new();
        op.inputs[0].default = Value::Float(0.0);
        op.inputs[1].default = Value::Float(1.0);
        op.inputs[2].default = Value::Float(0.5);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(0.5));
    }

    #[test]
    fn test_smoothstep_edges() {
        let mut op = SmoothStepOp::new();
        let ctx = EvalContext::new();

        // Before edge0
        op.inputs[0].default = Value::Float(0.0);
        op.inputs[1].default = Value::Float(1.0);
        op.inputs[2].default = Value::Float(-0.5);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(0.0));

        // After edge1
        op.inputs[2].default = Value::Float(1.5);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(1.0));
    }

    #[test]
    fn test_remap() {
        let mut op = RemapOp::new();
        op.inputs[0].default = Value::Float(5.0);
        op.inputs[1].default = Value::Float(0.0);
        op.inputs[2].default = Value::Float(10.0);
        op.inputs[3].default = Value::Float(0.0);
        op.inputs[4].default = Value::Float(100.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(50.0));
    }

    #[test]
    fn test_inverse_lerp() {
        let mut op = InverseLerpOp::new();
        op.inputs[0].default = Value::Float(0.0);
        op.inputs[1].default = Value::Float(10.0);
        op.inputs[2].default = Value::Float(5.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(0.5));
    }

    #[test]
    fn test_map_range() {
        let mut op = MapRangeOp::new();
        op.inputs[0].default = Value::Float(0.5);
        op.inputs[1].default = Value::Float(0.0);
        op.inputs[2].default = Value::Float(1.0);
        op.inputs[3].default = Value::Float(100.0);
        op.inputs[4].default = Value::Float(200.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(150.0));
    }

    // Vec3 tests (polymorphic)
    #[test]
    fn test_lerp_vec3() {
        let mut op = LerpOp::new();
        op.inputs[0].default = Value::Vec3([0.0, 0.0, 0.0]);
        op.inputs[1].default = Value::Vec3([10.0, 20.0, 30.0]);
        op.inputs[2].default = Value::Float(0.5);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value, Value::Vec3([5.0, 10.0, 15.0]));
    }

    #[test]
    fn test_lerp_color() {
        let mut op = LerpOp::new();
        op.inputs[0].default = Value::Color(Color::rgba(0.0, 0.0, 0.0, 1.0));
        op.inputs[1].default = Value::Color(Color::rgba(1.0, 1.0, 1.0, 1.0));
        op.inputs[2].default = Value::Float(0.5);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        if let Value::Color(c) = &op.outputs[0].value {
            assert!((c.r - 0.5).abs() < 0.001);
            assert!((c.g - 0.5).abs() < 0.001);
            assert!((c.b - 0.5).abs() < 0.001);
        } else {
            panic!("Expected Color");
        }
    }

    #[test]
    fn test_smoothstep_vec3() {
        let mut op = SmoothStepOp::new();
        op.inputs[0].default = Value::Float(0.0); // edge0
        op.inputs[1].default = Value::Float(1.0); // edge1
        op.inputs[2].default = Value::Vec3([0.0, 0.5, 1.0]); // x
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        if let Value::Vec3(v) = &op.outputs[0].value {
            assert!((v[0] - 0.0).abs() < 0.001);
            assert!((v[1] - 0.5).abs() < 0.001);
            assert!((v[2] - 1.0).abs() < 0.001);
        } else {
            panic!("Expected Vec3");
        }
    }
}
