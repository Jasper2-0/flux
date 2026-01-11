//! Scope operator - displays a waveform plot of the input signal
//!
//! This operator acts as a pass-through while maintaining a ring buffer
//! of recent values for visualization.

use std::any::Any;
use std::collections::VecDeque;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::port::{InputPort, OutputPort};

use flux_core::{category_colors, InputResolver, Operator, OperatorMeta, PinShape, PortMeta};

/// Default number of samples to keep in the buffer
const DEFAULT_BUFFER_SIZE: usize = 128;

/// Scope operator - visualizes signal values over time
pub struct ScopeOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    /// Ring buffer of recent values
    buffer: VecDeque<f32>,
    /// Maximum buffer size
    buffer_size: usize,
    /// Minimum value seen (for auto-scaling)
    min_value: f32,
    /// Maximum value seen (for auto-scaling)
    max_value: f32,
}

impl ScopeOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("In", 0.0)],
            outputs: [OutputPort::float("Out")],
            buffer: VecDeque::with_capacity(DEFAULT_BUFFER_SIZE),
            buffer_size: DEFAULT_BUFFER_SIZE,
            min_value: f32::MAX,
            max_value: f32::MIN,
        }
    }

    /// Get the sample buffer for rendering
    pub fn samples(&self) -> &VecDeque<f32> {
        &self.buffer
    }

    /// Get the value range for rendering (min, max)
    pub fn value_range(&self) -> (f32, f32) {
        if self.min_value > self.max_value {
            // No samples yet
            (-1.0, 1.0)
        } else {
            // Add a little padding
            let range = self.max_value - self.min_value;
            let padding = if range < 0.001 { 0.5 } else { range * 0.1 };
            (self.min_value - padding, self.max_value + padding)
        }
    }

    /// Reset the scope buffer and range
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.min_value = f32::MAX;
        self.max_value = f32::MIN;
    }
}

impl Default for ScopeOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ScopeOp {
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
        "Scope"
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
        // Get input value (connected or default)
        let value = match self.inputs[0].connection {
            Some((node_id, output_idx)) => {
                get_input(node_id, output_idx).as_float().unwrap_or(0.0)
            }
            None => self.inputs[0].default.as_float().unwrap_or(0.0),
        };

        // Add to ring buffer
        if self.buffer.len() >= self.buffer_size {
            self.buffer.pop_front();
        }
        self.buffer.push_back(value);

        // Update range tracking
        if value < self.min_value {
            self.min_value = value;
        }
        if value > self.max_value {
            self.max_value = value;
        }

        // Pass through to output
        self.outputs[0].set_float(value);
    }

    fn is_time_varying(&self) -> bool {
        // Always needs to be re-evaluated to update the buffer
        true
    }
}

impl OperatorMeta for ScopeOp {
    fn category(&self) -> &'static str {
        "Output"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::OUTPUT
    }

    fn description(&self) -> &'static str {
        "Displays a waveform plot of the input signal"
    }

    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("In").with_shape(PinShape::CircleFilled)),
            _ => None,
        }
    }

    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Out").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}
