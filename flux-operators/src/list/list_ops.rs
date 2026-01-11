//! List operators: FloatList, ListLength, ListGet, ListSum, ListAverage, ListMin, ListMax, ListMap

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};
use flux_core::Value;

fn get_float(input: &InputPort, get_input: InputResolver) -> f32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
        None => input.default.as_float().unwrap_or(0.0),
    }
}

fn get_int(input: &InputPort, get_input: InputResolver) -> i32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_int().unwrap_or(0),
        None => input.default.as_int().unwrap_or(0),
    }
}

fn get_list(input: &InputPort, get_input: InputResolver) -> Vec<f32> {
    match input.connection {
        Some((node_id, output_idx)) => {
            let value = get_input(node_id, output_idx);
            match value {
                Value::FloatList(list) => list,
                Value::Float(f) => vec![f],
                _ => Vec::new(),
            }
        }
        None => match &input.default {
            Value::FloatList(list) => list.clone(),
            Value::Float(f) => vec![*f],
            _ => Vec::new(),
        },
    }
}

// Helper to collect floats from multi-input
fn collect_floats(input: &InputPort, get_input: InputResolver) -> Vec<f32> {
    if !input.connections.is_empty() {
        input
            .connections
            .iter()
            .map(|(node_id, output_idx)| {
                get_input(*node_id, *output_idx).as_float().unwrap_or(0.0)
            })
            .collect()
    } else {
        match &input.default {
            Value::FloatList(list) => list.clone(),
            Value::Float(f) => vec![*f],
            _ => Vec::new(),
        }
    }
}

// ============================================================================
// FloatList Operator
// ============================================================================

pub struct FloatListOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: [OutputPort; 1],
}

impl FloatListOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![InputPort::float_multi("Values")],
            outputs: [OutputPort::float_list("List")],
        }
    }
}

impl Default for FloatListOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for FloatListOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "FloatList" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let values = collect_floats(&self.inputs[0], get_input);
        self.outputs[0].value = Value::FloatList(values);
    }
}

impl OperatorMeta for FloatListOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Create a list from float values" }
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
// ListLength Operator
// ============================================================================

pub struct ListLengthOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl ListLengthOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float_list("List")],
            outputs: [OutputPort::int("Length")],
        }
    }
}

impl Default for ListLengthOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ListLengthOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ListLength" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_list(&self.inputs[0], get_input);
        self.outputs[0].set_int(list.len() as i32);
    }
}

impl OperatorMeta for ListLengthOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Get the length of a list" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
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
// ListGet Operator
// ============================================================================

pub struct ListGetOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl ListGetOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float_list("List"),
                InputPort::int("Index", 0),
            ],
            outputs: [OutputPort::float("Value")],
        }
    }
}

impl Default for ListGetOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ListGetOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ListGet" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_list(&self.inputs[0], get_input);
        let index = get_int(&self.inputs[1], get_input);

        // Handle negative indexing from end
        let idx = if index < 0 {
            (list.len() as i32 + index).max(0) as usize
        } else {
            index as usize
        };

        let value = list.get(idx).copied().unwrap_or(0.0);
        self.outputs[0].set_float(value);
    }
}

impl OperatorMeta for ListGetOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Get value at index from list" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            1 => Some(PortMeta::new("Index")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// ListSum Operator
// ============================================================================

pub struct ListSumOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl ListSumOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float_list("List")],
            outputs: [OutputPort::float("Sum")],
        }
    }
}

impl Default for ListSumOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ListSumOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ListSum" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_list(&self.inputs[0], get_input);
        let sum: f32 = list.iter().sum();
        self.outputs[0].set_float(sum);
    }
}

impl OperatorMeta for ListSumOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Sum of all values in list" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
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
// ListAverage Operator
// ============================================================================

pub struct ListAverageOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl ListAverageOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float_list("List")],
            outputs: [OutputPort::float("Average")],
        }
    }
}

impl Default for ListAverageOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ListAverageOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ListAverage" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_list(&self.inputs[0], get_input);
        let avg = if list.is_empty() {
            0.0
        } else {
            list.iter().sum::<f32>() / list.len() as f32
        };
        self.outputs[0].set_float(avg);
    }
}

impl OperatorMeta for ListAverageOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Average of all values in list" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Average").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// ListMin Operator
// ============================================================================

pub struct ListMinOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl ListMinOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float_list("List")],
            outputs: [OutputPort::float("Min")],
        }
    }
}

impl Default for ListMinOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ListMinOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ListMin" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_list(&self.inputs[0], get_input);
        let min = list.iter().cloned().fold(f32::INFINITY, f32::min);
        let result = if min.is_infinite() { 0.0 } else { min };
        self.outputs[0].set_float(result);
    }
}

impl OperatorMeta for ListMinOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Minimum value in list" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Min").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// ListMax Operator
// ============================================================================

pub struct ListMaxOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl ListMaxOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float_list("List")],
            outputs: [OutputPort::float("Max")],
        }
    }
}

impl Default for ListMaxOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ListMaxOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ListMax" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_list(&self.inputs[0], get_input);
        let max = list.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let result = if max.is_infinite() { 0.0 } else { max };
        self.outputs[0].set_float(result);
    }
}

impl OperatorMeta for ListMaxOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Maximum value in list" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Max").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// ListMap Operator (Scale & Offset)
// ============================================================================

pub struct ListMapOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl ListMapOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float_list("List"),
                InputPort::float("Scale", 1.0),
                InputPort::float("Offset", 0.0),
            ],
            outputs: [OutputPort::float_list("Result")],
        }
    }
}

impl Default for ListMapOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ListMapOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ListMap" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_list(&self.inputs[0], get_input);
        let scale = get_float(&self.inputs[1], get_input);
        let offset = get_float(&self.inputs[2], get_input);

        let result: Vec<f32> = list.iter().map(|v| v * scale + offset).collect();
        self.outputs[0].value = Value::FloatList(result);
    }
}

impl OperatorMeta for ListMapOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Scale and offset all values in list" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            1 => Some(PortMeta::new("Scale")),
            2 => Some(PortMeta::new("Offset")),
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
            name: "FloatList",
            category: "List",
            description: "Create list from values",
        },
        || capture_meta(FloatListOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ListLength",
            category: "List",
            description: "Get list length",
        },
        || capture_meta(ListLengthOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ListGet",
            category: "List",
            description: "Get value at index",
        },
        || capture_meta(ListGetOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ListSum",
            category: "List",
            description: "Sum of list values",
        },
        || capture_meta(ListSumOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ListAverage",
            category: "List",
            description: "Average of list values",
        },
        || capture_meta(ListAverageOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ListMin",
            category: "List",
            description: "Minimum value in list",
        },
        || capture_meta(ListMinOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ListMax",
            category: "List",
            description: "Maximum value in list",
        },
        || capture_meta(ListMaxOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ListMap",
            category: "List",
            description: "Scale and offset list values",
        },
        || capture_meta(ListMapOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Float(0.0)
    }

    #[test]
    fn test_list_length() {
        let mut op = ListLengthOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::FloatList(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(5));
    }

    #[test]
    fn test_list_get() {
        let mut op = ListGetOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::FloatList(vec![10.0, 20.0, 30.0]);
        op.inputs[1].default = Value::Int(1);
        op.compute(&ctx, &no_connections);
        assert!((op.outputs[0].value.as_float().unwrap() - 20.0).abs() < 0.001);

        // Negative index
        op.inputs[1].default = Value::Int(-1);
        op.compute(&ctx, &no_connections);
        assert!((op.outputs[0].value.as_float().unwrap() - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_list_sum() {
        let mut op = ListSumOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::FloatList(vec![1.0, 2.0, 3.0, 4.0]);
        op.compute(&ctx, &no_connections);
        assert!((op.outputs[0].value.as_float().unwrap() - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_list_average() {
        let mut op = ListAverageOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::FloatList(vec![2.0, 4.0, 6.0, 8.0]);
        op.compute(&ctx, &no_connections);
        assert!((op.outputs[0].value.as_float().unwrap() - 5.0).abs() < 0.001);

        // Empty list
        op.inputs[0].default = Value::FloatList(vec![]);
        op.compute(&ctx, &no_connections);
        assert!((op.outputs[0].value.as_float().unwrap()).abs() < 0.001);
    }

    #[test]
    fn test_list_min_max() {
        let mut min_op = ListMinOp::new();
        let mut max_op = ListMaxOp::new();
        let ctx = EvalContext::new();

        min_op.inputs[0].default = Value::FloatList(vec![5.0, 2.0, 8.0, 1.0, 9.0]);
        max_op.inputs[0].default = Value::FloatList(vec![5.0, 2.0, 8.0, 1.0, 9.0]);

        min_op.compute(&ctx, &no_connections);
        max_op.compute(&ctx, &no_connections);

        assert!((min_op.outputs[0].value.as_float().unwrap() - 1.0).abs() < 0.001);
        assert!((max_op.outputs[0].value.as_float().unwrap() - 9.0).abs() < 0.001);
    }

    #[test]
    fn test_list_map() {
        let mut op = ListMapOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::FloatList(vec![1.0, 2.0, 3.0]);
        op.inputs[1].default = Value::Float(2.0); // Scale
        op.inputs[2].default = Value::Float(10.0); // Offset
        op.compute(&ctx, &no_connections);

        if let Value::FloatList(result) = &op.outputs[0].value {
            assert!((result[0] - 12.0).abs() < 0.001);
            assert!((result[1] - 14.0).abs() < 0.001);
            assert!((result[2] - 16.0).abs() < 0.001);
        } else {
            panic!("Expected FloatList");
        }
    }
}
