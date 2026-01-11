use serde::{Deserialize, Serialize};

use flux_core::id::Id;
use crate::instance_path::InstancePath;

/// Reference to a slot on an operator instance
///
/// This is used to create connections between operators, especially
/// across composite operator boundaries. It identifies:
/// - The path to the operator instance (through nested composites)
/// - The slot index on that operator
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SlotRef {
    /// Path to the operator instance
    pub instance_path: InstancePath,
    /// Index of the slot on the operator
    pub slot_index: usize,
    /// Whether this refers to an input or output slot
    pub is_output: bool,
}

impl SlotRef {
    /// Create a reference to an output slot
    pub fn output(instance_path: InstancePath, slot_index: usize) -> Self {
        Self {
            instance_path,
            slot_index,
            is_output: true,
        }
    }

    /// Create a reference to an input slot
    pub fn input(instance_path: InstancePath, slot_index: usize) -> Self {
        Self {
            instance_path,
            slot_index,
            is_output: false,
        }
    }

    /// Create a simple reference from a node ID and slot index (for flat graphs)
    pub fn simple_output(node_id: Id, slot_index: usize) -> Self {
        Self::output(InstancePath::root(node_id), slot_index)
    }

    /// Create a simple input reference from a node ID and slot index
    pub fn simple_input(node_id: Id, slot_index: usize) -> Self {
        Self::input(InstancePath::root(node_id), slot_index)
    }

    /// Get the immediate node ID (leaf of the instance path)
    pub fn node_id(&self) -> Option<Id> {
        self.instance_path.leaf()
    }

    /// Check if this reference points to a slot in a nested composite
    pub fn is_nested(&self) -> bool {
        self.instance_path.depth() > 1
    }

    /// Get the depth of nesting
    pub fn depth(&self) -> usize {
        self.instance_path.depth()
    }
}

impl std::fmt::Display for SlotRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slot_type = if self.is_output { "out" } else { "in" };
        write!(f, "{}[{}:{}]", self.instance_path, slot_type, self.slot_index)
    }
}

/// A connection between two slots
///
/// Represents an edge in the graph from a source output to a target input.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Connection {
    /// The source output slot
    pub source: SlotRef,
    /// The target input slot
    pub target: SlotRef,
}

impl Connection {
    /// Create a new connection
    pub fn new(source: SlotRef, target: SlotRef) -> Self {
        debug_assert!(source.is_output, "Source must be an output slot");
        debug_assert!(!target.is_output, "Target must be an input slot");
        Self { source, target }
    }

    /// Create a simple connection between two nodes in a flat graph
    pub fn simple(
        source_node: Id,
        source_output: usize,
        target_node: Id,
        target_input: usize,
    ) -> Self {
        Self::new(
            SlotRef::simple_output(source_node, source_output),
            SlotRef::simple_input(target_node, target_input),
        )
    }

    /// Check if this connection crosses composite boundaries
    pub fn is_cross_boundary(&self) -> bool {
        self.source.is_nested() || self.target.is_nested()
    }
}

impl std::fmt::Display for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.source, self.target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_ref_simple() {
        let node_id = Id::new();
        let slot_ref = SlotRef::simple_output(node_id, 0);

        assert!(slot_ref.is_output);
        assert_eq!(slot_ref.slot_index, 0);
        assert_eq!(slot_ref.node_id(), Some(node_id));
        assert!(!slot_ref.is_nested());
    }

    #[test]
    fn test_slot_ref_nested() {
        let parent = Id::new();
        let child = Id::new();
        let path = InstancePath::root(parent).child(child);

        let slot_ref = SlotRef::output(path, 1);

        assert!(slot_ref.is_nested());
        assert_eq!(slot_ref.depth(), 2);
        assert_eq!(slot_ref.node_id(), Some(child));
    }

    #[test]
    fn test_connection() {
        let source = Id::new();
        let target = Id::new();

        let conn = Connection::simple(source, 0, target, 1);

        assert_eq!(conn.source.node_id(), Some(source));
        assert_eq!(conn.target.node_id(), Some(target));
        assert_eq!(conn.source.slot_index, 0);
        assert_eq!(conn.target.slot_index, 1);
    }
}
