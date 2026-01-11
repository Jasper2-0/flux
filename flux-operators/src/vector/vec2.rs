//! Vec2 operators

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};

fn get_float(input: &InputPort, get_input: InputResolver) -> f32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
        None => input.default.as_float().unwrap_or(0.0),
    }
}

fn get_vec2(input: &InputPort, get_input: InputResolver) -> [f32; 2] {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_vec2().unwrap_or([0.0, 0.0]),
        None => input.default.as_vec2().unwrap_or([0.0, 0.0]),
    }
}

// ============================================================================
// Vec2Compose Operator
// ============================================================================

pub struct Vec2ComposeOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl Vec2ComposeOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("X", 0.0), InputPort::float("Y", 0.0)],
            outputs: [OutputPort::vec2("Vector")],
        }
    }
}

impl Default for Vec2ComposeOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec2ComposeOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec2Compose" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let x = get_float(&self.inputs[0], get_input);
        let y = get_float(&self.inputs[1], get_input);
        self.outputs[0].set_vec2([x, y]);
    }
}

impl OperatorMeta for Vec2ComposeOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Create Vec2 from X, Y components" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("X")),
            1 => Some(PortMeta::new("Y")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Vector").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Vec2Decompose Operator
// ============================================================================

pub struct Vec2DecomposeOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 2],
}

impl Vec2DecomposeOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::vec2("Vector", [0.0, 0.0])],
            outputs: [OutputPort::float("X"), OutputPort::float("Y")],
        }
    }
}

impl Default for Vec2DecomposeOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec2DecomposeOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec2Decompose" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = get_vec2(&self.inputs[0], get_input);
        self.outputs[0].set_float(v[0]);
        self.outputs[1].set_float(v[1]);
    }
}

impl OperatorMeta for Vec2DecomposeOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Split Vec2 into X, Y components" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Vector")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("X").with_shape(PinShape::TriangleFilled)),
            1 => Some(PortMeta::new("Y").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Vec2Add Operator
// ============================================================================

pub struct Vec2AddOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl Vec2AddOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::vec2("A", [0.0, 0.0]),
                InputPort::vec2("B", [0.0, 0.0]),
            ],
            outputs: [OutputPort::vec2("Result")],
        }
    }
}

impl Default for Vec2AddOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec2AddOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec2Add" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_vec2(&self.inputs[0], get_input);
        let b = get_vec2(&self.inputs[1], get_input);
        self.outputs[0].set_vec2([a[0] + b[0], a[1] + b[1]]);
    }
}

impl OperatorMeta for Vec2AddOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Add two Vec2 vectors" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Sum").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Vec2Scale Operator
// ============================================================================

pub struct Vec2ScaleOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl Vec2ScaleOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::vec2("Vector", [0.0, 0.0]),
                InputPort::float("Scale", 1.0),
            ],
            outputs: [OutputPort::vec2("Result")],
        }
    }
}

impl Default for Vec2ScaleOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec2ScaleOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec2Scale" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = get_vec2(&self.inputs[0], get_input);
        let s = get_float(&self.inputs[1], get_input);
        self.outputs[0].set_vec2([v[0] * s, v[1] * s]);
    }
}

impl OperatorMeta for Vec2ScaleOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Scale Vec2 by scalar" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Vector")),
            1 => Some(PortMeta::new("Scale")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Scaled").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Vec2Length Operator
// ============================================================================

pub struct Vec2LengthOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl Vec2LengthOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::vec2("Vector", [0.0, 0.0])],
            outputs: [OutputPort::float("Length")],
        }
    }
}

impl Default for Vec2LengthOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec2LengthOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec2Length" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = get_vec2(&self.inputs[0], get_input);
        let length = (v[0] * v[0] + v[1] * v[1]).sqrt();
        self.outputs[0].set_float(length);
    }
}

impl OperatorMeta for Vec2LengthOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Get length of Vec2" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Vector")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Length").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
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
