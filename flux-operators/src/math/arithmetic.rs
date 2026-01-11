//! Basic arithmetic operators

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};

/// Helper to get float from input slot
fn get_float(input: &InputPort, get_input: InputResolver) -> f32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
        None => input.default.as_float().unwrap_or(0.0),
    }
}

// ============================================================================
// Subtract Operator
// ============================================================================

pub struct SubtractOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl SubtractOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("A", 0.0), InputPort::float("B", 0.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for SubtractOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SubtractOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Subtract" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_float(&self.inputs[0], get_input);
        let b = get_float(&self.inputs[1], get_input);
        self.outputs[0].set_float(a - b);
    }
}

impl OperatorMeta for SubtractOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Subtracts B from A" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Diff").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Divide Operator
// ============================================================================

pub struct DivideOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl DivideOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("A", 0.0), InputPort::float("B", 1.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for DivideOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for DivideOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Divide" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_float(&self.inputs[0], get_input);
        let b = get_float(&self.inputs[1], get_input);
        // Division by zero returns infinity (matching GLSL behavior)
        self.outputs[0].set_float(a / b);
    }
}

impl OperatorMeta for DivideOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Divides A by B" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Quot").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Modulo Operator
// ============================================================================

pub struct ModuloOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl ModuloOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("A", 0.0), InputPort::float("B", 1.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for ModuloOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for ModuloOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Modulo" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_float(&self.inputs[0], get_input);
        let b = get_float(&self.inputs[1], get_input);
        self.outputs[0].set_float(a % b);
    }
}

impl OperatorMeta for ModuloOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "A modulo B" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Mod").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Pow Operator
// ============================================================================

pub struct PowOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl PowOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Base", 0.0), InputPort::float("Exponent", 1.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for PowOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for PowOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Pow" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let base = get_float(&self.inputs[0], get_input);
        let exp = get_float(&self.inputs[1], get_input);
        self.outputs[0].set_float(base.powf(exp));
    }
}

impl OperatorMeta for PowOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Raises base to exponent power" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Base")),
            1 => Some(PortMeta::new("Exp")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Pow").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Sqrt Operator
// ============================================================================

pub struct SqrtOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl SqrtOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Value", 0.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for SqrtOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SqrtOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Sqrt" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_float(&self.inputs[0], get_input);
        self.outputs[0].set_float(value.sqrt());
    }
}

impl OperatorMeta for SqrtOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Square root of value" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Sqrt").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Log Operator
// ============================================================================

pub struct LogOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl LogOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Value", 1.0),
                InputPort::float("Base", std::f32::consts::E),
            ],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for LogOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for LogOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Log" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_float(&self.inputs[0], get_input);
        let base = get_float(&self.inputs[1], get_input);
        self.outputs[0].set_float(value.log(base));
    }
}

impl OperatorMeta for LogOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Logarithm of value with base" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            1 => Some(PortMeta::new("Base")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Log").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Abs Operator
// ============================================================================

pub struct AbsOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl AbsOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Value", 0.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for AbsOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for AbsOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Abs" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_float(&self.inputs[0], get_input);
        self.outputs[0].set_float(value.abs());
    }
}

impl OperatorMeta for AbsOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Absolute value" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Abs").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Negate Operator
// ============================================================================

pub struct NegateOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl NegateOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Value", 0.0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for NegateOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for NegateOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Negate" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_float(&self.inputs[0], get_input);
        self.outputs[0].set_float(-value);
    }
}

impl OperatorMeta for NegateOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { "Negates the value" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Neg").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Registration
// ============================================================================

pub fn register(registry: &OperatorRegistry) {
    // Note: Add and Multiply are already in the legacy operator.rs
    // We register the new arithmetic operators here

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Subtract",
            category: "Math",
            description: "Subtracts B from A",
        },
        || capture_meta(SubtractOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Divide",
            category: "Math",
            description: "Divides A by B",
        },
        || capture_meta(DivideOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Modulo",
            category: "Math",
            description: "A modulo B",
        },
        || capture_meta(ModuloOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Pow",
            category: "Math",
            description: "Raises base to exponent power",
        },
        || capture_meta(PowOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Sqrt",
            category: "Math",
            description: "Square root of value",
        },
        || capture_meta(SqrtOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Log",
            category: "Math",
            description: "Logarithm of value with base",
        },
        || capture_meta(LogOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Abs",
            category: "Math",
            description: "Absolute value",
        },
        || capture_meta(AbsOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Negate",
            category: "Math",
            description: "Negates the value",
        },
        || capture_meta(NegateOp::new()),
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
    fn test_subtract() {
        let mut op = SubtractOp::new();
        op.inputs[0].default = Value::Float(10.0);
        op.inputs[1].default = Value::Float(3.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(7.0));
    }

    #[test]
    fn test_divide() {
        let mut op = DivideOp::new();
        op.inputs[0].default = Value::Float(10.0);
        op.inputs[1].default = Value::Float(2.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(5.0));
    }

    #[test]
    fn test_divide_by_zero() {
        let mut op = DivideOp::new();
        op.inputs[0].default = Value::Float(1.0);
        op.inputs[1].default = Value::Float(0.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert!(op.outputs[0].value.as_float().unwrap().is_infinite());
    }

    #[test]
    fn test_modulo() {
        let mut op = ModuloOp::new();
        op.inputs[0].default = Value::Float(10.0);
        op.inputs[1].default = Value::Float(3.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(1.0));
    }

    #[test]
    fn test_pow() {
        let mut op = PowOp::new();
        op.inputs[0].default = Value::Float(2.0);
        op.inputs[1].default = Value::Float(3.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(8.0));
    }

    #[test]
    fn test_sqrt() {
        let mut op = SqrtOp::new();
        op.inputs[0].default = Value::Float(16.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(4.0));
    }

    #[test]
    fn test_log() {
        let mut op = LogOp::new();
        op.inputs[0].default = Value::Float(100.0);
        op.inputs[1].default = Value::Float(10.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        let result = op.outputs[0].value.as_float().unwrap();
        assert!((result - 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_abs() {
        let mut op = AbsOp::new();
        op.inputs[0].default = Value::Float(-5.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(5.0));
    }

    #[test]
    fn test_negate() {
        let mut op = NegateOp::new();
        op.inputs[0].default = Value::Float(5.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(-5.0));
    }
}
