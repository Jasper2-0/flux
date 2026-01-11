use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::Symbol;
use flux_core::id::Id;
use crate::instance_path::InstancePath;
use flux_core::port::{InputPort, OutputPort};
use flux_core::value::Value;

/// Runtime instance of a Symbol
///
/// An Instance holds the actual runtime state for a Symbol during evaluation.
/// This includes:
/// - Runtime input/output slots with current values
/// - Child instances (lazily created)
/// - Instance status flags
#[derive(Clone, Debug)]
pub struct Instance {
    /// Unique identifier for this instance
    pub id: Id,
    /// Reference to the symbol definition
    pub symbol_id: Id,
    /// Path from root to this instance
    pub instance_path: InstancePath,

    /// Runtime input slots
    pub inputs: Vec<InputPort>,
    /// Runtime output slots
    pub outputs: Vec<OutputPort>,

    /// Child instances (lazily created)
    pub children: InstanceChildren,

    /// Current status flags
    pub status: InstanceStatus,
}

impl Instance {
    /// Create a new instance from a symbol
    pub fn from_symbol(symbol: &Symbol, path: InstancePath) -> Self {
        // Create runtime input slots from definitions
        // Note: We leak the name strings to get &'static str.
        // This is acceptable since symbols are typically long-lived.
        let inputs: Vec<InputPort> = symbol
            .input_definitions
            .iter()
            .map(|def| {
                let name: &'static str = Box::leak(def.name.clone().into_boxed_str());
                let mut slot = InputPort::new(name, def.default_value.clone());
                slot.value_type = def.value_type;
                slot.is_multi_input = def.is_multi_input;
                slot
            })
            .collect();

        // Create runtime output slots from definitions
        let outputs: Vec<OutputPort> = symbol
            .output_definitions
            .iter()
            .map(|def| {
                let name: &'static str = Box::leak(def.name.clone().into_boxed_str());
                OutputPort::new_typed(name, def.value_type)
            })
            .collect();

        Self {
            id: Id::new(),
            symbol_id: symbol.id,
            instance_path: path,
            inputs,
            outputs,
            children: InstanceChildren::new(symbol.id),
            status: InstanceStatus::UNINITIALIZED,
        }
    }

    /// Initialize the instance
    pub fn initialize(&mut self) {
        self.status.insert(InstanceStatus::INITIALIZED);
        self.status.remove(InstanceStatus::UNINITIALIZED);
    }

    /// Dispose the instance and release resources
    pub fn dispose(&mut self) {
        self.status.insert(InstanceStatus::DISPOSED);
        self.status.remove(InstanceStatus::ACTIVE);
        self.children.dispose_all();
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.status.contains(InstanceStatus::INITIALIZED)
    }

    /// Check if disposed
    pub fn is_disposed(&self) -> bool {
        self.status.contains(InstanceStatus::DISPOSED)
    }

    /// Check if active
    pub fn is_active(&self) -> bool {
        self.status.contains(InstanceStatus::ACTIVE)
    }

    /// Check if bypassed
    pub fn is_bypassed(&self) -> bool {
        self.status.contains(InstanceStatus::BYPASSED)
    }

    /// Set bypass state
    pub fn set_bypass(&mut self, bypassed: bool) {
        if bypassed {
            self.status.insert(InstanceStatus::BYPASSED);
        } else {
            self.status.remove(InstanceStatus::BYPASSED);
        }
    }

    /// Set active state
    pub fn set_active(&mut self, active: bool) {
        if active {
            self.status.insert(InstanceStatus::ACTIVE);
        } else {
            self.status.remove(InstanceStatus::ACTIVE);
        }
    }

    /// Mark that reconnection is needed
    pub fn mark_needs_reconnection(&mut self) {
        self.status.insert(InstanceStatus::IS_RECONNECTING);
    }

    /// Check if reconnection is needed
    pub fn needs_reconnection(&self) -> bool {
        self.status.contains(InstanceStatus::IS_RECONNECTING)
    }

    /// Clear reconnection flag
    pub fn clear_reconnection_flag(&mut self) {
        self.status.remove(InstanceStatus::IS_RECONNECTING);
    }

    /// Get an input slot by index
    pub fn get_input(&self, index: usize) -> Option<&InputPort> {
        self.inputs.get(index)
    }

    /// Get a mutable input slot by index
    pub fn get_input_mut(&mut self, index: usize) -> Option<&mut InputPort> {
        self.inputs.get_mut(index)
    }

    /// Get an output slot by index
    pub fn get_output(&self, index: usize) -> Option<&OutputPort> {
        self.outputs.get(index)
    }

    /// Get a mutable output slot by index
    pub fn get_output_mut(&mut self, index: usize) -> Option<&mut OutputPort> {
        self.outputs.get_mut(index)
    }

    /// Get output value by index
    pub fn get_output_value(&self, index: usize) -> Option<Value> {
        self.outputs.get(index).map(|o| o.get())
    }

    /// Set output value by index
    pub fn set_output_value(&mut self, index: usize, value: Value) {
        if let Some(output) = self.outputs.get_mut(index) {
            output.set(value);
        }
    }
}

/// Status flags for an instance
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceStatus(u8);

impl InstanceStatus {
    pub const UNINITIALIZED: Self = Self(0);
    pub const INITIALIZED: Self = Self(1 << 0);
    pub const ACTIVE: Self = Self(1 << 1);
    pub const CONNECTED_INTERNALLY: Self = Self(1 << 2);
    pub const IS_RECONNECTING: Self = Self(1 << 3);
    pub const RESOURCE_FOLDERS_DIRTY: Self = Self(1 << 4);
    pub const BYPASSED: Self = Self(1 << 5);
    pub const DISPOSED: Self = Self(1 << 6);

    /// Check if status contains a flag
    pub fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) == flag.0
    }

    /// Insert a flag
    pub fn insert(&mut self, flag: Self) {
        self.0 |= flag.0;
    }

    /// Remove a flag
    pub fn remove(&mut self, flag: Self) {
        self.0 &= !flag.0;
    }

    /// Check if empty (uninitialized)
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// Lazy-loading collection of child instances
#[derive(Clone, Debug, Default)]
pub struct InstanceChildren {
    /// Reference to parent symbol
    #[allow(dead_code)]
    symbol_id: Id,
    /// Created child instances (keyed by SymbolChild ID)
    instances: HashMap<Id, Instance>,
    /// Which children have been instantiated
    instantiated: HashMap<Id, bool>,
}

impl InstanceChildren {
    /// Create a new children collection
    pub fn new(symbol_id: Id) -> Self {
        Self {
            symbol_id,
            instances: HashMap::new(),
            instantiated: HashMap::new(),
        }
    }

    /// Check if a child has been instantiated
    pub fn is_instantiated(&self, child_id: Id) -> bool {
        self.instantiated.get(&child_id).copied().unwrap_or(false)
    }

    /// Get a child instance (if instantiated)
    pub fn get(&self, child_id: Id) -> Option<&Instance> {
        self.instances.get(&child_id)
    }

    /// Get a mutable child instance (if instantiated)
    pub fn get_mut(&mut self, child_id: Id) -> Option<&mut Instance> {
        self.instances.get_mut(&child_id)
    }

    /// Insert or update a child instance
    pub fn insert(&mut self, child_id: Id, instance: Instance) {
        self.instances.insert(child_id, instance);
        self.instantiated.insert(child_id, true);
    }

    /// Remove a child instance
    pub fn remove(&mut self, child_id: Id) -> Option<Instance> {
        self.instantiated.remove(&child_id);
        self.instances.remove(&child_id)
    }

    /// Get all instantiated child IDs
    pub fn child_ids(&self) -> impl Iterator<Item = Id> + '_ {
        self.instances.keys().copied()
    }

    /// Get all child instances
    pub fn iter(&self) -> impl Iterator<Item = (&Id, &Instance)> {
        self.instances.iter()
    }

    /// Get all child instances mutably
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Id, &mut Instance)> {
        self.instances.iter_mut()
    }

    /// Get the number of instantiated children
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// Dispose all child instances
    pub fn dispose_all(&mut self) {
        for (_, instance) in self.instances.iter_mut() {
            instance.dispose();
        }
        self.instances.clear();
        self.instantiated.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::{InputDefinition, OutputDefinition};
    

    fn make_test_symbol() -> Symbol {
        let mut symbol = Symbol::new("TestOp");
        symbol.add_input(InputDefinition::float("A", 1.0));
        symbol.add_input(InputDefinition::float("B", 2.0));
        symbol.add_output(OutputDefinition::float("Result"));
        symbol
    }

    #[test]
    fn test_instance_from_symbol() {
        let symbol = make_test_symbol();
        let instance = symbol.create_instance();

        assert_eq!(instance.symbol_id, symbol.id);
        assert_eq!(instance.inputs.len(), 2);
        assert_eq!(instance.outputs.len(), 1);
        assert!(!instance.is_initialized());
    }

    #[test]
    fn test_instance_lifecycle() {
        let symbol = make_test_symbol();
        let mut instance = symbol.create_instance();

        assert!(!instance.is_initialized());
        assert!(!instance.is_disposed());

        instance.initialize();
        assert!(instance.is_initialized());

        instance.set_active(true);
        assert!(instance.is_active());

        instance.dispose();
        assert!(instance.is_disposed());
        assert!(!instance.is_active());
    }

    #[test]
    fn test_instance_bypass() {
        let symbol = make_test_symbol();
        let mut instance = symbol.create_instance();

        assert!(!instance.is_bypassed());

        instance.set_bypass(true);
        assert!(instance.is_bypassed());

        instance.set_bypass(false);
        assert!(!instance.is_bypassed());
    }

    #[test]
    fn test_instance_status_flags() {
        let mut status = InstanceStatus::UNINITIALIZED;

        status.insert(InstanceStatus::INITIALIZED);
        assert!(status.contains(InstanceStatus::INITIALIZED));

        status.insert(InstanceStatus::ACTIVE);
        assert!(status.contains(InstanceStatus::INITIALIZED));
        assert!(status.contains(InstanceStatus::ACTIVE));

        status.remove(InstanceStatus::ACTIVE);
        assert!(status.contains(InstanceStatus::INITIALIZED));
        assert!(!status.contains(InstanceStatus::ACTIVE));
    }

    #[test]
    fn test_instance_children() {
        let mut children = InstanceChildren::new(Id::new());

        let child_id = Id::new();
        let symbol = make_test_symbol();
        let instance = symbol.create_instance();

        children.insert(child_id, instance);

        assert!(children.is_instantiated(child_id));
        assert!(children.get(child_id).is_some());
        assert_eq!(children.len(), 1);

        children.remove(child_id);
        assert!(!children.is_instantiated(child_id));
        assert!(children.is_empty());
    }
}
