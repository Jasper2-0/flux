use std::any::Any;

use flux_core::context::EvalContext;
use crate::graph::{Graph, GraphError};
use flux_core::id::Id;
use crate::instance_path::InstancePath;
use flux_core::operator::{InputResolver, Operator};
use flux_core::port::{InputPort, OutputPort};
use flux_core::value::{Value, ValueType};

/// An exposed slot that maps external inputs/outputs to internal nodes
#[derive(Clone, Debug)]
pub struct ExposedSlot {
    /// External slot ID (what the outside world sees)
    pub external_id: Id,
    /// Name of the exposed slot
    pub name: &'static str,
    /// Type of the slot
    pub value_type: ValueType,
    /// ID of the internal node this maps to
    pub internal_node: Id,
    /// Index of the slot on the internal node
    pub internal_slot_index: usize,
}

/// A composite operator that contains a subgraph
///
/// Composite operators allow creating reusable "macros" or "functions"
/// by grouping multiple operators into a single node. The internal
/// graph is hidden, and only the exposed inputs/outputs are visible.
///
/// This is similar to a Symbol concept where operators can contain
/// child operators (hierarchical composition).
pub struct CompositeOp {
    id: Id,
    name: &'static str,

    /// The internal subgraph
    subgraph: Graph,

    /// External input slots (visible from outside)
    inputs: Vec<InputPort>,
    /// External output slots (visible from outside)
    outputs: Vec<OutputPort>,

    /// Mapping from external input index to internal node/slot
    exposed_inputs: Vec<ExposedSlot>,
    /// Mapping from external output index to internal node/slot
    exposed_outputs: Vec<ExposedSlot>,

    /// Instance path for nested evaluation
    #[allow(dead_code)]
    instance_path: InstancePath,
}

impl CompositeOp {
    /// Create a new composite operator with a name
    pub fn new(name: &'static str) -> Self {
        let id = Id::new();
        Self {
            id,
            name,
            subgraph: Graph::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            exposed_inputs: Vec::new(),
            exposed_outputs: Vec::new(),
            instance_path: InstancePath::root(id),
        }
    }

    /// Add an operator to the internal subgraph
    pub fn add<O: Operator + 'static>(&mut self, op: O) -> Id {
        self.subgraph.add(op)
    }

    /// Connect two nodes within the subgraph.
    ///
    /// Returns `Ok(Some(id))` if a conversion node was auto-inserted, `Ok(None)` otherwise.
    pub fn connect_internal(
        &mut self,
        source_node: Id,
        source_output: usize,
        target_node: Id,
        target_input: usize,
    ) -> Result<Option<Id>, GraphError> {
        self.subgraph
            .connect(source_node, source_output, target_node, target_input)
    }

    /// Expose an internal input as an external input
    ///
    /// This creates an input slot on the composite that, when connected,
    /// passes values through to the internal node.
    pub fn expose_input(
        &mut self,
        name: &'static str,
        internal_node: Id,
        internal_slot_index: usize,
    ) -> Result<usize, &'static str> {
        // Get the type from the internal node's input
        let value_type = self
            .subgraph
            .get(internal_node)
            .and_then(|op| op.inputs().get(internal_slot_index))
            .map(|slot| slot.value_type)
            .ok_or("Internal input slot not found")?;

        let default = self
            .subgraph
            .get(internal_node)
            .and_then(|op| op.inputs().get(internal_slot_index))
            .map(|slot| slot.default.clone())
            .unwrap_or(Value::Float(0.0));

        let external_slot = InputPort::new_typed(name, value_type, default);
        let external_id = external_slot.id;
        let index = self.inputs.len();

        self.inputs.push(external_slot);
        self.exposed_inputs.push(ExposedSlot {
            external_id,
            name,
            value_type,
            internal_node,
            internal_slot_index,
        });

        Ok(index)
    }

    /// Expose an internal output as an external output
    ///
    /// This creates an output slot on the composite that provides
    /// the value from the internal node.
    pub fn expose_output(
        &mut self,
        name: &'static str,
        internal_node: Id,
        internal_slot_index: usize,
    ) -> Result<usize, &'static str> {
        // Get the type from the internal node's output
        let value_type = self
            .subgraph
            .get(internal_node)
            .and_then(|op| op.outputs().get(internal_slot_index))
            .map(|slot| slot.value_type)
            .ok_or("Internal output slot not found")?;

        let external_slot = OutputPort::new_typed(name, value_type);
        let external_id = external_slot.id;
        let index = self.outputs.len();

        self.outputs.push(external_slot);
        self.exposed_outputs.push(ExposedSlot {
            external_id,
            name,
            value_type,
            internal_node,
            internal_slot_index,
        });

        Ok(index)
    }

    /// Get the internal subgraph (for inspection)
    pub fn subgraph(&self) -> &Graph {
        &self.subgraph
    }

    /// Get the internal subgraph mutably
    pub fn subgraph_mut(&mut self) -> &mut Graph {
        &mut self.subgraph
    }

    /// Get information about exposed inputs
    pub fn exposed_inputs(&self) -> &[ExposedSlot] {
        &self.exposed_inputs
    }

    /// Get information about exposed outputs
    pub fn exposed_outputs(&self) -> &[ExposedSlot] {
        &self.exposed_outputs
    }
}

impl Operator for CompositeOp {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &'static str {
        self.name
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

    fn compute(&mut self, ctx: &EvalContext, get_input_value: InputResolver) {
        // Step 1: Collect external input values (before borrowing subgraph)
        let input_values: Vec<(Id, usize, Value)> = self
            .exposed_inputs
            .iter()
            .enumerate()
            .map(|(ext_idx, exposed)| {
                let value = match self.inputs[ext_idx].connection {
                    Some((node_id, output_idx)) => get_input_value(node_id, output_idx),
                    None => self.inputs[ext_idx].default.clone(),
                };
                (exposed.internal_node, exposed.internal_slot_index, value)
            })
            .collect();

        // Step 2: Apply input values to internal nodes using Graph helper
        for (internal_node, internal_slot_index, value) in input_values {
            self.subgraph.set_input_default(internal_node, internal_slot_index, value);
        }

        // Step 3: Collect output node/slot info (before borrowing subgraph again)
        let output_targets: Vec<(Id, usize)> = self
            .exposed_outputs
            .iter()
            .map(|exposed| (exposed.internal_node, exposed.internal_slot_index))
            .collect();

        // Step 4: Evaluate the internal subgraph for each exposed output
        for (ext_idx, (internal_node, internal_slot_index)) in output_targets.into_iter().enumerate()
        {
            match self.subgraph.evaluate(internal_node, internal_slot_index, ctx) {
                Ok(value) => {
                    self.outputs[ext_idx].set(value);
                }
                Err(e) => {
                    eprintln!(
                        "  [{}] Error evaluating internal graph: {}",
                        self.name, e
                    );
                }
            }
        }

        println!("  [{}] computed (composite)", self.name);
    }
}

/// Builder pattern for creating composite operators
pub struct CompositeBuilder {
    composite: CompositeOp,
}

impl CompositeBuilder {
    /// Start building a new composite operator
    pub fn new(name: &'static str) -> Self {
        Self {
            composite: CompositeOp::new(name),
        }
    }

    /// Add an operator to the composite
    pub fn with_operator<O: Operator + 'static>(mut self, op: O) -> (Self, Id) {
        let id = self.composite.add(op);
        (self, id)
    }

    /// Connect two internal nodes
    pub fn connect(
        mut self,
        source_node: Id,
        source_output: usize,
        target_node: Id,
        target_input: usize,
    ) -> Result<Self, GraphError> {
        self.composite
            .connect_internal(source_node, source_output, target_node, target_input)?;
        Ok(self)
    }

    /// Expose an input
    pub fn expose_input(
        mut self,
        name: &'static str,
        internal_node: Id,
        internal_slot_index: usize,
    ) -> Result<Self, &'static str> {
        self.composite
            .expose_input(name, internal_node, internal_slot_index)?;
        Ok(self)
    }

    /// Expose an output
    pub fn expose_output(
        mut self,
        name: &'static str,
        internal_node: Id,
        internal_slot_index: usize,
    ) -> Result<Self, &'static str> {
        self.composite
            .expose_output(name, internal_node, internal_slot_index)?;
        Ok(self)
    }

    /// Build the composite operator
    pub fn build(self) -> CompositeOp {
        self.composite
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_operators::{AddOp, ConstantOp, MultiplyOp};

    #[test]
    fn test_composite_basic() {
        // Create a composite that computes (A + B) * 2
        let mut composite = CompositeOp::new("AddAndDouble");

        // Add internal nodes
        let const_two = composite.add(ConstantOp::new(2.0));
        let add = composite.add(AddOp::new());
        let multiply = composite.add(MultiplyOp::new());

        // Connect: add -> multiply.A, const_two -> multiply.B
        composite
            .connect_internal(add, 0, multiply, 0)
            .expect("connect add to multiply");
        composite
            .connect_internal(const_two, 0, multiply, 1)
            .expect("connect const to multiply");

        // Expose Add's inputs as external inputs
        composite
            .expose_input("A", add, 0)
            .expect("expose input A");
        composite
            .expose_input("B", add, 1)
            .expect("expose input B");

        // Expose Multiply's output as external output
        composite
            .expose_output("Result", multiply, 0)
            .expect("expose output");

        // Verify structure
        assert_eq!(composite.inputs().len(), 2);
        assert_eq!(composite.outputs().len(), 1);
        assert_eq!(composite.name(), "AddAndDouble");
    }
}
