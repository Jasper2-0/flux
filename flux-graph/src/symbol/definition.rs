use serde::{Deserialize, Serialize};

use flux_core::id::Id;
use flux_core::value::{Value, ValueType};

/// Definition of an input slot on a Symbol
///
/// This describes the "shape" of an input - its type, name, and default value.
/// Multiple instances can be created from this definition, each with their own
/// runtime state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputDefinition {
    /// Unique identifier for this input
    pub id: Id,
    /// Display name
    pub name: String,
    /// Type of value this input accepts
    pub value_type: ValueType,
    /// Default value when no connection or override is present
    pub default_value: Value,
    /// Whether this input accepts multiple connections
    pub is_multi_input: bool,
    /// Optional description for UI/documentation
    #[serde(default)]
    pub description: Option<String>,
}

impl InputDefinition {
    /// Create a new input definition
    pub fn new(name: impl Into<String>, value_type: ValueType, default_value: Value) -> Self {
        Self {
            id: Id::new(),
            name: name.into(),
            value_type,
            default_value,
            is_multi_input: false,
            description: None,
        }
    }

    /// Create a float input with a default value
    pub fn float(name: impl Into<String>, default: f32) -> Self {
        Self::new(name, ValueType::Float, Value::Float(default))
    }

    /// Create an int input with a default value
    pub fn int(name: impl Into<String>, default: i32) -> Self {
        Self::new(name, ValueType::Int, Value::Int(default))
    }

    /// Create a bool input with a default value
    pub fn bool(name: impl Into<String>, default: bool) -> Self {
        Self::new(name, ValueType::Bool, Value::Bool(default))
    }

    /// Create a Vec3 input with a default value
    pub fn vec3(name: impl Into<String>, default: [f32; 3]) -> Self {
        Self::new(name, ValueType::Vec3, Value::Vec3(default))
    }

    /// Set this input as a multi-input (accepts multiple connections)
    pub fn multi_input(mut self) -> Self {
        self.is_multi_input = true;
        self
    }

    /// Add a description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Definition of an output slot on a Symbol
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputDefinition {
    /// Unique identifier for this output
    pub id: Id,
    /// Display name
    pub name: String,
    /// Type of value this output produces
    pub value_type: ValueType,
    /// What triggers this output to become dirty
    pub dirty_flag_trigger: DirtyFlagTrigger,
    /// Optional description for UI/documentation
    #[serde(default)]
    pub description: Option<String>,
}

impl OutputDefinition {
    /// Create a new output definition
    pub fn new(name: impl Into<String>, value_type: ValueType) -> Self {
        Self {
            id: Id::new(),
            name: name.into(),
            value_type,
            dirty_flag_trigger: DirtyFlagTrigger::default(),
            description: None,
        }
    }

    /// Create a float output
    pub fn float(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Float)
    }

    /// Create an int output
    pub fn int(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Int)
    }

    /// Create a bool output
    pub fn bool(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Bool)
    }

    /// Create a Vec3 output
    pub fn vec3(name: impl Into<String>) -> Self {
        Self::new(name, ValueType::Vec3)
    }

    /// Set the dirty flag trigger
    pub fn with_trigger(mut self, trigger: DirtyFlagTrigger) -> Self {
        self.dirty_flag_trigger = trigger;
        self
    }

    /// Add a description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// What causes an output to become dirty and need recomputation
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirtyFlagTrigger {
    /// Dirty when any input changes
    #[default]
    Animated,
    /// Always recompute (never clean)
    Always,
    /// Only dirty when explicitly marked
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_definition() {
        let input = InputDefinition::float("Amount", 1.0)
            .with_description("The amount to add");

        assert_eq!(input.name, "Amount");
        assert_eq!(input.value_type, ValueType::Float);
        assert_eq!(input.default_value, Value::Float(1.0));
        assert!(!input.is_multi_input);
        assert_eq!(input.description, Some("The amount to add".to_string()));
    }

    #[test]
    fn test_output_definition() {
        let output = OutputDefinition::vec3("Position")
            .with_trigger(DirtyFlagTrigger::Always);

        assert_eq!(output.name, "Position");
        assert_eq!(output.value_type, ValueType::Vec3);
        assert_eq!(output.dirty_flag_trigger, DirtyFlagTrigger::Always);
    }

    #[test]
    fn test_multi_input() {
        let input = InputDefinition::float("Values", 0.0).multi_input();
        assert!(input.is_multi_input);
    }
}
