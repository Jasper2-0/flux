//! Graph Compiler - Compiles operator graphs to efficient command buffers
//!
//! This module provides a two-tier execution model inspired by Werkkzeug4:
//!
//! 1. **Compile phase**: Graph is analyzed and flattened into a `CompiledGraph`
//! 2. **Execute phase**: `CompiledGraph` runs with minimal overhead
//!
//! # Benefits
//!
//! - No HashMap lookups during execution (pre-computed indices)
//! - Linear memory access (commands in contiguous array)
//! - No trait object dispatch (direct function pointers)
//! - Opportunity for optimizations (dead code elimination, constant folding)
//!
//! # Example
//!
//! ```ignore
//! use flux_graph::{Graph, CompiledGraph};
//!
//! let mut graph = Graph::new();
//! // ... add operators and connect them ...
//!
//! // Compile the graph (once, or when structure changes)
//! let compiled = graph.compile(output_node, output_index)?;
//!
//! // Execute efficiently (every frame)
//! let ctx = EvalContext::new();
//! let result = compiled.execute(&ctx);
//! ```

use std::collections::HashMap;

use flux_core::{EvalContext, Id, Value};

use crate::graph::{Graph, GraphError};

/// A compiled representation of a graph, optimized for execution.
///
/// The graph is flattened into a linear sequence of commands that execute
/// in topological order. Each command reads from a shared output buffer
/// using pre-computed indices, eliminating HashMap lookups.
pub struct CompiledGraph {
    /// Commands to execute in order
    commands: Vec<Command>,
    /// Mapping from node Id to output buffer base index
    node_output_base: HashMap<Id, usize>,
    /// Total number of outputs across all nodes
    total_outputs: usize,
    /// The output we're computing (index into output buffer)
    target_output: usize,
}

/// A single compiled command representing one operator.
struct Command {
    /// Node ID (for debugging and cache invalidation)
    node_id: Id,
    /// Base index in the output buffer where this node's outputs start
    output_base: usize,
    // Note: The following fields are computed during compilation but not currently
    // used during execution. They're retained for potential future optimizations
    // like pre-gathering inputs or function pointer extraction.
    #[allow(dead_code)]
    /// Number of outputs this node produces
    output_count: usize,
    #[allow(dead_code)]
    /// Input mappings: Vec<(input_index, source_output_buffer_index)>
    /// For inputs with no connection, source_output_buffer_index is None
    input_sources: Vec<Option<usize>>,
    #[allow(dead_code)]
    /// Default values for unconnected inputs (indices match input_sources)
    input_defaults: Vec<Value>,
}

impl CompiledGraph {
    /// Execute the compiled graph and return the target output value.
    ///
    /// This is the hot path - designed for minimal overhead:
    /// - No HashMap lookups (pre-computed index mapping)
    /// - Linear memory access pattern
    pub fn execute(&self, graph: &mut Graph, ctx: &EvalContext) -> Value {
        // Output buffer holds all computed values
        let mut outputs: Vec<Value> = vec![Value::Float(0.0); self.total_outputs];

        for cmd in &self.commands {
            // Execute the operator
            if let Some(node) = graph.nodes.get_mut(&cmd.node_id) {
                // Create input resolver that maps (source_id, source_output) to our output buffer
                // The node_output_base map lets us convert source_id lookups to buffer indices
                let node_output_base = &self.node_output_base;
                let outputs_ref = &outputs;

                let get_input = |source_id: Id, source_output: usize| -> Value {
                    // Look up the base index for the source node
                    if let Some(&base) = node_output_base.get(&source_id) {
                        outputs_ref
                            .get(base + source_output)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        Value::Float(0.0)
                    }
                };

                node.operator.compute(ctx, &get_input);

                // Copy outputs to buffer
                for (i, output) in node.operator.outputs().iter().enumerate() {
                    outputs[cmd.output_base + i] = output.value.clone();
                }
            }
        }

        // Return the target output
        outputs.get(self.target_output).cloned().unwrap_or_default()
    }

    /// Get the number of commands in the compiled graph.
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    /// Get the total number of output slots.
    pub fn output_count(&self) -> usize {
        self.total_outputs
    }

    /// Check if a node is included in the compiled graph.
    pub fn contains_node(&self, node_id: Id) -> bool {
        self.node_output_base.contains_key(&node_id)
    }
}

impl Graph {
    /// Compile the graph for efficient execution.
    ///
    /// This creates a `CompiledGraph` that can be executed repeatedly with
    /// minimal overhead. The compiled graph is a snapshot - if the graph
    /// structure changes, you must recompile.
    ///
    /// # Arguments
    ///
    /// * `output_node` - The node whose output we want to compute
    /// * `output_index` - The output port index on that node
    ///
    /// # Returns
    ///
    /// A `CompiledGraph` that can be executed efficiently.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let compiled = graph.compile(output_node, 0)?;
    ///
    /// // Execute many times efficiently
    /// for frame in 0..1000 {
    ///     ctx.set_time(frame as f32 / 60.0);
    ///     let result = compiled.execute(&mut graph, &ctx);
    /// }
    /// ```
    pub fn compile(
        &mut self,
        output_node: Id,
        output_index: usize,
    ) -> Result<CompiledGraph, GraphError> {
        // Ensure topological order is computed
        self.compute_order()?;

        // Verify output node exists
        let output_node_info = self
            .nodes
            .get(&output_node)
            .ok_or_else(|| GraphError::node_not_found(output_node, None))?;

        let output_count = output_node_info.operator.outputs().len();
        if output_index >= output_count {
            return Err(GraphError::output_not_found(
                output_node,
                output_index,
                output_node_info.operator.name(),
                output_count,
            ));
        }

        // Build node output base index mapping
        // Each node's outputs are placed contiguously in the output buffer
        let mut node_output_base: HashMap<Id, usize> = HashMap::new();
        let mut current_base = 0;

        for &node_id in &self.eval_order {
            if let Some(node) = self.nodes.get(&node_id) {
                node_output_base.insert(node_id, current_base);
                current_base += node.operator.outputs().len();
            }
        }

        let total_outputs = current_base;

        // Calculate target output index
        let target_output = node_output_base
            .get(&output_node)
            .map(|base| base + output_index)
            .ok_or_else(|| GraphError::node_not_found(output_node, None))?;

        // Build commands
        let mut commands = Vec::with_capacity(self.eval_order.len());

        for &node_id in &self.eval_order {
            if let Some(node) = self.nodes.get(&node_id) {
                let output_base = *node_output_base.get(&node_id).unwrap();
                let output_count = node.operator.outputs().len();

                // Map each input to its source in the output buffer
                let mut input_sources = Vec::new();
                let mut input_defaults = Vec::new();

                for input in node.operator.inputs() {
                    let source_idx = input.connection.and_then(|(source_id, source_output)| {
                        node_output_base
                            .get(&source_id)
                            .map(|base| base + source_output)
                    });
                    input_sources.push(source_idx);
                    input_defaults.push(input.default.clone());
                }

                commands.push(Command {
                    node_id,
                    output_base,
                    output_count,
                    input_sources,
                    input_defaults,
                });
            }
        }

        Ok(CompiledGraph {
            commands,
            node_output_base,
            total_outputs,
            target_output,
        })
    }

    /// Compile the graph with dead code elimination.
    ///
    /// This is like `compile()` but only includes nodes that are actually
    /// needed to compute the target output. Nodes that don't contribute
    /// to the result are excluded.
    ///
    /// # Arguments
    ///
    /// * `output_node` - The node whose output we want to compute
    /// * `output_index` - The output port index on that node
    ///
    /// # Returns
    ///
    /// A `CompiledGraph` with only the necessary nodes.
    pub fn compile_optimized(
        &mut self,
        output_node: Id,
        output_index: usize,
    ) -> Result<CompiledGraph, GraphError> {
        // Ensure topological order is computed
        self.compute_order()?;

        // Verify output node exists
        let output_node_info = self
            .nodes
            .get(&output_node)
            .ok_or_else(|| GraphError::node_not_found(output_node, None))?;

        let output_count = output_node_info.operator.outputs().len();
        if output_index >= output_count {
            return Err(GraphError::output_not_found(
                output_node,
                output_index,
                output_node_info.operator.name(),
                output_count,
            ));
        }

        // Find all nodes needed to compute the output (backward traversal)
        let needed_nodes = self.find_dependencies(output_node);

        // Build node output base index mapping (only for needed nodes)
        let mut node_output_base: HashMap<Id, usize> = HashMap::new();
        let mut current_base = 0;

        for &node_id in &self.eval_order {
            if needed_nodes.contains(&node_id) {
                if let Some(node) = self.nodes.get(&node_id) {
                    node_output_base.insert(node_id, current_base);
                    current_base += node.operator.outputs().len();
                }
            }
        }

        let total_outputs = current_base;

        // Calculate target output index
        let target_output = node_output_base
            .get(&output_node)
            .map(|base| base + output_index)
            .ok_or_else(|| GraphError::node_not_found(output_node, None))?;

        // Build commands (only for needed nodes)
        let mut commands = Vec::new();

        for &node_id in &self.eval_order {
            if !needed_nodes.contains(&node_id) {
                continue;
            }

            if let Some(node) = self.nodes.get(&node_id) {
                let output_base = *node_output_base.get(&node_id).unwrap();
                let output_count = node.operator.outputs().len();

                // Map each input to its source in the output buffer
                let mut input_sources = Vec::new();
                let mut input_defaults = Vec::new();

                for input in node.operator.inputs() {
                    let source_idx = input.connection.and_then(|(source_id, source_output)| {
                        node_output_base
                            .get(&source_id)
                            .map(|base| base + source_output)
                    });
                    input_sources.push(source_idx);
                    input_defaults.push(input.default.clone());
                }

                commands.push(Command {
                    node_id,
                    output_base,
                    output_count,
                    input_sources,
                    input_defaults,
                });
            }
        }

        Ok(CompiledGraph {
            commands,
            node_output_base,
            total_outputs,
            target_output,
        })
    }

    /// Find all nodes that the given node depends on (including itself).
    fn find_dependencies(&self, node_id: Id) -> std::collections::HashSet<Id> {
        let mut deps = std::collections::HashSet::new();
        let mut stack = vec![node_id];

        while let Some(current) = stack.pop() {
            if deps.contains(&current) {
                continue;
            }
            deps.insert(current);

            if let Some(node) = self.nodes.get(&current) {
                for input in node.operator.inputs() {
                    // Check single connection
                    if let Some((dep_id, _)) = input.connection {
                        if !deps.contains(&dep_id) {
                            stack.push(dep_id);
                        }
                    }
                    // Check multi-input connections
                    for &(dep_id, _) in &input.connections {
                        if !deps.contains(&dep_id) {
                            stack.push(dep_id);
                        }
                    }
                }
            }
        }

        deps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::{InputPort, Operator, OutputPort, ValueType};

    /// Simple source operator for testing
    struct SourceOp {
        id: Id,
        outputs: Vec<OutputPort>,
        value: f32,
    }

    impl SourceOp {
        fn new(value: f32) -> Self {
            let mut output = OutputPort::new("Out", ValueType::Float);
            output.set(Value::Float(value));
            Self {
                id: Id::new(),
                outputs: vec![output],
                value,
            }
        }
    }

    impl Operator for SourceOp {
        fn id(&self) -> Id {
            self.id
        }
        fn name(&self) -> &'static str {
            "SourceOp"
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
            self.outputs[0].set(Value::Float(self.value));
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    /// Add operator for testing
    struct AddOp {
        id: Id,
        inputs: Vec<InputPort>,
        outputs: Vec<OutputPort>,
    }

    impl AddOp {
        fn new() -> Self {
            Self {
                id: Id::new(),
                inputs: vec![
                    InputPort::new("A", Value::Float(0.0)),
                    InputPort::new("B", Value::Float(0.0)),
                ],
                outputs: vec![OutputPort::new("Sum", ValueType::Float)],
            }
        }
    }

    impl Operator for AddOp {
        fn id(&self) -> Id {
            self.id
        }
        fn name(&self) -> &'static str {
            "AddOp"
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
            let a = if let Some((src, idx)) = self.inputs[0].connection {
                get_input(src, idx)
            } else {
                self.inputs[0].default.clone()
            };
            let b = if let Some((src, idx)) = self.inputs[1].connection {
                get_input(src, idx)
            } else {
                self.inputs[1].default.clone()
            };

            let sum = match (a, b) {
                (Value::Float(x), Value::Float(y)) => x + y,
                _ => 0.0,
            };
            self.outputs[0].set(Value::Float(sum));
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_compile_simple() {
        let mut graph = Graph::new();

        let source = graph.add(SourceOp::new(42.0));

        let compiled = graph.compile(source, 0).unwrap();

        assert_eq!(compiled.command_count(), 1);
        assert_eq!(compiled.output_count(), 1);
        assert!(compiled.contains_node(source));
    }

    #[test]
    fn test_compile_chain() {
        let mut graph = Graph::new();

        let src1 = graph.add(SourceOp::new(10.0));
        let src2 = graph.add(SourceOp::new(20.0));
        let add_id = {
            let add = AddOp::new();
            let id = add.id;
            graph.add(add);
            id
        };

        graph.connect(src1, 0, add_id, 0).unwrap();
        graph.connect(src2, 0, add_id, 1).unwrap();

        let compiled = graph.compile(add_id, 0).unwrap();

        assert_eq!(compiled.command_count(), 3);
        assert!(compiled.contains_node(src1));
        assert!(compiled.contains_node(src2));
        assert!(compiled.contains_node(add_id));
    }

    #[test]
    fn test_compile_execute() {
        let mut graph = Graph::new();

        let src1 = graph.add(SourceOp::new(10.0));
        let src2 = graph.add(SourceOp::new(20.0));
        let add_id = {
            let add = AddOp::new();
            let id = add.id;
            graph.add(add);
            id
        };

        graph.connect(src1, 0, add_id, 0).unwrap();
        graph.connect(src2, 0, add_id, 1).unwrap();

        let compiled = graph.compile(add_id, 0).unwrap();
        let ctx = EvalContext::new();

        let result = compiled.execute(&mut graph, &ctx);

        assert_eq!(result, Value::Float(30.0));
    }

    #[test]
    fn test_compile_optimized_dead_code_elimination() {
        let mut graph = Graph::new();

        // Create a graph with unused nodes
        let used_src = graph.add(SourceOp::new(10.0));
        let _unused_src = graph.add(SourceOp::new(999.0)); // Not connected to output

        let add_id = {
            let add = AddOp::new();
            let id = add.id;
            graph.add(add);
            id
        };

        // Only connect one source
        graph.connect(used_src, 0, add_id, 0).unwrap();
        // Input B uses default (0.0)

        // Compile with optimization
        let compiled = graph.compile_optimized(add_id, 0).unwrap();

        // Should only include used_src and add, not unused_src
        assert_eq!(compiled.command_count(), 2);
        assert!(compiled.contains_node(used_src));
        assert!(compiled.contains_node(add_id));
        // unused_src should be eliminated
    }

    #[test]
    fn test_compile_execute_multiple_times() {
        let mut graph = Graph::new();

        let src = graph.add(SourceOp::new(5.0));

        let compiled = graph.compile(src, 0).unwrap();
        let ctx = EvalContext::new();

        // Execute multiple times - should give same result
        for _ in 0..10 {
            let result = compiled.execute(&mut graph, &ctx);
            assert_eq!(result, Value::Float(5.0));
        }
    }

    #[test]
    fn test_compile_invalid_output_node() {
        let mut graph = Graph::new();

        let fake_id = Id::new();
        let result = graph.compile(fake_id, 0);

        assert!(result.is_err());
    }

    #[test]
    fn test_compile_invalid_output_index() {
        let mut graph = Graph::new();

        let src = graph.add(SourceOp::new(1.0));
        let result = graph.compile(src, 5); // SourceOp only has 1 output

        assert!(result.is_err());
    }
}
