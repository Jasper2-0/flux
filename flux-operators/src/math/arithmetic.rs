//! Arithmetic operators - polymorphic versions that work with Float, Int, Vec2, Vec3, Vec4, Color
//!
//! All arithmetic operators are polymorphic. A single `AddOp` works with any arithmetic type.
//! Broadcasting is automatic: Float + Vec3 = Vec3 (scalar broadcasts to all components).
//!
//! ## Usage
//!
//! ```ignore
//! use flux_operators::{AddOp, MultiplyOp};
//!
//! let add = AddOp::new();      // Works with Float, Int, Vec3, etc.
//! let mul = MultiplyOp::new(); // Scalar * Vec3 = scaled Vec3
//! ```

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::port::{InputPort, OutputPort};
use flux_core::value::Value;
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};

use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};

// ============================================================================
// Binary Arithmetic Operations
// ============================================================================

/// The binary arithmetic operation to perform
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl BinaryArithOp {
    /// Get the operator name
    pub fn name(&self) -> &'static str {
        match self {
            BinaryArithOp::Add => "Add",
            BinaryArithOp::Sub => "Subtract",
            BinaryArithOp::Mul => "Multiply",
            BinaryArithOp::Div => "Divide",
            BinaryArithOp::Mod => "Modulo",
        }
    }

    /// Get the operator description
    pub fn description(&self) -> &'static str {
        match self {
            BinaryArithOp::Add => "Adds two values",
            BinaryArithOp::Sub => "Subtracts B from A",
            BinaryArithOp::Mul => "Multiplies two values",
            BinaryArithOp::Div => "Divides A by B",
            BinaryArithOp::Mod => "A modulo B",
        }
    }

    /// Get the output label
    pub fn output_label(&self) -> &'static str {
        match self {
            BinaryArithOp::Add => "Sum",
            BinaryArithOp::Sub => "Diff",
            BinaryArithOp::Mul => "Prod",
            BinaryArithOp::Div => "Quot",
            BinaryArithOp::Mod => "Rem",
        }
    }

    /// Apply the operation to two values
    pub fn apply(&self, a: Value, b: Value) -> Value {
        let result = match self {
            BinaryArithOp::Add => a + b,
            BinaryArithOp::Sub => a - b,
            BinaryArithOp::Mul => a * b,
            BinaryArithOp::Div => a / b,
            BinaryArithOp::Mod => a % b,
        };
        result.unwrap_or(Value::Float(0.0))
    }
}

/// Polymorphic binary arithmetic operator
///
/// Handles Add, Subtract, Multiply, Divide, and Modulo for all arithmetic types.
pub struct BinaryOp {
    id: Id,
    op: BinaryArithOp,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl BinaryOp {
    /// Create a new binary operator with the given operation
    pub fn new(op: BinaryArithOp) -> Self {
        let default_b = match op {
            BinaryArithOp::Mul | BinaryArithOp::Div | BinaryArithOp::Mod => Value::Float(1.0),
            _ => Value::Float(0.0),
        };

        Self {
            id: Id::new(),
            op,
            inputs: vec![
                InputPort::arithmetic("A", Value::Float(0.0)),
                InputPort::arithmetic("B", default_b),
            ],
            outputs: vec![OutputPort::wider_of_inputs("Result")],
        }
    }

    /// Create an Add operator
    pub fn add() -> Self {
        Self::new(BinaryArithOp::Add)
    }

    /// Create a Subtract operator
    pub fn sub() -> Self {
        Self::new(BinaryArithOp::Sub)
    }

    /// Create a Multiply operator
    pub fn mul() -> Self {
        Self::new(BinaryArithOp::Mul)
    }

    /// Create a Divide operator
    pub fn div() -> Self {
        Self::new(BinaryArithOp::Div)
    }

    /// Create a Modulo operator
    pub fn modulo() -> Self {
        Self::new(BinaryArithOp::Mod)
    }

    fn get_value(&self, index: usize, get_input: InputResolver) -> Value {
        let input = &self.inputs[index];
        match input.connection {
            Some((node_id, output_idx)) => get_input(node_id, output_idx),
            None => input.default.clone(),
        }
    }
}

impl Default for BinaryOp {
    fn default() -> Self {
        Self::add()
    }
}

impl Operator for BinaryOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { self.op.name() }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_value(0, get_input);
        let b = self.get_value(1, get_input);

        let input_types = vec![Some(a.value_type()), Some(b.value_type())];
        self.outputs[0].resolve_type(&input_types);

        let result = self.op.apply(a, b);
        self.outputs[0].set(result);
    }
}

impl OperatorMeta for BinaryOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { self.op.description() }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("A")),
            1 => Some(PortMeta::new("B")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new(self.op.output_label()).with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Unary Arithmetic Operations
// ============================================================================

/// The unary arithmetic operation to perform
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryArithOp {
    Negate,
    Abs,
    Sqrt,
    Floor,
    Ceil,
    Round,
    Truncate,
}

impl UnaryArithOp {
    pub fn name(&self) -> &'static str {
        match self {
            UnaryArithOp::Negate => "Negate",
            UnaryArithOp::Abs => "Abs",
            UnaryArithOp::Sqrt => "Sqrt",
            UnaryArithOp::Floor => "Floor",
            UnaryArithOp::Ceil => "Ceil",
            UnaryArithOp::Round => "Round",
            UnaryArithOp::Truncate => "Truncate",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            UnaryArithOp::Negate => "Negates the value",
            UnaryArithOp::Abs => "Absolute value",
            UnaryArithOp::Sqrt => "Square root",
            UnaryArithOp::Floor => "Floor (round down)",
            UnaryArithOp::Ceil => "Ceiling (round up)",
            UnaryArithOp::Round => "Round to nearest integer",
            UnaryArithOp::Truncate => "Truncate toward zero",
        }
    }

    pub fn output_label(&self) -> &'static str {
        match self {
            UnaryArithOp::Negate => "Neg",
            UnaryArithOp::Abs => "Abs",
            UnaryArithOp::Sqrt => "Sqrt",
            UnaryArithOp::Floor => "Floor",
            UnaryArithOp::Ceil => "Ceil",
            UnaryArithOp::Round => "Round",
            UnaryArithOp::Truncate => "Trunc",
        }
    }

    pub fn apply(&self, value: Value) -> Value {
        match self {
            UnaryArithOp::Negate => (-value).unwrap_or(Value::Float(0.0)),
            UnaryArithOp::Abs => value.abs().unwrap_or(Value::Float(0.0)),
            UnaryArithOp::Sqrt => value.sqrt().unwrap_or(Value::Float(0.0)),
            UnaryArithOp::Floor => value.floor().unwrap_or(Value::Float(0.0)),
            UnaryArithOp::Ceil => value.ceil().unwrap_or(Value::Float(0.0)),
            UnaryArithOp::Round => value.round().unwrap_or(Value::Float(0.0)),
            UnaryArithOp::Truncate => value.trunc().unwrap_or(Value::Float(0.0)),
        }
    }
}

/// Polymorphic unary arithmetic operator
pub struct UnaryOp {
    id: Id,
    op: UnaryArithOp,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl UnaryOp {
    pub fn new(op: UnaryArithOp) -> Self {
        Self {
            id: Id::new(),
            op,
            inputs: vec![InputPort::arithmetic("Value", Value::Float(0.0))],
            outputs: vec![OutputPort::same_as_first("Result")],
        }
    }

    pub fn negate() -> Self { Self::new(UnaryArithOp::Negate) }
    pub fn abs() -> Self { Self::new(UnaryArithOp::Abs) }
    pub fn sqrt() -> Self { Self::new(UnaryArithOp::Sqrt) }
    pub fn floor() -> Self { Self::new(UnaryArithOp::Floor) }
    pub fn ceil() -> Self { Self::new(UnaryArithOp::Ceil) }
    pub fn round() -> Self { Self::new(UnaryArithOp::Round) }
    pub fn trunc() -> Self { Self::new(UnaryArithOp::Truncate) }

    fn get_value(&self, get_input: InputResolver) -> Value {
        let input = &self.inputs[0];
        match input.connection {
            Some((node_id, output_idx)) => get_input(node_id, output_idx),
            None => input.default.clone(),
        }
    }
}

impl Default for UnaryOp {
    fn default() -> Self {
        Self::negate()
    }
}

impl Operator for UnaryOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { self.op.name() }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = self.get_value(get_input);
        let input_types = vec![Some(value.value_type())];
        self.outputs[0].resolve_type(&input_types);
        let result = self.op.apply(value);
        self.outputs[0].set(result);
    }
}

impl OperatorMeta for UnaryOp {
    fn category(&self) -> &'static str { "Math" }
    fn category_color(&self) -> [f32; 4] { category_colors::MATH }
    fn description(&self) -> &'static str { self.op.description() }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new(self.op.output_label()).with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Pow Operator
// ============================================================================

/// Polymorphic power operator (base^exponent)
pub struct PowOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl PowOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![
                InputPort::arithmetic("Base", Value::Float(0.0)),
                InputPort::float("Exponent", 1.0),
            ],
            outputs: vec![OutputPort::same_as_first("Result")],
        }
    }

    fn get_value(&self, index: usize, get_input: InputResolver) -> Value {
        let input = &self.inputs[index];
        match input.connection {
            Some((node_id, output_idx)) => get_input(node_id, output_idx),
            None => input.default.clone(),
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
        let base = self.get_value(0, get_input);
        let exp = self.get_value(1, get_input);
        let input_types = vec![Some(base.value_type())];
        self.outputs[0].resolve_type(&input_types);
        let result = base.pow(&exp).unwrap_or(Value::Float(0.0));
        self.outputs[0].set(result);
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
// Log Operator (float-only)
// ============================================================================

fn get_float(input: &InputPort, get_input: InputResolver) -> f32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
        None => input.default.as_float().unwrap_or(0.0),
    }
}

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
// Factory types with simple new() constructors
// ============================================================================

/// Factory for creating an Add operator
pub struct AddOp;
impl AddOp {
    pub fn new() -> BinaryOp { BinaryOp::add() }
}

/// Factory for creating a Subtract operator
pub struct SubtractOp;
impl SubtractOp {
    pub fn new() -> BinaryOp { BinaryOp::sub() }
}

/// Factory for creating a Multiply operator
pub struct MultiplyOp;
impl MultiplyOp {
    pub fn new() -> BinaryOp { BinaryOp::mul() }
}

/// Factory for creating a Divide operator
pub struct DivideOp;
impl DivideOp {
    pub fn new() -> BinaryOp { BinaryOp::div() }
}

/// Factory for creating a Modulo operator
pub struct ModuloOp;
impl ModuloOp {
    pub fn new() -> BinaryOp { BinaryOp::modulo() }
}

/// Factory for creating a Negate operator
pub struct NegateOp;
impl NegateOp {
    pub fn new() -> UnaryOp { UnaryOp::negate() }
}

/// Factory for creating an Abs operator
pub struct AbsOp;
impl AbsOp {
    pub fn new() -> UnaryOp { UnaryOp::abs() }
}

/// Factory for creating a Sqrt operator
pub struct SqrtOp;
impl SqrtOp {
    pub fn new() -> UnaryOp { UnaryOp::sqrt() }
}

/// Factory for creating a Floor operator
pub struct FloorOp;
impl FloorOp {
    pub fn new() -> UnaryOp { UnaryOp::floor() }
}

/// Factory for creating a Ceil operator
pub struct CeilOp;
impl CeilOp {
    pub fn new() -> UnaryOp { UnaryOp::ceil() }
}

/// Factory for creating a Round operator
pub struct RoundOp;
impl RoundOp {
    pub fn new() -> UnaryOp { UnaryOp::round() }
}

/// Factory for creating a Truncate operator
pub struct TruncateOp;
impl TruncateOp {
    pub fn new() -> UnaryOp { UnaryOp::trunc() }
}

// Legacy aliases
pub type PolyBinaryOp = BinaryOp;
pub type PolyUnaryOp = UnaryOp;
pub type PolyPowOp = PowOp;

// ============================================================================
// Registration
// ============================================================================

pub fn register(registry: &OperatorRegistry) {
    // Binary operators
    registry.register(
        RegistryEntry { type_id: Id::new(), name: "Add", category: "Math", description: "Adds two values" },
        || capture_meta(BinaryOp::add()),
    );
    registry.register(
        RegistryEntry { type_id: Id::new(), name: "Subtract", category: "Math", description: "Subtracts B from A" },
        || capture_meta(BinaryOp::sub()),
    );
    registry.register(
        RegistryEntry { type_id: Id::new(), name: "Multiply", category: "Math", description: "Multiplies two values" },
        || capture_meta(BinaryOp::mul()),
    );
    registry.register(
        RegistryEntry { type_id: Id::new(), name: "Divide", category: "Math", description: "Divides A by B" },
        || capture_meta(BinaryOp::div()),
    );
    registry.register(
        RegistryEntry { type_id: Id::new(), name: "Modulo", category: "Math", description: "A modulo B" },
        || capture_meta(BinaryOp::modulo()),
    );

    // Unary operators
    registry.register(
        RegistryEntry { type_id: Id::new(), name: "Negate", category: "Math", description: "Negates the value" },
        || capture_meta(UnaryOp::negate()),
    );
    registry.register(
        RegistryEntry { type_id: Id::new(), name: "Abs", category: "Math", description: "Absolute value" },
        || capture_meta(UnaryOp::abs()),
    );
    registry.register(
        RegistryEntry { type_id: Id::new(), name: "Sqrt", category: "Math", description: "Square root" },
        || capture_meta(UnaryOp::sqrt()),
    );
    registry.register(
        RegistryEntry { type_id: Id::new(), name: "Floor", category: "Math", description: "Floor (round down)" },
        || capture_meta(UnaryOp::floor()),
    );
    registry.register(
        RegistryEntry { type_id: Id::new(), name: "Ceil", category: "Math", description: "Ceiling (round up)" },
        || capture_meta(UnaryOp::ceil()),
    );
    registry.register(
        RegistryEntry { type_id: Id::new(), name: "Round", category: "Math", description: "Round to nearest integer" },
        || capture_meta(UnaryOp::round()),
    );
    registry.register(
        RegistryEntry { type_id: Id::new(), name: "Truncate", category: "Math", description: "Truncate toward zero" },
        || capture_meta(UnaryOp::trunc()),
    );

    // Pow and Log
    registry.register(
        RegistryEntry { type_id: Id::new(), name: "Pow", category: "Math", description: "Raises base to exponent power" },
        || capture_meta(PowOp::new()),
    );
    registry.register(
        RegistryEntry { type_id: Id::new(), name: "Log", category: "Math", description: "Logarithm of value with base" },
        || capture_meta(LogOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Float(0.0)
    }

    #[test]
    fn test_add_floats() {
        let mut op = BinaryOp::add();
        op.inputs[0].default = Value::Float(3.0);
        op.inputs[1].default = Value::Float(4.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(7.0));
    }

    #[test]
    fn test_add_ints() {
        let mut op = BinaryOp::add();
        op.inputs[0].default = Value::Int(3);
        op.inputs[1].default = Value::Int(4);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(7));
    }

    #[test]
    fn test_add_vec3() {
        let mut op = BinaryOp::add();
        op.inputs[0].default = Value::Vec3([1.0, 2.0, 3.0]);
        op.inputs[1].default = Value::Vec3([4.0, 5.0, 6.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_vec3(), Some([5.0, 7.0, 9.0]));
    }

    #[test]
    fn test_float_vec3_broadcast() {
        let mut op = BinaryOp::add();
        op.inputs[0].default = Value::Float(10.0);
        op.inputs[1].default = Value::Vec3([1.0, 2.0, 3.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_vec3(), Some([11.0, 12.0, 13.0]));
    }

    #[test]
    fn test_scalar_vec3_multiply() {
        let mut op = BinaryOp::mul();
        op.inputs[0].default = Value::Float(2.0);
        op.inputs[1].default = Value::Vec3([1.0, 2.0, 3.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_vec3(), Some([2.0, 4.0, 6.0]));
    }

    #[test]
    fn test_subtract() {
        let mut op = BinaryOp::sub();
        op.inputs[0].default = Value::Float(10.0);
        op.inputs[1].default = Value::Float(3.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(7.0));
    }

    #[test]
    fn test_divide() {
        let mut op = BinaryOp::div();
        op.inputs[0].default = Value::Float(10.0);
        op.inputs[1].default = Value::Float(2.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(5.0));
    }

    #[test]
    fn test_divide_by_zero() {
        let mut op = BinaryOp::div();
        op.inputs[0].default = Value::Float(1.0);
        op.inputs[1].default = Value::Float(0.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert!(op.outputs[0].value.as_float().unwrap().is_infinite());
    }

    #[test]
    fn test_modulo() {
        let mut op = BinaryOp::modulo();
        op.inputs[0].default = Value::Float(10.0);
        op.inputs[1].default = Value::Float(3.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(1.0));
    }

    #[test]
    fn test_negate() {
        let mut op = UnaryOp::negate();
        op.inputs[0].default = Value::Float(5.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(-5.0));
    }

    #[test]
    fn test_negate_vec3() {
        let mut op = UnaryOp::negate();
        op.inputs[0].default = Value::Vec3([1.0, -2.0, 3.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_vec3(), Some([-1.0, 2.0, -3.0]));
    }

    #[test]
    fn test_abs() {
        let mut op = UnaryOp::abs();
        op.inputs[0].default = Value::Float(-5.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(5.0));
    }

    #[test]
    fn test_sqrt() {
        let mut op = UnaryOp::sqrt();
        op.inputs[0].default = Value::Float(16.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(4.0));
    }

    #[test]
    fn test_sqrt_vec3() {
        let mut op = UnaryOp::sqrt();
        op.inputs[0].default = Value::Vec3([4.0, 9.0, 16.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_vec3(), Some([2.0, 3.0, 4.0]));
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
    fn test_pow_vec3() {
        let mut op = PowOp::new();
        op.inputs[0].default = Value::Vec3([2.0, 3.0, 4.0]);
        op.inputs[1].default = Value::Float(2.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_vec3(), Some([4.0, 9.0, 16.0]));
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
    fn test_type_resolution() {
        let mut op = BinaryOp::add();
        op.inputs[0].default = Value::Float(1.0);
        op.inputs[1].default = Value::Vec3([1.0, 2.0, 3.0]);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].effective_type(), flux_core::ValueType::Vec3);
    }

    #[test]
    fn test_int_preservation() {
        let mut op = BinaryOp::add();
        op.inputs[0].default = Value::Int(3);
        op.inputs[1].default = Value::Int(4);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].effective_type(), flux_core::ValueType::Int);
        assert_eq!(op.outputs[0].value.as_int(), Some(7));
    }

    #[test]
    fn test_int_float_promotion() {
        let mut op = BinaryOp::add();
        op.inputs[0].default = Value::Int(3);
        op.inputs[1].default = Value::Float(4.5);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].effective_type(), flux_core::ValueType::Float);
        assert_eq!(op.outputs[0].value.as_float(), Some(7.5));
    }

    // Test factory methods work
    #[test]
    fn test_factory_add() {
        let mut op = AddOp::new();
        op.inputs[0].default = Value::Float(1.0);
        op.inputs[1].default = Value::Float(2.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(3.0));
    }

    #[test]
    fn test_factory_multiply() {
        let mut op = MultiplyOp::new();
        op.inputs[0].default = Value::Float(3.0);
        op.inputs[1].default = Value::Float(4.0);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_float(), Some(12.0));
    }
}
