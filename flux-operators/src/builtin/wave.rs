//! Time-based wave operators

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::port::{InputPort, OutputPort};

use flux_core::{category_colors, InputResolver, Operator, OperatorMeta, PinShape, PortMeta};

/// SineWave Operator - time-based sine wave generator
pub struct SineWaveOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl SineWaveOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Frequency", 1.0), // Hz
                InputPort::float("Amplitude", 1.0),
                InputPort::float("Phase", 0.0), // radians
            ],
            outputs: [OutputPort::float("Value")],
        }
    }
}

impl Default for SineWaveOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SineWaveOp {
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
        "SineWave"
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

    fn compute(&mut self, ctx: &EvalContext, get_input: InputResolver) {
        let freq = match self.inputs[0].connection {
            Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(1.0),
            None => self.inputs[0].default.as_float().unwrap_or(1.0),
        };
        let amp = match self.inputs[1].connection {
            Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(1.0),
            None => self.inputs[1].default.as_float().unwrap_or(1.0),
        };
        let phase = match self.inputs[2].connection {
            Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
            None => self.inputs[2].default.as_float().unwrap_or(0.0),
        };

        let time = ctx.time as f32;
        let result = amp * (2.0 * std::f32::consts::PI * freq * time + phase).sin();
        self.outputs[0].set_float(result);
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for SineWaveOp {
    fn category(&self) -> &'static str {
        "Oscillators"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::OSCILLATORS
    }

    fn description(&self) -> &'static str {
        "Generates a sine wave: amplitude * sin(2π * frequency * time + phase)"
    }

    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(
                PortMeta::new("Frequency")
                    .with_range(0.01, 100.0)
                    .with_unit("Hz"),
            ),
            1 => Some(PortMeta::new("Amplitude").with_range(0.0, 10.0)),
            2 => Some(
                PortMeta::new("Phase")
                    .with_range(0.0, std::f32::consts::TAU)
                    .with_unit("rad"),
            ),
            _ => None,
        }
    }

    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Wave").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}
