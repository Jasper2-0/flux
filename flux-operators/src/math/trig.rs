//! Trigonometry operators: Sin, Cos, Tan, Atan2, DegreesToRadians, RadiansToDegrees
//!
//! Sin and Cos are polymorphic and work with:
//! Float, Int, Vec2, Vec3, Vec4

use crate::register_operators;
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::port::{InputPort, OutputPort};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta, Value};
use flux_macros::Operator;


// =============================================================================
// Sin Operator (polymorphic - NOT migrated)
// =============================================================================

pub struct SinOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl SinOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![InputPort::arithmetic("Angle", Value::Float(0.0))],
            outputs: vec![OutputPort::same_as_first("Result")],
        }
    }
}

impl Default for SinOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SinOp {
    fn id(&self) -> Id {
        self.id
    }
    fn name(&self) -> &'static str {
        "Sin"
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
        let angle = self.inputs[0].resolve(get_input);
        let result = angle.sin().unwrap_or(Value::Float(0.0));
        self.outputs[0].set(result);
    }
}

impl OperatorMeta for SinOp {
    fn category(&self) -> &'static str {
        "Math"
    }
    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }
    fn description(&self) -> &'static str {
        "Sine of angle (radians, per-component)"
    }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Angle").with_unit("rad")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Sin").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// =============================================================================
// Cos Operator (polymorphic - NOT migrated)
// =============================================================================

pub struct CosOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl CosOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![InputPort::arithmetic("Angle", Value::Float(0.0))],
            outputs: vec![OutputPort::same_as_first("Result")],
        }
    }
}

impl Default for CosOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for CosOp {
    fn id(&self) -> Id {
        self.id
    }
    fn name(&self) -> &'static str {
        "Cos"
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
        let angle = self.inputs[0].resolve(get_input);
        let result = angle.cos().unwrap_or(Value::Float(1.0));
        self.outputs[0].set(result);
    }
}

impl OperatorMeta for CosOp {
    fn category(&self) -> &'static str {
        "Math"
    }
    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }
    fn description(&self) -> &'static str {
        "Cosine of angle (radians, per-component)"
    }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Angle").with_unit("rad")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Cos").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// =============================================================================
// Tan Operator (using derive macro)
// =============================================================================

#[derive(Operator)]
#[operator(name = "Tan", category = "Math", description = "Tangent of angle (radians)")]
#[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
#[allow(dead_code)]
pub struct TanOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    #[input(label = "Angle", default = 0.0)]
    angle: f32,
    #[output(label = "Tan")]
    tan: f32,
}

impl TanOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let angle = self.get_angle(get_input);
        self.set_tan(angle.tan());
    }
}

// =============================================================================
// Atan2 Operator (using derive macro)
// =============================================================================

#[derive(Operator)]
#[operator(name = "Atan2", category = "Math", description = "Two-argument arctangent")]
#[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
#[allow(dead_code)]
pub struct Atan2Op {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "Y", default = 0.0)]
    y: f32,
    #[input(label = "X", default = 1.0)]
    x: f32,
    #[output(label = "Angle")]
    angle: f32,
}

impl Atan2Op {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let y = self.get_y(get_input);
        let x = self.get_x(get_input);
        self.set_angle(y.atan2(x));
    }
}

// =============================================================================
// DegreesToRadians Operator (using derive macro)
// =============================================================================

#[derive(Operator)]
#[operator(name = "DegreesToRadians", category = "Math", description = "Converts degrees to radians")]
#[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
#[allow(dead_code)]
pub struct DegreesToRadiansOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    #[input(label = "Degrees", default = 0.0)]
    degrees: f32,
    #[output(label = "Radians")]
    radians: f32,
}

impl DegreesToRadiansOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let degrees = self.get_degrees(get_input);
        self.set_radians(degrees.to_radians());
    }
}

// =============================================================================
// RadiansToDegrees Operator (using derive macro)
// =============================================================================

#[derive(Operator)]
#[operator(name = "RadiansToDegrees", category = "Math", description = "Converts radians to degrees")]
#[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
#[allow(dead_code)]
pub struct RadiansToDegreesOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    #[input(label = "Radians", default = 0.0)]
    radians: f32,
    #[output(label = "Degrees")]
    degrees: f32,
}

impl RadiansToDegreesOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let radians = self.get_radians(get_input);
        self.set_degrees(radians.to_degrees());
    }
}

// =============================================================================
// Registration
// =============================================================================

pub fn register(registry: &OperatorRegistry) {
    // Operators using #[derive(Operator)]
    register_operators!(
        registry,
        TanOp,
        Atan2Op,
        DegreesToRadiansOp,
        RadiansToDegreesOp,
    );

    // Polymorphic operators (manual impl)
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Sin",
            category: "Math",
            description: "Sine of angle (radians, per-component)",
        },
        || capture_meta(SinOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Cos",
            category: "Math",
            description: "Cosine of angle (radians, per-component)",
        },
        || capture_meta(CosOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Float(0.0)
    }

    // Float tests (backward compatibility)
    #[test]
    fn test_sin_float() {
        let mut op = SinOp::new();
        op.inputs[0].default = Value::Float(PI / 2.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        let result = op.outputs[0].value.as_float().unwrap();
        assert!((result - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_cos_float() {
        let mut op = CosOp::new();
        op.inputs[0].default = Value::Float(0.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(1.0));
    }

    #[test]
    fn test_tan() {
        let mut op = TanOp::new();
        op.inputs[0].default = Value::Float(PI / 4.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        let result = op.outputs[0].value.as_float().unwrap();
        assert!((result - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_atan2() {
        let mut op = Atan2Op::new();
        op.inputs[0].default = Value::Float(1.0);
        op.inputs[1].default = Value::Float(1.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        let result = op.outputs[0].value.as_float().unwrap();
        assert!((result - PI / 4.0).abs() < 0.0001);
    }

    #[test]
    fn test_degrees_to_radians() {
        let mut op = DegreesToRadiansOp::new();
        op.inputs[0].default = Value::Float(180.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        let result = op.outputs[0].value.as_float().unwrap();
        assert!((result - PI).abs() < 0.0001);
    }

    #[test]
    fn test_radians_to_degrees() {
        let mut op = RadiansToDegreesOp::new();
        op.inputs[0].default = Value::Float(PI);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        let result = op.outputs[0].value.as_float().unwrap();
        assert!((result - 180.0).abs() < 0.0001);
    }

    // Vec3 tests (polymorphic)
    #[test]
    fn test_sin_vec3() {
        let mut op = SinOp::new();
        op.inputs[0].default = Value::Vec3([0.0, PI / 2.0, PI]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        if let Value::Vec3(v) = &op.outputs[0].value {
            assert!((v[0] - 0.0).abs() < 0.0001);
            assert!((v[1] - 1.0).abs() < 0.0001);
            assert!((v[2] - 0.0).abs() < 0.0001);
        } else {
            panic!("Expected Vec3");
        }
    }

    #[test]
    fn test_cos_vec3() {
        let mut op = CosOp::new();
        op.inputs[0].default = Value::Vec3([0.0, PI, 2.0 * PI]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        if let Value::Vec3(v) = &op.outputs[0].value {
            assert!((v[0] - 1.0).abs() < 0.0001);
            assert!((v[1] - (-1.0)).abs() < 0.0001);
            assert!((v[2] - 1.0).abs() < 0.0001);
        } else {
            panic!("Expected Vec3");
        }
    }
}
