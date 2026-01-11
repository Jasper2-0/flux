//! Integer operators: IntAdd, IntMultiply, IntDivide, IntModulo, IntClamp, IntToFloat

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};

fn get_int(input: &InputPort, get_input: InputResolver) -> i32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_int().unwrap_or(0),
        None => input.default.as_int().unwrap_or(0),
    }
}

// ============================================================================
// IntAdd Operator
// ============================================================================

pub struct IntAddOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl IntAddOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::int("A", 0), InputPort::int("B", 0)],
            outputs: [OutputPort::int("Result")],
        }
    }
}

impl Default for IntAddOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for IntAddOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "IntAdd" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_int(&self.inputs[0], get_input);
        let b = get_int(&self.inputs[1], get_input);
        self.outputs[0].set_int(a.wrapping_add(b));
    }
}

impl OperatorMeta for IntAddOp {
    fn category(&self) -> &'static str { "Logic" }
    fn category_color(&self) -> [f32; 4] { category_colors::LOGIC }
    fn description(&self) -> &'static str { "Adds two integers" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
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
// IntMultiply Operator
// ============================================================================

pub struct IntMultiplyOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl IntMultiplyOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::int("A", 0), InputPort::int("B", 1)],
            outputs: [OutputPort::int("Result")],
        }
    }
}

impl Default for IntMultiplyOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for IntMultiplyOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "IntMultiply" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_int(&self.inputs[0], get_input);
        let b = get_int(&self.inputs[1], get_input);
        self.outputs[0].set_int(a.wrapping_mul(b));
    }
}

impl OperatorMeta for IntMultiplyOp {
    fn category(&self) -> &'static str { "Logic" }
    fn category_color(&self) -> [f32; 4] { category_colors::LOGIC }
    fn description(&self) -> &'static str { "Multiplies two integers" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
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
// IntDivide Operator
// ============================================================================

pub struct IntDivideOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl IntDivideOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::int("A", 0), InputPort::int("B", 1)],
            outputs: [OutputPort::int("Result")],
        }
    }
}

impl Default for IntDivideOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for IntDivideOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "IntDivide" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_int(&self.inputs[0], get_input);
        let b = get_int(&self.inputs[1], get_input);
        // Division by zero returns 0
        let result = if b == 0 { 0 } else { a / b };
        self.outputs[0].set_int(result);
    }
}

impl OperatorMeta for IntDivideOp {
    fn category(&self) -> &'static str { "Logic" }
    fn category_color(&self) -> [f32; 4] { category_colors::LOGIC }
    fn description(&self) -> &'static str { "Divides two integers" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
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
// IntModulo Operator
// ============================================================================

pub struct IntModuloOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl IntModuloOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::int("A", 0), InputPort::int("B", 1)],
            outputs: [OutputPort::int("Result")],
        }
    }
}

impl Default for IntModuloOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for IntModuloOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "IntModulo" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_int(&self.inputs[0], get_input);
        let b = get_int(&self.inputs[1], get_input);
        let result = if b == 0 { 0 } else { a % b };
        self.outputs[0].set_int(result);
    }
}

impl OperatorMeta for IntModuloOp {
    fn category(&self) -> &'static str { "Logic" }
    fn category_color(&self) -> [f32; 4] { category_colors::LOGIC }
    fn description(&self) -> &'static str { "Returns remainder of integer division" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
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
// IntClamp Operator
// ============================================================================

pub struct IntClampOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl IntClampOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::int("Value", 0),
                InputPort::int("Min", 0),
                InputPort::int("Max", 100),
            ],
            outputs: [OutputPort::int("Result")],
        }
    }
}

impl Default for IntClampOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for IntClampOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "IntClamp" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_int(&self.inputs[0], get_input);
        let min = get_int(&self.inputs[1], get_input);
        let max = get_int(&self.inputs[2], get_input);
        self.outputs[0].set_int(value.clamp(min, max));
    }
}

impl OperatorMeta for IntClampOp {
    fn category(&self) -> &'static str { "Logic" }
    fn category_color(&self) -> [f32; 4] { category_colors::LOGIC }
    fn description(&self) -> &'static str { "Clamps an integer to a range" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            1 => Some(PortMeta::new("Min")),
            2 => Some(PortMeta::new("Max")),
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
// IntToFloat Operator
// ============================================================================

pub struct IntToFloatOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl IntToFloatOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::int("Value", 0)],
            outputs: [OutputPort::float("Result")],
        }
    }
}

impl Default for IntToFloatOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for IntToFloatOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "IntToFloat" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_int(&self.inputs[0], get_input);
        self.outputs[0].set_float(value as f32);
    }
}

impl OperatorMeta for IntToFloatOp {
    fn category(&self) -> &'static str { "Logic" }
    fn category_color(&self) -> [f32; 4] { category_colors::LOGIC }
    fn description(&self) -> &'static str { "Converts an integer to a float" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
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
            name: "IntAdd",
            category: "Logic",
            description: "Integer addition",
        },
        || capture_meta(IntAddOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "IntMultiply",
            category: "Logic",
            description: "Integer multiplication",
        },
        || capture_meta(IntMultiplyOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "IntDivide",
            category: "Logic",
            description: "Integer division",
        },
        || capture_meta(IntDivideOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "IntModulo",
            category: "Logic",
            description: "Integer modulo",
        },
        || capture_meta(IntModuloOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "IntClamp",
            category: "Logic",
            description: "Clamp integer to range",
        },
        || capture_meta(IntClampOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "IntToFloat",
            category: "Logic",
            description: "Convert integer to float",
        },
        || capture_meta(IntToFloatOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::Value;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Int(0)
    }

    #[test]
    fn test_int_add() {
        let mut op = IntAddOp::new();
        op.inputs[0].default = Value::Int(5);
        op.inputs[1].default = Value::Int(3);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(8));
    }

    #[test]
    fn test_int_multiply() {
        let mut op = IntMultiplyOp::new();
        op.inputs[0].default = Value::Int(4);
        op.inputs[1].default = Value::Int(3);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(12));
    }

    #[test]
    fn test_int_divide() {
        let mut op = IntDivideOp::new();
        op.inputs[0].default = Value::Int(10);
        op.inputs[1].default = Value::Int(3);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(3));
    }

    #[test]
    fn test_int_divide_by_zero() {
        let mut op = IntDivideOp::new();
        op.inputs[0].default = Value::Int(10);
        op.inputs[1].default = Value::Int(0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(0));
    }

    #[test]
    fn test_int_modulo() {
        let mut op = IntModuloOp::new();
        op.inputs[0].default = Value::Int(10);
        op.inputs[1].default = Value::Int(3);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(1));
    }

    #[test]
    fn test_int_clamp() {
        let mut op = IntClampOp::new();
        op.inputs[0].default = Value::Int(150);
        op.inputs[1].default = Value::Int(0);
        op.inputs[2].default = Value::Int(100);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(100));
    }

    #[test]
    fn test_int_to_float() {
        let mut op = IntToFloatOp::new();
        op.inputs[0].default = Value::Int(42);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(42.0));
    }
}
