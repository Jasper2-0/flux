//! ArrayIterator - Trigger-based iteration over lists
//!
//! Implements the cables.gl-style iteration pattern where:
//! - User fires the `Iterate` trigger to advance to the next element
//! - `OnElement` trigger fires after each element is output
//! - `OnComplete` trigger fires when iteration finishes
//! - User can wire `OnElement` back to `Iterate` for auto-continuation

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort, TriggerInput, TriggerOutput};
use flux_core::Value;

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
fn list_get(value: &Value, index: usize) -> Value {
    use flux_core::value::Color;

    let len = list_length(value);
    if len == 0 || index >= len {
        return value.value_type().default_value();
    }

    match value {
        Value::FloatList(l) => Value::Float(l.get(index).copied().unwrap_or(0.0)),
        Value::IntList(l) => Value::Int(l.get(index).copied().unwrap_or(0)),
        Value::BoolList(l) => Value::Bool(l.get(index).copied().unwrap_or(false)),
        Value::Vec2List(l) => Value::Vec2(l.get(index).copied().unwrap_or([0.0, 0.0])),
        Value::Vec3List(l) => Value::Vec3(l.get(index).copied().unwrap_or([0.0, 0.0, 0.0])),
        Value::Vec4List(l) => Value::Vec4(l.get(index).copied().unwrap_or([0.0, 0.0, 0.0, 0.0])),
        Value::ColorList(l) => Value::Color(l.get(index).copied().unwrap_or(Color::BLACK)),
        Value::StringList(l) => Value::String(l.get(index).cloned().unwrap_or_default()),
        // Scalars return themselves at index 0
        Value::Float(f) if index == 0 => Value::Float(*f),
        Value::Int(i) if index == 0 => Value::Int(*i),
        Value::Bool(b) if index == 0 => Value::Bool(*b),
        Value::Vec2(v) if index == 0 => Value::Vec2(*v),
        Value::Vec3(v) if index == 0 => Value::Vec3(*v),
        Value::Vec4(v) if index == 0 => Value::Vec4(*v),
        Value::Color(c) if index == 0 => Value::Color(*c),
        Value::String(s) if index == 0 => Value::String(s.clone()),
        _ => value.value_type().default_value(),
    }
}

/// ArrayIterator: State-machine based iterator over any list type
///
/// ## Inputs
/// - `List`: Any list type to iterate over
///
/// ## Trigger Inputs
/// - `Iterate`: Advances to next element (or starts iteration if at beginning)
///
/// ## Outputs
/// - `Element`: The current element (preserves type from input list)
/// - `Index`: Current index (Int)
///
/// ## Trigger Outputs
/// - `OnElement`: Fires after each element is output
/// - `OnComplete`: Fires when iteration is complete
///
/// ## Usage Pattern
/// Wire `OnElement` → `Iterate` for automatic full-list iteration
/// Or trigger `Iterate` manually for stepped iteration
pub struct ArrayIterator {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 2],
    trigger_inputs: Vec<TriggerInput>,
    trigger_outputs: Vec<TriggerOutput>,

    // Internal state
    current_index: usize,
    list_length: usize,
    // Flag to track if we're in an active iteration
    is_iterating: bool,
}

impl ArrayIterator {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::any("List", Value::float_list(Vec::new())),
            ],
            outputs: [
                OutputPort::new("Element", flux_core::value::ValueType::Float),
                OutputPort::int("Index"),
            ],
            trigger_inputs: vec![TriggerInput::new("Iterate")],
            trigger_outputs: vec![
                TriggerOutput::new("OnElement"),
                TriggerOutput::new("OnComplete"),
            ],
            current_index: 0,
            list_length: 0,
            is_iterating: false,
        }
    }

    /// Get the list value from input
    fn get_list_value(&self, get_input: InputResolver) -> Value {
        match self.inputs[0].connection {
            Some((node_id, output_idx)) => get_input(node_id, output_idx),
            None => self.inputs[0].default.clone(),
        }
    }
}

impl Default for ArrayIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ArrayIterator {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "ArrayIterator" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn trigger_inputs(&self) -> &[TriggerInput] { &self.trigger_inputs }
    fn trigger_inputs_mut(&mut self) -> &mut [TriggerInput] { &mut self.trigger_inputs }
    fn trigger_outputs(&self) -> &[TriggerOutput] { &self.trigger_outputs }
    fn trigger_outputs_mut(&mut self) -> &mut [TriggerOutput] { &mut self.trigger_outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        // Update list length when compute is called
        let list = self.get_list_value(get_input);
        self.list_length = list_length(&list);

        // Update current element if in iteration
        if self.current_index < self.list_length {
            self.outputs[0].value = list_get(&list, self.current_index);
        }
        self.outputs[1].value = Value::Int(self.current_index as i32);
    }

    fn on_triggered(
        &mut self,
        trigger_index: usize,
        _ctx: &EvalContext,
        get_input: InputResolver,
    ) -> Vec<usize> {
        if trigger_index != 0 {
            return vec![];  // Only respond to "Iterate" trigger (index 0)
        }

        let list = self.get_list_value(get_input);
        let len = list_length(&list);

        // If starting a new iteration, reset state
        if !self.is_iterating {
            self.current_index = 0;
            self.list_length = len;
            self.is_iterating = true;
        }

        if self.current_index < len {
            // Output current element and index
            let element = list_get(&list, self.current_index);
            self.outputs[0].value = element;
            self.outputs[1].value = Value::Int(self.current_index as i32);

            // Advance index for next call
            self.current_index += 1;

            // Check if this was the last element
            if self.current_index >= len {
                // Reset for next iteration
                self.current_index = 0;
                self.is_iterating = false;
                // Fire both OnElement and OnComplete
                return vec![0, 1];
            }

            // Fire OnElement (index 0)
            return vec![0];
        } else {
            // Iteration complete or empty list - reset state and fire OnComplete
            self.current_index = 0;
            self.is_iterating = false;

            // Fire OnComplete (index 1)
            return vec![1];
        }
    }
}

impl OperatorMeta for ArrayIterator {
    fn category(&self) -> &'static str { "List" }
    fn category_color(&self) -> [f32; 4] { category_colors::LIST }
    fn description(&self) -> &'static str {
        "Iterate over list elements with triggers. Wire OnElement→Iterate for auto-loop."
    }

    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("List")),
            _ => None,
        }
    }

    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Element").with_shape(PinShape::TriangleFilled)),
            1 => Some(PortMeta::new("Index")),
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
            name: "ArrayIterator",
            category: "List",
            description: "Trigger-based list iteration",
        },
        || capture_meta(ArrayIterator::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Float(0.0)
    }

    #[test]
    fn test_array_iterator_basic() {
        let mut op = ArrayIterator::new();
        let ctx = EvalContext::new();

        // Set up a list to iterate
        op.inputs[0].default = Value::float_list(vec![10.0, 20.0, 30.0]);
        op.compute(&ctx, &no_connections);  // Initialize list_length

        // First trigger: should output first element
        let triggers = op.on_triggered(0, &ctx, &no_connections);
        assert_eq!(triggers, vec![0]);  // OnElement
        assert!((op.outputs[0].value.as_float().unwrap() - 10.0).abs() < 0.001);
        assert_eq!(op.outputs[1].value.as_int().unwrap(), 0);

        // Second trigger: should output second element
        let triggers = op.on_triggered(0, &ctx, &no_connections);
        assert_eq!(triggers, vec![0]);  // OnElement
        assert!((op.outputs[0].value.as_float().unwrap() - 20.0).abs() < 0.001);
        assert_eq!(op.outputs[1].value.as_int().unwrap(), 1);

        // Third trigger: should output third element (last one, fires both)
        let triggers = op.on_triggered(0, &ctx, &no_connections);
        assert_eq!(triggers, vec![0, 1]);  // OnElement + OnComplete
        assert!((op.outputs[0].value.as_float().unwrap() - 30.0).abs() < 0.001);
        assert_eq!(op.outputs[1].value.as_int().unwrap(), 2);
    }

    #[test]
    fn test_array_iterator_empty_list() {
        let mut op = ArrayIterator::new();
        let ctx = EvalContext::new();

        // Empty list
        op.inputs[0].default = Value::float_list(vec![]);
        op.compute(&ctx, &no_connections);

        // Trigger should immediately fire OnComplete
        let triggers = op.on_triggered(0, &ctx, &no_connections);
        assert_eq!(triggers, vec![1]);  // OnComplete
    }

    #[test]
    fn test_array_iterator_restart() {
        let mut op = ArrayIterator::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::float_list(vec![1.0, 2.0]);
        op.compute(&ctx, &no_connections);

        // First iteration
        let _ = op.on_triggered(0, &ctx, &no_connections);  // Element 0
        let triggers = op.on_triggered(0, &ctx, &no_connections);  // Element 1 + Complete
        assert_eq!(triggers, vec![0, 1]);

        // After completion, next trigger starts fresh
        let triggers = op.on_triggered(0, &ctx, &no_connections);
        assert_eq!(triggers, vec![0]);  // OnElement (back to start)
        assert!((op.outputs[0].value.as_float().unwrap() - 1.0).abs() < 0.001);
        assert_eq!(op.outputs[1].value.as_int().unwrap(), 0);
    }

    #[test]
    fn test_array_iterator_int_list() {
        let mut op = ArrayIterator::new();
        let ctx = EvalContext::new();

        // IntList should work polymorphically
        op.inputs[0].default = Value::int_list(vec![100, 200, 300]);
        op.compute(&ctx, &no_connections);

        let _ = op.on_triggered(0, &ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int().unwrap(), 100);
        assert_eq!(op.outputs[1].value.as_int().unwrap(), 0);

        let _ = op.on_triggered(0, &ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int().unwrap(), 200);
        assert_eq!(op.outputs[1].value.as_int().unwrap(), 1);
    }

    #[test]
    fn test_array_iterator_vec3_list() {
        let mut op = ArrayIterator::new();
        let ctx = EvalContext::new();

        // Vec3List should work polymorphically
        op.inputs[0].default = Value::vec3_list(vec![
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
        ]);
        op.compute(&ctx, &no_connections);

        let _ = op.on_triggered(0, &ctx, &no_connections);
        let v = op.outputs[0].value.as_vec3().unwrap();
        assert!((v[0] - 1.0).abs() < 0.001);
        assert!((v[1] - 2.0).abs() < 0.001);
        assert!((v[2] - 3.0).abs() < 0.001);
    }
}
