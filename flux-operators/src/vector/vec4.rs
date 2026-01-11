//! Vec4 operators

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

fn get_vec3(input: &InputPort, get_input: InputResolver) -> [f32; 3] {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_vec3().unwrap_or([0.0, 0.0, 0.0]),
        None => input.default.as_vec3().unwrap_or([0.0, 0.0, 0.0]),
    }
}

fn get_vec4(input: &InputPort, get_input: InputResolver) -> [f32; 4] {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_vec4().unwrap_or([0.0, 0.0, 0.0, 0.0]),
        None => input.default.as_vec4().unwrap_or([0.0, 0.0, 0.0, 0.0]),
    }
}

// ============================================================================
// Vec4Compose Operator
// ============================================================================

pub struct Vec4ComposeOp {
    id: Id,
    inputs: [InputPort; 4],
    outputs: [OutputPort; 1],
}

impl Vec4ComposeOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("X", 0.0),
                InputPort::float("Y", 0.0),
                InputPort::float("Z", 0.0),
                InputPort::float("W", 1.0),
            ],
            outputs: [OutputPort::vec4("Vector")],
        }
    }
}

impl Default for Vec4ComposeOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec4ComposeOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec4Compose" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let x = get_float(&self.inputs[0], get_input);
        let y = get_float(&self.inputs[1], get_input);
        let z = get_float(&self.inputs[2], get_input);
        let w = get_float(&self.inputs[3], get_input);
        self.outputs[0].set_vec4([x, y, z, w]);
    }
}

impl OperatorMeta for Vec4ComposeOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Create Vec4 from X, Y, Z, W components" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("X")),
            1 => Some(PortMeta::new("Y")),
            2 => Some(PortMeta::new("Z")),
            3 => Some(PortMeta::new("W")),
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
// Vec4Decompose Operator
// ============================================================================

pub struct Vec4DecomposeOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 4],
}

impl Vec4DecomposeOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::vec4("Vector", [0.0, 0.0, 0.0, 1.0])],
            outputs: [
                OutputPort::float("X"),
                OutputPort::float("Y"),
                OutputPort::float("Z"),
                OutputPort::float("W"),
            ],
        }
    }
}

impl Default for Vec4DecomposeOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec4DecomposeOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec4Decompose" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = get_vec4(&self.inputs[0], get_input);
        self.outputs[0].set_float(v[0]);
        self.outputs[1].set_float(v[1]);
        self.outputs[2].set_float(v[2]);
        self.outputs[3].set_float(v[3]);
    }
}

impl OperatorMeta for Vec4DecomposeOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Split Vec4 into X, Y, Z, W components" }
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
            2 => Some(PortMeta::new("Z").with_shape(PinShape::TriangleFilled)),
            3 => Some(PortMeta::new("W").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Vec3ToVec4 Operator
// ============================================================================

pub struct Vec3ToVec4Op {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl Vec3ToVec4Op {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::vec3("Vector", [0.0, 0.0, 0.0]),
                InputPort::float("W", 1.0),
            ],
            outputs: [OutputPort::vec4("Result")],
        }
    }
}

impl Default for Vec3ToVec4Op {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3ToVec4Op {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3ToVec4" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = get_vec3(&self.inputs[0], get_input);
        let w = get_float(&self.inputs[1], get_input);
        self.outputs[0].set_vec4([v[0], v[1], v[2], w]);
    }
}

impl OperatorMeta for Vec3ToVec4Op {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Extend Vec3 to Vec4 with W component" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Vector")),
            1 => Some(PortMeta::new("W")),
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

// ============================================================================
// Registration
// ============================================================================

pub fn register(registry: &OperatorRegistry) {
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec4Compose",
            category: "Vector",
            description: "Create Vec4 from X, Y, Z, W components",
        },
        || capture_meta(Vec4ComposeOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec4Decompose",
            category: "Vector",
            description: "Split Vec4 into X, Y, Z, W components",
        },
        || capture_meta(Vec4DecomposeOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec3ToVec4",
            category: "Vector",
            description: "Extend Vec3 to Vec4 with W component",
        },
        || capture_meta(Vec3ToVec4Op::new()),
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
