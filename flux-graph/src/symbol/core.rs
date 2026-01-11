use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{InputDefinition, Instance, OutputDefinition, SymbolChild};
use crate::animation::Animator;
use flux_core::id::Id;
use crate::instance_path::InstancePath;
use crate::slot_ref::Connection;

/// A Symbol is the definition/blueprint of an operator
///
/// Symbols define:
/// - Input and output slot definitions
/// - Nested child operators (SymbolChild)
/// - Connections between children
/// - Animation data
///
/// Multiple Instances can be created from a single Symbol, each with
/// their own runtime state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Symbol {
    /// Unique identifier for this symbol
    pub id: Id,
    /// Display name
    pub name: String,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
    /// Category for UI organization
    #[serde(default)]
    pub category: Option<String>,

    /// Input slot definitions
    pub input_definitions: Vec<InputDefinition>,
    /// Output slot definitions
    pub output_definitions: Vec<OutputDefinition>,

    /// Nested child operators
    pub children: HashMap<Id, SymbolChild>,
    /// Connections between children
    pub connections: Vec<Connection>,

    /// Animation data for this symbol
    #[serde(skip)]
    pub animator: Animator,

    /// Whether this symbol supports bypass
    #[serde(default)]
    pub is_bypassable: bool,
}

impl Symbol {
    /// Create a new symbol with a name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Id::new(),
            name: name.into(),
            description: None,
            category: None,
            input_definitions: Vec::new(),
            output_definitions: Vec::new(),
            children: HashMap::new(),
            connections: Vec::new(),
            animator: Animator::new(),
            is_bypassable: false,
        }
    }

    /// Create a symbol with a specific ID
    pub fn with_id(id: Id, name: impl Into<String>) -> Self {
        let mut symbol = Self::new(name);
        symbol.id = id;
        symbol
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Mark as bypassable
    pub fn bypassable(mut self) -> Self {
        self.is_bypassable = true;
        self
    }

    // ========== Input Management ==========

    /// Add an input definition
    pub fn add_input(&mut self, input: InputDefinition) -> Id {
        let id = input.id;
        self.input_definitions.push(input);
        id
    }

    /// Get an input definition by ID
    pub fn get_input(&self, input_id: Id) -> Option<&InputDefinition> {
        self.input_definitions.iter().find(|i| i.id == input_id)
    }

    /// Get an input definition by name
    pub fn get_input_by_name(&self, name: &str) -> Option<&InputDefinition> {
        self.input_definitions.iter().find(|i| i.name == name)
    }

    /// Get input definition by index
    pub fn get_input_at(&self, index: usize) -> Option<&InputDefinition> {
        self.input_definitions.get(index)
    }

    /// Remove an input definition
    pub fn remove_input(&mut self, input_id: Id) -> Option<InputDefinition> {
        if let Some(idx) = self.input_definitions.iter().position(|i| i.id == input_id) {
            Some(self.input_definitions.remove(idx))
        } else {
            None
        }
    }

    // ========== Output Management ==========

    /// Add an output definition
    pub fn add_output(&mut self, output: OutputDefinition) -> Id {
        let id = output.id;
        self.output_definitions.push(output);
        id
    }

    /// Get an output definition by ID
    pub fn get_output(&self, output_id: Id) -> Option<&OutputDefinition> {
        self.output_definitions.iter().find(|o| o.id == output_id)
    }

    /// Get an output definition by name
    pub fn get_output_by_name(&self, name: &str) -> Option<&OutputDefinition> {
        self.output_definitions.iter().find(|o| o.name == name)
    }

    /// Get output definition by index
    pub fn get_output_at(&self, index: usize) -> Option<&OutputDefinition> {
        self.output_definitions.get(index)
    }

    /// Remove an output definition
    pub fn remove_output(&mut self, output_id: Id) -> Option<OutputDefinition> {
        if let Some(idx) = self.output_definitions.iter().position(|o| o.id == output_id) {
            Some(self.output_definitions.remove(idx))
        } else {
            None
        }
    }

    // ========== Child Management ==========

    /// Add a child operator
    pub fn add_child(&mut self, child: SymbolChild) -> Id {
        let id = child.id;
        self.children.insert(id, child);
        id
    }

    /// Get a child by ID
    pub fn get_child(&self, child_id: Id) -> Option<&SymbolChild> {
        self.children.get(&child_id)
    }

    /// Get a mutable reference to a child
    pub fn get_child_mut(&mut self, child_id: Id) -> Option<&mut SymbolChild> {
        self.children.get_mut(&child_id)
    }

    /// Remove a child
    pub fn remove_child(&mut self, child_id: Id) -> Option<SymbolChild> {
        // Also remove any connections involving this child
        self.connections.retain(|c| {
            c.source.node_id() != Some(child_id) && c.target.node_id() != Some(child_id)
        });
        self.children.remove(&child_id)
    }

    /// Get all child IDs
    pub fn child_ids(&self) -> impl Iterator<Item = Id> + '_ {
        self.children.keys().copied()
    }

    /// Get child count
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    // ========== Connection Management ==========

    /// Add a connection between children
    pub fn add_connection(&mut self, connection: Connection) -> Result<(), SymbolError> {
        // Validate source exists
        let source_id = connection.source.node_id().ok_or(SymbolError::InvalidConnection)?;
        if !self.children.contains_key(&source_id) {
            return Err(SymbolError::ChildNotFound(source_id));
        }

        // Validate target exists
        let target_id = connection.target.node_id().ok_or(SymbolError::InvalidConnection)?;
        if !self.children.contains_key(&target_id) {
            return Err(SymbolError::ChildNotFound(target_id));
        }

        self.connections.push(connection);
        Ok(())
    }

    /// Remove a connection to a specific target
    pub fn remove_connection(&mut self, target_child: Id, target_slot: usize) {
        self.connections.retain(|c| {
            !(c.target.node_id() == Some(target_child) && c.target.slot_index == target_slot)
        });
    }

    /// Remove all connections from a source
    pub fn remove_connections_from(&mut self, source_child: Id) {
        self.connections
            .retain(|c| c.source.node_id() != Some(source_child));
    }

    /// Remove all connections to a target
    pub fn remove_connections_to(&mut self, target_child: Id) {
        self.connections
            .retain(|c| c.target.node_id() != Some(target_child));
    }

    /// Get connections to a specific slot
    pub fn get_connections_to(&self, target_child: Id, target_slot: usize) -> Vec<&Connection> {
        self.connections
            .iter()
            .filter(|c| {
                c.target.node_id() == Some(target_child) && c.target.slot_index == target_slot
            })
            .collect()
    }

    /// Get connections from a specific slot
    pub fn get_connections_from(&self, source_child: Id, source_slot: usize) -> Vec<&Connection> {
        self.connections
            .iter()
            .filter(|c| {
                c.source.node_id() == Some(source_child) && c.source.slot_index == source_slot
            })
            .collect()
    }

    // ========== Instance Creation ==========

    /// Create a runtime instance from this symbol
    pub fn create_instance(&self) -> Instance {
        Instance::from_symbol(self, InstancePath::root(self.id))
    }

    /// Create a runtime instance with a specific parent path
    pub fn create_instance_with_path(&self, parent_path: InstancePath) -> Instance {
        Instance::from_symbol(self, parent_path.child(self.id))
    }
}

/// Errors that can occur when manipulating symbols
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolError {
    /// Child not found
    ChildNotFound(Id),
    /// Invalid connection
    InvalidConnection,
    /// Input not found
    InputNotFound(Id),
    /// Output not found
    OutputNotFound(Id),
}

impl std::fmt::Display for SymbolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolError::ChildNotFound(id) => write!(f, "Child not found: {:?}", id),
            SymbolError::InvalidConnection => write!(f, "Invalid connection"),
            SymbolError::InputNotFound(id) => write!(f, "Input not found: {:?}", id),
            SymbolError::OutputNotFound(id) => write!(f, "Output not found: {:?}", id),
        }
    }
}

impl std::error::Error for SymbolError {}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_symbol_creation() {
        let symbol = Symbol::new("Add")
            .with_description("Adds two values")
            .with_category("Math");

        assert_eq!(symbol.name, "Add");
        assert_eq!(symbol.description, Some("Adds two values".to_string()));
        assert_eq!(symbol.category, Some("Math".to_string()));
    }

    #[test]
    fn test_input_output_management() {
        let mut symbol = Symbol::new("Test");

        let input_id = symbol.add_input(InputDefinition::float("A", 0.0));
        symbol.add_input(InputDefinition::float("B", 0.0));
        symbol.add_output(OutputDefinition::float("Result"));

        assert_eq!(symbol.input_definitions.len(), 2);
        assert_eq!(symbol.output_definitions.len(), 1);

        assert!(symbol.get_input(input_id).is_some());
        assert!(symbol.get_input_by_name("A").is_some());
        assert!(symbol.get_output_by_name("Result").is_some());

        symbol.remove_input(input_id);
        assert_eq!(symbol.input_definitions.len(), 1);
    }

    #[test]
    fn test_child_management() {
        let mut parent = Symbol::new("Parent");
        let child_symbol_id = Id::new();

        let child = SymbolChild::new(Id::new(), child_symbol_id);
        let child_id = parent.add_child(child);

        assert_eq!(parent.child_count(), 1);
        assert!(parent.get_child(child_id).is_some());

        parent.remove_child(child_id);
        assert_eq!(parent.child_count(), 0);
    }
}
