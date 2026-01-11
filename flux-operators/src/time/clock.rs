//! Time/clock operators: Time, LocalTime, DeltaTime, Frame

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};

// ============================================================================
// Time Operator
// ============================================================================

pub struct TimeOp {
    id: Id,
    outputs: [OutputPort; 1],
}

impl TimeOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            outputs: [OutputPort::float("Time")],
        }
    }
}

impl Default for TimeOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for TimeOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Time" }
    fn inputs(&self) -> &[InputPort] { &[] }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut [] }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, ctx: &EvalContext, _get_input: InputResolver) {
        self.outputs[0].set_float(ctx.time as f32);
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for TimeOp {
    fn category(&self) -> &'static str {
        "Time"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::TIME
    }

    fn description(&self) -> &'static str {
        "Current global time in seconds"
    }

    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Time").with_shape(PinShape::TriangleFilled).with_unit("s")),
            _ => None,
        }
    }
}

// ============================================================================
// LocalTime Operator
// ============================================================================

pub struct LocalTimeOp {
    id: Id,
    outputs: [OutputPort; 1],
}

impl LocalTimeOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            outputs: [OutputPort::float("LocalTime")],
        }
    }
}

impl Default for LocalTimeOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for LocalTimeOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "LocalTime" }
    fn inputs(&self) -> &[InputPort] { &[] }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut [] }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, ctx: &EvalContext, _get_input: InputResolver) {
        self.outputs[0].set_float(ctx.local_time as f32);
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for LocalTimeOp {
    fn category(&self) -> &'static str {
        "Time"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::TIME
    }

    fn description(&self) -> &'static str {
        "Local time in current composition"
    }

    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("LocalTime").with_shape(PinShape::TriangleFilled).with_unit("s")),
            _ => None,
        }
    }
}

// ============================================================================
// DeltaTime Operator
// ============================================================================

pub struct DeltaTimeOp {
    id: Id,
    outputs: [OutputPort; 1],
}

impl DeltaTimeOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            outputs: [OutputPort::float("DeltaTime")],
        }
    }
}

impl Default for DeltaTimeOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for DeltaTimeOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "DeltaTime" }
    fn inputs(&self) -> &[InputPort] { &[] }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut [] }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, ctx: &EvalContext, _get_input: InputResolver) {
        self.outputs[0].set_float(ctx.delta_time as f32);
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for DeltaTimeOp {
    fn category(&self) -> &'static str {
        "Time"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::TIME
    }

    fn description(&self) -> &'static str {
        "Time since last frame"
    }

    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("DT").with_shape(PinShape::TriangleFilled).with_unit("s")),
            _ => None,
        }
    }
}

// ============================================================================
// Frame Operator
// ============================================================================

pub struct FrameOp {
    id: Id,
    outputs: [OutputPort; 1],
}

impl FrameOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            outputs: [OutputPort::int("Frame")],
        }
    }
}

impl Default for FrameOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for FrameOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Frame" }
    fn inputs(&self) -> &[InputPort] { &[] }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut [] }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, ctx: &EvalContext, _get_input: InputResolver) {
        self.outputs[0].set_int(ctx.frame as i32);
    }

    fn is_time_varying(&self) -> bool {
        true
    }
}

impl OperatorMeta for FrameOp {
    fn category(&self) -> &'static str {
        "Time"
    }

    fn category_color(&self) -> [f32; 4] {
        category_colors::TIME
    }

    fn description(&self) -> &'static str {
        "Current frame number"
    }

    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Frame").with_shape(PinShape::TriangleFilled)),
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
            name: "Time",
            category: "Time",
            description: "Current global time in seconds",
        },
        || capture_meta(TimeOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "LocalTime",
            category: "Time",
            description: "Local time in current composition",
        },
        || capture_meta(LocalTimeOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "DeltaTime",
            category: "Time",
            description: "Time since last frame",
        },
        || capture_meta(DeltaTimeOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Frame",
            category: "Time",
            description: "Current frame number",
        },
        || capture_meta(FrameOp::new()),
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
    fn test_time() {
        let mut op = TimeOp::new();
        let mut ctx = EvalContext::new();
        ctx.time = 5.5;
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(5.5));
    }

    #[test]
    fn test_delta_time() {
        let mut op = DeltaTimeOp::new();
        let mut ctx = EvalContext::new();
        ctx.delta_time = 0.016;
        op.compute(&ctx, &no_connections);
        let result = op.outputs[0].value.as_float().unwrap();
        assert!((result - 0.016).abs() < 0.0001);
    }

    #[test]
    fn test_frame() {
        let mut op = FrameOp::new();
        let mut ctx = EvalContext::new();
        ctx.frame = 100;
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(100));
    }
}
