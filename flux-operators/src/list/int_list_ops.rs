//! Integer list operators
//!
//! Type-specific operators for IntList:
//! - IntListOp: Create IntList from multi-input
//! - IntListSum: Sum all integers
//! - IntListMin/IntListMax: Integer extrema
//! - IntListRange: Generate range [start..end]

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};
use flux_core::Value;

fn get_int(input: &InputPort, get_input: InputResolver) -> i32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_int().unwrap_or(0),
        None => input.default.as_int().unwrap_or(0),
    }
}

fn get_int_list(input: &InputPort, get_input: InputResolver) -> Vec<i32> {
    match input.connection {
        Some((node_id, output_idx)) => {
            let value = get_input(node_id, output_idx);
            match value {
                Value::IntList(list) => list,
                Value::Int(i) => vec![i],
                Value::FloatList(fl) => fl.iter().map(|f| *f as i32).collect(),
                Value::Float(f) => vec![f as i32],
                _ => Vec::new(),
            }
        }
        None => match &input.default {
            Value::IntList(list) => list.clone(),
            Value::Int(i) => vec![*i],
            _ => Vec::new(),
        },
    }
}

fn collect_ints(input: &InputPort, get_input: InputResolver) -> Vec<i32> {
    if !input.connections.is_empty() {
        input
            .connections
            .iter()
            .map(|(node_id, output_idx)| {
                get_input(*node_id, *output_idx).as_int().unwrap_or(0)
            })
            .collect()
    } else {
        match &input.default {
            Value::IntList(list) => list.clone(),
            Value::Int(i) => vec![*i],
            _ => Vec::new(),
        }
    }
}

// ============================================================================
// IntList Operator (Creation)
// ============================================================================

pub struct IntListOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: [OutputPort; 1],
}

impl IntListOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![InputPort::int_multi("Values")],
            outputs: [OutputPort::int_list("List")],
        }
    }
}

impl Default for IntListOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for IntListOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "IntList" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let values = collect_ints(&self.inputs[0], get_input);
        self.outputs[0].value = Value::IntList(values);
    }
}

impl OperatorMeta for IntListOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Create an integer list from values" }
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
// IntListSum Operator
// ============================================================================

pub struct IntListSumOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl IntListSumOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::int_list("List")],
            outputs: [OutputPort::int("Sum")],
        }
    }
}

impl Default for IntListSumOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for IntListSumOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "IntListSum" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_int_list(&self.inputs[0], get_input);
        let sum: i32 = list.iter().sum();
        self.outputs[0].set_int(sum);
    }
}

impl OperatorMeta for IntListSumOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Sum of all integers in list" }
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
// IntListMin Operator
// ============================================================================

pub struct IntListMinOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl IntListMinOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::int_list("List")],
            outputs: [OutputPort::int("Min")],
        }
    }
}

impl Default for IntListMinOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for IntListMinOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "IntListMin" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_int_list(&self.inputs[0], get_input);
        let min = list.iter().cloned().min().unwrap_or(0);
        self.outputs[0].set_int(min);
    }
}

impl OperatorMeta for IntListMinOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Minimum value in integer list" }
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
// IntListMax Operator
// ============================================================================

pub struct IntListMaxOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl IntListMaxOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::int_list("List")],
            outputs: [OutputPort::int("Max")],
        }
    }
}

impl Default for IntListMaxOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for IntListMaxOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "IntListMax" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_int_list(&self.inputs[0], get_input);
        let max = list.iter().cloned().max().unwrap_or(0);
        self.outputs[0].set_int(max);
    }
}

impl OperatorMeta for IntListMaxOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Maximum value in integer list" }
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
// IntListRange Operator
// ============================================================================

pub struct IntListRangeOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl IntListRangeOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::int("Start", 0),
                InputPort::int("End", 10),
                InputPort::int("Step", 1),
            ],
            outputs: [OutputPort::int_list("Range")],
        }
    }
}

impl Default for IntListRangeOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for IntListRangeOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "IntListRange" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let start = get_int(&self.inputs[0], get_input);
        let end = get_int(&self.inputs[1], get_input);
        let step = get_int(&self.inputs[2], get_input).max(1); // Ensure step >= 1

        let mut result = Vec::new();
        let mut i = start;
        if start <= end {
            while i < end {
                result.push(i);
                i += step;
            }
        } else {
            // Descending range
            while i > end {
                result.push(i);
                i -= step;
            }
        }

        self.outputs[0].value = Value::IntList(result);
    }
}

impl OperatorMeta for IntListRangeOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Generate a range of integers [start..end)" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Start")),
            1 => Some(PortMeta::new("End")),
            2 => Some(PortMeta::new("Step")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Range").with_shape(PinShape::TriangleFilled)),
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
            name: "IntList",
            category: "List",
            description: "Create integer list from values",
        },
        || capture_meta(IntListOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "IntListSum",
            category: "List",
            description: "Sum of integer list",
        },
        || capture_meta(IntListSumOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "IntListMin",
            category: "List",
            description: "Minimum value in integer list",
        },
        || capture_meta(IntListMinOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "IntListMax",
            category: "List",
            description: "Maximum value in integer list",
        },
        || capture_meta(IntListMaxOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "IntListRange",
            category: "List",
            description: "Generate integer range",
        },
        || capture_meta(IntListRangeOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Int(0)
    }

    #[test]
    fn test_int_list_sum() {
        let mut op = IntListSumOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::IntList(vec![1, 2, 3, 4, 5]);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(15));
    }

    #[test]
    fn test_int_list_min_max() {
        let mut min_op = IntListMinOp::new();
        let mut max_op = IntListMaxOp::new();
        let ctx = EvalContext::new();

        min_op.inputs[0].default = Value::IntList(vec![5, 2, 8, 1, 9]);
        max_op.inputs[0].default = Value::IntList(vec![5, 2, 8, 1, 9]);

        min_op.compute(&ctx, &no_connections);
        max_op.compute(&ctx, &no_connections);

        assert_eq!(min_op.outputs[0].value.as_int(), Some(1));
        assert_eq!(max_op.outputs[0].value.as_int(), Some(9));
    }

    #[test]
    fn test_int_list_range() {
        let mut op = IntListRangeOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Int(0);
        op.inputs[1].default = Value::Int(5);
        op.inputs[2].default = Value::Int(1);
        op.compute(&ctx, &no_connections);

        if let Value::IntList(result) = &op.outputs[0].value {
            assert_eq!(result, &vec![0, 1, 2, 3, 4]);
        } else {
            panic!("Expected IntList");
        }
    }

    #[test]
    fn test_int_list_range_step() {
        let mut op = IntListRangeOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Int(0);
        op.inputs[1].default = Value::Int(10);
        op.inputs[2].default = Value::Int(2);
        op.compute(&ctx, &no_connections);

        if let Value::IntList(result) = &op.outputs[0].value {
            assert_eq!(result, &vec![0, 2, 4, 6, 8]);
        } else {
            panic!("Expected IntList");
        }
    }
}
