//! Trigonometry operators: Sin, Cos, Tan, Atan2, DegreesToRadians, RadiansToDegrees
//!
//! Sin and Cos are polymorphic and work with:
//! Float, Int, Vec2, Vec3, Vec4

use std::any::Any;

use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::port::{InputPort, OutputPort};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta, Value};

// =============================================================================
// Helper functions
// =============================================================================

fn get_float(input: &InputPort, get_input: InputResolver) -> f32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
        None => input.default.as_float().unwrap_or(0.0),
    }
}

fn get_value(input: &InputPort, get_input: InputResolver) -> Value {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx),
        None => input.default.clone(),
    }
}

// =============================================================================
// Sin Operator (polymorphic)
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
        let angle = get_value(&self.inputs[0], get_input);
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
// Cos Operator (polymorphic)
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
        let angle = get_value(&self.inputs[0], get_input);
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
// Tan Operator (float-only)
// =============================================================================

pub struct TanOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl TanOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Angle", 0.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for TanOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for TanOp {
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
        "Tan"
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
        let angle = get_float(&self.inputs[0], get_input);
        self.outputs[0].set_float(angle.tan());
    }
}

impl OperatorMeta for TanOp {
    fn category(&self) -> &'static str {
        "Math"
    }
    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }
    fn description(&self) -> &'static str {
        "Tangent of angle (radians)"
    }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Angle").with_unit("rad")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Tan").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// =============================================================================
// Atan2 Operator (float-only, inherently scalar)
// =============================================================================

pub struct Atan2Op {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl Atan2Op {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Y", 0.0), InputPort::float("X", 1.0)],
            outputs: [OutputPort::float("Angle")],
        }
    }
}

impl Default for Atan2Op {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Atan2Op {
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
        "Atan2"
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
        let y = get_float(&self.inputs[0], get_input);
        let x = get_float(&self.inputs[1], get_input);
        self.outputs[0].set_float(y.atan2(x));
    }
}

impl OperatorMeta for Atan2Op {
    fn category(&self) -> &'static str {
        "Math"
    }
    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }
    fn description(&self) -> &'static str {
        "Two-argument arctangent"
    }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Y")),
            1 => Some(PortMeta::new("X")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(
                PortMeta::new("Angle")
                    .with_shape(PinShape::TriangleFilled)
                    .with_unit("rad"),
            ),
            _ => None,
        }
    }
}

// =============================================================================
// DegreesToRadians Operator (float-only)
// =============================================================================

pub struct DegreesToRadiansOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl DegreesToRadiansOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Degrees", 0.0)],
            outputs: [OutputPort::float("Radians")],
        }
    }
}

impl Default for DegreesToRadiansOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for DegreesToRadiansOp {
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
        "DegreesToRadians"
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
        let degrees = get_float(&self.inputs[0], get_input);
        self.outputs[0].set_float(degrees.to_radians());
    }
}

impl OperatorMeta for DegreesToRadiansOp {
    fn category(&self) -> &'static str {
        "Math"
    }
    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }
    fn description(&self) -> &'static str {
        "Converts degrees to radians"
    }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Degrees").with_unit("deg")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(
                PortMeta::new("Radians")
                    .with_shape(PinShape::TriangleFilled)
                    .with_unit("rad"),
            ),
            _ => None,
        }
    }
}

// =============================================================================
// RadiansToDegrees Operator (float-only)
// =============================================================================

pub struct RadiansToDegreesOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl RadiansToDegreesOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Radians", 0.0)],
            outputs: [OutputPort::float("Degrees")],
        }
    }
}

impl Default for RadiansToDegreesOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for RadiansToDegreesOp {
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
        "RadiansToDegrees"
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
        let radians = get_float(&self.inputs[0], get_input);
        self.outputs[0].set_float(radians.to_degrees());
    }
}

impl OperatorMeta for RadiansToDegreesOp {
    fn category(&self) -> &'static str {
        "Math"
    }
    fn category_color(&self) -> [f32; 4] {
        category_colors::MATH
    }
    fn description(&self) -> &'static str {
        "Converts radians to degrees"
    }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Radians").with_unit("rad")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(
                PortMeta::new("Degrees")
                    .with_shape(PinShape::TriangleFilled)
                    .with_unit("deg"),
            ),
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

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Tan",
            category: "Math",
            description: "Tangent of angle (radians)",
        },
        || capture_meta(TanOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Atan2",
            category: "Math",
            description: "Two-argument arctangent",
        },
        || capture_meta(Atan2Op::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "DegreesToRadians",
            category: "Math",
            description: "Converts degrees to radians",
        },
        || capture_meta(DegreesToRadiansOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "RadiansToDegrees",
            category: "Math",
            description: "Converts radians to degrees",
        },
        || capture_meta(RadiansToDegreesOp::new()),
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
