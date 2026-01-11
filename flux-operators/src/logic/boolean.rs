//! Boolean logic operators: And, Or, Not, Xor, All, Any

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};

fn get_bool(input: &InputPort, get_input: InputResolver) -> bool {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_bool().unwrap_or(false),
        None => input.default.as_bool().unwrap_or(false),
    }
}

// ============================================================================
// And Operator
// ============================================================================

pub struct AndOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl AndOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::bool("A", false), InputPort::bool("B", false)],
            outputs: [OutputPort::bool("Result")],
        }
    }
}

impl Default for AndOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for AndOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "And" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_bool(&self.inputs[0], get_input);
        let b = get_bool(&self.inputs[1], get_input);
        self.outputs[0].set_bool(a && b);
    }
}

impl OperatorMeta for AndOp {
    fn category(&self) -> &'static str { "Logic" }
    fn category_color(&self) -> [f32; 4] { category_colors::LOGIC }
    fn description(&self) -> &'static str { "Logical AND of two booleans" }
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
// Or Operator
// ============================================================================

pub struct OrOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl OrOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::bool("A", false), InputPort::bool("B", false)],
            outputs: [OutputPort::bool("Result")],
        }
    }
}

impl Default for OrOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for OrOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Or" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_bool(&self.inputs[0], get_input);
        let b = get_bool(&self.inputs[1], get_input);
        self.outputs[0].set_bool(a || b);
    }
}

impl OperatorMeta for OrOp {
    fn category(&self) -> &'static str { "Logic" }
    fn category_color(&self) -> [f32; 4] { category_colors::LOGIC }
    fn description(&self) -> &'static str { "Logical OR of two booleans" }
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
// Not Operator
// ============================================================================

pub struct NotOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl NotOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::bool("Value", false)],
            outputs: [OutputPort::bool("Result")],
        }
    }
}

impl Default for NotOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for NotOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Not" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_bool(&self.inputs[0], get_input);
        self.outputs[0].set_bool(!value);
    }
}

impl OperatorMeta for NotOp {
    fn category(&self) -> &'static str { "Logic" }
    fn category_color(&self) -> [f32; 4] { category_colors::LOGIC }
    fn description(&self) -> &'static str { "Logical NOT" }
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
// Xor Operator
// ============================================================================

pub struct XorOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl XorOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::bool("A", false), InputPort::bool("B", false)],
            outputs: [OutputPort::bool("Result")],
        }
    }
}

impl Default for XorOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for XorOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Xor" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_bool(&self.inputs[0], get_input);
        let b = get_bool(&self.inputs[1], get_input);
        self.outputs[0].set_bool(a ^ b);
    }
}

impl OperatorMeta for XorOp {
    fn category(&self) -> &'static str { "Logic" }
    fn category_color(&self) -> [f32; 4] { category_colors::LOGIC }
    fn description(&self) -> &'static str { "Exclusive OR" }
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
// All Operator (multi-input)
// ============================================================================

pub struct AllOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: [OutputPort; 1],
}

impl AllOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![InputPort::bool_multi("Values")],
            outputs: [OutputPort::bool("Result")],
        }
    }
}

impl Default for AllOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for AllOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "All" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let input = &self.inputs[0];

        if input.connections.is_empty() {
            // No inputs = true (vacuously true)
            self.outputs[0].set_bool(true);
            return;
        }

        let result = input
            .connections
            .iter()
            .all(|&(node_id, output_idx)| {
                get_input(node_id, output_idx).as_bool().unwrap_or(false)
            });

        self.outputs[0].set_bool(result);
    }
}

impl OperatorMeta for AllOp {
    fn category(&self) -> &'static str { "Logic" }
    fn category_color(&self) -> [f32; 4] { category_colors::LOGIC }
    fn description(&self) -> &'static str { "True if all inputs are true" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Values")),
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
// Any Operator (multi-input)
// ============================================================================

pub struct AnyOp {
    id: Id,
    inputs: Vec<InputPort>,
    outputs: [OutputPort; 1],
}

impl AnyOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: vec![InputPort::bool_multi("Values")],
            outputs: [OutputPort::bool("Result")],
        }
    }
}

impl Default for AnyOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for AnyOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Any" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let input = &self.inputs[0];

        if input.connections.is_empty() {
            // No inputs = false
            self.outputs[0].set_bool(false);
            return;
        }

        let result = input
            .connections
            .iter()
            .any(|&(node_id, output_idx)| {
                get_input(node_id, output_idx).as_bool().unwrap_or(false)
            });

        self.outputs[0].set_bool(result);
    }
}

impl OperatorMeta for AnyOp {
    fn category(&self) -> &'static str { "Logic" }
    fn category_color(&self) -> [f32; 4] { category_colors::LOGIC }
    fn description(&self) -> &'static str { "True if any input is true" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Values")),
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
            name: "And",
            category: "Logic",
            description: "Logical AND of two booleans",
        },
        || capture_meta(AndOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Or",
            category: "Logic",
            description: "Logical OR of two booleans",
        },
        || capture_meta(OrOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Not",
            category: "Logic",
            description: "Logical NOT",
        },
        || capture_meta(NotOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Xor",
            category: "Logic",
            description: "Exclusive OR",
        },
        || capture_meta(XorOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "All",
            category: "Logic",
            description: "True if all inputs are true",
        },
        || capture_meta(AllOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Any",
            category: "Logic",
            description: "True if any input is true",
        },
        || capture_meta(AnyOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::Value;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Bool(false)
    }

    #[test]
    fn test_and() {
        let mut op = AndOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Bool(true);
        op.inputs[1].default = Value::Bool(true);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(true));

        op.inputs[1].default = Value::Bool(false);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(false));
    }

    #[test]
    fn test_or() {
        let mut op = OrOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Bool(false);
        op.inputs[1].default = Value::Bool(true);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(true));

        op.inputs[1].default = Value::Bool(false);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(false));
    }

    #[test]
    fn test_not() {
        let mut op = NotOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Bool(true);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(false));

        op.inputs[0].default = Value::Bool(false);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(true));
    }

    #[test]
    fn test_xor() {
        let mut op = XorOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Bool(true);
        op.inputs[1].default = Value::Bool(false);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(true));

        op.inputs[1].default = Value::Bool(true);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(false));
    }
}
