//! Integer operators: IntAdd, IntMultiply, IntDivide, IntModulo, IntClamp, IntToFloat

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::OperatorMeta;
use flux_macros::Operator;
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};

// ============================================================================
// IntAdd Operator
// ============================================================================

#[derive(Operator)]
#[operator(name = "IntAdd", category = "Logic", description = "Adds two integers")]
#[operator(category_color = [0.55, 0.45, 0.25, 1.0])]
#[allow(dead_code)]
pub struct IntAddOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "A", default = 0)]
    a: i32,
    #[input(label = "B", default = 0)]
    b: i32,
    #[output(label = "Result")]
    result: i32,
}

impl IntAddOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_a(get_input);
        let b = self.get_b(get_input);
        self.set_result(a.wrapping_add(b));
    }
}

// ============================================================================
// IntMultiply Operator
// ============================================================================

#[derive(Operator)]
#[operator(name = "IntMultiply", category = "Logic", description = "Multiplies two integers")]
#[operator(category_color = [0.55, 0.45, 0.25, 1.0])]
#[allow(dead_code)]
pub struct IntMultiplyOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "A", default = 0)]
    a: i32,
    #[input(label = "B", default = 1)]
    b: i32,
    #[output(label = "Result")]
    result: i32,
}

impl IntMultiplyOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_a(get_input);
        let b = self.get_b(get_input);
        self.set_result(a.wrapping_mul(b));
    }
}

// ============================================================================
// IntDivide Operator
// ============================================================================

#[derive(Operator)]
#[operator(name = "IntDivide", category = "Logic", description = "Divides two integers")]
#[operator(category_color = [0.55, 0.45, 0.25, 1.0])]
#[allow(dead_code)]
pub struct IntDivideOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "A", default = 0)]
    a: i32,
    #[input(label = "B", default = 1)]
    b: i32,
    #[output(label = "Result")]
    result: i32,
}

impl IntDivideOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_a(get_input);
        let b = self.get_b(get_input);
        // Division by zero returns 0
        self.set_result(if b == 0 { 0 } else { a / b });
    }
}

// ============================================================================
// IntModulo Operator
// ============================================================================

#[derive(Operator)]
#[operator(name = "IntModulo", category = "Logic", description = "Returns remainder of integer division")]
#[operator(category_color = [0.55, 0.45, 0.25, 1.0])]
#[allow(dead_code)]
pub struct IntModuloOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "A", default = 0)]
    a: i32,
    #[input(label = "B", default = 1)]
    b: i32,
    #[output(label = "Result")]
    result: i32,
}

impl IntModuloOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_a(get_input);
        let b = self.get_b(get_input);
        self.set_result(if b == 0 { 0 } else { a % b });
    }
}

// ============================================================================
// IntClamp Operator
// ============================================================================

#[derive(Operator)]
#[operator(name = "IntClamp", category = "Logic", description = "Clamps an integer to a range")]
#[operator(category_color = [0.55, 0.45, 0.25, 1.0])]
#[allow(dead_code)]
pub struct IntClampOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
    #[input(label = "Value", default = 0)]
    value: i32,
    #[input(label = "Min", default = 0)]
    min: i32,
    #[input(label = "Max", default = 100)]
    max: i32,
    #[output(label = "Result")]
    result: i32,
}

impl IntClampOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = self.get_value(get_input);
        let min = self.get_min(get_input);
        let max = self.get_max(get_input);
        self.set_result(value.clamp(min, max));
    }
}

// ============================================================================
// IntToFloat Operator
// ============================================================================

#[derive(Operator)]
#[operator(name = "IntToFloat", category = "Logic", description = "Converts an integer to a float")]
#[operator(category_color = [0.55, 0.45, 0.25, 1.0])]
#[allow(dead_code)]
pub struct IntToFloatOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    #[input(label = "Value", default = 0)]
    value: i32,
    #[output(label = "Result")]
    result: f32,
}

impl IntToFloatOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = self.get_value(get_input);
        self.set_result(value as f32);
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
