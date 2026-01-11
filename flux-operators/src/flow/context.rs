//! Context variable operators: GetFloatVar, SetFloatVar, GetIntVar

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};

fn get_string(input: &InputPort, get_input: InputResolver) -> String {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx)
            .as_string()
            .unwrap_or_default()
            .to_string(),
        None => input.default.as_string().unwrap_or_default().to_string(),
    }
}

fn get_float(input: &InputPort, get_input: InputResolver) -> f32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_float().unwrap_or(0.0),
        None => input.default.as_float().unwrap_or(0.0),
    }
}

fn get_int(input: &InputPort, get_input: InputResolver) -> i32 {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_int().unwrap_or(0),
        None => input.default.as_int().unwrap_or(0),
    }
}

// ============================================================================
// GetFloatVar Operator
// ============================================================================

pub struct GetFloatVarOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl GetFloatVarOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::string("Name", ""),
                InputPort::float("Default", 0.0),
            ],
            outputs: [OutputPort::float("Value")],
        }
    }
}

impl Default for GetFloatVarOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for GetFloatVarOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "GetFloatVar" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, ctx: &EvalContext, get_input: InputResolver) {
        let name = get_string(&self.inputs[0], get_input);
        let default = get_float(&self.inputs[1], get_input);
        let value = ctx.get_float_var_or(&name, default);
        self.outputs[0].set_float(value);
    }
}

impl OperatorMeta for GetFloatVarOp {
    fn category(&self) -> &'static str { "Flow" }
    fn category_color(&self) -> [f32; 4] { category_colors::FLOW }
    fn description(&self) -> &'static str { "Get float variable from context" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Name")),
            1 => Some(PortMeta::new("Default")),
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
// SetFloatVar Operator
// ============================================================================

pub struct SetFloatVarOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    var_name: String,
    var_value: f32,
}

impl SetFloatVarOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::string("Name", ""),
                InputPort::float("Value", 0.0),
            ],
            outputs: [OutputPort::float("Value")],
            var_name: String::new(),
            var_value: 0.0,
        }
    }

    /// Get the variable name and value that should be set in context
    pub fn get_pending_var(&self) -> Option<(&str, f32)> {
        if !self.var_name.is_empty() {
            Some((&self.var_name, self.var_value))
        } else {
            None
        }
    }
}

impl Default for SetFloatVarOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SetFloatVarOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "SetFloatVar" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let name = get_string(&self.inputs[0], get_input);
        let value = get_float(&self.inputs[1], get_input);

        // Store for later application to context
        self.var_name = name;
        self.var_value = value;

        // Pass through the value
        self.outputs[0].set_float(value);
    }
}

impl OperatorMeta for SetFloatVarOp {
    fn category(&self) -> &'static str { "Flow" }
    fn category_color(&self) -> [f32; 4] { category_colors::FLOW }
    fn description(&self) -> &'static str { "Set float variable in context" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Name")),
            1 => Some(PortMeta::new("Value")),
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
// GetIntVar Operator
// ============================================================================

pub struct GetIntVarOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl GetIntVarOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::string("Name", ""),
                InputPort::int("Default", 0),
            ],
            outputs: [OutputPort::int("Value")],
        }
    }
}

impl Default for GetIntVarOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for GetIntVarOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "GetIntVar" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, ctx: &EvalContext, get_input: InputResolver) {
        let name = get_string(&self.inputs[0], get_input);
        let default = get_int(&self.inputs[1], get_input);
        let value = ctx.get_int_var_or(&name, default);
        self.outputs[0].set_int(value);
    }
}

impl OperatorMeta for GetIntVarOp {
    fn category(&self) -> &'static str { "Flow" }
    fn category_color(&self) -> [f32; 4] { category_colors::FLOW }
    fn description(&self) -> &'static str { "Get integer variable from context" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Name")),
            1 => Some(PortMeta::new("Default")),
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
            name: "GetFloatVar",
            category: "Flow",
            description: "Get float variable from context",
        },
        || capture_meta(GetFloatVarOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "SetFloatVar",
            category: "Flow",
            description: "Set float variable in context",
        },
        || capture_meta(SetFloatVarOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "GetIntVar",
            category: "Flow",
            description: "Get integer variable from context",
        },
        || capture_meta(GetIntVarOp::new()),
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
    fn test_get_float_var() {
        let mut op = GetFloatVarOp::new();
        let mut ctx = EvalContext::new();
        ctx.set_float_var("speed", 10.5);

        op.inputs[0].default = Value::String("speed".to_string());
        op.inputs[1].default = Value::Float(0.0);
        op.compute(&ctx, &no_connections);
        assert!((op.outputs[0].value.as_float().unwrap() - 10.5).abs() < 0.001);
    }

    #[test]
    fn test_get_float_var_default() {
        let mut op = GetFloatVarOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::String("missing".to_string());
        op.inputs[1].default = Value::Float(42.0);
        op.compute(&ctx, &no_connections);
        assert!((op.outputs[0].value.as_float().unwrap() - 42.0).abs() < 0.001);
    }

    #[test]
    fn test_set_float_var() {
        let mut op = SetFloatVarOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::String("myvar".to_string());
        op.inputs[1].default = Value::Float(99.0);
        op.compute(&ctx, &no_connections);

        // Check the pending var
        let (name, value) = op.get_pending_var().unwrap();
        assert_eq!(name, "myvar");
        assert!((value - 99.0).abs() < 0.001);

        // Output should pass through
        assert!((op.outputs[0].value.as_float().unwrap() - 99.0).abs() < 0.001);
    }

    #[test]
    fn test_get_int_var() {
        let mut op = GetIntVarOp::new();
        let mut ctx = EvalContext::new();
        ctx.set_int_var("count", 42);

        op.inputs[0].default = Value::String("count".to_string());
        op.inputs[1].default = Value::Int(0);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(42));
    }
}
