//! Output port definitions

use crate::dirty_flag::DirtyFlag;
use crate::id::Id;
use crate::value::{Color, Value, ValueType};

use super::OutputTypeRule;

/// An output port that produces a value
#[derive(Clone, Debug)]
pub struct OutputPort {
    pub id: Id,
    pub name: &'static str,
    /// The declared type of value this port produces (for backward compatibility)
    pub value_type: ValueType,
    /// Type rule for polymorphic outputs
    pub type_rule: OutputTypeRule,
    /// Resolved type based on connected inputs (for polymorphic ports)
    pub resolved_type: Option<ValueType>,
    /// Cached value
    pub value: Value,
    dirty_flag: DirtyFlag,
}

impl OutputPort {
    /// Create a new output port with fixed type
    pub fn new(name: &'static str, value_type: ValueType) -> Self {
        Self {
            id: Id::new(),
            name,
            value_type,
            type_rule: OutputTypeRule::Fixed(value_type),
            resolved_type: None,
            value: value_type.default_value(),
            dirty_flag: DirtyFlag::new(),
        }
    }

    /// Create a polymorphic output that matches the first input
    pub fn same_as_first(name: &'static str) -> Self {
        Self {
            id: Id::new(),
            name,
            value_type: ValueType::Float, // Default until resolved
            type_rule: OutputTypeRule::SameAsInput(0),
            resolved_type: None,
            value: Value::Float(0.0),
            dirty_flag: DirtyFlag::new(),
        }
    }

    /// Create a polymorphic output that matches a specific input
    pub fn same_as_input(name: &'static str, input_index: usize) -> Self {
        Self {
            id: Id::new(),
            name,
            value_type: ValueType::Float, // Default until resolved
            type_rule: OutputTypeRule::SameAsInput(input_index),
            resolved_type: None,
            value: Value::Float(0.0),
            dirty_flag: DirtyFlag::new(),
        }
    }

    /// Create a polymorphic output that uses the wider of the first two inputs
    pub fn wider_of_inputs(name: &'static str) -> Self {
        Self {
            id: Id::new(),
            name,
            value_type: ValueType::Float, // Default until resolved
            type_rule: OutputTypeRule::Wider(vec![0, 1]),
            resolved_type: None,
            value: Value::Float(0.0),
            dirty_flag: DirtyFlag::new(),
        }
    }

    /// Create a polymorphic output with custom type rule
    pub fn polymorphic(name: &'static str, type_rule: OutputTypeRule) -> Self {
        Self {
            id: Id::new(),
            name,
            value_type: ValueType::Float, // Default until resolved
            type_rule,
            resolved_type: None,
            value: Value::Float(0.0),
            dirty_flag: DirtyFlag::new(),
        }
    }

    /// Convenience constructor for float output
    pub fn float(name: &'static str) -> Self {
        Self::new(name, ValueType::Float)
    }

    /// Convenience constructor for int output
    pub fn int(name: &'static str) -> Self {
        Self::new(name, ValueType::Int)
    }

    /// Convenience constructor for bool output
    pub fn bool(name: &'static str) -> Self {
        Self::new(name, ValueType::Bool)
    }

    /// Convenience constructor for vec3 output
    pub fn vec3(name: &'static str) -> Self {
        Self::new(name, ValueType::Vec3)
    }

    /// Create a new output port with explicit type
    pub fn new_typed(name: &'static str, value_type: ValueType) -> Self {
        Self::new(name, value_type)
    }

    /// Check if this output needs recomputation
    pub fn is_dirty(&self) -> bool {
        self.dirty_flag.is_dirty()
    }

    /// Mark this output as needing recomputation
    pub fn mark_dirty(&mut self) {
        self.dirty_flag.mark_dirty();
    }

    /// Set the value and mark as clean
    pub fn set(&mut self, value: Value) {
        self.value = value;
        self.dirty_flag.mark_clean();
    }

    /// Set float value (convenience method)
    pub fn set_float(&mut self, value: f32) {
        self.set(Value::Float(value));
    }

    /// Set int value (convenience method)
    pub fn set_int(&mut self, value: i32) {
        self.set(Value::Int(value));
    }

    /// Set bool value (convenience method)
    pub fn set_bool(&mut self, value: bool) {
        self.set(Value::Bool(value));
    }

    /// Set vec3 value (convenience method)
    pub fn set_vec3(&mut self, value: [f32; 3]) {
        self.set(Value::Vec3(value));
    }

    /// Get the value as f32 (returns 0.0 if wrong type)
    pub fn as_float(&self) -> f32 {
        self.value.as_float().unwrap_or(0.0)
    }

    /// Get the current value
    pub fn get(&self) -> Value {
        self.value.clone()
    }

    /// Convenience constructor for vec4 output
    pub fn vec4(name: &'static str) -> Self {
        Self::new(name, ValueType::Vec4)
    }

    /// Convenience constructor for color output
    pub fn color(name: &'static str) -> Self {
        Self::new(name, ValueType::Color)
    }

    /// Convenience constructor for gradient output
    pub fn gradient(name: &'static str) -> Self {
        Self::new(name, ValueType::Gradient)
    }

    /// Convenience constructor for matrix4 output
    pub fn matrix4(name: &'static str) -> Self {
        Self::new(name, ValueType::Matrix4)
    }

    /// Set vec4 value (convenience method)
    pub fn set_vec4(&mut self, value: [f32; 4]) {
        self.set(Value::Vec4(value));
    }

    /// Convenience constructor for vec2 output
    pub fn vec2(name: &'static str) -> Self {
        Self::new(name, ValueType::Vec2)
    }

    /// Convenience constructor for string output
    pub fn string(name: &'static str) -> Self {
        Self::new(name, ValueType::String)
    }

    /// Convenience constructor for float list output
    pub fn float_list(name: &'static str) -> Self {
        Self::new(name, ValueType::FloatList)
    }

    /// Convenience constructor for int list output
    pub fn int_list(name: &'static str) -> Self {
        Self::new(name, ValueType::IntList)
    }

    /// Convenience constructor for bool list output
    pub fn bool_list(name: &'static str) -> Self {
        Self::new(name, ValueType::BoolList)
    }

    /// Convenience constructor for vec2 list output
    pub fn vec2_list(name: &'static str) -> Self {
        Self::new(name, ValueType::Vec2List)
    }

    /// Convenience constructor for vec3 list output
    pub fn vec3_list(name: &'static str) -> Self {
        Self::new(name, ValueType::Vec3List)
    }

    /// Convenience constructor for vec4 list output
    pub fn vec4_list(name: &'static str) -> Self {
        Self::new(name, ValueType::Vec4List)
    }

    /// Convenience constructor for color list output
    pub fn color_list(name: &'static str) -> Self {
        Self::new(name, ValueType::ColorList)
    }

    /// Convenience constructor for string list output
    pub fn string_list(name: &'static str) -> Self {
        Self::new(name, ValueType::StringList)
    }

    /// Set vec2 value (convenience method)
    pub fn set_vec2(&mut self, value: [f32; 2]) {
        self.set(Value::Vec2(value));
    }

    /// Set string value (convenience method)
    pub fn set_string(&mut self, value: &str) {
        self.set(Value::String(value.to_string()));
    }

    /// Set color value (convenience method)
    pub fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.set(Value::Color(Color::rgba(r, g, b, a)));
    }

    /// Resolve the output type based on connected input types
    ///
    /// Call this when inputs are connected/disconnected to update the output type.
    pub fn resolve_type(&mut self, input_types: &[Option<ValueType>]) {
        let resolved = self.type_rule.resolve(input_types);
        self.resolved_type = Some(resolved);
        self.value_type = resolved;
    }

    /// Clear the resolved type (reset to default)
    pub fn clear_resolved_type(&mut self) {
        self.resolved_type = None;
        // Reset to default based on type rule
        self.value_type = match &self.type_rule {
            OutputTypeRule::Fixed(t) => *t,
            _ => ValueType::Float,
        };
    }

    /// Get the effective output type (resolved or declared)
    pub fn effective_type(&self) -> ValueType {
        self.resolved_type.unwrap_or(self.value_type)
    }

    /// Check if this is a polymorphic output
    pub fn is_polymorphic(&self) -> bool {
        !matches!(self.type_rule, OutputTypeRule::Fixed(_))
    }
}
