//! Vec3 operators
//! Note: Vec3Compose is already in legacy operator.rs, but we add it here for completeness

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

// ============================================================================
// Vec3Decompose Operator
// ============================================================================

pub struct Vec3DecomposeOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 3],
}

impl Vec3DecomposeOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::vec3("Vector", [0.0, 0.0, 0.0])],
            outputs: [
                OutputPort::float("X"),
                OutputPort::float("Y"),
                OutputPort::float("Z"),
            ],
        }
    }
}

impl Default for Vec3DecomposeOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3DecomposeOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3Decompose" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = get_vec3(&self.inputs[0], get_input);
        self.outputs[0].set_float(v[0]);
        self.outputs[1].set_float(v[1]);
        self.outputs[2].set_float(v[2]);
    }
}

impl OperatorMeta for Vec3DecomposeOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Split Vec3 into X, Y, Z components" }
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
            _ => None,
        }
    }
}

// ============================================================================
// Vec3Add Operator
// ============================================================================

pub struct Vec3AddOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl Vec3AddOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::vec3("A", [0.0, 0.0, 0.0]),
                InputPort::vec3("B", [0.0, 0.0, 0.0]),
            ],
            outputs: [OutputPort::vec3("Result")],
        }
    }
}

impl Default for Vec3AddOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3AddOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3Add" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_vec3(&self.inputs[0], get_input);
        let b = get_vec3(&self.inputs[1], get_input);
        self.outputs[0].set_vec3([a[0] + b[0], a[1] + b[1], a[2] + b[2]]);
    }
}

impl OperatorMeta for Vec3AddOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Add two Vec3 vectors" }
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
// Vec3Subtract Operator
// ============================================================================

pub struct Vec3SubtractOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl Vec3SubtractOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::vec3("A", [0.0, 0.0, 0.0]),
                InputPort::vec3("B", [0.0, 0.0, 0.0]),
            ],
            outputs: [OutputPort::vec3("Result")],
        }
    }
}

impl Default for Vec3SubtractOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3SubtractOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3Subtract" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_vec3(&self.inputs[0], get_input);
        let b = get_vec3(&self.inputs[1], get_input);
        self.outputs[0].set_vec3([a[0] - b[0], a[1] - b[1], a[2] - b[2]]);
    }
}

impl OperatorMeta for Vec3SubtractOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Subtract Vec3 B from A" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Diff").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Vec3Scale Operator
// ============================================================================

pub struct Vec3ScaleOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl Vec3ScaleOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::vec3("Vector", [0.0, 0.0, 0.0]),
                InputPort::float("Scale", 1.0),
            ],
            outputs: [OutputPort::vec3("Result")],
        }
    }
}

impl Default for Vec3ScaleOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3ScaleOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3Scale" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = get_vec3(&self.inputs[0], get_input);
        let s = get_float(&self.inputs[1], get_input);
        self.outputs[0].set_vec3([v[0] * s, v[1] * s, v[2] * s]);
    }
}

impl OperatorMeta for Vec3ScaleOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Scale Vec3 by scalar" }
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
// Vec3Normalize Operator
// ============================================================================

pub struct Vec3NormalizeOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl Vec3NormalizeOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::vec3("Vector", [0.0, 0.0, 1.0])],
            outputs: [OutputPort::vec3("Result")],
        }
    }
}

impl Default for Vec3NormalizeOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3NormalizeOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3Normalize" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = get_vec3(&self.inputs[0], get_input);
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if len > f32::EPSILON {
            self.outputs[0].set_vec3([v[0] / len, v[1] / len, v[2] / len]);
        } else {
            self.outputs[0].set_vec3([0.0, 0.0, 0.0]);
        }
    }
}

impl OperatorMeta for Vec3NormalizeOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Normalize Vec3 to unit length" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Vector")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Normal").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Vec3Dot Operator
// ============================================================================

pub struct Vec3DotOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl Vec3DotOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::vec3("A", [0.0, 0.0, 0.0]),
                InputPort::vec3("B", [0.0, 0.0, 0.0]),
            ],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for Vec3DotOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3DotOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3Dot" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_vec3(&self.inputs[0], get_input);
        let b = get_vec3(&self.inputs[1], get_input);
        self.outputs[0].set_float(a[0] * b[0] + a[1] * b[1] + a[2] * b[2]);
    }
}

impl OperatorMeta for Vec3DotOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Dot product of two Vec3" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Dot").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Vec3Cross Operator
// ============================================================================

pub struct Vec3CrossOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl Vec3CrossOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::vec3("A", [1.0, 0.0, 0.0]),
                InputPort::vec3("B", [0.0, 1.0, 0.0]),
            ],
            outputs: [OutputPort::vec3("Result")],
        }
    }
}

impl Default for Vec3CrossOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3CrossOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3Cross" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_vec3(&self.inputs[0], get_input);
        let b = get_vec3(&self.inputs[1], get_input);
        self.outputs[0].set_vec3([
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]);
    }
}

impl OperatorMeta for Vec3CrossOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Cross product of two Vec3" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Cross").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Vec3Length Operator
// ============================================================================

pub struct Vec3LengthOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl Vec3LengthOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::vec3("Vector", [0.0, 0.0, 0.0])],
            outputs: [OutputPort::float("Length")],
        }
    }
}

impl Default for Vec3LengthOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3LengthOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3Length" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let v = get_vec3(&self.inputs[0], get_input);
        let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        self.outputs[0].set_float(length);
    }
}

impl OperatorMeta for Vec3LengthOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Get length of Vec3" }
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
// Vec3Distance Operator
// ============================================================================

pub struct Vec3DistanceOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl Vec3DistanceOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::vec3("A", [0.0, 0.0, 0.0]),
                InputPort::vec3("B", [0.0, 0.0, 0.0]),
            ],
            outputs: [OutputPort::float("Distance")],
        }
    }
}

impl Default for Vec3DistanceOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3DistanceOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3Distance" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_vec3(&self.inputs[0], get_input);
        let b = get_vec3(&self.inputs[1], get_input);
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        self.outputs[0].set_float((dx * dx + dy * dy + dz * dz).sqrt());
    }
}

impl OperatorMeta for Vec3DistanceOp {
    fn category(&self) -> &'static str { "Vector" }
    fn category_color(&self) -> [f32; 4] { category_colors::VECTORS }
    fn description(&self) -> &'static str { "Distance between two Vec3 points" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Dist").with_shape(PinShape::TriangleFilled)),
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
            name: "Vec3Decompose",
            category: "Vector",
            description: "Split Vec3 into X, Y, Z components",
        },
        || capture_meta(Vec3DecomposeOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec3Add",
            category: "Vector",
            description: "Add two Vec3 vectors",
        },
        || capture_meta(Vec3AddOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec3Subtract",
            category: "Vector",
            description: "Subtract Vec3 B from A",
        },
        || capture_meta(Vec3SubtractOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec3Scale",
            category: "Vector",
            description: "Scale Vec3 by scalar",
        },
        || capture_meta(Vec3ScaleOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec3Normalize",
            category: "Vector",
            description: "Normalize Vec3 to unit length",
        },
        || capture_meta(Vec3NormalizeOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec3Dot",
            category: "Vector",
            description: "Dot product of two Vec3",
        },
        || capture_meta(Vec3DotOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec3Cross",
            category: "Vector",
            description: "Cross product of two Vec3",
        },
        || capture_meta(Vec3CrossOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec3Length",
            category: "Vector",
            description: "Get length of Vec3",
        },
        || capture_meta(Vec3LengthOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec3Distance",
            category: "Vector",
            description: "Distance between two Vec3 points",
        },
        || capture_meta(Vec3DistanceOp::new()),
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
