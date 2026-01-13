//! Input port definitions

use crate::error::{OperatorError, OperatorResult};
use crate::id::Id;
use crate::value::{Color, Gradient, Value, ValueType};

use super::TypeConstraint;

/// An input port that can be connected to an output
#[derive(Clone, Debug)]
pub struct InputPort {
    pub id: Id,
    pub name: &'static str,
    /// The type of value this port accepts (for backward compatibility)
    pub value_type: ValueType,
    /// Type constraint for polymorphic ports
    pub constraint: TypeConstraint,
    /// Default value when not connected
    pub default: Value,
    /// Connected source: (node_id, output_index)
    pub connection: Option<(Id, usize)>,
    /// Whether this is a multi-input port (can accept multiple connections)
    pub is_multi_input: bool,
    /// For multi-input ports: all connections in order
    pub connections: Vec<(Id, usize)>,
    /// Resolved type after connection (for polymorphic ports)
    pub resolved_type: Option<ValueType>,
}

impl InputPort {
    /// Create a new single-input port
    pub fn new(name: &'static str, default: Value) -> Self {
        let value_type = default.value_type();
        Self {
            id: Id::new(),
            name,
            value_type,
            constraint: TypeConstraint::Exact(value_type),
            default,
            connection: None,
            is_multi_input: false,
            connections: Vec::new(),
            resolved_type: None,
        }
    }

    /// Create a new multi-input port (can accept multiple connections)
    pub fn new_multi(name: &'static str, value_type: ValueType) -> Self {
        Self {
            id: Id::new(),
            name,
            value_type,
            constraint: TypeConstraint::Exact(value_type),
            default: value_type.default_value(),
            connection: None,
            is_multi_input: true,
            connections: Vec::new(),
            resolved_type: None,
        }
    }

    /// Create a new polymorphic input port with a type constraint
    pub fn constrained(name: &'static str, constraint: TypeConstraint, default: Value) -> Self {
        let value_type = default.value_type();
        Self {
            id: Id::new(),
            name,
            value_type,
            constraint,
            default,
            connection: None,
            is_multi_input: false,
            connections: Vec::new(),
            resolved_type: None,
        }
    }

    /// Create an arithmetic input (accepts Float, Int, Vec2, Vec3, Vec4, Color)
    pub fn arithmetic(name: &'static str, default: Value) -> Self {
        let value_type = default.value_type();
        Self {
            id: Id::new(),
            name,
            value_type,
            constraint: TypeConstraint::arithmetic(),
            default,
            connection: None,
            is_multi_input: false,
            connections: Vec::new(),
            resolved_type: None,
        }
    }

    /// Create a numeric input (accepts Float, Int)
    pub fn numeric(name: &'static str, default: f32) -> Self {
        Self {
            id: Id::new(),
            name,
            value_type: ValueType::Float,
            constraint: TypeConstraint::numeric(),
            default: Value::Float(default),
            connection: None,
            is_multi_input: false,
            connections: Vec::new(),
            resolved_type: None,
        }
    }

    /// Create a vector input (accepts Vec2, Vec3, Vec4)
    pub fn vector(name: &'static str, default: [f32; 3]) -> Self {
        Self {
            id: Id::new(),
            name,
            value_type: ValueType::Vec3,
            constraint: TypeConstraint::vector(),
            default: Value::Vec3(default),
            connection: None,
            is_multi_input: false,
            connections: Vec::new(),
            resolved_type: None,
        }
    }

    /// Create an any-type input (accepts all types)
    pub fn any(name: &'static str, default: Value) -> Self {
        let value_type = default.value_type();
        Self {
            id: Id::new(),
            name,
            value_type,
            constraint: TypeConstraint::any(),
            default,
            connection: None,
            is_multi_input: false,
            connections: Vec::new(),
            resolved_type: None,
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

    /// Convenience constructor for int list input
    pub fn int_list(name: &'static str) -> Self {
        Self::new(name, Value::IntList(Vec::new()))
    }

    /// Convenience constructor for bool list input
    pub fn bool_list(name: &'static str) -> Self {
        Self::new(name, Value::BoolList(Vec::new()))
    }

    /// Convenience constructor for vec2 list input
    pub fn vec2_list(name: &'static str) -> Self {
        Self::new(name, Value::Vec2List(Vec::new()))
    }

    /// Convenience constructor for vec3 list input
    pub fn vec3_list(name: &'static str) -> Self {
        Self::new(name, Value::Vec3List(Vec::new()))
    }

    /// Convenience constructor for vec4 list input
    pub fn vec4_list(name: &'static str) -> Self {
        Self::new(name, Value::Vec4List(Vec::new()))
    }

    /// Convenience constructor for color list input
    pub fn color_list(name: &'static str) -> Self {
        Self::new(name, Value::ColorList(Vec::new()))
    }

    /// Convenience constructor for string list input
    pub fn string_list(name: &'static str) -> Self {
        Self::new(name, Value::StringList(Vec::new()))
    }

    /// Convenience constructor for multi-input bool
    pub fn bool_multi(name: &'static str) -> Self {
        Self::new_multi(name, ValueType::Bool)
    }

    /// Convenience constructor for multi-input int
    pub fn int_multi(name: &'static str) -> Self {
        Self::new_multi(name, ValueType::Int)
    }

    /// Convenience constructor for multi-input vec2
    pub fn vec2_multi(name: &'static str) -> Self {
        Self::new_multi(name, ValueType::Vec2)
    }

    /// Convenience constructor for multi-input vec3
    pub fn vec3_multi(name: &'static str) -> Self {
        Self::new_multi(name, ValueType::Vec3)
    }

    /// Convenience constructor for multi-input vec4
    pub fn vec4_multi(name: &'static str) -> Self {
        Self::new_multi(name, ValueType::Vec4)
    }

    /// Convenience constructor for multi-input color
    pub fn color_multi(name: &'static str) -> Self {
        Self::new_multi(name, ValueType::Color)
    }

    /// Convenience constructor for multi-input string
    pub fn string_multi(name: &'static str) -> Self {
        Self::new_multi(name, ValueType::String)
    }

    /// Create a new input port with explicit type and default
    pub fn new_typed(name: &'static str, value_type: ValueType, default: Value) -> Self {
        Self {
            id: Id::new(),
            name,
            value_type,
            constraint: TypeConstraint::Exact(value_type),
            default,
            connection: None,
            is_multi_input: false,
            connections: Vec::new(),
            resolved_type: None,
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
    ///
    /// For polymorphic ports, uses the constraint system.
    /// For exact-type ports, falls back to traditional type checking.
    pub fn can_accept(&self, value: &Value) -> bool {
        let incoming_type = value.value_type();
        self.can_accept_type(incoming_type)
    }

    /// Check if a value type can be accepted (with optional coercion)
    ///
    /// For polymorphic ports, checks against the type constraint.
    pub fn can_accept_type(&self, value_type: ValueType) -> bool {
        // Check constraint first
        if self.constraint.accepts(value_type) {
            return true;
        }
        // Fall back to exact type match or coercion
        value_type == self.value_type || value_type.can_coerce_to(self.value_type)
    }

    /// Check if a value type can be accepted with context (for SameAsInput constraints)
    pub fn can_accept_type_with_context(
        &self,
        value_type: ValueType,
        other_input_types: &[Option<ValueType>],
    ) -> bool {
        self.constraint.accepts_with_context(value_type, other_input_types)
    }

    /// Accept a value, coercing if necessary
    ///
    /// For polymorphic ports, accepts the value as-is if it satisfies the constraint.
    /// Returns the coerced value if coercion was needed, or the original if types match.
    pub fn accept(&self, value: Value) -> OperatorResult<Value> {
        let incoming_type = value.value_type();

        // If constraint accepts the type, use it directly
        if self.constraint.accepts(incoming_type) {
            return Ok(value);
        }

        // Try exact match
        if incoming_type == self.value_type {
            return Ok(value);
        }

        // Try coercion
        if let Some(coerced) = value.coerce_to(self.value_type) {
            return Ok(coerced);
        }

        Err(OperatorError::coercion_failed(incoming_type, self.value_type))
    }

    /// Accept a value for polymorphic computation (no coercion, just validation)
    ///
    /// Unlike `accept()`, this returns the original value without coercion,
    /// which is what polymorphic operators need to preserve type information.
    pub fn accept_polymorphic(&self, value: Value) -> OperatorResult<Value> {
        let incoming_type = value.value_type();

        if self.constraint.accepts(incoming_type) {
            Ok(value)
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

    /// Get the value for polymorphic computation (no coercion)
    pub fn get_value_polymorphic(&self, connected_value: Option<Value>) -> Value {
        match connected_value {
            Some(v) => self.accept_polymorphic(v).unwrap_or_else(|_| self.default.clone()),
            None => self.default.clone(),
        }
    }

    /// Update resolved type after connection
    pub fn resolve_type(&mut self, connected_type: ValueType) {
        if self.constraint.accepts(connected_type) {
            self.resolved_type = Some(connected_type);
        }
    }

    /// Clear resolved type (when disconnected)
    pub fn clear_resolved_type(&mut self) {
        self.resolved_type = None;
    }

    /// Get the effective type (resolved or default)
    pub fn effective_type(&self) -> ValueType {
        self.resolved_type.unwrap_or(self.value_type)
    }

    /// Check if this is a polymorphic port
    pub fn is_polymorphic(&self) -> bool {
        !matches!(self.constraint, TypeConstraint::Exact(_))
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
