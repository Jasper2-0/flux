//! Output port definitions

use crate::dirty_flag::DirtyFlag;
use crate::id::Id;
use crate::value::{Color, Value, ValueType};

/// An output port that produces a value
#[derive(Clone, Debug)]
pub struct OutputPort {
    pub id: Id,
    pub name: &'static str,
    /// The type of value this port produces
    pub value_type: ValueType,
    /// Cached value
    pub value: Value,
    dirty_flag: DirtyFlag,
}

impl OutputPort {
    /// Create a new output port
    pub fn new(name: &'static str, value_type: ValueType) -> Self {
        Self {
            id: Id::new(),
            name,
            value_type,
            value: value_type.default_value(),
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
}
