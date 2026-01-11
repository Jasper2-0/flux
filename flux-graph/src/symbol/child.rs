use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use flux_core::id::Id;
use flux_core::value::Value;

/// A reference to another Symbol nested within a parent Symbol
///
/// SymbolChild represents a "usage" of a symbol within another symbol.
/// It stores per-instance configuration like input overrides and bypass state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolChild {
    /// Unique identifier for this child within the parent
    pub id: Id,
    /// Reference to the Symbol this child instantiates
    pub symbol_id: Id,
    /// Custom display name (overrides symbol name if set)
    pub name: Option<String>,

    /// Per-child input value overrides
    pub inputs: HashMap<Id, ChildInput>,
    /// Per-child output configuration
    pub outputs: HashMap<Id, ChildOutput>,

    /// Whether this child is bypassed (passes input through)
    pub is_bypassed: bool,
    /// Whether this child is disabled (excluded from computation)
    pub is_disabled: bool,

    /// UI position for graph editor (optional)
    pub position: Option<(f32, f32)>,
    /// UI size for graph editor (optional)
    pub size: Option<(f32, f32)>,
}

impl SymbolChild {
    /// Create a new symbol child
    pub fn new(id: Id, symbol_id: Id) -> Self {
        Self {
            id,
            symbol_id,
            name: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            is_bypassed: false,
            is_disabled: false,
            position: None,
            size: None,
        }
    }

    /// Create a child with a custom name
    pub fn named(id: Id, symbol_id: Id, name: impl Into<String>) -> Self {
        let mut child = Self::new(id, symbol_id);
        child.name = Some(name.into());
        child
    }

    /// Set an input value override
    pub fn set_input_value(&mut self, input_id: Id, value: Value) {
        self.inputs.insert(
            input_id,
            ChildInput {
                definition_id: input_id,
                value,
                is_default: false,
            },
        );
    }

    /// Get an input value override
    pub fn get_input_value(&self, input_id: Id) -> Option<&Value> {
        self.inputs.get(&input_id).map(|i| &i.value)
    }

    /// Reset an input to use the symbol's default value
    pub fn reset_input_to_default(&mut self, input_id: Id) {
        if let Some(input) = self.inputs.get_mut(&input_id) {
            input.is_default = true;
        }
    }

    /// Mark the current input value as the new default
    pub fn set_current_as_default(&mut self, input_id: Id) {
        if let Some(input) = self.inputs.get_mut(&input_id) {
            input.is_default = true;
        }
    }

    /// Check if an input has an override value
    pub fn has_input_override(&self, input_id: Id) -> bool {
        self.inputs
            .get(&input_id)
            .map(|i| !i.is_default)
            .unwrap_or(false)
    }

    /// Set bypass state
    pub fn set_bypassed(&mut self, bypassed: bool) {
        self.is_bypassed = bypassed;
    }

    /// Set disabled state
    pub fn set_disabled(&mut self, disabled: bool) {
        self.is_disabled = disabled;
    }

    /// Set UI position
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = Some((x, y));
    }

    /// Get the display name (custom name or symbol name placeholder)
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or("<unnamed>")
    }

    /// Set a custom display name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Clear custom name (will use symbol's name)
    pub fn clear_name(&mut self) {
        self.name = None;
    }

    /// Check if an output is disabled
    pub fn is_output_disabled(&self, output_id: Id) -> bool {
        self.outputs
            .get(&output_id)
            .map(|o| o.is_disabled)
            .unwrap_or(false)
    }

    /// Set output disabled state
    pub fn set_output_disabled(&mut self, output_id: Id, disabled: bool) {
        self.outputs
            .entry(output_id)
            .or_insert_with(|| ChildOutput {
                definition_id: output_id,
                is_disabled: false,
            })
            .is_disabled = disabled;
    }
}

/// Per-child input configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChildInput {
    /// Reference to the input definition
    pub definition_id: Id,
    /// Override value for this child
    pub value: Value,
    /// Whether this uses the symbol's default value
    pub is_default: bool,
}

impl ChildInput {
    /// Create a new child input with an override value
    pub fn new(definition_id: Id, value: Value) -> Self {
        Self {
            definition_id,
            value,
            is_default: false,
        }
    }

    /// Create a child input that uses the default value
    pub fn default_value(definition_id: Id, value: Value) -> Self {
        Self {
            definition_id,
            value,
            is_default: true,
        }
    }
}

/// Per-child output configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChildOutput {
    /// Reference to the output definition
    pub definition_id: Id,
    /// Whether this output is disabled
    pub is_disabled: bool,
}

impl ChildOutput {
    /// Create a new child output configuration
    pub fn new(definition_id: Id) -> Self {
        Self {
            definition_id,
            is_disabled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_child_creation() {
        let child_id = Id::new();
        let symbol_id = Id::new();

        let child = SymbolChild::new(child_id, symbol_id);

        assert_eq!(child.id, child_id);
        assert_eq!(child.symbol_id, symbol_id);
        assert!(child.name.is_none());
        assert!(!child.is_bypassed);
        assert!(!child.is_disabled);
    }

    #[test]
    fn test_input_overrides() {
        let mut child = SymbolChild::new(Id::new(), Id::new());
        let input_id = Id::new();

        // No override initially
        assert!(!child.has_input_override(input_id));

        // Set override
        child.set_input_value(input_id, Value::Float(42.0));
        assert!(child.has_input_override(input_id));
        assert_eq!(child.get_input_value(input_id), Some(&Value::Float(42.0)));

        // Reset to default
        child.reset_input_to_default(input_id);
        assert!(!child.has_input_override(input_id));
    }

    #[test]
    fn test_bypass_and_disable() {
        let mut child = SymbolChild::new(Id::new(), Id::new());

        child.set_bypassed(true);
        assert!(child.is_bypassed);

        child.set_disabled(true);
        assert!(child.is_disabled);
    }

    #[test]
    fn test_display_name() {
        let mut child = SymbolChild::new(Id::new(), Id::new());

        assert_eq!(child.display_name(), "<unnamed>");

        child.set_name("MyNode");
        assert_eq!(child.display_name(), "MyNode");

        child.clear_name();
        assert_eq!(child.display_name(), "<unnamed>");
    }
}
