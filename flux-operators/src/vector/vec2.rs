//! Vec2 operators

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::OperatorMeta;
use flux_macros::Operator;
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};

// ============================================================================
// Vec2Compose Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec2Compose", category = "Vector", description = "Create Vec2 from X, Y components")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec2ComposeOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "X", default = 0.0)]
    x: f32,
    #[input(label = "Y", default = 0.0)]
    y: f32,
    #[output(label = "Vector")]
    vector: [f32; 2],
}

impl Vec2ComposeOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let x = self.get_x(get_input);
        let y = self.get_y(get_input);
        self.set_vector([x, y]);
    }
}

// ============================================================================
// Vec2Decompose Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec2Decompose", category = "Vector", description = "Split Vec2 into X, Y components")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec2DecomposeOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 2],
    #[input(label = "Vector", default = [0.0, 0.0])]
    vector: [f32; 2],
    #[output(label = "X")]
    x: f32,
    #[output(label = "Y")]
    y: f32,
}

impl Vec2DecomposeOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = self.get_vector(get_input);
        self.set_x(v[0]);
        self.set_y(v[1]);
    }
}

// ============================================================================
// Vec2Add Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec2Add", category = "Vector", description = "Add two Vec2 vectors")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec2AddOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "A", default = [0.0, 0.0])]
    a: [f32; 2],
    #[input(label = "B", default = [0.0, 0.0])]
    b: [f32; 2],
    #[output(label = "Sum")]
    result: [f32; 2],
}

impl Vec2AddOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_a(get_input);
        let b = self.get_b(get_input);
        self.set_result([a[0] + b[0], a[1] + b[1]]);
    }
}

// ============================================================================
// Vec2Scale Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec2Scale", category = "Vector", description = "Scale Vec2 by scalar")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec2ScaleOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "Vector", default = [0.0, 0.0])]
    vector: [f32; 2],
    #[input(label = "Scale", default = 1.0)]
    scale: f32,
    #[output(label = "Scaled")]
    result: [f32; 2],
}

impl Vec2ScaleOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = self.get_vector(get_input);
        let s = self.get_scale(get_input);
        self.set_result([v[0] * s, v[1] * s]);
    }
}

// ============================================================================
// Vec2Length Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Vec2Length", category = "Vector", description = "Get length of Vec2")]
#[operator(category_color = [0.20, 0.55, 0.50, 1.0])]
#[allow(dead_code)]
pub struct Vec2LengthOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    #[input(label = "Vector", default = [0.0, 0.0])]
    vector: [f32; 2],
    #[output(label = "Length")]
    length: f32,
}

impl Vec2LengthOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = self.get_vector(get_input);
        self.set_length((v[0] * v[0] + v[1] * v[1]).sqrt());
    }
}

// ============================================================================
// Registration
// ============================================================================

pub fn register(registry: &OperatorRegistry) {
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec2Compose",
            category: "Vector",
            description: "Create Vec2 from X, Y components",
        },
        || capture_meta(Vec2ComposeOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec2Decompose",
            category: "Vector",
            description: "Split Vec2 into X, Y components",
        },
        || capture_meta(Vec2DecomposeOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec2Add",
            category: "Vector",
            description: "Add two Vec2 vectors",
        },
        || capture_meta(Vec2AddOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec2Scale",
            category: "Vector",
            description: "Scale Vec2 by scalar",
        },
        || capture_meta(Vec2ScaleOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec2Length",
            category: "Vector",
            description: "Get length of Vec2",
        },
        || capture_meta(Vec2LengthOp::new()),
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
    fn test_vec2_compose() {
        let mut op = Vec2ComposeOp::new();
        op.inputs[0].default = Value::Float(3.0);
        op.inputs[1].default = Value::Float(4.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_vec2(), Some([3.0, 4.0]));
    }

    #[test]
    fn test_vec2_decompose() {
        let mut op = Vec2DecomposeOp::new();
        op.inputs[0].default = Value::Vec2([5.0, 7.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(5.0));
        assert_eq!(op.outputs[1].value.as_float(), Some(7.0));
    }

    #[test]
    fn test_vec2_add() {
        let mut op = Vec2AddOp::new();
        op.inputs[0].default = Value::Vec2([1.0, 2.0]);
        op.inputs[1].default = Value::Vec2([3.0, 4.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_vec2(), Some([4.0, 6.0]));
    }

    #[test]
    fn test_vec2_scale() {
        let mut op = Vec2ScaleOp::new();
        op.inputs[0].default = Value::Vec2([2.0, 3.0]);
        op.inputs[1].default = Value::Float(2.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_vec2(), Some([4.0, 6.0]));
    }

    #[test]
    fn test_vec2_length() {
        let mut op = Vec2LengthOp::new();
        op.inputs[0].default = Value::Vec2([3.0, 4.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(5.0));
    }
}
