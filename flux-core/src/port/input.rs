//! Input port definitions

use crate::error::{OperatorError, OperatorResult};
use crate::id::Id;
use crate::value::{Color, Gradient, Value, ValueType};

/// An input port that can be connected to an output
#[derive(Clone, Debug)]
pub struct InputPort {
    pub id: Id,
    pub name: &'static str,
    /// The type of value this port accepts
    pub value_type: ValueType,
    /// Default value when not connected
    pub default: Value,
    /// Connected source: (node_id, output_index)
    pub connection: Option<(Id, usize)>,
    /// Whether this is a multi-input port (can accept multiple connections)
    pub is_multi_input: bool,
    /// For multi-input ports: all connections in order
    pub connections: Vec<(Id, usize)>,
}

impl InputPort {
    /// Create a new single-input port
    pub fn new(name: &'static str, default: Value) -> Self {
        let value_type = default.value_type();
        Self {
            id: Id::new(),
            name,
            value_type,
            default,
            connection: None,
            is_multi_input: false,
            connections: Vec::new(),
        }
    }

    /// Create a new multi-input port (can accept multiple connections)
    pub fn new_multi(name: &'static str, value_type: ValueType) -> Self {
        Self {
            id: Id::new(),
            name,
            value_type,
            default: value_type.default_value(),
            connection: None,
            is_multi_input: true,
            connections: Vec::new(),
        }
    }

    /// Convenience constructor for float input
    pub fn float(name: &'static str, default: f32) -> Self {
        Self::new(name, Value::Float(default))
    }

    /// Convenience constructor for int input
    pub fn int(name: &'static str, default: i32) -> Self {
        Self::new(name, Value::Int(default))
    }

    /// Convenience constructor for bool input
    pub fn bool(name: &'static str, default: bool) -> Self {
        Self::new(name, Value::Bool(default))
    }

    /// Convenience constructor for vec3 input
    pub fn vec3(name: &'static str, default: [f32; 3]) -> Self {
        Self::new(name, Value::Vec3(default))
    }

    /// Convenience constructor for multi-input float
    pub fn float_multi(name: &'static str) -> Self {
        Self::new_multi(name, ValueType::Float)
    }

    /// Convenience constructor for vec2 input
    pub fn vec2(name: &'static str, default: [f32; 2]) -> Self {
        Self::new(name, Value::Vec2(default))
    }

    /// Convenience constructor for vec4 input
    pub fn vec4(name: &'static str, default: [f32; 4]) -> Self {
        Self::new(name, Value::Vec4(default))
    }

    /// Convenience constructor for string input
    pub fn string(name: &'static str, default: &str) -> Self {
        Self::new(name, Value::String(default.to_string()))
    }

    /// Convenience constructor for color input
    pub fn color(name: &'static str, default: [f32; 4]) -> Self {
        Self::new(name, Value::Color(Color::rgba(default[0], default[1], default[2], default[3])))
    }

    /// Convenience constructor for gradient input
    pub fn gradient(name: &'static str) -> Self {
        Self::new(name, Value::Gradient(Gradient::new()))
    }

    /// Convenience constructor for float list input
    pub fn float_list(name: &'static str) -> Self {
        Self::new(name, Value::FloatList(Vec::new()))
    }

    /// Convenience constructor for multi-input bool
    pub fn bool_multi(name: &'static str) -> Self {
        Self::new_multi(name, ValueType::Bool)
    }

    /// Create a new input port with explicit type and default
    pub fn new_typed(name: &'static str, value_type: ValueType, default: Value) -> Self {
        Self {
            id: Id::new(),
            name,
            value_type,
            default,
            connection: None,
            is_multi_input: false,
            connections: Vec::new(),
        }
    }

    pub fn is_connected(&self) -> bool {
        if self.is_multi_input {
            !self.connections.is_empty()
        } else {
            self.connection.is_some()
        }
    }

    pub fn connection_count(&self) -> usize {
        if self.is_multi_input {
            self.connections.len()
        } else if self.connection.is_some() {
            1
        } else {
            0
        }
    }

    pub fn connect(&mut self, source_node: Id, output_index: usize) {
        if self.is_multi_input {
            self.connections.push((source_node, output_index));
        } else {
            self.connection = Some((source_node, output_index));
        }
    }

    pub fn disconnect(&mut self) {
        self.connection = None;
        self.connections.clear();
    }

    /// Disconnect a specific connection (for multi-input)
    pub fn disconnect_at(&mut self, index: usize) {
        if self.is_multi_input {
            if index < self.connections.len() {
                self.connections.remove(index);
            }
        } else {
            self.connection = None;
        }
    }

    /// Check if a value can be accepted (with optional coercion)
    pub fn can_accept(&self, value: &Value) -> bool {
        let incoming_type = value.value_type();
        incoming_type == self.value_type || incoming_type.can_coerce_to(self.value_type)
    }

    /// Check if a value type can be accepted (with optional coercion)
    pub fn can_accept_type(&self, value_type: ValueType) -> bool {
        value_type == self.value_type || value_type.can_coerce_to(self.value_type)
    }

    /// Accept a value, coercing if necessary
    ///
    /// Returns the coerced value if coercion was needed, or the original if types match.
    pub fn accept(&self, value: Value) -> OperatorResult<Value> {
        let incoming_type = value.value_type();
        if incoming_type == self.value_type {
            Ok(value)
        } else if let Some(coerced) = value.coerce_to(self.value_type) {
            Ok(coerced)
        } else {
            Err(OperatorError::coercion_failed(incoming_type, self.value_type))
        }
    }

    /// Get the current value (default or from connection), coercing if needed
    pub fn get_value(&self, connected_value: Option<Value>) -> Value {
        match connected_value {
            Some(v) => self.accept(v).unwrap_or_else(|_| self.default.clone()),
            None => self.default.clone(),
        }
    }

    /// Extract a float value from input, with coercion
    pub fn get_float(&self, connected_value: Option<Value>) -> f32 {
        self.get_value(connected_value).as_float().unwrap_or(0.0)
    }

    /// Extract an int value from input, with coercion
    pub fn get_int(&self, connected_value: Option<Value>) -> i32 {
        self.get_value(connected_value).as_int().unwrap_or(0)
    }

    /// Extract a bool value from input, with coercion
    pub fn get_bool(&self, connected_value: Option<Value>) -> bool {
        self.get_value(connected_value).as_bool().unwrap_or(false)
    }

    /// Extract a vec3 value from input, with coercion
    pub fn get_vec3(&self, connected_value: Option<Value>) -> [f32; 3] {
        self.get_value(connected_value)
            .as_vec3()
            .unwrap_or([0.0, 0.0, 0.0])
    }

    /// Extract a vec4 value from input, with coercion
    pub fn get_vec4(&self, connected_value: Option<Value>) -> [f32; 4] {
        self.get_value(connected_value)
            .as_vec4()
            .unwrap_or([0.0, 0.0, 0.0, 0.0])
    }
}
