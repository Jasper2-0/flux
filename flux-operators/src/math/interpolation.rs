//! Interpolation operators: Lerp, SmoothStep, Remap, InverseLerp, MapRange

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
// Lerp Operator
// ============================================================================

pub struct LerpOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl LerpOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("A", 0.0),
                InputPort::float("B", 1.0),
                InputPort::float("T", 0.5),
            ],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for LerpOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for LerpOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Lerp" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_float(&self.inputs[0], get_input);
        let b = get_float(&self.inputs[1], get_input);
        let t = get_float(&self.inputs[2], get_input);
        self.outputs[0].set_float(a + (b - a) * t);
    }
}

impl OperatorMeta for LerpOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Linear interpolation between A and B" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            2 => Some(PortMeta::new("T").with_range(0.0, 1.0)),
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

// ============================================================================
// SmoothStep Operator
// ============================================================================

pub struct SmoothStepOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl SmoothStepOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Edge0", 0.0),
                InputPort::float("Edge1", 1.0),
                InputPort::float("X", 0.5),
            ],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for SmoothStepOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SmoothStepOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "SmoothStep" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let edge0 = get_float(&self.inputs[0], get_input);
        let edge1 = get_float(&self.inputs[1], get_input);
        let x = get_float(&self.inputs[2], get_input);

        // GLSL smoothstep: t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0)
        // return t * t * (3.0 - 2.0 * t)
        let range = edge1 - edge0;

        // Handle edge case where edge0 == edge1 (avoid division by zero)
        let t = if range.abs() < f32::EPSILON {
            if x < edge0 { 0.0 } else { 1.0 }
        } else {
            ((x - edge0) / range).clamp(0.0, 1.0)
        };

        let result = t * t * (3.0 - 2.0 * t);
        self.outputs[0].set_float(result);
    }
}

impl OperatorMeta for SmoothStepOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Hermite interpolation with smooth edges" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Edge0")),
            1 => Some(PortMeta::new("Edge1")),
            2 => Some(PortMeta::new("X")),
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
// Remap Operator
// ============================================================================

pub struct RemapOp {
    id: Id,
    inputs: [InputPort; 5],
    outputs: [OutputPort; 1],
}

impl RemapOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Value", 0.0),
                InputPort::float("InMin", 0.0),
                InputPort::float("InMax", 1.0),
                InputPort::float("OutMin", 0.0),
                InputPort::float("OutMax", 1.0),
            ],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for RemapOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for RemapOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Remap" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_float(&self.inputs[0], get_input);
        let in_min = get_float(&self.inputs[1], get_input);
        let in_max = get_float(&self.inputs[2], get_input);
        let out_min = get_float(&self.inputs[3], get_input);
        let out_max = get_float(&self.inputs[4], get_input);

        // Avoid division by zero
        let in_range = in_max - in_min;
        if in_range.abs() < f32::EPSILON {
            self.outputs[0].set_float(out_min);
            return;
        }

        let t = (value - in_min) / in_range;
        let result = out_min + t * (out_max - out_min);
        self.outputs[0].set_float(result);
    }
}

impl OperatorMeta for RemapOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Remaps value from one range to another" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            1 => Some(PortMeta::new("InMin")),
            2 => Some(PortMeta::new("InMax")),
            3 => Some(PortMeta::new("OutMin")),
            4 => Some(PortMeta::new("OutMax")),
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
// InverseLerp Operator
// ============================================================================

pub struct InverseLerpOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl InverseLerpOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("A", 0.0),
                InputPort::float("B", 1.0),
                InputPort::float("Value", 0.5),
            ],
            outputs: [OutputPort::float("T")],
        }
    }
}

impl Default for InverseLerpOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for InverseLerpOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "InverseLerp" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_float(&self.inputs[0], get_input);
        let b = get_float(&self.inputs[1], get_input);
        let value = get_float(&self.inputs[2], get_input);

        let range = b - a;
        if range.abs() < f32::EPSILON {
            self.outputs[0].set_float(0.0);
            return;
        }

        let t = (value - a) / range;
        self.outputs[0].set_float(t);
    }
}

impl OperatorMeta for InverseLerpOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Gets T from lerp result" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            2 => Some(PortMeta::new("Value")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("T").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// MapRange Operator (simplified Remap with Vec2 ranges)
// ============================================================================

pub struct MapRangeOp {
    id: Id,
    inputs: [InputPort; 5],
    outputs: [OutputPort; 1],
}

impl MapRangeOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Value", 0.0),
                InputPort::float("FromMin", 0.0),
                InputPort::float("FromMax", 1.0),
                InputPort::float("ToMin", 0.0),
                InputPort::float("ToMax", 1.0),
            ],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for MapRangeOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for MapRangeOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "MapRange" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_float(&self.inputs[0], get_input);
        let from_min = get_float(&self.inputs[1], get_input);
        let from_max = get_float(&self.inputs[2], get_input);
        let to_min = get_float(&self.inputs[3], get_input);
        let to_max = get_float(&self.inputs[4], get_input);

        let from_range = from_max - from_min;
        if from_range.abs() < f32::EPSILON {
            self.outputs[0].set_float(to_min);
            return;
        }

        let t = (value - from_min) / from_range;
        let result = to_min + t * (to_max - to_min);
        self.outputs[0].set_float(result);
    }
}

impl OperatorMeta for MapRangeOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Maps value from one range to another" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            1 => Some(PortMeta::new("FromMin")),
            2 => Some(PortMeta::new("FromMax")),
            3 => Some(PortMeta::new("ToMin")),
            4 => Some(PortMeta::new("ToMax")),
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
// Registration
// ============================================================================

pub fn register(registry: &OperatorRegistry) {
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Lerp",
            category: "Math",
            description: "Linear interpolation between A and B",
        },
        || capture_meta(LerpOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "SmoothStep",
            category: "Math",
            description: "Hermite interpolation with smooth edges",
        },
        || capture_meta(SmoothStepOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Remap",
            category: "Math",
            description: "Remaps value from one range to another",
        },
        || capture_meta(RemapOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "InverseLerp",
            category: "Math",
            description: "Gets T from lerp result",
        },
        || capture_meta(InverseLerpOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "MapRange",
            category: "Math",
            description: "Maps value from one range to another",
        },
        || capture_meta(MapRangeOp::new()),
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
    fn test_lerp() {
        let mut op = LerpOp::new();
        op.inputs[0].default = Value::Float(0.0);
        op.inputs[1].default = Value::Float(10.0);
        op.inputs[2].default = Value::Float(0.5);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(5.0));
    }

    #[test]
    fn test_lerp_extrapolation() {
        let mut op = LerpOp::new();
        op.inputs[0].default = Value::Float(0.0);
        op.inputs[1].default = Value::Float(10.0);
        op.inputs[2].default = Value::Float(1.5); // Extrapolate beyond
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(15.0));
    }

    #[test]
    fn test_smoothstep() {
        let mut op = SmoothStepOp::new();
        op.inputs[0].default = Value::Float(0.0);
        op.inputs[1].default = Value::Float(1.0);
        op.inputs[2].default = Value::Float(0.5);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(0.5));
    }

    #[test]
    fn test_smoothstep_edges() {
        let mut op = SmoothStepOp::new();
        let ctx = EvalContext::new();

        // Before edge0
        op.inputs[0].default = Value::Float(0.0);
        op.inputs[1].default = Value::Float(1.0);
        op.inputs[2].default = Value::Float(-0.5);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(0.0));

        // After edge1
        op.inputs[2].default = Value::Float(1.5);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(1.0));
    }

    #[test]
    fn test_remap() {
        let mut op = RemapOp::new();
        op.inputs[0].default = Value::Float(5.0);   // Value
        op.inputs[1].default = Value::Float(0.0);   // InMin
        op.inputs[2].default = Value::Float(10.0);  // InMax
        op.inputs[3].default = Value::Float(0.0);   // OutMin
        op.inputs[4].default = Value::Float(100.0); // OutMax
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(50.0));
    }

    #[test]
    fn test_inverse_lerp() {
        let mut op = InverseLerpOp::new();
        op.inputs[0].default = Value::Float(0.0);
        op.inputs[1].default = Value::Float(10.0);
        op.inputs[2].default = Value::Float(5.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(0.5));
    }

    #[test]
    fn test_map_range() {
        let mut op = MapRangeOp::new();
        op.inputs[0].default = Value::Float(0.5);  // Value
        op.inputs[1].default = Value::Float(0.0);  // FromMin
        op.inputs[2].default = Value::Float(1.0);  // FromMax
        op.inputs[3].default = Value::Float(100.0); // ToMin
        op.inputs[4].default = Value::Float(200.0); // ToMax
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(150.0));
    }
}
