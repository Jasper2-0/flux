//! Associated Graph - A graph wrapper that maintains external ID associations.
//!
//! This module provides [`AssociatedGraph<E>`], which wraps a [`Graph`] and maintains
//! bidirectional mappings between flux IDs and external system IDs (like nodal's NodeId).
//!
//! # Purpose
//!
//! When integrating flux with visual editors or other systems, you often need to track
//! which external ID corresponds to which flux operator. `AssociatedGraph` eliminates
//! the need for manual HashMap management by providing a unified API.
//!
//! # Example
//!
//! ```ignore
//! use flux_graph::{AssociatedGraph, NodeHandle};
//! use flux_operators::AddOp;
//!
//! // E could be nodal::NodeId or any Copy + Eq + Hash type
//! let mut graph: AssociatedGraph<u32> = AssociatedGraph::new();
//!
//! // Add with external ID association
//! let handle = graph.add_with_external(AddOp::new(), 42u32);
//!
//! // Access by either ID
//! let op_by_flux = graph.get(handle.flux_id);
//! let op_by_ext = graph.get_by_external(42u32);
//!
//! // Remove by external ID
//! graph.remove_by_external(42u32);
//! ```

use std::collections::HashMap;
use std::hash::Hash;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::Operator;
use flux_core::value::Value;

use crate::graph::{Connection, Graph, GraphError, GraphEvent, GraphStats};

/// A handle that combines both flux and external IDs.
///
/// Returned when adding operators with external associations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeHandle<E> {
    /// Internal flux ID for graph operations.
    pub flux_id: Id,
    /// External system ID (e.g., nodal::NodeId).
    pub external_id: E,
}

impl<E> NodeHandle<E> {
    /// Create a new node handle.
    pub fn new(flux_id: Id, external_id: E) -> Self {
        Self {
            flux_id,
            external_id,
        }
    }
}

/// A graph wrapper that maintains bidirectional external ID associations.
///
/// This eliminates the need for manual HashMap management when integrating
/// flux with visual editors or other external systems.
///
/// # Type Parameter
///
/// - `E`: The external ID type. Must be `Copy + Eq + Hash`.
///   Common choices include `usize`, `u32`, or `nodal::NodeId`.
pub struct AssociatedGraph<E: Copy + Eq + Hash> {
    /// The underlying flux graph.
    inner: Graph,
    /// External ID → Flux ID mapping.
    external_to_flux: HashMap<E, Id>,
    /// Flux ID → External ID mapping.
    flux_to_external: HashMap<Id, E>,
}

impl<E: Copy + Eq + Hash> AssociatedGraph<E> {
    /// Create a new empty associated graph.
    pub fn new() -> Self {
        Self {
            inner: Graph::new(),
            external_to_flux: HashMap::new(),
            flux_to_external: HashMap::new(),
        }
    }

    // =========================================================================
    // Node Operations with External ID
    // =========================================================================

    /// Add an operator with an associated external ID.
    ///
    /// Returns a [`NodeHandle`] containing both the flux ID and external ID.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let handle = graph.add_with_external(AddOp::new(), nodal_node_id);
    /// // handle.flux_id - use for graph operations
    /// // handle.external_id - use for visual editor operations
    /// ```
    pub fn add_with_external<O: Operator + 'static>(
        &mut self,
        op: O,
        external_id: E,
    ) -> NodeHandle<E> {
        self.add_boxed_with_external(Box::new(op), external_id)
    }

    /// Add a pre-boxed operator with an external ID association.
    ///
    /// Use this when you have a `Box<dyn Operator>` from a factory.
    pub fn add_boxed_with_external(
        &mut self,
        op: Box<dyn Operator>,
        external_id: E,
    ) -> NodeHandle<E> {
        let flux_id = self.inner.add_boxed(op);
        self.external_to_flux.insert(external_id, flux_id);
        self.flux_to_external.insert(flux_id, external_id);
        NodeHandle::new(flux_id, external_id)
    }

    /// Get an operator by external ID.
    pub fn get_by_external(&self, external_id: E) -> Option<&dyn Operator> {
        let flux_id = self.external_to_flux.get(&external_id)?;
        self.inner.get(*flux_id)
    }

    /// Get a mutable operator by external ID.
    pub fn get_mut_by_external(&mut self, external_id: E) -> Option<&mut (dyn Operator + '_)> {
        let flux_id = *self.external_to_flux.get(&external_id)?;
        self.inner.get_mut(flux_id)
    }

    /// Get a typed mutable operator by external ID.
    pub fn get_mut_as_by_external<O: 'static>(&mut self, external_id: E) -> Option<&mut O> {
        let flux_id = *self.external_to_flux.get(&external_id)?;
        self.inner.get_mut_as::<O>(flux_id)
    }

    /// Remove an operator by external ID.
    ///
    /// Returns the removed operator if found.
    pub fn remove_by_external(&mut self, external_id: E) -> Option<Box<dyn Operator>> {
        let flux_id = self.external_to_flux.remove(&external_id)?;
        self.flux_to_external.remove(&flux_id);
        self.inner.remove(flux_id)
    }

    /// Get the flux ID for an external ID.
    pub fn flux_id_for(&self, external_id: E) -> Option<Id> {
        self.external_to_flux.get(&external_id).copied()
    }

    /// Get the external ID for a flux ID.
    pub fn external_id_for(&self, flux_id: Id) -> Option<E> {
        self.flux_to_external.get(&flux_id).copied()
    }

    // =========================================================================
    // Direct Graph Operations (delegated to inner)
    // =========================================================================

    /// Get an operator by flux ID.
    pub fn get(&self, id: Id) -> Option<&dyn Operator> {
        self.inner.get(id)
    }

    /// Get a mutable operator by flux ID.
    pub fn get_mut(&mut self, id: Id) -> Option<&mut (dyn Operator + '_)> {
        self.inner.get_mut(id)
    }

    /// Get a typed mutable operator by flux ID.
    pub fn get_mut_as<O: 'static>(&mut self, id: Id) -> Option<&mut O> {
        self.inner.get_mut_as::<O>(id)
    }

    /// Remove an operator by flux ID.
    ///
    /// Also removes the external ID association.
    pub fn remove(&mut self, id: Id) -> Option<Box<dyn Operator>> {
        if let Some(external_id) = self.flux_to_external.remove(&id) {
            self.external_to_flux.remove(&external_id);
        }
        self.inner.remove(id)
    }

    /// Get the name of a node.
    pub fn node_name(&self, id: Id) -> Option<&'static str> {
        self.inner.node_name(id)
    }

    /// Returns the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    /// Returns an iterator over all flux node IDs.
    pub fn node_ids(&self) -> impl Iterator<Item = Id> + '_ {
        self.inner.node_ids()
    }

    /// Returns an iterator over all node handles.
    pub fn handles(&self) -> impl Iterator<Item = NodeHandle<E>> + '_ {
        self.flux_to_external
            .iter()
            .map(|(&flux_id, &external_id)| NodeHandle::new(flux_id, external_id))
    }

    // =========================================================================
    // Connection Operations
    // =========================================================================

    /// Connect two nodes using flux IDs.
    ///
    /// Returns `Ok(Some(id))` if a conversion node was auto-inserted, `Ok(None)` otherwise.
    pub fn connect(
        &mut self,
        source_node: Id,
        source_output: usize,
        target_node: Id,
        target_input: usize,
    ) -> Result<Option<Id>, GraphError> {
        self.inner
            .connect(source_node, source_output, target_node, target_input)
    }

    /// Connect two nodes using external IDs.
    ///
    /// Returns `Ok(Some(id))` if a conversion node was auto-inserted, `Ok(None)` otherwise.
    pub fn connect_by_external(
        &mut self,
        source_external: E,
        source_output: usize,
        target_external: E,
        target_input: usize,
    ) -> Result<Option<Id>, GraphError> {
        let source_id = self
            .external_to_flux
            .get(&source_external)
            .copied()
            .ok_or(GraphError::NodeNotFound {
                id: Id::NIL,
                name: None,
            })?;
        let target_id = self
            .external_to_flux
            .get(&target_external)
            .copied()
            .ok_or(GraphError::NodeNotFound {
                id: Id::NIL,
                name: None,
            })?;
        self.inner
            .connect(source_id, source_output, target_id, target_input)
    }

    /// Disconnect an input by flux ID.
    pub fn disconnect(&mut self, target_node: Id, target_input: usize) -> Result<(), GraphError> {
        self.inner.disconnect(target_node, target_input)
    }

    /// Disconnect an input by external ID.
    pub fn disconnect_by_external(
        &mut self,
        target_external: E,
        target_input: usize,
    ) -> Result<(), GraphError> {
        let target_id = self
            .external_to_flux
            .get(&target_external)
            .copied()
            .ok_or(GraphError::NodeNotFound {
                id: Id::NIL,
                name: None,
            })?;
        self.inner.disconnect(target_id, target_input)
    }

    /// Set the default value for an input port.
    pub fn set_input_default(&mut self, node_id: Id, input_index: usize, value: Value) -> bool {
        self.inner.set_input_default(node_id, input_index, value)
    }

    /// Set the default value for an input port by external ID.
    pub fn set_input_default_by_external(
        &mut self,
        external_id: E,
        input_index: usize,
        value: Value,
    ) -> bool {
        if let Some(&flux_id) = self.external_to_flux.get(&external_id) {
            self.inner.set_input_default(flux_id, input_index, value)
        } else {
            false
        }
    }

    // =========================================================================
    // Port Override Operations
    // =========================================================================

    /// Get the override for an input port.
    pub fn get_input_override(
        &self,
        node_id: Id,
        input_index: usize,
    ) -> Option<&flux_core::PortOverride> {
        self.inner.get_input_override(node_id, input_index)
    }

    /// Set an override for an input port.
    pub fn set_input_override(
        &mut self,
        node_id: Id,
        input_index: usize,
        override_: flux_core::PortOverride,
    ) {
        self.inner.set_input_override(node_id, input_index, override_)
    }

    /// Clear an override for an input port.
    pub fn clear_input_override(&mut self, node_id: Id, input_index: usize) {
        self.inner.clear_input_override(node_id, input_index)
    }

    // =========================================================================
    // Query Operations
    // =========================================================================

    /// Iterate over all connections.
    pub fn connections(&self) -> impl Iterator<Item = Connection> + '_ {
        self.inner.connections()
    }

    /// Get downstream connections from a node.
    pub fn downstream_of(&self, id: Id) -> Vec<Connection> {
        self.inner.downstream_of(id)
    }

    /// Get upstream connections to a node.
    pub fn upstream_of(&self, id: Id) -> Vec<Connection> {
        self.inner.upstream_of(id)
    }

    /// Get graph statistics.
    pub fn stats(&self) -> GraphStats {
        self.inner.stats()
    }

    // =========================================================================
    // Evaluation
    // =========================================================================

    /// Evaluate the graph and return output value.
    pub fn evaluate(
        &mut self,
        output_node: Id,
        output_index: usize,
        ctx: &EvalContext,
    ) -> Result<Value, GraphError> {
        self.inner.evaluate(output_node, output_index, ctx)
    }

    /// Evaluate the graph by external ID.
    pub fn evaluate_by_external(
        &mut self,
        output_external: E,
        output_index: usize,
        ctx: &EvalContext,
    ) -> Result<Value, GraphError> {
        let output_id = self
            .external_to_flux
            .get(&output_external)
            .copied()
            .ok_or(GraphError::NodeNotFound {
                id: Id::NIL,
                name: None,
            })?;
        self.inner.evaluate(output_id, output_index, ctx)
    }

    // =========================================================================
    // Event System
    // =========================================================================

    /// Drain all pending events.
    pub fn drain_events(&mut self) -> impl Iterator<Item = GraphEvent> + '_ {
        self.inner.drain_events()
    }

    /// Check if there are pending events.
    pub fn has_pending_events(&self) -> bool {
        self.inner.has_pending_events()
    }

    /// Get the number of pending events.
    pub fn pending_event_count(&self) -> usize {
        self.inner.pending_event_count()
    }

    /// Clear all pending events.
    pub fn clear_events(&mut self) {
        self.inner.clear_events();
    }

    // =========================================================================
    // Direct Access
    // =========================================================================

    /// Get a reference to the underlying graph.
    pub fn inner(&self) -> &Graph {
        &self.inner
    }

    /// Get a mutable reference to the underlying graph.
    ///
    /// **Warning**: Direct modifications may desync the external ID mappings.
    /// Prefer using the `AssociatedGraph` methods instead.
    pub fn inner_mut(&mut self) -> &mut Graph {
        &mut self.inner
    }
}

impl<E: Copy + Eq + Hash> Default for AssociatedGraph<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::{InputPort, Operator, OutputPort, ValueType};

    /// Simple test operator
    struct TestOp {
        id: Id,
        inputs: Vec<InputPort>,
        outputs: Vec<OutputPort>,
    }

    impl TestOp {
        fn new() -> Self {
            Self {
                id: Id::new(),
                inputs: vec![InputPort::new("in", Value::Float(0.0))],
                outputs: vec![OutputPort::new("out", ValueType::Float)],
            }
        }

        fn source() -> Self {
            Self {
                id: Id::new(),
                inputs: vec![],
                outputs: vec![OutputPort::new("out", ValueType::Float)],
            }
        }
    }

    impl Operator for TestOp {
        fn id(&self) -> Id {
            self.id
        }
        fn name(&self) -> &'static str {
            "Test"
        }
        fn inputs(&self) -> &[InputPort] {
            &self.inputs
        }
        fn inputs_mut(&mut self) -> &mut [InputPort] {
            &mut self.inputs
        }
        fn outputs(&self) -> &[OutputPort] {
            &self.outputs
        }
        fn outputs_mut(&mut self) -> &mut [OutputPort] {
            &mut self.outputs
        }
        fn compute(&mut self, _ctx: &EvalContext, _get_input: &dyn Fn(Id, usize) -> Value) {}
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_add_with_external() {
        let mut graph: AssociatedGraph<u32> = AssociatedGraph::new();

        let handle = graph.add_with_external(TestOp::source(), 42);

        assert_eq!(handle.external_id, 42);
        assert!(graph.get(handle.flux_id).is_some());
        assert!(graph.get_by_external(42).is_some());
    }

    #[test]
    fn test_id_lookups() {
        let mut graph: AssociatedGraph<u32> = AssociatedGraph::new();

        let handle = graph.add_with_external(TestOp::source(), 100);

        assert_eq!(graph.flux_id_for(100), Some(handle.flux_id));
        assert_eq!(graph.external_id_for(handle.flux_id), Some(100));

        // Non-existent IDs
        assert_eq!(graph.flux_id_for(999), None);
    }

    #[test]
    fn test_remove_by_external() {
        let mut graph: AssociatedGraph<u32> = AssociatedGraph::new();

        let handle = graph.add_with_external(TestOp::source(), 42);

        // Remove by external ID
        let removed = graph.remove_by_external(42);
        assert!(removed.is_some());

        // Both mappings should be gone
        assert!(graph.get_by_external(42).is_none());
        assert!(graph.get(handle.flux_id).is_none());
        assert_eq!(graph.flux_id_for(42), None);
        assert_eq!(graph.external_id_for(handle.flux_id), None);
    }

    #[test]
    fn test_remove_by_flux_id() {
        let mut graph: AssociatedGraph<u32> = AssociatedGraph::new();

        let handle = graph.add_with_external(TestOp::source(), 42);

        // Remove by flux ID
        let removed = graph.remove(handle.flux_id);
        assert!(removed.is_some());

        // Both mappings should be gone
        assert!(graph.get_by_external(42).is_none());
        assert_eq!(graph.flux_id_for(42), None);
    }

    #[test]
    fn test_connect_by_external() {
        let mut graph: AssociatedGraph<u32> = AssociatedGraph::new();

        let source = graph.add_with_external(TestOp::source(), 1);
        let target = graph.add_with_external(TestOp::new(), 2);

        // Connect by external IDs
        graph.connect_by_external(1, 0, 2, 0).unwrap();

        // Verify connection exists
        let connections: Vec<_> = graph.connections().collect();
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].source_node, source.flux_id);
        assert_eq!(connections[0].target_node, target.flux_id);
    }

    #[test]
    fn test_handles_iterator() {
        let mut graph: AssociatedGraph<u32> = AssociatedGraph::new();

        let h1 = graph.add_with_external(TestOp::source(), 10);
        let h2 = graph.add_with_external(TestOp::source(), 20);
        let h3 = graph.add_with_external(TestOp::source(), 30);

        let handles: Vec<_> = graph.handles().collect();
        assert_eq!(handles.len(), 3);

        // All handles should be present
        assert!(handles.contains(&h1));
        assert!(handles.contains(&h2));
        assert!(handles.contains(&h3));
    }

    #[test]
    fn test_events_propagate() {
        let mut graph: AssociatedGraph<u32> = AssociatedGraph::new();

        let handle = graph.add_with_external(TestOp::source(), 42);

        assert!(graph.has_pending_events());
        let events: Vec<_> = graph.drain_events().collect();
        assert_eq!(events.len(), 1);
        match &events[0] {
            GraphEvent::NodeAdded { id } => assert_eq!(*id, handle.flux_id),
            _ => panic!("Expected NodeAdded event"),
        }
    }
}
