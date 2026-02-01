//! Vec3 operators

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::OperatorMeta;
use flux_macros::Operator;
use crate::register_operators;
use crate::registry::OperatorRegistry;
use flux_core::port::{InputPort, OutputPort};

// ============================================================================
// Vec3Decompose Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec3Decompose", category = "Vector", description = "Split Vec3 into X, Y, Z components")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec3DecomposeOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 3],
    #[input(label = "Vector", default = [0.0, 0.0, 0.0])]
    vector: [f32; 3],
    #[output(label = "X")]
    x: f32,
    #[output(label = "Y")]
    y: f32,
    #[output(label = "Z")]
    z: f32,
}

impl Vec3DecomposeOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = self.get_vector(get_input);
        self.set_x(v[0]);
        self.set_y(v[1]);
        self.set_z(v[2]);
    }
}

// ============================================================================
// Vec3Add Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec3Add", category = "Vector", description = "Add two Vec3 vectors")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec3AddOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "A", default = [0.0, 0.0, 0.0])]
    a: [f32; 3],
    #[input(label = "B", default = [0.0, 0.0, 0.0])]
    b: [f32; 3],
    #[output(label = "Sum")]
    result: [f32; 3],
}

impl Vec3AddOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_a(get_input);
        let b = self.get_b(get_input);
        self.set_result([a[0] + b[0], a[1] + b[1], a[2] + b[2]]);
    }
}

// ============================================================================
// Vec3Subtract Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec3Subtract", category = "Vector", description = "Subtract Vec3 B from A")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec3SubtractOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "A", default = [0.0, 0.0, 0.0])]
    a: [f32; 3],
    #[input(label = "B", default = [0.0, 0.0, 0.0])]
    b: [f32; 3],
    #[output(label = "Diff")]
    result: [f32; 3],
}

impl Vec3SubtractOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_a(get_input);
        let b = self.get_b(get_input);
        self.set_result([a[0] - b[0], a[1] - b[1], a[2] - b[2]]);
    }
}

// ============================================================================
// Vec3Scale Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec3Scale", category = "Vector", description = "Scale Vec3 by scalar")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec3ScaleOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "Vector", default = [0.0, 0.0, 0.0])]
    vector: [f32; 3],
    #[input(label = "Scale", default = 1.0)]
    scale: f32,
    #[output(label = "Scaled")]
    result: [f32; 3],
}

impl Vec3ScaleOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = self.get_vector(get_input);
        let s = self.get_scale(get_input);
        self.set_result([v[0] * s, v[1] * s, v[2] * s]);
    }
}

// ============================================================================
// Vec3Normalize Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec3Normalize", category = "Vector", description = "Normalize Vec3 to unit length")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec3NormalizeOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    #[input(label = "Vector", default = [0.0, 0.0, 1.0])]
    vector: [f32; 3],
    #[output(label = "Normal")]
    result: [f32; 3],
}

impl Vec3NormalizeOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = self.get_vector(get_input);
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if len > f32::EPSILON {
            self.set_result([v[0] / len, v[1] / len, v[2] / len]);
        } else {
            self.set_result([0.0, 0.0, 0.0]);
        }
    }
}

// ============================================================================
// Vec3Dot Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec3Dot", category = "Vector", description = "Dot product of two Vec3")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec3DotOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "A", default = [0.0, 0.0, 0.0])]
    a: [f32; 3],
    #[input(label = "B", default = [0.0, 0.0, 0.0])]
    b: [f32; 3],
    #[output(label = "Dot")]
    result: f32,
}

impl Vec3DotOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_a(get_input);
        let b = self.get_b(get_input);
        self.set_result(a[0] * b[0] + a[1] * b[1] + a[2] * b[2]);
    }
}

// ============================================================================
// Vec3Cross Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec3Cross", category = "Vector", description = "Cross product of two Vec3")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec3CrossOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "A", default = [1.0, 0.0, 0.0])]
    a: [f32; 3],
    #[input(label = "B", default = [0.0, 1.0, 0.0])]
    b: [f32; 3],
    #[output(label = "Cross")]
    result: [f32; 3],
}

impl Vec3CrossOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_a(get_input);
        let b = self.get_b(get_input);
        self.set_result([
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]);
    }
}

// ============================================================================
// Vec3Length Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec3Length", category = "Vector", description = "Get length of Vec3")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec3LengthOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    #[input(label = "Vector", default = [0.0, 0.0, 0.0])]
    vector: [f32; 3],
    #[output(label = "Length")]
    length: f32,
}

impl Vec3LengthOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = self.get_vector(get_input);
        self.set_length((v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt());
    }
}

// ============================================================================
// Vec3Distance Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec3Distance", category = "Vector", description = "Distance between two Vec3 points")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec3DistanceOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "A", default = [0.0, 0.0, 0.0])]
    a: [f32; 3],
    #[input(label = "B", default = [0.0, 0.0, 0.0])]
    b: [f32; 3],
    #[output(label = "Dist")]
    distance: f32,
}

impl Vec3DistanceOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_a(get_input);
        let b = self.get_b(get_input);
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        self.set_distance((dx * dx + dy * dy + dz * dz).sqrt());
    }
}

// ============================================================================
// Registration
// ============================================================================

pub fn register(registry: &OperatorRegistry) {
    register_operators!(
        registry,
        Vec3DecomposeOp,
        Vec3AddOp,
        Vec3SubtractOp,
        Vec3ScaleOp,
        Vec3NormalizeOp,
        Vec3DotOp,
        Vec3CrossOp,
        Vec3LengthOp,
        Vec3DistanceOp,
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
    fn test_vec3_decompose() {
        let mut op = Vec3DecomposeOp::new();
        op.inputs[0].default = Value::Vec3([1.0, 2.0, 3.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(1.0));
        assert_eq!(op.outputs[1].value.as_float(), Some(2.0));
        assert_eq!(op.outputs[2].value.as_float(), Some(3.0));
    }

    #[test]
    fn test_vec3_add() {
        let mut op = Vec3AddOp::new();
        op.inputs[0].default = Value::Vec3([1.0, 2.0, 3.0]);
        op.inputs[1].default = Value::Vec3([4.0, 5.0, 6.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_vec3(), Some([5.0, 7.0, 9.0]));
    }

    #[test]
    fn test_vec3_dot() {
        let mut op = Vec3DotOp::new();
        op.inputs[0].default = Value::Vec3([1.0, 0.0, 0.0]);
        op.inputs[1].default = Value::Vec3([0.0, 1.0, 0.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(0.0)); // Perpendicular

        op.inputs[1].default = Value::Vec3([1.0, 0.0, 0.0]);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(1.0)); // Parallel
    }

    #[test]
    fn test_vec3_cross() {
        let mut op = Vec3CrossOp::new();
        op.inputs[0].default = Value::Vec3([1.0, 0.0, 0.0]);
        op.inputs[1].default = Value::Vec3([0.0, 1.0, 0.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_vec3(), Some([0.0, 0.0, 1.0]));
    }

    #[test]
    fn test_vec3_normalize() {
        let mut op = Vec3NormalizeOp::new();
        op.inputs[0].default = Value::Vec3([3.0, 0.0, 4.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        let result = op.outputs[0].value.as_vec3().unwrap();
        assert!((result[0] - 0.6).abs() < 0.0001);
        assert!((result[1] - 0.0).abs() < 0.0001);
        assert!((result[2] - 0.8).abs() < 0.0001);
    }

    #[test]
    fn test_vec3_distance() {
        let mut op = Vec3DistanceOp::new();
        op.inputs[0].default = Value::Vec3([0.0, 0.0, 0.0]);
        op.inputs[1].default = Value::Vec3([3.0, 4.0, 0.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(5.0));
    }
}
