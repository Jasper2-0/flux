//! List operators: FloatList, ListLength, ListGet, ListSum, ListAverage, ListMin, ListMax, ListMap
//!
//! ## Polymorphic vs Type-Specific
//!
//! **Polymorphic operators** (work with any list type):
//! - ListLength, ListGet, ListSlice, ListConcat, ListReverse, ListFirst, ListLast
//!
//! **Type-specific operators** (require specific element types):
//! - FloatList (creation), ListSum, ListAverage, ListMin, ListMax, ListMap, ListFilter

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::value::{Color, ValueType};
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

/// Get float list (legacy helper for FloatList-specific operators)
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

// ============================================================================
// Polymorphic List Helpers
// ============================================================================

/// Get any list value (for polymorphic operators)
fn get_any_list(input: &InputPort, get_input: InputResolver) -> Value {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx),
        None => input.default.clone(),
    }
}

/// Get length of any list type
fn list_length(value: &Value) -> usize {
    match value {
        Value::FloatList(l) => l.len(),
        Value::IntList(l) => l.len(),
        Value::BoolList(l) => l.len(),
        Value::Vec2List(l) => l.len(),
        Value::Vec3List(l) => l.len(),
        Value::Vec4List(l) => l.len(),
        Value::ColorList(l) => l.len(),
        Value::StringList(l) => l.len(),
        // Scalars are treated as single-element lists for compatibility
        Value::Float(_) | Value::Int(_) | Value::Bool(_) => 1,
        Value::Vec2(_) | Value::Vec3(_) | Value::Vec4(_) | Value::Color(_) | Value::String(_) => 1,
        _ => 0,
    }
}

/// Get element at index from any list type (returns Value)
fn list_get(value: &Value, index: i32) -> Value {
    let len = list_length(value);
    if len == 0 {
        return value.value_type().default_value();
    }

    // Handle negative indexing
    let idx = if index < 0 {
        (len as i32 + index).max(0) as usize
    } else {
        index as usize
    };

    if idx >= len {
        return value.value_type().default_value();
    }

    match value {
        Value::FloatList(l) => Value::Float(l.get(idx).copied().unwrap_or(0.0)),
        Value::IntList(l) => Value::Int(l.get(idx).copied().unwrap_or(0)),
        Value::BoolList(l) => Value::Bool(l.get(idx).copied().unwrap_or(false)),
        Value::Vec2List(l) => Value::Vec2(l.get(idx).copied().unwrap_or([0.0, 0.0])),
        Value::Vec3List(l) => Value::Vec3(l.get(idx).copied().unwrap_or([0.0, 0.0, 0.0])),
        Value::Vec4List(l) => Value::Vec4(l.get(idx).copied().unwrap_or([0.0, 0.0, 0.0, 0.0])),
        Value::ColorList(l) => Value::Color(l.get(idx).copied().unwrap_or(Color::BLACK)),
        Value::StringList(l) => Value::String(l.get(idx).cloned().unwrap_or_default()),
        // Scalars return themselves at index 0
        Value::Float(f) if idx == 0 => Value::Float(*f),
        Value::Int(i) if idx == 0 => Value::Int(*i),
        Value::Bool(b) if idx == 0 => Value::Bool(*b),
        Value::Vec2(v) if idx == 0 => Value::Vec2(*v),
        Value::Vec3(v) if idx == 0 => Value::Vec3(*v),
        Value::Vec4(v) if idx == 0 => Value::Vec4(*v),
        Value::Color(c) if idx == 0 => Value::Color(*c),
        Value::String(s) if idx == 0 => Value::String(s.clone()),
        _ => value.value_type().default_value(),
    }
}

/// Slice any list type (returns same list type)
fn list_slice(value: &Value, start: i32, end: i32) -> Value {
    let len = list_length(value) as i32;
    if len == 0 {
        return value.clone();
    }

    // Handle negative indices (Python-style)
    let start_idx = if start < 0 {
        (len + start).max(0) as usize
    } else {
        (start as usize).min(len as usize)
    };

    let end_idx = if end < 0 {
        (len + end).max(0) as usize
    } else {
        (end as usize).min(len as usize)
    };

    if start_idx >= end_idx {
        // Return empty list of same type
        return match value {
            Value::FloatList(_) => Value::FloatList(vec![]),
            Value::IntList(_) => Value::IntList(vec![]),
            Value::BoolList(_) => Value::BoolList(vec![]),
            Value::Vec2List(_) => Value::Vec2List(vec![]),
            Value::Vec3List(_) => Value::Vec3List(vec![]),
            Value::Vec4List(_) => Value::Vec4List(vec![]),
            Value::ColorList(_) => Value::ColorList(vec![]),
            Value::StringList(_) => Value::StringList(vec![]),
            _ => value.clone(),
        };
    }

    match value {
        Value::FloatList(l) => Value::FloatList(l[start_idx..end_idx].to_vec()),
        Value::IntList(l) => Value::IntList(l[start_idx..end_idx].to_vec()),
        Value::BoolList(l) => Value::BoolList(l[start_idx..end_idx].to_vec()),
        Value::Vec2List(l) => Value::Vec2List(l[start_idx..end_idx].to_vec()),
        Value::Vec3List(l) => Value::Vec3List(l[start_idx..end_idx].to_vec()),
        Value::Vec4List(l) => Value::Vec4List(l[start_idx..end_idx].to_vec()),
        Value::ColorList(l) => Value::ColorList(l[start_idx..end_idx].to_vec()),
        Value::StringList(l) => Value::StringList(l[start_idx..end_idx].to_vec()),
        _ => value.clone(),
    }
}

/// Concatenate two lists of the same type
fn list_concat(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::FloatList(la), Value::FloatList(lb)) => {
            let mut result = la.clone();
            result.extend(lb);
            Value::FloatList(result)
        }
        (Value::IntList(la), Value::IntList(lb)) => {
            let mut result = la.clone();
            result.extend(lb);
            Value::IntList(result)
        }
        (Value::BoolList(la), Value::BoolList(lb)) => {
            let mut result = la.clone();
            result.extend(lb);
            Value::BoolList(result)
        }
        (Value::Vec2List(la), Value::Vec2List(lb)) => {
            let mut result = la.clone();
            result.extend(lb);
            Value::Vec2List(result)
        }
        (Value::Vec3List(la), Value::Vec3List(lb)) => {
            let mut result = la.clone();
            result.extend(lb);
            Value::Vec3List(result)
        }
        (Value::Vec4List(la), Value::Vec4List(lb)) => {
            let mut result = la.clone();
            result.extend(lb);
            Value::Vec4List(result)
        }
        (Value::ColorList(la), Value::ColorList(lb)) => {
            let mut result = la.clone();
            result.extend(lb);
            Value::ColorList(result)
        }
        (Value::StringList(la), Value::StringList(lb)) => {
            let mut result = la.clone();
            result.extend(lb.iter().cloned());
            Value::StringList(result)
        }
        // Cross-type: try coercion or return first list
        _ => {
            // Attempt to coerce b to a's type
            if let Some(coerced) = b.coerce_to(a.value_type()) {
                list_concat(a, &coerced)
            } else {
                a.clone()
            }
        }
    }
}

/// Reverse any list type
fn list_reverse(value: &Value) -> Value {
    match value {
        Value::FloatList(l) => Value::FloatList(l.iter().rev().copied().collect()),
        Value::IntList(l) => Value::IntList(l.iter().rev().copied().collect()),
        Value::BoolList(l) => Value::BoolList(l.iter().rev().copied().collect()),
        Value::Vec2List(l) => Value::Vec2List(l.iter().rev().copied().collect()),
        Value::Vec3List(l) => Value::Vec3List(l.iter().rev().copied().collect()),
        Value::Vec4List(l) => Value::Vec4List(l.iter().rev().copied().collect()),
        Value::ColorList(l) => Value::ColorList(l.iter().rev().copied().collect()),
        Value::StringList(l) => Value::StringList(l.iter().rev().cloned().collect()),
        _ => value.clone(),
    }
}

/// Get output type for element extraction from a list type
fn element_type_for_list(list_type: ValueType) -> ValueType {
    match list_type {
        ValueType::FloatList => ValueType::Float,
        ValueType::IntList => ValueType::Int,
        ValueType::BoolList => ValueType::Bool,
        ValueType::Vec2List => ValueType::Vec2,
        ValueType::Vec3List => ValueType::Vec3,
        ValueType::Vec4List => ValueType::Vec4,
        ValueType::ColorList => ValueType::Color,
        ValueType::StringList => ValueType::String,
        // For non-list types (scalar passthrough), return the same type
        other => other,
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
// ListLength Operator (Polymorphic)
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
            // Use FloatList as default, but accepts any list via TypeCategory::List
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
        let list_value = get_any_list(&self.inputs[0], get_input);
        self.outputs[0].set_int(list_length(&list_value) as i32);
    }
}

impl OperatorMeta for ListLengthOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Get the length of any list type" }
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
// ListGet Operator (Polymorphic)
// ============================================================================

pub struct ListGetOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: Vec<OutputPort>,
}

impl ListGetOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float_list("List"),
                InputPort::int("Index", 0),
            ],
            // Dynamic output type based on input list type
            outputs: vec![OutputPort::float("Value")],
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
        let list_value = get_any_list(&self.inputs[0], get_input);
        let index = get_int(&self.inputs[1], get_input);

        // Use polymorphic list_get
        let value = list_get(&list_value, index);

        // Update output type if needed and set value
        let elem_type = element_type_for_list(list_value.value_type());
        if self.outputs[0].value_type != elem_type {
            self.outputs[0] = OutputPort::new("Value", elem_type);
        }
        self.outputs[0].value = value;
    }
}

impl OperatorMeta for ListGetOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Get value at index from any list (supports negative indexing)" }
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
// ListFilter Operator
// ============================================================================

pub struct ListFilterOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl ListFilterOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float_list("List"),
                InputPort::float("Threshold", 0.0),
                InputPort::int("Mode", 0), // 0=GT, 1=LT, 2=GTE, 3=LTE
            ],
            outputs: [OutputPort::float_list("Filtered")],
        }
    }
}

impl Default for ListFilterOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ListFilterOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ListFilter" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list = get_list(&self.inputs[0], get_input);
        let threshold = get_float(&self.inputs[1], get_input);
        let mode = get_int(&self.inputs[2], get_input);

        let result: Vec<f32> = list.into_iter().filter(|&v| {
            match mode {
                0 => v > threshold,   // GT
                1 => v < threshold,   // LT
                2 => v >= threshold,  // GTE
                3 => v <= threshold,  // LTE
                _ => v > threshold,   // Default to GT
            }
        }).collect();

        self.outputs[0].value = Value::FloatList(result);
    }
}

impl OperatorMeta for ListFilterOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Filter list elements by threshold comparison" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            1 => Some(PortMeta::new("Threshold")),
            2 => Some(PortMeta::new("Mode")), // 0=GT, 1=LT, 2=GTE, 3=LTE
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Filtered").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// ListConcat Operator (Polymorphic)
// ============================================================================

pub struct ListConcatOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: Vec<OutputPort>,
}

impl ListConcatOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float_list("ListA"),
                InputPort::float_list("ListB"),
            ],
            outputs: vec![OutputPort::float_list("Combined")],
        }
    }
}

impl Default for ListConcatOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ListConcatOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ListConcat" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list_a = get_any_list(&self.inputs[0], get_input);
        let list_b = get_any_list(&self.inputs[1], get_input);

        let result = list_concat(&list_a, &list_b);

        // Update output type if needed
        if self.outputs[0].value_type != result.value_type() {
            self.outputs[0] = OutputPort::new("Combined", result.value_type());
        }
        self.outputs[0].value = result;
    }
}

impl OperatorMeta for ListConcatOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Concatenate two lists of the same type" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("ListA")),
            1 => Some(PortMeta::new("ListB")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Combined").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// ListSlice Operator (Polymorphic)
// ============================================================================

pub struct ListSliceOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: Vec<OutputPort>,
}

impl ListSliceOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float_list("List"),
                InputPort::int("Start", 0),
                InputPort::int("End", i32::MAX), // i32::MAX means end of list
            ],
            outputs: vec![OutputPort::float_list("Slice")],
        }
    }
}

impl Default for ListSliceOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ListSliceOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ListSlice" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list_value = get_any_list(&self.inputs[0], get_input);
        let start = get_int(&self.inputs[1], get_input);
        let end = get_int(&self.inputs[2], get_input);

        let result = list_slice(&list_value, start, end);

        // Update output type if needed
        if self.outputs[0].value_type != result.value_type() {
            self.outputs[0] = OutputPort::new("Slice", result.value_type());
        }
        self.outputs[0].value = result;
    }
}

impl OperatorMeta for ListSliceOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Extract a slice from any list (supports negative indices)" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            1 => Some(PortMeta::new("Start")), // Negative = from end
            2 => Some(PortMeta::new("End")),   // Exclusive; negative = from end
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Slice").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// ListReverse Operator (Polymorphic)
// ============================================================================

pub struct ListReverseOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: Vec<OutputPort>,
}

impl ListReverseOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float_list("List")],
            outputs: vec![OutputPort::float_list("Reversed")],
        }
    }
}

impl Default for ListReverseOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ListReverseOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ListReverse" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list_value = get_any_list(&self.inputs[0], get_input);
        let result = list_reverse(&list_value);

        // Update output type if needed
        if self.outputs[0].value_type != result.value_type() {
            self.outputs[0] = OutputPort::new("Reversed", result.value_type());
        }
        self.outputs[0].value = result;
    }
}

impl OperatorMeta for ListReverseOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Reverse the order of elements in any list" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Reversed").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// ListFirst Operator (Polymorphic)
// ============================================================================

pub struct ListFirstOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: Vec<OutputPort>,
}

impl ListFirstOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float_list("List")],
            outputs: vec![OutputPort::float("First")],
        }
    }
}

impl Default for ListFirstOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ListFirstOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ListFirst" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list_value = get_any_list(&self.inputs[0], get_input);
        let value = list_get(&list_value, 0);

        // Update output type if needed
        let elem_type = element_type_for_list(list_value.value_type());
        if self.outputs[0].value_type != elem_type {
            self.outputs[0] = OutputPort::new("First", elem_type);
        }
        self.outputs[0].value = value;
    }
}

impl OperatorMeta for ListFirstOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Get the first element of any list" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("First").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// ListLast Operator (Polymorphic)
// ============================================================================

pub struct ListLastOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: Vec<OutputPort>,
}

impl ListLastOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float_list("List")],
            outputs: vec![OutputPort::float("Last")],
        }
    }
}

impl Default for ListLastOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ListLastOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ListLast" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let list_value = get_any_list(&self.inputs[0], get_input);
        let value = list_get(&list_value, -1); // -1 = last element

        // Update output type if needed
        let elem_type = element_type_for_list(list_value.value_type());
        if self.outputs[0].value_type != elem_type {
            self.outputs[0] = OutputPort::new("Last", elem_type);
        }
        self.outputs[0].value = value;
    }
}

impl OperatorMeta for ListLastOp {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str { "Get the last element of any list" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Last").with_shape(PinShape::TriangleFilled)),
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

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ListFilter",
            category: "List",
            description: "Filter list by threshold",
        },
        || capture_meta(ListFilterOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ListConcat",
            category: "List",
            description: "Concatenate two lists",
        },
        || capture_meta(ListConcatOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ListSlice",
            category: "List",
            description: "Extract slice from list",
        },
        || capture_meta(ListSliceOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ListReverse",
            category: "List",
            description: "Reverse list order",
        },
        || capture_meta(ListReverseOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ListFirst",
            category: "List",
            description: "Get first list element",
        },
        || capture_meta(ListFirstOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "ListLast",
            category: "List",
            description: "Get last list element",
        },
        || capture_meta(ListLastOp::new()),
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

    #[test]
    fn test_list_filter() {
        let mut op = ListFilterOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::FloatList(vec![1.0, 5.0, 2.0, 8.0, 3.0]);
        op.inputs[1].default = Value::Float(3.0); // Threshold
        op.inputs[2].default = Value::Int(0); // Mode: GT (greater than)
        op.compute(&ctx, &no_connections);

        if let Value::FloatList(result) = &op.outputs[0].value {
            assert_eq!(result.len(), 2);
            assert!((result[0] - 5.0).abs() < 0.001);
            assert!((result[1] - 8.0).abs() < 0.001);
        } else {
            panic!("Expected FloatList");
        }

        // Test LT mode
        op.inputs[2].default = Value::Int(1); // Mode: LT (less than)
        op.compute(&ctx, &no_connections);

        if let Value::FloatList(result) = &op.outputs[0].value {
            assert_eq!(result.len(), 2);
            assert!((result[0] - 1.0).abs() < 0.001);
            assert!((result[1] - 2.0).abs() < 0.001);
        } else {
            panic!("Expected FloatList");
        }
    }

    #[test]
    fn test_list_concat() {
        let mut op = ListConcatOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::FloatList(vec![1.0, 2.0]);
        op.inputs[1].default = Value::FloatList(vec![3.0, 4.0, 5.0]);
        op.compute(&ctx, &no_connections);

        if let Value::FloatList(result) = &op.outputs[0].value {
            assert_eq!(result.len(), 5);
            assert!((result[0] - 1.0).abs() < 0.001);
            assert!((result[1] - 2.0).abs() < 0.001);
            assert!((result[2] - 3.0).abs() < 0.001);
            assert!((result[3] - 4.0).abs() < 0.001);
            assert!((result[4] - 5.0).abs() < 0.001);
        } else {
            panic!("Expected FloatList");
        }
    }

    #[test]
    fn test_list_slice() {
        let mut op = ListSliceOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::FloatList(vec![10.0, 20.0, 30.0, 40.0, 50.0]);

        // Slice [1:3] -> [20, 30]
        op.inputs[1].default = Value::Int(1);
        op.inputs[2].default = Value::Int(3);
        op.compute(&ctx, &no_connections);

        if let Value::FloatList(result) = &op.outputs[0].value {
            assert_eq!(result.len(), 2);
            assert!((result[0] - 20.0).abs() < 0.001);
            assert!((result[1] - 30.0).abs() < 0.001);
        } else {
            panic!("Expected FloatList");
        }

        // Negative index: [-2:-1] -> last element excluding the very last
        // Actually [-2:-1] in Python means [index -2 to -1) = [40]
        op.inputs[1].default = Value::Int(-2);
        op.inputs[2].default = Value::Int(-1);
        op.compute(&ctx, &no_connections);

        if let Value::FloatList(result) = &op.outputs[0].value {
            assert_eq!(result.len(), 1);
            assert!((result[0] - 40.0).abs() < 0.001);
        } else {
            panic!("Expected FloatList");
        }

        // First 3 elements: [0:3]
        op.inputs[1].default = Value::Int(0);
        op.inputs[2].default = Value::Int(3);
        op.compute(&ctx, &no_connections);

        if let Value::FloatList(result) = &op.outputs[0].value {
            assert_eq!(result.len(), 3);
            assert!((result[0] - 10.0).abs() < 0.001);
            assert!((result[2] - 30.0).abs() < 0.001);
        } else {
            panic!("Expected FloatList");
        }
    }
}
