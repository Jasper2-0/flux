//! Sum operator - sums multiple inputs (variadic)

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::port::{InputPort, OutputPort};

use flux_core::{InputResolver, Operator};

pub struct SumOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: [OutputPort; 1],
}

impl SumOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![InputPort::float_multi("Values")],
            outputs: [OutputPort::float("Sum")],
        }
    }
}

impl Default for SumOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SumOp {
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
        "Sum"
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

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let input = &self.inputs[0];
        let mut sum = 0.0;
        let mut values = Vec::new();

        for &(node_id, output_idx) in &input.connections {
            let val = get_input(node_id, output_idx).as_float().unwrap_or(0.0);
            values.push(val);
            sum += val;
        }

        if values.is_empty() {
            println!("  [Sum] (no inputs) = 0");
        } else {
            let values_str: Vec<String> = values.iter().map(|v| format!("{}", v)).collect();
            println!("  [Sum] {} = {}", values_str.join(" + "), sum);
        }

        self.outputs[0].set_float(sum);
    }
}
