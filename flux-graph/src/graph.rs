use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::conversion::ConversionOp;
use flux_core::context::{CallContext, EvalContext};
use flux_core::id::Id;
use flux_core::operator::Operator;
use flux_core::operator_meta::{EffectivePortMeta, PortOverride};
use flux_core::value::{Value, ValueType};

/// Cache key combining node ID and call context for context-aware caching.
///
/// This ensures that the same operator evaluated in different subroutine calls
/// or loop iterations gets separate cache entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CacheKey {
    node_id: Id,
    call_context: CallContext,
}

/// A node in the graph (wraps an operator)
pub(crate) struct Node {
    pub(crate) operator: Box<dyn Operator>,
    /// Per-instance overrides for input port UI behavior.
    /// Sparse storage - only extends to highest overridden index.
    input_overrides: Vec<Option<PortOverride>>,
}

/// Events emitted by the graph when its structure changes.
///
/// These events enable reactive synchronization with visual layers (like nodal)
/// without requiring the integration layer to poll for changes.
///
/// # Example
///
/// ```ignore
/// // Process events after graph operations
/// for event in graph.drain_events() {
///     match event {
///         GraphEvent::NodeAdded { id } => {
///             // Create visual node
///         }
///         GraphEvent::Connected { source, target, .. } => {
///             // Create visual link
///         }
///         GraphEvent::ConversionInserted { conversion_node, .. } => {
///             // Handle auto-inserted conversion node (may want to hide in UI)
///         }
///         _ => {}
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum GraphEvent {
    /// A node was added to the graph.
    NodeAdded { id: Id },
    /// A node was removed from the graph.
    NodeRemoved { id: Id },
    /// A connection was created between two nodes.
    Connected {
        source: Id,
        source_output: usize,
        target: Id,
        target_input: usize,
    },
    /// A connection was removed.
    Disconnected { target: Id, target_input: usize },
    /// An input's default value was changed.
    InputDefaultChanged {
        node: Id,
        input: usize,
        value: Value,
    },
    /// The evaluation order was recomputed.
    OrderRecomputed,
    /// A conversion node was auto-inserted to bridge incompatible types.
    ///
    /// This event is emitted when `connect()` detects that the source and target
    /// types differ but can be coerced. A ConversionOp is automatically inserted
    /// between them to make the conversion explicit.
    ConversionInserted {
        /// The auto-generated conversion node
        conversion_node: Id,
        /// The source type being converted from
        source_type: ValueType,
        /// The target type being converted to
        target_type: ValueType,
    },
    /// A trigger connection was created between two nodes.
    TriggerConnected {
        source: Id,
        source_output: usize,
        target: Id,
        target_input: usize,
    },
    /// A trigger connection was removed.
    TriggerDisconnected {
        source: Id,
        source_output: usize,
        target: Id,
        target_input: usize,
    },
}

/// The operator graph
pub struct Graph {
    pub(crate) nodes: HashMap<Id, Node>,
    /// Topological order for evaluation (computed on demand)
    pub(crate) eval_order: Vec<Id>,
    /// Whether the evaluation order needs recomputation
    order_dirty: bool,
    /// Cache of output values (CacheKey -> Vec<Arc<Value>>)
    ///
    /// The cache key includes both node ID and call context, ensuring that
    /// the same operator in different subroutine calls or loop iterations
    /// gets separate cache entries.
    ///
    /// Values are wrapped in `Arc` to enable reference stealing: when an
    /// operator is the sole consumer of a value (refcount == 1), we can
    /// pass ownership instead of cloning, avoiding unnecessary allocations.
    value_cache: HashMap<CacheKey, Vec<Arc<Value>>>,
    /// Pending events since last drain
    pending_events: Vec<GraphEvent>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            eval_order: Vec::new(),
            order_dirty: true,
            value_cache: HashMap::new(),
            pending_events: Vec::new(),
        }
    }

    // =========================================================================
    // Cache Management
    // =========================================================================

    /// Invalidate all cached values for a specific node (all call contexts).
    ///
    /// This is called when a node's structure changes (connections, defaults)
    /// to ensure stale cached values are not used.
    fn invalidate_cache_for_node(&mut self, node_id: Id) {
        self.value_cache.retain(|key, _| key.node_id != node_id);
    }

    /// Clear the entire value cache (all nodes, all contexts).
    pub fn clear_cache(&mut self) {
        self.value_cache.clear();
    }

    // =========================================================================
    // Event System
    // =========================================================================

    /// Drain all pending events since the last call.
    ///
    /// Events are accumulated during graph operations (add, remove, connect, etc.)
    /// and can be processed by calling this method.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Perform graph operations
    /// graph.add(my_operator);
    /// graph.connect(a, 0, b, 0)?;
    ///
    /// // Process events
    /// for event in graph.drain_events() {
    ///     match event {
    ///         GraphEvent::NodeAdded { id } => println!("Added node {:?}", id),
    ///         GraphEvent::Connected { source, target, .. } => {
    ///             println!("Connected {:?} -> {:?}", source, target)
    ///         }
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub fn drain_events(&mut self) -> impl Iterator<Item = GraphEvent> + '_ {
        self.pending_events.drain(..)
    }

    /// Check if there are any pending events.
    pub fn has_pending_events(&self) -> bool {
        !self.pending_events.is_empty()
    }

    /// Get the number of pending events.
    pub fn pending_event_count(&self) -> usize {
        self.pending_events.len()
    }

    /// Clear all pending events without processing them.
    pub fn clear_events(&mut self) {
        self.pending_events.clear();
    }

    /// Push an event to the pending queue.
    fn emit(&mut self, event: GraphEvent) {
        self.pending_events.push(event);
    }

    // =========================================================================
    // Node Operations
    // =========================================================================

    /// Add an operator to the graph, returns its ID
    pub fn add<O: Operator + 'static>(&mut self, op: O) -> Id {
        self.add_boxed(Box::new(op))
    }

    /// Add a pre-boxed operator to the graph, returns its ID
    pub fn add_boxed(&mut self, op: Box<dyn Operator>) -> Id {
        let id = op.id();
        self.nodes.insert(
            id,
            Node {
                operator: op,
                input_overrides: Vec::new(),
            },
        );
        self.order_dirty = true;
        self.emit(GraphEvent::NodeAdded { id });
        id
    }

    /// Get a reference to an operator by ID
    pub fn get(&self, id: Id) -> Option<&dyn Operator> {
        self.nodes.get(&id).map(|n| n.operator.as_ref())
    }

    /// Get a mutable reference to an operator by ID
    pub fn get_mut(&mut self, id: Id) -> Option<&mut (dyn Operator + '_)> {
        self.nodes.get_mut(&id).map(|n| n.operator.as_mut())
    }

    /// Get a mutable reference to a specific operator type by ID
    pub fn get_mut_as<O: 'static>(&mut self, id: Id) -> Option<&mut O> {
        self.nodes
            .get_mut(&id)
            .and_then(|n| n.operator.as_any_mut().downcast_mut::<O>())
    }

    /// Get the name of a node
    pub fn node_name(&self, id: Id) -> Option<&'static str> {
        self.nodes.get(&id).map(|n| n.operator.name())
    }

    /// Returns the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns an iterator over all node IDs in the graph.
    pub fn node_ids(&self) -> impl Iterator<Item = Id> + '_ {
        self.nodes.keys().copied()
    }

    /// Remove a node from the graph.
    ///
    /// This will:
    /// 1. Disconnect all inputs that connect FROM this node to other nodes
    /// 2. Remove the node from the graph
    /// 3. Invalidate evaluation order
    ///
    /// Note: Connections TO this node (from other nodes) are stored on the target,
    /// so they'll be cleared when the node is removed. However, nodes that were
    /// connected FROM this node will have stale connection references that point
    /// to a non-existent node. These will safely return default values during evaluation.
    ///
    /// Returns the removed operator if found.
    pub fn remove(&mut self, id: Id) -> Option<Box<dyn Operator>> {
        // First, find all nodes that have connections FROM the node being removed
        // and disconnect them (connections are stored on the target side)
        let nodes_to_update: Vec<(Id, usize)> = self
            .nodes
            .iter()
            .filter(|(&node_id, _)| node_id != id)
            .flat_map(|(&node_id, node)| {
                node.operator
                    .inputs()
                    .iter()
                    .enumerate()
                    .filter_map(move |(input_idx, input)| {
                        // Check if this input connects from the node being removed
                        let connects_from_removed = input
                            .connection
                            .map(|(src, _)| src == id)
                            .unwrap_or(false)
                            || input.connections.iter().any(|(src, _)| *src == id);

                        if connects_from_removed {
                            Some((node_id, input_idx))
                        } else {
                            None
                        }
                    })
            })
            .collect();

        // Disconnect those inputs
        for (node_id, input_idx) in nodes_to_update {
            if let Some(node) = self.nodes.get_mut(&node_id) {
                let input = &mut node.operator.inputs_mut()[input_idx];
                // Clear single connection if it points to removed node
                if input.connection.map(|(src, _)| src == id).unwrap_or(false) {
                    input.connection = None;
                }
                // Remove from multi-input connections
                input.connections.retain(|(src, _)| *src != id);
            }
            self.invalidate_cache_for_node(node_id);
        }

        // Remove from cache
        self.invalidate_cache_for_node(id);

        // Remove the node itself
        let node = self.nodes.remove(&id)?;

        // Mark order as dirty
        self.order_dirty = true;

        // Emit event
        self.emit(GraphEvent::NodeRemoved { id });

        Some(node.operator)
    }

    /// Iterate over all connections in the graph.
    ///
    /// Returns an iterator of `Connection` structs describing each edge.
    pub fn connections(&self) -> impl Iterator<Item = Connection> + '_ {
        self.nodes.iter().flat_map(|(&target_id, node)| {
            node.operator
                .inputs()
                .iter()
                .enumerate()
                .flat_map(move |(input_idx, input)| {
                    // Collect single connection
                    let single = input.connection.map(|(source_id, source_output)| Connection {
                        source_node: source_id,
                        source_output,
                        target_node: target_id,
                        target_input: input_idx,
                    });

                    // Collect multi-input connections
                    let multi = input
                        .connections
                        .iter()
                        .map(move |&(source_id, source_output)| Connection {
                            source_node: source_id,
                            source_output,
                            target_node: target_id,
                            target_input: input_idx,
                        });

                    single.into_iter().chain(multi)
                })
        })
    }

    /// Get all nodes that this node's outputs connect to (downstream).
    pub fn downstream_of(&self, id: Id) -> Vec<Connection> {
        self.connections()
            .filter(|c| c.source_node == id)
            .collect()
    }

    /// Get all nodes that connect to this node's inputs (upstream).
    pub fn upstream_of(&self, id: Id) -> Vec<Connection> {
        self.connections()
            .filter(|c| c.target_node == id)
            .collect()
    }

    /// Set the default value for an input port on a node
    /// This is used by composite operators to pass values to internal nodes
    pub fn set_input_default(&mut self, node_id: Id, input_index: usize, value: Value) -> bool {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            if let Some(input_port) = node.operator.inputs_mut().get_mut(input_index) {
                input_port.default = value.clone();
                // Mark outputs as dirty since input changed
                for output in node.operator.outputs_mut() {
                    output.mark_dirty();
                }
                // Invalidate cache for this node and dependents
                self.invalidate_cache_for_node(node_id);

                // Emit event
                self.emit(GraphEvent::InputDefaultChanged {
                    node: node_id,
                    input: input_index,
                    value,
                });

                return true;
            }
        }
        false
    }

    // =========================================================================
    // Port Override API
    // =========================================================================

    /// Get the override for an input port, if any.
    pub fn get_input_override(&self, node_id: Id, input_index: usize) -> Option<&PortOverride> {
        self.nodes
            .get(&node_id)?
            .input_overrides
            .get(input_index)?
            .as_ref()
    }

    /// Set an override for an input port.
    ///
    /// Extends the override vector if necessary. If the override is empty
    /// (all fields None), it's equivalent to clearing the override.
    pub fn set_input_override(&mut self, node_id: Id, input_index: usize, override_: PortOverride) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            // Extend vector if needed
            if node.input_overrides.len() <= input_index {
                node.input_overrides.resize(input_index + 1, None);
            }
            // Store override (or None if empty)
            node.input_overrides[input_index] = if override_.is_empty() {
                None
            } else {
                Some(override_)
            };
        }
    }

    /// Clear an override for an input port.
    pub fn clear_input_override(&mut self, node_id: Id, input_index: usize) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            if let Some(slot) = node.input_overrides.get_mut(input_index) {
                *slot = None;
            }
        }
    }

    /// Get effective metadata for an input (combines PortMeta defaults + per-instance override).
    ///
    /// Returns resolved metadata ready for UI rendering.
    ///
    /// **Note**: Currently, PortMeta from operator is not accessible through `dyn Operator`.
    /// For full OperatorMeta support, use FluxNodalBridge which can access concrete types
    /// during node creation. This method applies overrides to sensible defaults.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The node to get metadata for
    /// * `input_index` - The input port index
    /// * `port_meta` - Optional PortMeta from the operator (caller must provide if known)
    pub fn get_effective_input_meta_with_default(
        &self,
        node_id: Id,
        input_index: usize,
        port_meta: Option<flux_core::PortMeta>,
    ) -> Option<EffectivePortMeta> {
        let node = self.nodes.get(&node_id)?;

        // Get per-instance override if any
        let override_ = node
            .input_overrides
            .get(input_index)
            .and_then(|o| o.as_ref());

        Some(EffectivePortMeta::from_meta(port_meta, override_))
    }

    /// Get per-instance override for an input, if any exists.
    ///
    /// This is useful when you need to check if a specific override is set
    /// before applying defaults.
    pub fn get_input_override_raw(&self, node_id: Id, input_index: usize) -> Option<&PortOverride> {
        self.get_input_override(node_id, input_index)
    }

    /// Connect a source output to a target input with type checking and auto-conversion.
    ///
    /// If the source and target types differ but can be coerced, a [`ConversionOp`]
    /// is automatically inserted between them. This makes type conversion explicit
    /// and visible in the graph.
    ///
    /// # Returns
    ///
    /// - `Ok(None)` - Direct connection (types match exactly)
    /// - `Ok(Some(id))` - Connection via auto-inserted conversion node
    /// - `Err(...)` - Connection failed (incompatible types, cycle, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Float to Vec3 connection - auto-inserts ConversionOp
    /// let conversion_id = graph.connect(float_node, 0, vec3_node, 0)?;
    /// if let Some(conv_id) = conversion_id {
    ///     println!("Conversion node inserted: {:?}", conv_id);
    /// }
    /// ```
    pub fn connect(
        &mut self,
        source_node: Id,
        source_output: usize,
        target_node: Id,
        target_input: usize,
    ) -> Result<Option<Id>, GraphError> {
        // Get source output type
        let source = self
            .nodes
            .get(&source_node)
            .ok_or(GraphError::NodeNotFound { id: source_node, name: None })?;

        let source_name = source.operator.name();
        let outputs = source.operator.outputs();
        if source_output >= outputs.len() {
            return Err(GraphError::output_not_found(
                source_node,
                source_output,
                source_name,
                outputs.len(),
            ));
        }
        let source_type = outputs[source_output].value_type;

        // Get target input type
        let target = self
            .nodes
            .get(&target_node)
            .ok_or(GraphError::NodeNotFound { id: target_node, name: None })?;

        let target_name = target.operator.name();
        let input_count = target.operator.inputs().len();

        if target_input >= input_count {
            return Err(GraphError::input_not_found(
                target_node,
                target_input,
                target_name,
                input_count,
            ));
        }

        let target_type = target.operator.inputs()[target_input].value_type;

        // Determine connection strategy based on types
        if source_type == target_type {
            // Direct connection - types match exactly
            self.connect_direct(source_node, source_output, target_node, target_input)?;
            Ok(None)
        } else if source_type.can_coerce_to(target_type) {
            // Auto-insert conversion operator
            let conv_op = ConversionOp::new(source_type, target_type);
            let conv_id = conv_op.id();
            self.add(conv_op);

            // Connect: source -> conversion -> target
            self.connect_direct(source_node, source_output, conv_id, 0)?;
            self.connect_direct(conv_id, 0, target_node, target_input)?;

            // Emit conversion insertion event
            self.emit(GraphEvent::ConversionInserted {
                conversion_node: conv_id,
                source_type,
                target_type,
            });

            Ok(Some(conv_id))
        } else {
            // Incompatible types - cannot connect
            Err(GraphError::type_mismatch(
                source_node,
                source_type,
                target_node,
                target_type,
            ))
        }
    }

    /// Connect a source output to a target input directly, without auto-conversion.
    ///
    /// This method performs the raw connection without checking for type compatibility
    /// beyond exact equality. It's used internally by `connect()` and can be used
    /// when you want to bypass auto-conversion (e.g., when manually inserting
    /// conversion nodes).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Source or target node doesn't exist
    /// - Output or input index is out of bounds
    /// - Types don't match exactly
    /// - Connection would create a cycle
    pub fn connect_direct(
        &mut self,
        source_node: Id,
        source_output: usize,
        target_node: Id,
        target_input: usize,
    ) -> Result<(), GraphError> {
        // Get source output type
        let source = self
            .nodes
            .get(&source_node)
            .ok_or(GraphError::NodeNotFound { id: source_node, name: None })?;

        let source_name = source.operator.name();
        let outputs = source.operator.outputs();
        if source_output >= outputs.len() {
            return Err(GraphError::output_not_found(
                source_node,
                source_output,
                source_name,
                outputs.len(),
            ));
        }
        let source_type = outputs[source_output].value_type;

        // Get target input type and connect
        let target = self
            .nodes
            .get_mut(&target_node)
            .ok_or(GraphError::NodeNotFound { id: target_node, name: None })?;

        let target_name = target.operator.name();
        let input_count = target.operator.inputs().len();

        if target_input >= input_count {
            return Err(GraphError::input_not_found(
                target_node,
                target_input,
                target_name,
                input_count,
            ));
        }

        let inputs = target.operator.inputs_mut();
        let target_type = inputs[target_input].value_type;

        // Type check - require exact match for direct connection
        if source_type != target_type {
            return Err(GraphError::type_mismatch(
                source_node,
                source_type,
                target_node,
                target_type,
            ));
        }

        // Track previous connection state for multi-input rollback
        let was_multi = inputs[target_input].is_multi_input;
        let prev_connection_count = inputs[target_input].connections.len();

        inputs[target_input].connect(source_node, source_output);

        // Check for cycles after connecting
        if let Err(cycle_nodes) = self.check_for_cycles() {
            // Undo only the newly-added connection
            if let Some(target) = self.nodes.get_mut(&target_node) {
                let input = &mut target.operator.inputs_mut()[target_input];
                if was_multi {
                    // For multi-input, remove only the last added connection
                    if input.connections.len() > prev_connection_count {
                        input.connections.pop();
                    }
                } else {
                    // For single-input, clear the connection
                    input.connection = None;
                }
            }
            return Err(GraphError::CycleDetected { nodes: cycle_nodes });
        }

        // Invalidate cache for target node since its input changed
        self.invalidate_cache_for_node(target_node);
        self.order_dirty = true;

        // Emit event
        self.emit(GraphEvent::Connected {
            source: source_node,
            source_output,
            target: target_node,
            target_input,
        });

        Ok(())
    }

    /// Disconnect a target input
    pub fn disconnect(&mut self, target_node: Id, target_input: usize) -> Result<(), GraphError> {
        let target = self
            .nodes
            .get_mut(&target_node)
            .ok_or(GraphError::NodeNotFound { id: target_node, name: None })?;

        let target_name = target.operator.name();
        let input_count = target.operator.inputs().len();

        if target_input >= input_count {
            return Err(GraphError::input_not_found(
                target_node,
                target_input,
                target_name,
                input_count,
            ));
        }
        target.operator.inputs_mut()[target_input].disconnect();
        // Invalidate cache for target node since its input changed
        self.invalidate_cache_for_node(target_node);
        self.order_dirty = true;

        // Emit event
        self.emit(GraphEvent::Disconnected {
            target: target_node,
            target_input,
        });

        Ok(())
    }

    // =========================================================================
    // Trigger Connections
    // =========================================================================

    /// Connect a trigger output to a trigger input.
    ///
    /// Unlike value connections, trigger connections don't carry data - they
    /// signal "execute now" to the target operator.
    ///
    /// # Arguments
    ///
    /// * `source_node` - Node emitting the trigger
    /// * `source_output` - Index of the trigger output on the source
    /// * `target_node` - Node receiving the trigger
    /// * `target_input` - Index of the trigger input on the target
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Source or target node doesn't exist
    /// - Trigger output or input index is out of bounds
    pub fn connect_trigger(
        &mut self,
        source_node: Id,
        source_output: usize,
        target_node: Id,
        target_input: usize,
    ) -> Result<(), GraphError> {
        // Verify source node and trigger output exist
        {
            let source = self
                .nodes
                .get(&source_node)
                .ok_or(GraphError::NodeNotFound { id: source_node, name: None })?;

            let trigger_outputs = source.operator.trigger_outputs();
            if source_output >= trigger_outputs.len() {
                return Err(GraphError::TriggerNotFound {
                    node_id: source_node,
                    is_output: true,
                    index: source_output,
                    available: trigger_outputs.len(),
                });
            }
        }

        // Verify target node and trigger input exist, then connect
        {
            let target = self
                .nodes
                .get_mut(&target_node)
                .ok_or(GraphError::NodeNotFound { id: target_node, name: None })?;

            let trigger_input_count = target.operator.trigger_inputs().len();
            if target_input >= trigger_input_count {
                return Err(GraphError::TriggerNotFound {
                    node_id: target_node,
                    is_output: false,
                    index: target_input,
                    available: trigger_input_count,
                });
            }

            // Connect the target's trigger input
            target.operator.trigger_inputs_mut()[target_input].connect(source_node, source_output);
        }

        // Add connection to source's trigger output
        {
            let source = self
                .nodes
                .get_mut(&source_node)
                .expect("Source node verified above");

            source.operator.trigger_outputs_mut()[source_output].connect(target_node, target_input);
        }

        // Emit event
        self.emit(GraphEvent::TriggerConnected {
            source: source_node,
            source_output,
            target: target_node,
            target_input,
        });

        Ok(())
    }

    /// Disconnect a trigger input from its source.
    ///
    /// # Arguments
    ///
    /// * `target_node` - Node with the trigger input to disconnect
    /// * `target_input` - Index of the trigger input
    ///
    /// # Returns
    ///
    /// The previous connection (source_node, source_output) if there was one.
    pub fn disconnect_trigger(
        &mut self,
        target_node: Id,
        target_input: usize,
    ) -> Result<Option<(Id, usize)>, GraphError> {
        let prev_connection;

        // Get the current connection and disconnect target's trigger input
        {
            let target = self
                .nodes
                .get_mut(&target_node)
                .ok_or(GraphError::NodeNotFound { id: target_node, name: None })?;

            let trigger_input_count = target.operator.trigger_inputs().len();
            if target_input >= trigger_input_count {
                return Err(GraphError::TriggerNotFound {
                    node_id: target_node,
                    is_output: false,
                    index: target_input,
                    available: trigger_input_count,
                });
            }

            prev_connection = target.operator.trigger_inputs()[target_input].connection;
            target.operator.trigger_inputs_mut()[target_input].disconnect();
        }

        // Remove connection from source's trigger output
        if let Some((source_node, source_output)) = prev_connection {
            if let Some(source) = self.nodes.get_mut(&source_node) {
                source.operator.trigger_outputs_mut()[source_output]
                    .disconnect(target_node, target_input);
            }

            // Emit event
            self.emit(GraphEvent::TriggerDisconnected {
                source: source_node,
                source_output,
                target: target_node,
                target_input,
            });
        }

        Ok(prev_connection)
    }

    /// Fire a trigger output and propagate to all connected trigger inputs.
    ///
    /// This initiates push-based execution. When a trigger fires:
    /// 1. All connected trigger inputs receive the signal
    /// 2. Each target operator's `on_triggered()` is called
    /// 3. Any triggers returned by `on_triggered()` are fired recursively
    ///
    /// # Arguments
    ///
    /// * `node_id` - Node whose trigger output to fire
    /// * `trigger_output` - Index of the trigger output to fire
    /// * `ctx` - Evaluation context for timing information
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Fire the "OnFrame" trigger from the main loop
    /// graph.fire_trigger(main_loop_id, 0, &ctx);
    /// ```
    pub fn fire_trigger(&mut self, node_id: Id, trigger_output: usize, ctx: &EvalContext) {
        // Get the targets for this trigger output
        let targets: Vec<(Id, usize)> = {
            let node = match self.nodes.get(&node_id) {
                Some(n) => n,
                None => return,
            };

            let trigger_outputs = node.operator.trigger_outputs();
            if trigger_output >= trigger_outputs.len() {
                return;
            }

            trigger_outputs[trigger_output].connections.clone()
        };

        // Fire each connected target
        for (target_id, target_input) in targets {
            self.trigger_node(target_id, target_input, ctx);
        }
    }

    /// Internal: Trigger a specific node's trigger input and handle cascading triggers.
    fn trigger_node(&mut self, node_id: Id, trigger_input: usize, ctx: &EvalContext) {
        // Create the input resolver closure
        let get_input_value = |source_id: Id, output_idx: usize| -> Value {
            // Try to get from cache first
            let cache_key = CacheKey {
                node_id: source_id,
                call_context: ctx.call_context,
            };

            if let Some(cached) = self.value_cache.get(&cache_key) {
                if let Some(value) = cached.get(output_idx) {
                    return (**value).clone();
                }
            }

            // Not cached - return a default value
            // In practice, trigger-based operators should either:
            // 1. Use inputs that are already cached from prior evaluation
            // 2. Not depend on value inputs for their triggered behavior
            Value::Float(0.0)
        };

        // Call the operator's on_triggered method
        let triggers_to_fire: Vec<usize> = {
            let node = match self.nodes.get_mut(&node_id) {
                Some(n) => n,
                None => return,
            };

            node.operator.on_triggered(trigger_input, ctx, &get_input_value)
        };

        // Fire any cascading triggers
        for output_idx in triggers_to_fire {
            self.fire_trigger(node_id, output_idx, ctx);
        }
    }

    /// Check for cycles in the graph using DFS
    fn check_for_cycles(&self) -> Result<(), Vec<Id>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut cycle_nodes = Vec::new();

        for &node_id in self.nodes.keys() {
            if self.has_cycle_dfs(node_id, &mut visited, &mut rec_stack, &mut cycle_nodes) {
                return Err(cycle_nodes);
            }
        }
        Ok(())
    }

    fn has_cycle_dfs(
        &self,
        node_id: Id,
        visited: &mut HashSet<Id>,
        rec_stack: &mut HashSet<Id>,
        cycle_nodes: &mut Vec<Id>,
    ) -> bool {
        if rec_stack.contains(&node_id) {
            cycle_nodes.push(node_id);
            return true;
        }
        if visited.contains(&node_id) {
            return false;
        }

        visited.insert(node_id);
        rec_stack.insert(node_id);

        if let Some(node) = self.nodes.get(&node_id) {
            for input in node.operator.inputs() {
                // Check single connection
                if let Some((dep_id, _)) = input.connection {
                    if self.has_cycle_dfs(dep_id, visited, rec_stack, cycle_nodes) {
                        cycle_nodes.push(node_id);
                        return true;
                    }
                }
                // Check multi-input connections
                for &(dep_id, _) in &input.connections {
                    if self.has_cycle_dfs(dep_id, visited, rec_stack, cycle_nodes) {
                        cycle_nodes.push(node_id);
                        return true;
                    }
                }
            }
        }

        rec_stack.remove(&node_id);
        false
    }

    /// Compute topological order for evaluation using Kahn's algorithm
    pub(crate) fn compute_order(&mut self) -> Result<(), GraphError> {
        if !self.order_dirty {
            return Ok(());
        }

        let mut remaining: Vec<Id> = self.nodes.keys().copied().collect();
        let mut order = Vec::with_capacity(remaining.len());
        // HashSet for O(1) dependency lookups instead of O(n) Vec::contains
        let mut order_set: HashSet<Id> = HashSet::with_capacity(remaining.len());
        let mut made_progress = true;

        while !remaining.is_empty() && made_progress {
            made_progress = false;

            remaining.retain(|&id| {
                let node = match self.nodes.get(&id) {
                    Some(n) => n,
                    None => return false, // Node disappeared, remove from remaining
                };

                // Check if all dependencies are already in order
                let deps_satisfied = node.operator.inputs().iter().all(|input| {
                    // Check single connection
                    let single_ok = match input.connection {
                        None => true,
                        Some((dep_id, _)) => order_set.contains(&dep_id),
                    };
                    // Check multi-input connections
                    let multi_ok = input
                        .connections
                        .iter()
                        .all(|(dep_id, _)| order_set.contains(dep_id));

                    single_ok && multi_ok
                });

                if deps_satisfied {
                    order.push(id);
                    order_set.insert(id);
                    made_progress = true;
                    false // remove from remaining
                } else {
                    true // keep in remaining
                }
            });
        }

        if !remaining.is_empty() {
            return Err(GraphError::CycleDetected { nodes: remaining });
        }

        self.eval_order = order;
        self.order_dirty = false;

        // Emit event when order is recomputed
        self.emit(GraphEvent::OrderRecomputed);

        Ok(())
    }

    /// Check if a node needs evaluation based on its dirty state and dependencies
    fn needs_evaluation(
        &self,
        node_id: Id,
        call_context: CallContext,
        computed_nodes: &HashSet<Id>,
    ) -> bool {
        let node = match self.nodes.get(&node_id) {
            Some(n) => n,
            None => return false,
        };

        // Create cache key with call context
        let cache_key = CacheKey {
            node_id,
            call_context,
        };

        // If node has never been computed (not in cache for this context), it needs evaluation
        if !self.value_cache.contains_key(&cache_key) {
            return true;
        }

        // Time-varying operators always need to be recomputed
        if node.operator.is_time_varying() {
            return true;
        }

        // Check if any output is dirty
        if node.operator.outputs().iter().any(|o| o.is_dirty()) {
            return true;
        }

        // Check if any connected input comes from a node that was just computed
        for input in node.operator.inputs() {
            if let Some((source_id, _)) = input.connection {
                if computed_nodes.contains(&source_id) {
                    return true;
                }
            }
            // Check multi-input connections
            for &(source_id, _) in &input.connections {
                if computed_nodes.contains(&source_id) {
                    return true;
                }
            }
        }

        false
    }

    /// Evaluate the graph and return the output value of a specific node
    pub fn evaluate(
        &mut self,
        output_node: Id,
        output_index: usize,
        ctx: &EvalContext,
    ) -> Result<Value, GraphError> {
        self.compute_order()?;

        // Get the call context for this evaluation
        let call_context = ctx.call_context;

        // Track which nodes were computed this frame (HashSet for O(1) lookups)
        let mut computed_nodes: HashSet<Id> = HashSet::new();

        // Clone eval_order to avoid borrow issues
        let eval_order = self.eval_order.clone();

        for &node_id in &eval_order {
            let needs_eval = self.needs_evaluation(node_id, call_context, &computed_nodes);

            if !needs_eval {
                continue;
            }

            // Get node reference safely
            let node = match self.nodes.get_mut(&node_id) {
                Some(n) => n,
                None => {
                    // Node was removed during evaluation, skip it
                    continue;
                }
            };

            // Create lookup closure that captures a reference to value_cache
            // We need to use a separate reference because we can't borrow self
            // while also having a mutable borrow of node
            //
            // Note: The closure looks up values using the same call context,
            // ensuring context-aware cache isolation for subroutines/loops.
            //
            // Reference stealing: When an Arc has refcount == 1, we could pass
            // ownership instead of cloning. However, since the closure captures
            // an immutable reference, we clone here. Full reference stealing
            // would require a more complex evaluation model where we pre-collect
            // inputs before computing.
            let cache_ref = &self.value_cache;
            let get_input = |dep_id: Id, idx: usize| -> Value {
                let key = CacheKey {
                    node_id: dep_id,
                    call_context,
                };
                cache_ref
                    .get(&key)
                    .and_then(|outputs| outputs.get(idx))
                    .map(|arc| {
                        // Try to steal the reference if we're the sole owner
                        // Note: This won't work with the immutable borrow, but we
                        // set up the infrastructure for future optimization
                        Arc::unwrap_or_clone(arc.clone())
                    })
                    .unwrap_or_default()
            };

            node.operator.compute(ctx, &get_input);

            // Update the cache with new output values wrapped in Arc
            let cache_key = CacheKey {
                node_id,
                call_context,
            };
            let outputs: Vec<Arc<Value>> = node
                .operator
                .outputs()
                .iter()
                .map(|o| Arc::new(o.value.clone()))
                .collect();
            self.value_cache.insert(cache_key, outputs);

            computed_nodes.insert(node_id);
        }

        // Return requested output (using the current call context)
        let output_key = CacheKey {
            node_id: output_node,
            call_context,
        };
        self.value_cache
            .get(&output_key)
            .and_then(|outputs| outputs.get(output_index))
            .map(|arc| Arc::unwrap_or_clone(arc.clone()))
            .ok_or_else(|| GraphError::node_not_found(output_node, self.node_name(output_node)))
    }

    /// Get statistics about the graph
    pub fn stats(&self) -> GraphStats {
        let mut connection_count = 0;
        for node in self.nodes.values() {
            for input in node.operator.inputs() {
                if input.connection.is_some() {
                    connection_count += 1;
                }
                connection_count += input.connections.len();
            }
        }

        GraphStats {
            node_count: self.nodes.len(),
            connection_count,
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the graph
#[derive(Debug, Clone)]
pub struct GraphStats {
    pub node_count: usize,
    pub connection_count: usize,
}

/// Represents a connection between two nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connection {
    /// The node that produces the value.
    pub source_node: Id,
    /// The output index on the source node.
    pub source_output: usize,
    /// The node that consumes the value.
    pub target_node: Id,
    /// The input index on the target node.
    pub target_input: usize,
}

/// Errors that can occur during graph operations
#[derive(Debug)]
pub enum GraphError {
    NodeNotFound {
        id: Id,
        name: Option<&'static str>,
    },
    InputNotFound {
        node_id: Id,
        input_index: usize,
        node_name: &'static str,
        input_count: usize,
    },
    OutputNotFound {
        node_id: Id,
        output_index: usize,
        node_name: &'static str,
        output_count: usize,
    },
    TypeMismatch {
        source_node: Id,
        source_type: ValueType,
        target_node: Id,
        target_type: ValueType,
    },
    CycleDetected {
        nodes: Vec<Id>,
    },
    /// Trigger port not found on a node
    TriggerNotFound {
        node_id: Id,
        is_output: bool,
        index: usize,
        available: usize,
    },
}

impl GraphError {
    pub(crate) fn node_not_found(id: Id, name: Option<&'static str>) -> Self {
        GraphError::NodeNotFound { id, name }
    }

    pub(crate) fn input_not_found(
        node_id: Id,
        input_index: usize,
        node_name: &'static str,
        input_count: usize,
    ) -> Self {
        GraphError::InputNotFound {
            node_id,
            input_index,
            node_name,
            input_count,
        }
    }

    pub(crate) fn output_not_found(
        node_id: Id,
        output_index: usize,
        node_name: &'static str,
        output_count: usize,
    ) -> Self {
        GraphError::OutputNotFound {
            node_id,
            output_index,
            node_name,
            output_count,
        }
    }

    pub(crate) fn type_mismatch(
        source_node: Id,
        source_type: ValueType,
        target_node: Id,
        target_type: ValueType,
    ) -> Self {
        GraphError::TypeMismatch {
            source_node,
            source_type,
            target_node,
            target_type,
        }
    }
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::NodeNotFound { id, name } => {
                if let Some(name) = name {
                    write!(f, "Node '{}' ({}) not found", name, id)
                } else {
                    write!(f, "Node {} not found", id)
                }
            }
            GraphError::InputNotFound {
                node_id,
                input_index,
                node_name,
                input_count,
            } => {
                write!(
                    f,
                    "Input index {} not found on '{}' ({}). Node has {} input(s).",
                    input_index, node_name, node_id, input_count
                )
            }
            GraphError::OutputNotFound {
                node_id,
                output_index,
                node_name,
                output_count,
            } => {
                write!(
                    f,
                    "Output index {} not found on '{}' ({}). Node has {} output(s).",
                    output_index, node_name, node_id, output_count
                )
            }
            GraphError::TypeMismatch {
                source_node,
                source_type,
                target_node,
                target_type,
            } => {
                write!(
                    f,
                    "Type mismatch: cannot connect {} output ({}) to {} input ({})",
                    source_type, source_node, target_type, target_node
                )
            }
            GraphError::CycleDetected { nodes } => {
                write!(f, "Cycle detected in graph involving {} node(s)", nodes.len())
            }
            GraphError::TriggerNotFound {
                node_id,
                is_output,
                index,
                available,
            } => {
                let port_type = if *is_output { "output" } else { "input" };
                write!(
                    f,
                    "Trigger {} index {} not found on node {}. Node has {} trigger {}(s).",
                    port_type, index, node_id, available, port_type
                )
            }
        }
    }
}

impl std::error::Error for GraphError {}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::{InputPort, Operator, OutputPort, Value, ValueType};

    /// Simple test operator for event system tests
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
        fn compute(&mut self, _ctx: &EvalContext, get_input: &dyn Fn(Id, usize) -> Value) {
            if !self.inputs.is_empty() {
                if let Some((source_id, source_output)) = self.inputs[0].connection {
                    let val = get_input(source_id, source_output);
                    self.outputs[0].value = val;
                }
            }
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_node_added_event() {
        let mut graph = Graph::new();
        assert!(!graph.has_pending_events());

        let op = TestOp::source();
        let id = graph.add(op);

        assert!(graph.has_pending_events());
        assert_eq!(graph.pending_event_count(), 1);

        let events: Vec<_> = graph.drain_events().collect();
        assert_eq!(events.len(), 1);
        match &events[0] {
            GraphEvent::NodeAdded { id: event_id } => assert_eq!(*event_id, id),
            _ => panic!("Expected NodeAdded event"),
        }

        assert!(!graph.has_pending_events());
    }

    #[test]
    fn test_node_removed_event() {
        let mut graph = Graph::new();
        let op = TestOp::source();
        let id = graph.add(op);

        // Clear add event
        graph.clear_events();

        graph.remove(id);

        let events: Vec<_> = graph.drain_events().collect();
        assert_eq!(events.len(), 1);
        match &events[0] {
            GraphEvent::NodeRemoved { id: event_id } => assert_eq!(*event_id, id),
            _ => panic!("Expected NodeRemoved event"),
        }
    }

    #[test]
    fn test_connected_event() {
        let mut graph = Graph::new();
        let source = graph.add(TestOp::source());
        let target = graph.add(TestOp::new());

        // Clear add events
        graph.clear_events();

        graph.connect(source, 0, target, 0).unwrap();

        let events: Vec<_> = graph.drain_events().collect();
        // We expect Connected + OrderRecomputed (from evaluation order)
        assert!(!events.is_empty());

        let connected = events.iter().find(|e| matches!(e, GraphEvent::Connected { .. }));
        assert!(connected.is_some());

        match connected.unwrap() {
            GraphEvent::Connected {
                source: src,
                source_output,
                target: tgt,
                target_input,
            } => {
                assert_eq!(*src, source);
                assert_eq!(*source_output, 0);
                assert_eq!(*tgt, target);
                assert_eq!(*target_input, 0);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_disconnected_event() {
        let mut graph = Graph::new();
        let source = graph.add(TestOp::source());
        let target = graph.add(TestOp::new());
        graph.connect(source, 0, target, 0).unwrap();

        // Clear previous events
        graph.clear_events();

        graph.disconnect(target, 0).unwrap();

        let events: Vec<_> = graph.drain_events().collect();
        assert!(!events.is_empty());

        let disconnected = events
            .iter()
            .find(|e| matches!(e, GraphEvent::Disconnected { .. }));
        assert!(disconnected.is_some());

        match disconnected.unwrap() {
            GraphEvent::Disconnected {
                target: tgt,
                target_input,
            } => {
                assert_eq!(*tgt, target);
                assert_eq!(*target_input, 0);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_input_default_changed_event() {
        let mut graph = Graph::new();
        let node = graph.add(TestOp::new());

        // Clear add event
        graph.clear_events();

        let success = graph.set_input_default(node, 0, Value::Float(42.0));
        assert!(success);

        let events: Vec<_> = graph.drain_events().collect();
        assert_eq!(events.len(), 1);

        match &events[0] {
            GraphEvent::InputDefaultChanged {
                node: n,
                input,
                value,
            } => {
                assert_eq!(*n, node);
                assert_eq!(*input, 0);
                assert_eq!(*value, Value::Float(42.0));
            }
            _ => panic!("Expected InputDefaultChanged event"),
        }
    }

    #[test]
    fn test_order_recomputed_event() {
        let mut graph = Graph::new();
        let source = graph.add(TestOp::source());
        let target = graph.add(TestOp::new());
        graph.connect(source, 0, target, 0).unwrap();

        // Clear previous events
        graph.clear_events();

        // Trigger order recomputation via evaluate
        let ctx = EvalContext::default();
        let _ = graph.evaluate(target, 0, &ctx);

        let events: Vec<_> = graph.drain_events().collect();
        let order_recomputed = events
            .iter()
            .any(|e| matches!(e, GraphEvent::OrderRecomputed));
        assert!(order_recomputed, "Expected OrderRecomputed event");
    }

    #[test]
    fn test_multiple_events_accumulate() {
        let mut graph = Graph::new();

        // Add multiple nodes without draining
        let _a = graph.add(TestOp::source());
        let _b = graph.add(TestOp::source());
        let _c = graph.add(TestOp::source());

        assert_eq!(graph.pending_event_count(), 3);

        let events: Vec<_> = graph.drain_events().collect();
        assert_eq!(events.len(), 3);
        assert!(events.iter().all(|e| matches!(e, GraphEvent::NodeAdded { .. })));
    }

    // =========================================================================
    // Phase 1 Feature Tests: CallContext-Aware Caching
    // =========================================================================

    /// Test operator that tracks how many times compute() is called
    struct CountingOp {
        id: Id,
        inputs: Vec<InputPort>,
        outputs: Vec<OutputPort>,
        compute_count: std::cell::Cell<u32>,
    }

    impl CountingOp {
        fn new() -> Self {
            Self {
                id: Id::new(),
                inputs: vec![InputPort::new("in", Value::Float(1.0))],
                outputs: vec![OutputPort::new("out", ValueType::Float)],
                compute_count: std::cell::Cell::new(0),
            }
        }

        fn get_compute_count(&self) -> u32 {
            self.compute_count.get()
        }
    }

    impl Operator for CountingOp {
        fn id(&self) -> Id {
            self.id
        }
        fn name(&self) -> &'static str {
            "CountingOp"
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
        fn compute(&mut self, _ctx: &EvalContext, get_input: &dyn Fn(Id, usize) -> Value) {
            self.compute_count.set(self.compute_count.get() + 1);
            // Double the input value
            if let Some((source_id, source_output)) = self.inputs[0].connection {
                let val = get_input(source_id, source_output);
                if let Value::Float(f) = val {
                    // Use set() to mark output as clean after computation
                    self.outputs[0].set(Value::Float(f * 2.0));
                }
            } else if let Value::Float(f) = self.inputs[0].default {
                // Use set() to mark output as clean after computation
                self.outputs[0].set(Value::Float(f * 2.0));
            }
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_call_context_cache_isolation() {
        // Test that the same operator evaluated with different CallContexts
        // gets separate cache entries

        let mut graph = Graph::new();
        let op = CountingOp::new();
        let op_id = op.id;
        graph.add(op);

        // First evaluation with root context
        let ctx_root = EvalContext::new();
        let result1 = graph.evaluate(op_id, 0, &ctx_root).unwrap();

        // Second evaluation with different call context (simulating a subroutine call)
        let ctx_child1 = ctx_root.with_call_context(1);
        let result2 = graph.evaluate(op_id, 0, &ctx_child1).unwrap();

        // Third evaluation with another different call context
        let ctx_child2 = ctx_root.with_call_context(2);
        let result3 = graph.evaluate(op_id, 0, &ctx_child2).unwrap();

        // All results should be the same value (2.0 = 1.0 * 2)
        assert_eq!(result1, Value::Float(2.0));
        assert_eq!(result2, Value::Float(2.0));
        assert_eq!(result3, Value::Float(2.0));

        // The operator should have been computed 3 times (once per context)
        let op = graph.get(op_id).unwrap();
        let counting_op = op.as_any().downcast_ref::<CountingOp>().unwrap();
        assert_eq!(counting_op.get_compute_count(), 3);
    }

    #[test]
    fn test_same_context_uses_cache() {
        // Test that evaluating with the same context reuses cached values

        let mut graph = Graph::new();
        let op = CountingOp::new();
        let op_id = op.id;
        graph.add(op);

        let ctx = EvalContext::new();

        // First evaluation - should compute
        let result1 = graph.evaluate(op_id, 0, &ctx).unwrap();

        // Second evaluation with same context - should use cache
        let result2 = graph.evaluate(op_id, 0, &ctx).unwrap();

        // Third evaluation with same context - should still use cache
        let result3 = graph.evaluate(op_id, 0, &ctx).unwrap();

        // All results should be the same
        assert_eq!(result1, Value::Float(2.0));
        assert_eq!(result2, Value::Float(2.0));
        assert_eq!(result3, Value::Float(2.0));

        // The operator should have been computed only once
        let op = graph.get(op_id).unwrap();
        let counting_op = op.as_any().downcast_ref::<CountingOp>().unwrap();
        assert_eq!(counting_op.get_compute_count(), 1);
    }

    #[test]
    fn test_nested_call_contexts_are_isolated() {
        // Test that nested call contexts (like nested loop iterations) are isolated

        let mut graph = Graph::new();
        let op = CountingOp::new();
        let op_id = op.id;
        graph.add(op);

        let ctx_root = EvalContext::new();

        // Simulate nested loops: outer loop iterations 0 and 1
        let ctx_outer_0 = ctx_root.with_call_context(0);
        let ctx_outer_1 = ctx_root.with_call_context(1);

        // Inner loop iterations within outer loop 0
        let ctx_0_0 = ctx_outer_0.with_call_context(0);
        let ctx_0_1 = ctx_outer_0.with_call_context(1);

        // Inner loop iterations within outer loop 1
        let ctx_1_0 = ctx_outer_1.with_call_context(0);
        let ctx_1_1 = ctx_outer_1.with_call_context(1);

        // Evaluate all 4 nested contexts
        graph.evaluate(op_id, 0, &ctx_0_0).unwrap();
        graph.evaluate(op_id, 0, &ctx_0_1).unwrap();
        graph.evaluate(op_id, 0, &ctx_1_0).unwrap();
        graph.evaluate(op_id, 0, &ctx_1_1).unwrap();

        // Each nested context should have its own cache entry
        let op = graph.get(op_id).unwrap();
        let counting_op = op.as_any().downcast_ref::<CountingOp>().unwrap();
        assert_eq!(counting_op.get_compute_count(), 4);
    }

    #[test]
    fn test_can_operate_in_place_default() {
        // Test that the default can_operate_in_place() returns false

        let op = TestOp::new();
        assert!(!op.can_operate_in_place());
    }

    /// Test operator that declares it can operate in-place
    struct InPlaceOp {
        id: Id,
        inputs: Vec<InputPort>,
        outputs: Vec<OutputPort>,
    }

    impl InPlaceOp {
        fn new() -> Self {
            Self {
                id: Id::new(),
                inputs: vec![InputPort::new("in", Value::Float(0.0))],
                outputs: vec![OutputPort::new("out", ValueType::Float)],
            }
        }
    }

    impl Operator for InPlaceOp {
        fn id(&self) -> Id {
            self.id
        }
        fn name(&self) -> &'static str {
            "InPlaceOp"
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
        fn compute(&mut self, _ctx: &EvalContext, _get_input: &dyn Fn(Id, usize) -> Value) {
            // Use set() to mark output as clean after computation
            self.outputs[0].set(Value::Float(42.0));
        }
        fn can_operate_in_place(&self) -> bool {
            true
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_can_operate_in_place_override() {
        // Test that operators can override can_operate_in_place() to return true

        let op = InPlaceOp::new();
        assert!(op.can_operate_in_place());
    }

    #[test]
    fn test_clear_cache_clears_all_contexts() {
        // Test that clear_cache() removes entries for all call contexts

        let mut graph = Graph::new();
        let op = CountingOp::new();
        let op_id = op.id;
        graph.add(op);

        let ctx_root = EvalContext::new();
        let ctx_child = ctx_root.with_call_context(1);

        // Evaluate with both contexts to populate cache
        graph.evaluate(op_id, 0, &ctx_root).unwrap();
        graph.evaluate(op_id, 0, &ctx_child).unwrap();

        // Clear the cache
        graph.clear_cache();

        // Evaluate again - should recompute since cache was cleared
        graph.evaluate(op_id, 0, &ctx_root).unwrap();
        graph.evaluate(op_id, 0, &ctx_child).unwrap();

        // Should have computed 4 times total (2 before clear, 2 after)
        let op = graph.get(op_id).unwrap();
        let counting_op = op.as_any().downcast_ref::<CountingOp>().unwrap();
        assert_eq!(counting_op.get_compute_count(), 4);
    }

    // =========================================================================
    // Phase 2 Feature Tests: Auto-Conversion at Connect Time
    // =========================================================================

    /// Test operator that outputs a Float
    struct FloatSourceOp {
        id: Id,
        outputs: Vec<OutputPort>,
    }

    impl FloatSourceOp {
        fn new(value: f32) -> Self {
            let mut output = OutputPort::float("Out");
            output.set(Value::Float(value));
            Self {
                id: Id::new(),
                outputs: vec![output],
            }
        }
    }

    impl Operator for FloatSourceOp {
        fn id(&self) -> Id {
            self.id
        }
        fn name(&self) -> &'static str {
            "FloatSource"
        }
        fn inputs(&self) -> &[InputPort] {
            &[]
        }
        fn inputs_mut(&mut self) -> &mut [InputPort] {
            &mut []
        }
        fn outputs(&self) -> &[OutputPort] {
            &self.outputs
        }
        fn outputs_mut(&mut self) -> &mut [OutputPort] {
            &mut self.outputs
        }
        fn compute(&mut self, _ctx: &EvalContext, _get_input: &dyn Fn(Id, usize) -> Value) {
            // Value is already set in constructor
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    /// Test operator that accepts a Vec3 input
    struct Vec3SinkOp {
        id: Id,
        inputs: Vec<InputPort>,
        outputs: Vec<OutputPort>,
    }

    impl Vec3SinkOp {
        fn new() -> Self {
            Self {
                id: Id::new(),
                inputs: vec![InputPort::new("In", Value::Vec3([0.0, 0.0, 0.0]))],
                outputs: vec![OutputPort::vec3("Out")],
            }
        }
    }

    impl Operator for Vec3SinkOp {
        fn id(&self) -> Id {
            self.id
        }
        fn name(&self) -> &'static str {
            "Vec3Sink"
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
        fn compute(&mut self, _ctx: &EvalContext, get_input: &dyn Fn(Id, usize) -> Value) {
            let input = if let Some((node_id, output_idx)) = self.inputs[0].connection {
                get_input(node_id, output_idx)
            } else {
                self.inputs[0].default.clone()
            };
            self.outputs[0].set(input);
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_connect_exact_type_match() {
        // When types match exactly, connect directly without conversion node
        let mut graph = Graph::new();
        let source = graph.add(TestOp::source());
        let target = graph.add(TestOp::new());

        // Clear events from adding nodes
        graph.clear_events();

        // Connect Float -> Float (exact match)
        let result = graph.connect(source, 0, target, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None); // No conversion node inserted

        // Should have emitted Connected event but no ConversionInserted event
        let events: Vec<_> = graph.drain_events().collect();
        assert!(events.iter().any(|e| matches!(e, GraphEvent::Connected { .. })));
        assert!(!events.iter().any(|e| matches!(e, GraphEvent::ConversionInserted { .. })));
    }

    #[test]
    fn test_connect_auto_conversion() {
        // When types can be coerced, auto-insert conversion node
        let mut graph = Graph::new();
        let float_source = graph.add(FloatSourceOp::new(2.5));
        let vec3_sink = graph.add(Vec3SinkOp::new());

        // Clear events from adding nodes
        graph.clear_events();

        // Connect Float -> Vec3 (requires conversion)
        let result = graph.connect(float_source, 0, vec3_sink, 0);
        assert!(result.is_ok());

        let conversion_id = result.unwrap();
        assert!(conversion_id.is_some()); // Conversion node was inserted

        let conv_id = conversion_id.unwrap();

        // Verify the conversion node exists and has correct types
        let conv_op = graph.get(conv_id).unwrap();
        assert_eq!(conv_op.name(), "Convert");

        // Check events
        let events: Vec<_> = graph.drain_events().collect();
        let conversion_event = events.iter().find(|e| {
            matches!(e, GraphEvent::ConversionInserted { .. })
        });
        assert!(conversion_event.is_some());

        if let Some(GraphEvent::ConversionInserted {
            conversion_node,
            source_type,
            target_type,
        }) = conversion_event
        {
            assert_eq!(*conversion_node, conv_id);
            assert_eq!(*source_type, ValueType::Float);
            assert_eq!(*target_type, ValueType::Vec3);
        }
    }

    #[test]
    fn test_connect_auto_conversion_evaluation() {
        // Verify that auto-conversion works correctly during evaluation
        let mut graph = Graph::new();
        let float_source = graph.add(FloatSourceOp::new(2.5));
        let vec3_sink_id = {
            let sink = Vec3SinkOp::new();
            let id = sink.id;
            graph.add(sink);
            id
        };

        // Connect with auto-conversion
        let conversion_id = graph.connect(float_source, 0, vec3_sink_id, 0).unwrap();
        assert!(conversion_id.is_some());

        // Evaluate the graph
        let ctx = EvalContext::new();
        let result = graph.evaluate(vec3_sink_id, 0, &ctx).unwrap();

        // Float 2.5 should be broadcast to Vec3 [2.5, 2.5, 2.5]
        assert_eq!(result, Value::Vec3([2.5, 2.5, 2.5]));
    }

    #[test]
    fn test_connect_incompatible_types() {
        // When types cannot be coerced, return error
        let mut graph = Graph::new();

        // String source
        struct StringSourceOp {
            id: Id,
            outputs: Vec<OutputPort>,
        }
        impl StringSourceOp {
            fn new() -> Self {
                Self {
                    id: Id::new(),
                    outputs: vec![OutputPort::string("Out")],
                }
            }
        }
        impl Operator for StringSourceOp {
            fn id(&self) -> Id { self.id }
            fn name(&self) -> &'static str { "StringSource" }
            fn inputs(&self) -> &[InputPort] { &[] }
            fn inputs_mut(&mut self) -> &mut [InputPort] { &mut [] }
            fn outputs(&self) -> &[OutputPort] { &self.outputs }
            fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }
            fn compute(&mut self, _ctx: &EvalContext, _get_input: &dyn Fn(Id, usize) -> Value) {}
            fn as_any(&self) -> &dyn std::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
        }

        let string_source = graph.add(StringSourceOp::new());
        let vec3_sink = graph.add(Vec3SinkOp::new());

        // Connect String -> Vec3 (incompatible)
        let result = graph.connect(string_source, 0, vec3_sink, 0);
        assert!(result.is_err());

        if let Err(GraphError::TypeMismatch { source_type, target_type, .. }) = result {
            assert_eq!(source_type, ValueType::String);
            assert_eq!(target_type, ValueType::Vec3);
        } else {
            panic!("Expected TypeMismatch error");
        }
    }

    #[test]
    fn test_connect_direct_requires_exact_match() {
        // connect_direct() should require exact type match, no auto-conversion
        let mut graph = Graph::new();
        let float_source = graph.add(FloatSourceOp::new(2.5));
        let vec3_sink = graph.add(Vec3SinkOp::new());

        // connect_direct Float -> Vec3 should fail
        let result = graph.connect_direct(float_source, 0, vec3_sink, 0);
        assert!(result.is_err());

        if let Err(GraphError::TypeMismatch { .. }) = result {
            // Expected
        } else {
            panic!("Expected TypeMismatch error from connect_direct");
        }
    }

    // =========================================================================
    // Trigger System Tests
    // =========================================================================

    /// Operator with trigger ports for testing push-based execution
    struct TriggerTestOp {
        id: Id,
        inputs: Vec<InputPort>,
        outputs: Vec<OutputPort>,
        trigger_inputs: Vec<flux_core::TriggerInput>,
        trigger_outputs: Vec<flux_core::TriggerOutput>,
        trigger_count: std::cell::Cell<usize>,
    }

    impl TriggerTestOp {
        fn new() -> Self {
            Self {
                id: Id::new(),
                inputs: vec![InputPort::new("in", Value::Float(0.0))],
                outputs: vec![OutputPort::new("out", ValueType::Float)],
                trigger_inputs: vec![flux_core::TriggerInput::new("OnFrame")],
                trigger_outputs: vec![flux_core::TriggerOutput::new("Done")],
                trigger_count: std::cell::Cell::new(0),
            }
        }

        fn trigger_count(&self) -> usize {
            self.trigger_count.get()
        }
    }

    impl Operator for TriggerTestOp {
        fn id(&self) -> Id {
            self.id
        }
        fn name(&self) -> &'static str {
            "TriggerTestOp"
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
        fn trigger_inputs(&self) -> &[flux_core::TriggerInput] {
            &self.trigger_inputs
        }
        fn trigger_inputs_mut(&mut self) -> &mut [flux_core::TriggerInput] {
            &mut self.trigger_inputs
        }
        fn trigger_outputs(&self) -> &[flux_core::TriggerOutput] {
            &self.trigger_outputs
        }
        fn trigger_outputs_mut(&mut self) -> &mut [flux_core::TriggerOutput] {
            &mut self.trigger_outputs
        }
        fn compute(&mut self, _ctx: &EvalContext, _get_input: &dyn Fn(Id, usize) -> Value) {
            self.outputs[0].set(Value::Float(42.0));
        }
        fn on_triggered(
            &mut self,
            trigger_index: usize,
            _ctx: &EvalContext,
            _get_input: flux_core::InputResolver,
        ) -> Vec<usize> {
            if trigger_index == 0 {
                self.trigger_count.set(self.trigger_count.get() + 1);
                // Fire "Done" trigger
                vec![0]
            } else {
                vec![]
            }
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    /// Source operator that has trigger outputs but no inputs
    struct TriggerSourceOp {
        id: Id,
        outputs: Vec<OutputPort>,
        trigger_outputs: Vec<flux_core::TriggerOutput>,
    }

    impl TriggerSourceOp {
        fn new() -> Self {
            Self {
                id: Id::new(),
                outputs: vec![OutputPort::new("out", ValueType::Float)],
                trigger_outputs: vec![flux_core::TriggerOutput::new("OnFrame")],
            }
        }
    }

    impl Operator for TriggerSourceOp {
        fn id(&self) -> Id {
            self.id
        }
        fn name(&self) -> &'static str {
            "TriggerSourceOp"
        }
        fn inputs(&self) -> &[InputPort] {
            &[]
        }
        fn inputs_mut(&mut self) -> &mut [InputPort] {
            &mut []
        }
        fn outputs(&self) -> &[OutputPort] {
            &self.outputs
        }
        fn outputs_mut(&mut self) -> &mut [OutputPort] {
            &mut self.outputs
        }
        fn trigger_outputs(&self) -> &[flux_core::TriggerOutput] {
            &self.trigger_outputs
        }
        fn trigger_outputs_mut(&mut self) -> &mut [flux_core::TriggerOutput] {
            &mut self.trigger_outputs
        }
        fn compute(&mut self, _ctx: &EvalContext, _get_input: &dyn Fn(Id, usize) -> Value) {
            self.outputs[0].set(Value::Float(1.0));
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_trigger_port_connection() {
        let mut graph = Graph::new();

        let source = graph.add(TriggerSourceOp::new());
        let target_id = {
            let op = TriggerTestOp::new();
            let id = op.id;
            graph.add(op);
            id
        };

        // Clear events from node additions
        graph.clear_events();

        // Connect trigger output to trigger input
        let result = graph.connect_trigger(source, 0, target_id, 0);
        assert!(result.is_ok());

        // Check events
        let events: Vec<_> = graph.drain_events().collect();
        assert_eq!(events.len(), 1);

        match &events[0] {
            GraphEvent::TriggerConnected {
                source: s,
                source_output,
                target: t,
                target_input,
            } => {
                assert_eq!(*s, source);
                assert_eq!(*source_output, 0);
                assert_eq!(*t, target_id);
                assert_eq!(*target_input, 0);
            }
            _ => panic!("Expected TriggerConnected event"),
        }
    }

    #[test]
    fn test_trigger_port_connection_invalid_source() {
        let mut graph = Graph::new();

        let source = graph.add(TestOp::source()); // No trigger outputs
        let target_id = {
            let op = TriggerTestOp::new();
            let id = op.id;
            graph.add(op);
            id
        };

        // Connect should fail - source has no trigger outputs
        let result = graph.connect_trigger(source, 0, target_id, 0);
        assert!(result.is_err());

        match result {
            Err(GraphError::TriggerNotFound {
                node_id,
                is_output,
                index,
                available,
            }) => {
                assert_eq!(node_id, source);
                assert!(is_output);
                assert_eq!(index, 0);
                assert_eq!(available, 0);
            }
            _ => panic!("Expected TriggerNotFound error"),
        }
    }

    #[test]
    fn test_trigger_port_connection_invalid_target() {
        let mut graph = Graph::new();

        let source = graph.add(TriggerSourceOp::new());
        let target = graph.add(TestOp::new()); // No trigger inputs

        // Connect should fail - target has no trigger inputs
        let result = graph.connect_trigger(source, 0, target, 0);
        assert!(result.is_err());

        match result {
            Err(GraphError::TriggerNotFound {
                node_id,
                is_output,
                index,
                available,
            }) => {
                assert_eq!(node_id, target);
                assert!(!is_output);
                assert_eq!(index, 0);
                assert_eq!(available, 0);
            }
            _ => panic!("Expected TriggerNotFound error"),
        }
    }

    #[test]
    fn test_trigger_disconnection() {
        let mut graph = Graph::new();

        let source = graph.add(TriggerSourceOp::new());
        let target_id = {
            let op = TriggerTestOp::new();
            let id = op.id;
            graph.add(op);
            id
        };

        // Connect
        graph.connect_trigger(source, 0, target_id, 0).unwrap();
        graph.clear_events();

        // Disconnect
        let prev = graph.disconnect_trigger(target_id, 0).unwrap();
        assert_eq!(prev, Some((source, 0)));

        // Check events
        let events: Vec<_> = graph.drain_events().collect();
        assert_eq!(events.len(), 1);

        match &events[0] {
            GraphEvent::TriggerDisconnected {
                source: s,
                source_output,
                target: t,
                target_input,
            } => {
                assert_eq!(*s, source);
                assert_eq!(*source_output, 0);
                assert_eq!(*t, target_id);
                assert_eq!(*target_input, 0);
            }
            _ => panic!("Expected TriggerDisconnected event"),
        }
    }

    #[test]
    fn test_fire_trigger_propagation() {
        let mut graph = Graph::new();

        let source = graph.add(TriggerSourceOp::new());
        let target_id = {
            let op = TriggerTestOp::new();
            let id = op.id;
            graph.add(op);
            id
        };

        // Connect trigger
        graph.connect_trigger(source, 0, target_id, 0).unwrap();

        // Fire trigger from source
        let ctx = EvalContext::new();
        graph.fire_trigger(source, 0, &ctx);

        // Check that target was triggered
        let target = graph.get(target_id).unwrap();
        let test_op = target.as_any().downcast_ref::<TriggerTestOp>().unwrap();
        assert_eq!(test_op.trigger_count(), 1);
    }

    #[test]
    fn test_fire_trigger_cascading() {
        // Test trigger chain: source -> op1 -> op2
        let mut graph = Graph::new();

        let source = graph.add(TriggerSourceOp::new());

        let op1_id = {
            let op = TriggerTestOp::new();
            let id = op.id;
            graph.add(op);
            id
        };

        let op2_id = {
            let op = TriggerTestOp::new();
            let id = op.id;
            graph.add(op);
            id
        };

        // Connect: source[0] -> op1[0]
        graph.connect_trigger(source, 0, op1_id, 0).unwrap();

        // Connect: op1.Done -> op2.OnFrame
        graph.connect_trigger(op1_id, 0, op2_id, 0).unwrap();

        // Fire trigger from source
        let ctx = EvalContext::new();
        graph.fire_trigger(source, 0, &ctx);

        // Both ops should have been triggered
        let op1 = graph.get(op1_id).unwrap();
        let test_op1 = op1.as_any().downcast_ref::<TriggerTestOp>().unwrap();
        assert_eq!(test_op1.trigger_count(), 1);

        let op2 = graph.get(op2_id).unwrap();
        let test_op2 = op2.as_any().downcast_ref::<TriggerTestOp>().unwrap();
        assert_eq!(test_op2.trigger_count(), 1);
    }

    #[test]
    fn test_fire_trigger_fan_out() {
        // Test trigger fan-out: source -> [op1, op2]
        let mut graph = Graph::new();

        let source = graph.add(TriggerSourceOp::new());

        let op1_id = {
            let op = TriggerTestOp::new();
            let id = op.id;
            graph.add(op);
            id
        };

        let op2_id = {
            let op = TriggerTestOp::new();
            let id = op.id;
            graph.add(op);
            id
        };

        // Connect both to the same trigger output
        graph.connect_trigger(source, 0, op1_id, 0).unwrap();
        graph.connect_trigger(source, 0, op2_id, 0).unwrap();

        // Fire trigger from source
        let ctx = EvalContext::new();
        graph.fire_trigger(source, 0, &ctx);

        // Both ops should have been triggered
        let op1 = graph.get(op1_id).unwrap();
        let test_op1 = op1.as_any().downcast_ref::<TriggerTestOp>().unwrap();
        assert_eq!(test_op1.trigger_count(), 1);

        let op2 = graph.get(op2_id).unwrap();
        let test_op2 = op2.as_any().downcast_ref::<TriggerTestOp>().unwrap();
        assert_eq!(test_op2.trigger_count(), 1);
    }
}
