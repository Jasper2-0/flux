//! List conversion operators
//!
//! Explicit conversion between list types:
//! - IntListToFloatList / FloatListToIntList
//! - Vec3ListFlatten / FloatListToVec3List
//! - ColorListToVec4List / Vec4ListToColorList

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::value::Color;
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};
use flux_core::Value;

fn get_value(input: &InputPort, get_input: InputResolver) -> Value {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx),
        None => input.default.clone(),
    }
}

// ============================================================================
// IntListToFloatList Operator
// ============================================================================

pub struct IntListToFloatListOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl IntListToFloatListOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::int_list("IntList")],
            outputs: [OutputPort::float_list("FloatList")],
        }
    }
}

impl Default for IntListToFloatListOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for IntListToFloatListOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "IntListToFloatList" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_value(&self.inputs[0], get_input);
        match value {
            Value::IntList(il) => {
                let fl: Vec<f32> = il.iter().map(|i| *i as f32).collect();
                self.outputs[0].value = Value::float_list(fl);
            }
            _ => {
                self.outputs[0].value = Value::float_list(vec![]);
            }
        }
    }
}

impl OperatorMeta for IntListToFloatListOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Convert IntList to FloatList" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("IntList")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("FloatList").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// FloatListToIntList Operator
// ============================================================================

pub struct FloatListToIntListOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl FloatListToIntListOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float_list("FloatList")],
            outputs: [OutputPort::int_list("IntList")],
        }
    }
}

impl Default for FloatListToIntListOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for FloatListToIntListOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "FloatListToIntList" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_value(&self.inputs[0], get_input);
        match value {
            Value::FloatList(fl) => {
                let il: Vec<i32> = fl.iter().map(|f| *f as i32).collect();
                self.outputs[0].value = Value::int_list(il);
            }
            _ => {
                self.outputs[0].value = Value::int_list(vec![]);
            }
        }
    }
}

impl OperatorMeta for FloatListToIntListOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Convert FloatList to IntList (truncates)" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("FloatList")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("IntList").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Vec3ListFlatten Operator
// ============================================================================

pub struct Vec3ListFlattenOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl Vec3ListFlattenOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::vec3_list("Vec3List")],
            outputs: [OutputPort::float_list("FloatList")],
        }
    }
}

impl Default for Vec3ListFlattenOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3ListFlattenOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec3ListFlatten" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_value(&self.inputs[0], get_input);
        match value {
            Value::Vec3List(vl) => {
                let fl: Vec<f32> = vl.iter().flat_map(|v| vec![v[0], v[1], v[2]]).collect();
                self.outputs[0].value = Value::float_list(fl);
            }
            _ => {
                self.outputs[0].value = Value::float_list(vec![]);
            }
        }
    }
}

impl OperatorMeta for Vec3ListFlattenOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Flatten Vec3List to FloatList (xyz, xyz, ...)" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Vec3List")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("FloatList").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// FloatListToVec3List Operator
// ============================================================================

pub struct FloatListToVec3ListOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl FloatListToVec3ListOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float_list("FloatList")],
            outputs: [OutputPort::vec3_list("Vec3List")],
        }
    }
}

impl Default for FloatListToVec3ListOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for FloatListToVec3ListOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "FloatListToVec3List" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_value(&self.inputs[0], get_input);
        match value {
            Value::FloatList(fl) => {
                let vl: Vec<[f32; 3]> = fl
                    .chunks(3)
                    .filter(|c| c.len() == 3)
                    .map(|c| [c[0], c[1], c[2]])
                    .collect();
                self.outputs[0].value = Value::vec3_list(vl);
            }
            _ => {
                self.outputs[0].value = Value::vec3_list(vec![]);
            }
        }
    }
}

impl OperatorMeta for FloatListToVec3ListOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Group FloatList into Vec3List (by 3s)" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("FloatList")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Vec3List").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// ColorListToVec4List Operator
// ============================================================================

pub struct ColorListToVec4ListOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl ColorListToVec4ListOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::color_list("ColorList")],
            outputs: [OutputPort::vec4_list("Vec4List")],
        }
    }
}

impl Default for ColorListToVec4ListOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ColorListToVec4ListOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ColorListToVec4List" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_value(&self.inputs[0], get_input);
        match value {
            Value::ColorList(cl) => {
                let vl: Vec<[f32; 4]> = cl.iter().map(|c| c.to_array()).collect();
                self.outputs[0].value = Value::vec4_list(vl);
            }
            _ => {
                self.outputs[0].value = Value::vec4_list(vec![]);
            }
        }
    }
}

impl OperatorMeta for ColorListToVec4ListOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Convert ColorList to Vec4List" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("ColorList")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Vec4List").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Vec4ListToColorList Operator
// ============================================================================

pub struct Vec4ListToColorListOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl Vec4ListToColorListOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::vec4_list("Vec4List")],
            outputs: [OutputPort::color_list("ColorList")],
        }
    }
}

impl Default for Vec4ListToColorListOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec4ListToColorListOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Vec4ListToColorList" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_value(&self.inputs[0], get_input);
        match value {
            Value::Vec4List(vl) => {
                let cl: Vec<Color> = vl.iter().map(|v| Color::from_array(*v)).collect();
                self.outputs[0].value = Value::color_list(cl);
            }
            _ => {
                self.outputs[0].value = Value::color_list(vec![]);
            }
        }
    }
}

impl OperatorMeta for Vec4ListToColorListOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Convert Vec4List to ColorList" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Vec4List")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("ColorList").with_shape(PinShape::TriangleFilled)),
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
            name: "IntListToFloatList",
            category: "List",
            description: "Convert IntList to FloatList",
        },
        || capture_meta(IntListToFloatListOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "FloatListToIntList",
            category: "List",
            description: "Convert FloatList to IntList",
        },
        || capture_meta(FloatListToIntListOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec3ListFlatten",
            category: "List",
            description: "Flatten Vec3List to FloatList",
        },
        || capture_meta(Vec3ListFlattenOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "FloatListToVec3List",
            category: "List",
            description: "Group FloatList to Vec3List",
        },
        || capture_meta(FloatListToVec3ListOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ColorListToVec4List",
            category: "List",
            description: "Convert ColorList to Vec4List",
        },
        || capture_meta(ColorListToVec4ListOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Vec4ListToColorList",
            category: "List",
            description: "Convert Vec4List to ColorList",
        },
        || capture_meta(Vec4ListToColorListOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Float(0.0)
    }

    #[test]
    fn test_int_list_to_float_list() {
        let mut op = IntListToFloatListOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::int_list(vec![1, 2, 3, 4, 5]);
        op.compute(&ctx, &no_connections);

        if let Value::FloatList(result) = &op.outputs[0].value {
            assert_eq!(result.as_ref(), &[1.0, 2.0, 3.0, 4.0, 5.0]);
        } else {
            panic!("Expected FloatList");
        }
    }

    #[test]
    fn test_float_list_to_int_list() {
        let mut op = FloatListToIntListOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::float_list(vec![1.9, 2.1, 3.5, 4.0, 5.99]);
        op.compute(&ctx, &no_connections);

        if let Value::IntList(result) = &op.outputs[0].value {
            assert_eq!(result.as_ref(), &[1, 2, 3, 4, 5]);
        } else {
            panic!("Expected IntList");
        }
    }

    #[test]
    fn test_vec3_list_flatten() {
        let mut op = Vec3ListFlattenOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::vec3_list(vec![
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
        ]);
        op.compute(&ctx, &no_connections);

        if let Value::FloatList(result) = &op.outputs[0].value {
            assert_eq!(result.as_ref(), &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        } else {
            panic!("Expected FloatList");
        }
    }

    #[test]
    fn test_float_list_to_vec3_list() {
        let mut op = FloatListToVec3ListOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::float_list(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
        op.compute(&ctx, &no_connections);

        if let Value::Vec3List(result) = &op.outputs[0].value {
            assert_eq!(result.as_ref(), &[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
            // Note: 7.0 is truncated since it doesn't complete a Vec3
        } else {
            panic!("Expected Vec3List");
        }
    }

    #[test]
    fn test_color_list_to_vec4_list() {
        let mut op = ColorListToVec4ListOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::color_list(vec![
            Color::rgba(1.0, 0.0, 0.0, 1.0),
            Color::rgba(0.0, 1.0, 0.0, 0.5),
        ]);
        op.compute(&ctx, &no_connections);

        if let Value::Vec4List(result) = &op.outputs[0].value {
            assert_eq!(result.len(), 2);
            assert!((result[0][0] - 1.0).abs() < 0.001);
            assert!((result[1][1] - 1.0).abs() < 0.001);
            assert!((result[1][3] - 0.5).abs() < 0.001);
        } else {
            panic!("Expected Vec4List");
        }
    }

    #[test]
    fn test_vec4_list_to_color_list() {
        let mut op = Vec4ListToColorListOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::vec4_list(vec![
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 0.5],
        ]);
        op.compute(&ctx, &no_connections);

        if let Value::ColorList(result) = &op.outputs[0].value {
            assert_eq!(result.len(), 2);
            assert!((result[0].r - 1.0).abs() < 0.001);
            assert!((result[1].g - 1.0).abs() < 0.001);
            assert!((result[1].a - 0.5).abs() < 0.001);
        } else {
            panic!("Expected ColorList");
        }
    }
}
