//! Vec3 list operators
//!
//! Type-specific operators for Vec3List:
//! - Vec3ListOp: Create Vec3List from multi-input
//! - Vec3ListNormalize: Normalize all vectors
//! - Vec3ListCentroid: Average position (returns Vec3)
//! - Vec3ListBounds: Bounding box (returns min/max Vec3)

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};
use flux_core::Value;

fn get_vec3_list(input: &InputPort, get_input: InputResolver) -> Vec<[f32; 3]> {
    match input.connection {
        Some((node_id, output_idx)) => {
            let value = get_input(node_id, output_idx);
            match value {
                Value::Vec3List(list) => list.to_vec(),
                Value::Vec3(v) => vec![v],
                _ => Vec::new(),
            }
        }
        None => match &input.default {
            Value::Vec3List(list) => list.to_vec(),
            Value::Vec3(v) => vec![*v],
            _ => Vec::new(),
        },
    }
}

fn collect_vec3s(input: &InputPort, get_input: InputResolver) -> Vec<[f32; 3]> {
    if !input.connections.is_empty() {
        input
            .connections
            .iter()
            .map(|(node_id, output_idx)| {
                get_input(*node_id, *output_idx).as_vec3().unwrap_or([0.0, 0.0, 0.0])
            })
            .collect()
    } else {
        match &input.default {
            Value::Vec3List(list) => list.to_vec(),
            Value::Vec3(v) => vec![*v],
            _ => Vec::new(),
        }
    }
}

fn normalize_vec3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 1e-10 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 0.0]
    }
}

// ============================================================================
// Vec3List Operator (Creation)
// ============================================================================

pub struct Vec3ListOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: [OutputPort; 1],
}

impl Vec3ListOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![InputPort::vec3_multi("Values")],
            outputs: [OutputPort::vec3_list("List")],
        }
    }
}

impl Default for Vec3ListOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3ListOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3List" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let values = collect_vec3s(&self.inputs[0], get_input);
        self.outputs[0].value = Value::vec3_list(values);
    }
}

impl OperatorMeta for Vec3ListOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Create a Vec3 list from values" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Values")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Vec3ListNormalize Operator
// ============================================================================

pub struct Vec3ListNormalizeOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl Vec3ListNormalizeOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::vec3_list("List")],
            outputs: [OutputPort::vec3_list("Normalized")],
        }
    }
}

impl Default for Vec3ListNormalizeOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3ListNormalizeOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3ListNormalize" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_vec3_list(&self.inputs[0], get_input);
        let normalized: Vec<[f32; 3]> = list.iter().map(|v| normalize_vec3(*v)).collect();
        self.outputs[0].value = Value::vec3_list(normalized);
    }
}

impl OperatorMeta for Vec3ListNormalizeOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Normalize all vectors in list" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Normalized").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Vec3ListCentroid Operator
// ============================================================================

pub struct Vec3ListCentroidOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl Vec3ListCentroidOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::vec3_list("List")],
            outputs: [OutputPort::vec3("Centroid")],
        }
    }
}

impl Default for Vec3ListCentroidOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3ListCentroidOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3ListCentroid" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_vec3_list(&self.inputs[0], get_input);
        if list.is_empty() {
            self.outputs[0].set_vec3([0.0, 0.0, 0.0]);
        } else {
            let mut sum = [0.0f32, 0.0, 0.0];
            for v in &list {
                sum[0] += v[0];
                sum[1] += v[1];
                sum[2] += v[2];
            }
            let n = list.len() as f32;
            self.outputs[0].set_vec3([sum[0] / n, sum[1] / n, sum[2] / n]);
        }
    }
}

impl OperatorMeta for Vec3ListCentroidOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Average position of all vectors (centroid)" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Centroid").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Vec3ListBounds Operator
// ============================================================================

pub struct Vec3ListBoundsOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 2],
}

impl Vec3ListBoundsOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::vec3_list("List")],
            outputs: [
                OutputPort::vec3("Min"),
                OutputPort::vec3("Max"),
            ],
        }
    }
}

impl Default for Vec3ListBoundsOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3ListBoundsOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3ListBounds" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_vec3_list(&self.inputs[0], get_input);
        if list.is_empty() {
            self.outputs[0].set_vec3([0.0, 0.0, 0.0]);
            self.outputs[1].set_vec3([0.0, 0.0, 0.0]);
        } else {
            let mut min = [f32::INFINITY, f32::INFINITY, f32::INFINITY];
            let mut max = [f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY];

            for v in &list {
                min[0] = min[0].min(v[0]);
                min[1] = min[1].min(v[1]);
                min[2] = min[2].min(v[2]);
                max[0] = max[0].max(v[0]);
                max[1] = max[1].max(v[1]);
                max[2] = max[2].max(v[2]);
            }

            self.outputs[0].set_vec3(min);
            self.outputs[1].set_vec3(max);
        }
    }
}

impl OperatorMeta for Vec3ListBoundsOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Compute axis-aligned bounding box" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Min").with_shape(PinShape::TriangleFilled)),
            1 => Some(PortMeta::new("Max").with_shape(PinShape::TriangleFilled)),
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
            name: "Vec3List",
            category: "List",
            description: "Create Vec3 list from values",
        },
        || capture_meta(Vec3ListOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec3ListNormalize",
            category: "List",
            description: "Normalize all vectors",
        },
        || capture_meta(Vec3ListNormalizeOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec3ListCentroid",
            category: "List",
            description: "Average position (centroid)",
        },
        || capture_meta(Vec3ListCentroidOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec3ListBounds",
            category: "List",
            description: "Compute bounding box",
        },
        || capture_meta(Vec3ListBoundsOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Vec3([0.0, 0.0, 0.0])
    }

    #[test]
    fn test_vec3_list_centroid() {
        let mut op = Vec3ListCentroidOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::vec3_list(vec![
            [0.0, 0.0, 0.0],
            [2.0, 4.0, 6.0],
        ]);
        op.compute(&ctx, &no_connections);

        if let Some(result) = op.outputs[0].value.as_vec3() {
            assert!((result[0] - 1.0).abs() < 0.001);
            assert!((result[1] - 2.0).abs() < 0.001);
            assert!((result[2] - 3.0).abs() < 0.001);
        } else {
            panic!("Expected Vec3");
        }
    }

    #[test]
    fn test_vec3_list_bounds() {
        let mut op = Vec3ListBoundsOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::vec3_list(vec![
            [1.0, 5.0, 2.0],
            [3.0, 2.0, 8.0],
            [-1.0, 4.0, 1.0],
        ]);
        op.compute(&ctx, &no_connections);

        if let (Some(min), Some(max)) = (
            op.outputs[0].value.as_vec3(),
            op.outputs[1].value.as_vec3()
        ) {
            assert!((min[0] - (-1.0)).abs() < 0.001);
            assert!((min[1] - 2.0).abs() < 0.001);
            assert!((min[2] - 1.0).abs() < 0.001);
            assert!((max[0] - 3.0).abs() < 0.001);
            assert!((max[1] - 5.0).abs() < 0.001);
            assert!((max[2] - 8.0).abs() < 0.001);
        } else {
            panic!("Expected Vec3s");
        }
    }

    #[test]
    fn test_vec3_list_normalize() {
        let mut op = Vec3ListNormalizeOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::vec3_list(vec![
            [3.0, 0.0, 0.0],
            [0.0, 4.0, 0.0],
        ]);
        op.compute(&ctx, &no_connections);

        if let Some(result) = op.outputs[0].value.as_vec3_list() {
            assert!((result[0][0] - 1.0).abs() < 0.001);
            assert!((result[0][1]).abs() < 0.001);
            assert!((result[1][0]).abs() < 0.001);
            assert!((result[1][1] - 1.0).abs() < 0.001);
        } else {
            panic!("Expected Vec3List");
        }
    }
}
