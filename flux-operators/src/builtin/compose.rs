//! Vector composition operators

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::port::{InputPort, OutputPort};

use flux_core::{InputResolver, Operator};

/// Vec3 Compose Operator - creates a Vec3 from X, Y, Z components
pub struct Vec3ComposeOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl Vec3ComposeOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("X", 0.0),
                InputPort::float("Y", 0.0),
                InputPort::float("Z", 0.0),
            ],
            outputs: [OutputPort::vec3("Vector")],
        }
    }
}

impl Default for Vec3ComposeOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for Vec3ComposeOp {
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
        "Vec3Compose"
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
        let get_float = |input: &InputPort| -> f32 {
            match input.connection {
                Some((node_id, output_idx)) => {
                    get_input(node_id, output_idx).as_float().unwrap_or(0.0)
                }
                None => input.default.as_float().unwrap_or(0.0),
            }
        };

        let x = get_float(&self.inputs[0]);
        let y = get_float(&self.inputs[1]);
        let z = get_float(&self.inputs[2]);

        println!("  [Vec3Compose] ({}, {}, {})", x, y, z);
        self.outputs[0].set_vec3([x, y, z]);
    }
}
