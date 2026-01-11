//! Oscillator operators: SawWave, TriangleWave, PulseWave, Accumulator, Spring
//! Note: SineWave is in the legacy operator.rs

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};

fn get_float(input: &InputPort, get_input: InputResolver) -> f32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
        None => input.default.as_float().unwrap_or(0.0),
    }
}

// ============================================================================
// SawWave Operator
// ============================================================================

pub struct SawWaveOp {
    id: Id,
    inputs: [InputPort; 4],
    outputs: [OutputPort; 1],
}

impl SawWaveOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Frequency", 1.0),
                InputPort::float("Amplitude", 1.0),
                InputPort::float("Phase", 0.0),
                InputPort::float("Offset", 0.0),
            ],
            outputs: [OutputPort::float("Value")],
        }
    }
}

impl Default for SawWaveOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SawWaveOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "SawWave" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, ctx: &EvalContext, get_input: InputResolver) {
        let freq = get_float(&self.inputs[0], get_input);
        let amp = get_float(&self.inputs[1], get_input);
        let phase = get_float(&self.inputs[2], get_input);
        let offset = get_float(&self.inputs[3], get_input);

        let t = ctx.time as f32;
        // Sawtooth: goes from -1 to 1 over one period
        let cycle = (t * freq + phase).rem_euclid(1.0);
        let value = (cycle * 2.0 - 1.0) * amp + offset;
        self.outputs[0].set_float(value);
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for SawWaveOp {
    fn category(&self) -> &'static str {
        "Oscillators"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::OSCILLATORS
    }

    fn description(&self) -> &'static str {
        "Sawtooth wave oscillator"
    }

    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Frequency").with_shape(PinShape::CircleFilled).with_unit("Hz")),
            1 => Some(PortMeta::new("Amplitude").with_shape(PinShape::CircleFilled).with_range(0.0,10.0)),
            2 => Some(PortMeta::new("Phase").with_shape(PinShape::CircleFilled)),
            3 => Some(PortMeta::new("Offset").with_shape(PinShape::CircleFilled)),
            _ => None,
        }
    }

    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// TriangleWave Operator
// ============================================================================

pub struct TriangleWaveOp {
    id: Id,
    inputs: [InputPort; 4],
    outputs: [OutputPort; 1],
}

impl TriangleWaveOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Frequency", 1.0),
                InputPort::float("Amplitude", 1.0),
                InputPort::float("Phase", 0.0),
                InputPort::float("Offset", 0.0),
            ],
            outputs: [OutputPort::float("Value")],
        }
    }
}

impl Default for TriangleWaveOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for TriangleWaveOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "TriangleWave" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, ctx: &EvalContext, get_input: InputResolver) {
        let freq = get_float(&self.inputs[0], get_input);
        let amp = get_float(&self.inputs[1], get_input);
        let phase = get_float(&self.inputs[2], get_input);
        let offset = get_float(&self.inputs[3], get_input);

        let t = ctx.time as f32;
        let cycle = (t * freq + phase).rem_euclid(1.0);
        // Triangle: goes from -1 to 1 to -1 over one period
        let value = (1.0 - (cycle * 2.0 - 1.0).abs() * 2.0) * amp + offset;
        self.outputs[0].set_float(value);
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for TriangleWaveOp {
    fn category(&self) -> &'static str {
        "Oscillators"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::OSCILLATORS
    }

    fn description(&self) -> &'static str {
        "Triangle wave oscillator"
    }

    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Frequency").with_shape(PinShape::CircleFilled).with_unit("Hz")),
            1 => Some(PortMeta::new("Amplitude").with_shape(PinShape::CircleFilled)),
            2 => Some(PortMeta::new("Phase").with_shape(PinShape::CircleFilled)),
            3 => Some(PortMeta::new("Offset").with_shape(PinShape::CircleFilled)),
            _ => None,
        }
    }

    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// PulseWave Operator
// ============================================================================

pub struct PulseWaveOp {
    id: Id,
    inputs: [InputPort; 4],
    outputs: [OutputPort; 1],
}

impl PulseWaveOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Frequency", 1.0),
                InputPort::float("Duty", 0.5),
                InputPort::float("Amplitude", 1.0),
                InputPort::float("Offset", 0.0),
            ],
            outputs: [OutputPort::float("Value")],
        }
    }
}

impl Default for PulseWaveOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for PulseWaveOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "PulseWave" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, ctx: &EvalContext, get_input: InputResolver) {
        let freq = get_float(&self.inputs[0], get_input);
        let duty = get_float(&self.inputs[1], get_input).clamp(0.0, 1.0);
        let amp = get_float(&self.inputs[2], get_input);
        let offset = get_float(&self.inputs[3], get_input);

        let t = ctx.time as f32;
        let cycle = (t * freq).rem_euclid(1.0);
        let value = if cycle < duty { amp } else { -amp };
        self.outputs[0].set_float(value + offset);
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for PulseWaveOp {
    fn category(&self) -> &'static str {
        "Oscillators"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::OSCILLATORS
    }

    fn description(&self) -> &'static str {
        "Pulse/square wave oscillator"
    }

    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Frequency").with_shape(PinShape::CircleFilled).with_unit("Hz")),
            1 => Some(PortMeta::new("Duty").with_shape(PinShape::CircleFilled)),
            2 => Some(PortMeta::new("Amplitude").with_shape(PinShape::CircleFilled)),
            3 => Some(PortMeta::new("Offset").with_shape(PinShape::CircleFilled)),
            _ => None,
        }
    }

    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Accumulator Operator
// ============================================================================

pub struct AccumulatorOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    accumulated: f32,
    last_time: f64,
}

impl AccumulatorOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Value", 0.0),
                InputPort::float("Rate", 1.0),
            ],
            outputs: [OutputPort::float("Result")],
            accumulated: 0.0,
            last_time: 0.0,
        }
    }
}

impl Default for AccumulatorOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for AccumulatorOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Accumulator" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, ctx: &EvalContext, get_input: InputResolver) {
        let value = get_float(&self.inputs[0], get_input);
        let rate = get_float(&self.inputs[1], get_input);

        let dt = if self.last_time > 0.0 {
            (ctx.time - self.last_time) as f32
        } else {
            0.0
        };
        self.last_time = ctx.time;

        self.accumulated += value * rate * dt;
        self.outputs[0].set_float(self.accumulated);
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for AccumulatorOp {
    fn category(&self) -> &'static str {
        "Time"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::TIME
    }

    fn description(&self) -> &'static str {
        "Accumulate value over time"
    }

    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value").with_shape(PinShape::CircleFilled)),
            1 => Some(PortMeta::new("Rate").with_shape(PinShape::CircleFilled)),
            _ => None,
        }
    }

    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Result").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Spring Operator
// ============================================================================

pub struct SpringOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
    current: f32,
    velocity: f32,
    last_time: f64,
}

impl SpringOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Target", 0.0),
                InputPort::float("Stiffness", 100.0),
                InputPort::float("Damping", 10.0),
            ],
            outputs: [OutputPort::float("Value")],
            current: 0.0,
            velocity: 0.0,
            last_time: 0.0,
        }
    }
}

impl Default for SpringOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SpringOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Spring" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, ctx: &EvalContext, get_input: InputResolver) {
        let target = get_float(&self.inputs[0], get_input);
        let stiffness = get_float(&self.inputs[1], get_input);
        let damping = get_float(&self.inputs[2], get_input);

        let dt = if self.last_time > 0.0 {
            (ctx.time - self.last_time) as f32
        } else {
            0.016
        };
        self.last_time = ctx.time;

        // Spring physics: F = -k * x - d * v
        let force = (target - self.current) * stiffness;
        let damping_force = -self.velocity * damping;
        let acceleration = force + damping_force;

        self.velocity += acceleration * dt;
        self.current += self.velocity * dt;

        self.outputs[0].set_float(self.current);
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for SpringOp {
    fn category(&self) -> &'static str {
        "Time"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::TIME
    }

    fn description(&self) -> &'static str {
        "Spring physics simulation"
    }

    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Target").with_shape(PinShape::CircleFilled)),
            1 => Some(PortMeta::new("Stiffness").with_shape(PinShape::CircleFilled)),
            2 => Some(PortMeta::new("Damping").with_shape(PinShape::CircleFilled)),
            _ => None,
        }
    }

    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Registration
// ============================================================================

pub fn register(registry: &OperatorRegistry) {
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "SawWave",
            category: "Oscillators",
            description: "Sawtooth wave oscillator",
        },
        || capture_meta(SawWaveOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "TriangleWave",
            category: "Oscillators",
            description: "Triangle wave oscillator",
        },
        || capture_meta(TriangleWaveOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "PulseWave",
            category: "Oscillators",
            description: "Pulse/square wave oscillator",
        },
        || capture_meta(PulseWaveOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Accumulator",
            category: "Time",
            description: "Accumulate value over time",
        },
        || capture_meta(AccumulatorOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Spring",
            category: "Time",
            description: "Spring physics simulation",
        },
        || capture_meta(SpringOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::Value;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Float(0.0)
    }

    #[test]
    fn test_saw_wave() {
        let mut op = SawWaveOp::new();
        op.inputs[0].default = Value::Float(1.0); // 1 Hz
        op.inputs[1].default = Value::Float(1.0);
        op.inputs[2].default = Value::Float(0.0);
        op.inputs[3].default = Value::Float(0.0);

        let mut ctx = EvalContext::new();

        // At t=0, should be at start of cycle
        ctx.time = 0.0;
        op.compute(&ctx, &no_connections);
        let v0 = op.outputs[0].value.as_float().unwrap();
        assert!((v0 - (-1.0)).abs() < 0.01);

        // At t=0.5, should be at 0
        ctx.time = 0.5;
        op.compute(&ctx, &no_connections);
        let v05 = op.outputs[0].value.as_float().unwrap();
        assert!((v05 - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_pulse_wave() {
        let mut op = PulseWaveOp::new();
        op.inputs[0].default = Value::Float(1.0);
        op.inputs[1].default = Value::Float(0.5); // 50% duty
        op.inputs[2].default = Value::Float(1.0);
        op.inputs[3].default = Value::Float(0.0);

        let mut ctx = EvalContext::new();

        // First half should be high
        ctx.time = 0.25;
        op.compute(&ctx, &no_connections);
        let v1 = op.outputs[0].value.as_float().unwrap();
        assert_eq!(v1, 1.0);

        // Second half should be low
        ctx.time = 0.75;
        op.compute(&ctx, &no_connections);
        let v2 = op.outputs[0].value.as_float().unwrap();
        assert_eq!(v2, -1.0);
    }

    #[test]
    fn test_spring_convergence() {
        let mut op = SpringOp::new();
        op.inputs[0].default = Value::Float(1.0); // Target
        op.inputs[1].default = Value::Float(100.0); // Stiffness
        op.inputs[2].default = Value::Float(10.0); // Damping

        let mut ctx = EvalContext::new();

        // Simulate for a while
        for i in 0..200 {
            ctx.time = i as f64 * 0.016;
            op.compute(&ctx, &no_connections);
        }

        let result = op.outputs[0].value.as_float().unwrap();
        assert!((result - 1.0).abs() < 0.1, "Spring should converge to target");
    }
}
