//! Vec4 operators

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::OperatorMeta;
use flux_macros::Operator;
use crate::register_operators;
use crate::registry::OperatorRegistry;
use flux_core::port::{InputPort, OutputPort};

// ============================================================================
// Vec4Compose Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec4Compose", category = "Vector", description = "Create Vec4 from X, Y, Z, W components")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec4ComposeOp {
    id: Id,
    inputs: [InputPort; 4],
    outputs: [OutputPort; 1],
    #[input(label = "X", default = 0.0)]
    x: f32,
    #[input(label = "Y", default = 0.0)]
    y: f32,
    #[input(label = "Z", default = 0.0)]
    z: f32,
    #[input(label = "W", default = 1.0)]
    w: f32,
    #[output(label = "Vector")]
    vector: [f32; 4],
}

impl Vec4ComposeOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let x = self.get_x(get_input);
        let y = self.get_y(get_input);
        let z = self.get_z(get_input);
        let w = self.get_w(get_input);
        self.set_vector([x, y, z, w]);
    }
}

// ============================================================================
// Vec4Decompose Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec4Decompose", category = "Vector", description = "Split Vec4 into X, Y, Z, W components")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec4DecomposeOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 4],
    #[input(label = "Vector", default = [0.0, 0.0, 0.0, 1.0])]
    vector: [f32; 4],
    #[output(label = "X")]
    x: f32,
    #[output(label = "Y")]
    y: f32,
    #[output(label = "Z")]
    z: f32,
    #[output(label = "W")]
    w: f32,
}

impl Vec4DecomposeOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = self.get_vector(get_input);
        self.set_x(v[0]);
        self.set_y(v[1]);
        self.set_z(v[2]);
        self.set_w(v[3]);
    }
}

// ============================================================================
// Vec3ToVec4 Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec3ToVec4", category = "Vector", description = "Extend Vec3 to Vec4 with W component")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec3ToVec4Op {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "Vector", default = [0.0, 0.0, 0.0])]
    vector: [f32; 3],
    #[input(label = "W", default = 1.0)]
    w: f32,
    #[output(label = "Result")]
    result: [f32; 4],
}

impl Vec3ToVec4Op {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = self.get_vector(get_input);
        let w = self.get_w(get_input);
        self.set_result([v[0], v[1], v[2], w]);
    }
}

// ============================================================================
// Registration
// ============================================================================

pub fn register(registry: &OperatorRegistry) {
    register_operators!(
        registry,
        Vec4ComposeOp,
        Vec4DecomposeOp,
        Vec3ToVec4Op,
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
    fn test_vec4_compose() {
        let mut op = Vec4ComposeOp::new();
        op.inputs[0].default = Value::Float(1.0);
        op.inputs[1].default = Value::Float(2.0);
        op.inputs[2].default = Value::Float(3.0);
        op.inputs[3].default = Value::Float(4.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_vec4(), Some([1.0, 2.0, 3.0, 4.0]));
    }

    #[test]
    fn test_vec4_decompose() {
        let mut op = Vec4DecomposeOp::new();
        op.inputs[0].default = Value::Vec4([1.0, 2.0, 3.0, 4.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(1.0));
        assert_eq!(op.outputs[1].value.as_float(), Some(2.0));
        assert_eq!(op.outputs[2].value.as_float(), Some(3.0));
        assert_eq!(op.outputs[3].value.as_float(), Some(4.0));
    }

    #[test]
    fn test_vec3_to_vec4() {
        let mut op = Vec3ToVec4Op::new();
        op.inputs[0].default = Value::Vec3([1.0, 2.0, 3.0]);
        op.inputs[1].default = Value::Float(0.5);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_vec4(), Some([1.0, 2.0, 3.0, 0.5]));
    }
}
