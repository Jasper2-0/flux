//! Utility/Debug operators: Print, Passthrough, Comment, Bookmark, TypeOf, IsNull

use std::any::Any;

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::{category_colors, OperatorMeta, PinShape, PortMeta};
use crate::registry::{capture_meta, OperatorRegistry, RegistryEntry};
use flux_core::port::{InputPort, OutputPort};
use flux_core::Value;

fn get_value(input: &InputPort, get_input: InputResolver) -> Value {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx),
        None => input.default.clone(),
    }
}

fn get_string(input: &InputPort, get_input: InputResolver) -> String {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx)
            .as_string()
            .unwrap_or_default()
            .to_string(),
        None => input.default.as_string().unwrap_or_default().to_string(),
    }
}

fn get_bool(input: &InputPort, get_input: InputResolver) -> bool {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_bool().unwrap_or(false),
        None => input.default.as_bool().unwrap_or(false),
    }
}

// ============================================================================
// Print Operator (Debug output)
// ============================================================================

pub struct PrintOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
    last_printed: String,
}

impl PrintOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Value", 0.0),
                InputPort::string("Label", ""),
                InputPort::bool("Enabled", true),
            ],
            outputs: [OutputPort::float("Passthrough")],
            last_printed: String::new(),
        }
    }

    /// Get the last printed message
    pub fn last_message(&self) -> &str {
        &self.last_printed
    }
}

impl Default for PrintOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for PrintOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Print" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_value(&self.inputs[0], get_input);
        let label = get_string(&self.inputs[1], get_input);
        let enabled = get_bool(&self.inputs[2], get_input);

        if enabled {
            let message = if label.is_empty() {
                format!("{:?}", value)
            } else {
                format!("{}: {:?}", label, value)
            };
            self.last_printed = message;
            // In a real implementation, this would emit to a debug console
            #[cfg(debug_assertions)]
            println!("[Print] {}", self.last_printed);
        }

        // Pass through the value
        self.outputs[0].value = value;
    }
}

impl OperatorMeta for PrintOp {
    fn category(&self) -> &'static str { "Util" }
    fn category_color(&self) -> [f32; 4] { category_colors::UTIL }
    fn description(&self) -> &'static str { "Debug print value with optional label" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            1 => Some(PortMeta::new("Label")),
            2 => Some(PortMeta::new("Enabled")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Passthrough").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// Passthrough Operator
// ============================================================================

pub struct PassthroughOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl PassthroughOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Value", 0.0)],
            outputs: [OutputPort::float("Value")],
        }
    }
}

impl Default for PassthroughOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for PassthroughOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Passthrough" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_value(&self.inputs[0], get_input);
        self.outputs[0].value = value;
    }
}

impl OperatorMeta for PassthroughOp {
    fn category(&self) -> &'static str { "Util" }
    fn category_color(&self) -> [f32; 4] { category_colors::UTIL }
    fn description(&self) -> &'static str { "Pass value through unchanged" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
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
// Comment Operator (No-op with text annotation)
// ============================================================================

pub struct CommentOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 0],
    comment: String,
}

impl CommentOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::string("Text", "")],
            outputs: [],
            comment: String::new(),
        }
    }

    pub fn get_comment(&self) -> &str {
        &self.comment
    }

    pub fn set_comment(&mut self, text: &str) {
        self.comment = text.to_string();
    }
}

impl Default for CommentOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for CommentOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Comment" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        self.comment = get_string(&self.inputs[0], get_input);
    }
}

impl OperatorMeta for CommentOp {
    fn category(&self) -> &'static str { "Util" }
    fn category_color(&self) -> [f32; 4] { category_colors::UTIL }
    fn description(&self) -> &'static str { "Add annotation comment to graph" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Text")),
            _ => None,
        }
    }
    fn output_meta(&self, _index: usize) -> Option<PortMeta> {
        None
    }
}

// ============================================================================
// Bookmark Operator (Named reference point)
// ============================================================================

pub struct BookmarkOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    bookmark_name: String,
}

impl BookmarkOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::string("Name", "Bookmark"),
                InputPort::float("Value", 0.0),
            ],
            outputs: [OutputPort::float("Value")],
            bookmark_name: String::from("Bookmark"),
        }
    }

    pub fn name_str(&self) -> &str {
        &self.bookmark_name
    }
}

impl Default for BookmarkOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for BookmarkOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "Bookmark" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        self.bookmark_name = get_string(&self.inputs[0], get_input);
        let value = get_value(&self.inputs[1], get_input);
        self.outputs[0].value = value;
    }
}

impl OperatorMeta for BookmarkOp {
    fn category(&self) -> &'static str { "Util" }
    fn category_color(&self) -> [f32; 4] { category_colors::UTIL }
    fn description(&self) -> &'static str { "Named reference point in graph" }
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
// TypeOf Operator
// ============================================================================

pub struct TypeOfOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl TypeOfOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Value", 0.0)],
            outputs: [OutputPort::string("Type")],
        }
    }
}

impl Default for TypeOfOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for TypeOfOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "TypeOf" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_value(&self.inputs[0], get_input);
        let type_name = match value {
            Value::Float(_) => "Float",
            Value::Int(_) => "Int",
            Value::Bool(_) => "Bool",
            Value::String(_) => "String",
            Value::Vec2(_) => "Vec2",
            Value::Vec3(_) => "Vec3",
            Value::Vec4(_) => "Vec4",
            Value::Color(_) => "Color",
            Value::Gradient(_) => "Gradient",
            Value::Matrix4(_) => "Matrix4",
            Value::FloatList(_) => "FloatList",
            Value::IntList(_) => "IntList",
            Value::BoolList(_) => "BoolList",
            Value::Vec2List(_) => "Vec2List",
            Value::Vec3List(_) => "Vec3List",
            Value::Vec4List(_) => "Vec4List",
            Value::ColorList(_) => "ColorList",
            Value::StringList(_) => "StringList",
        };
        self.outputs[0].set_string(type_name);
    }
}

impl OperatorMeta for TypeOfOp {
    fn category(&self) -> &'static str { "Util" }
    fn category_color(&self) -> [f32; 4] { category_colors::UTIL }
    fn description(&self) -> &'static str { "Get value type name as string" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Type").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// IsConnected Operator (replaces IsNull)
// ============================================================================

pub struct IsConnectedOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl IsConnectedOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::float("Value", 0.0)],
            outputs: [OutputPort::bool("IsConnected")],
        }
    }
}

impl Default for IsConnectedOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for IsConnectedOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "IsConnected" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, _get_input: InputResolver) {
        // Check if input is connected
        let is_connected = self.inputs[0].is_connected();
        self.outputs[0].set_bool(is_connected);
    }
}

impl OperatorMeta for IsConnectedOp {
    fn category(&self) -> &'static str { "Util" }
    fn category_color(&self) -> [f32; 4] { category_colors::UTIL }
    fn description(&self) -> &'static str { "Check if input port is connected" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("IsConnected").with_shape(PinShape::TriangleFilled)),
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
            name: "Print",
            category: "Utility",
            description: "Debug print value",
        },
        || capture_meta(PrintOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Passthrough",
            category: "Utility",
            description: "Pass value through",
        },
        || capture_meta(PassthroughOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Comment",
            category: "Utility",
            description: "Add annotation comment",
        },
        || capture_meta(CommentOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Bookmark",
            category: "Utility",
            description: "Named reference point",
        },
        || capture_meta(BookmarkOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "TypeOf",
            category: "Utility",
            description: "Get value type name",
        },
        || capture_meta(TypeOfOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "IsConnected",
            category: "Utility",
            description: "Check if input is connected",
        },
        || capture_meta(IsConnectedOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Float(0.0)
    }

    #[test]
    fn test_passthrough() {
        let mut op = PassthroughOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Float(42.0);
        op.compute(&ctx, &no_connections);
        assert!((op.outputs[0].value.as_float().unwrap() - 42.0).abs() < 0.001);
    }

    #[test]
    fn test_typeof() {
        let mut op = TypeOfOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Float(42.0);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_string(), Some("Float"));

        op.inputs[0].default = Value::Int(42);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_string(), Some("Int"));

        op.inputs[0].default = Value::Bool(true);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_string(), Some("Bool"));

        op.inputs[0].default = Value::String("test".to_string());
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_string(), Some("String"));

        op.inputs[0].default = Value::Vec3([1.0, 2.0, 3.0]);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_string(), Some("Vec3"));
    }

    #[test]
    fn test_is_connected() {
        let mut op = IsConnectedOp::new();
        let ctx = EvalContext::new();

        // Not connected - should return false
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(false));

        // Connected - should return true
        op.inputs[0].connection = Some((Id::new(), 0));
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(true));
    }

    #[test]
    fn test_print() {
        let mut op = PrintOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Float(123.0);
        op.inputs[1].default = Value::String("test".to_string());
        op.inputs[2].default = Value::Bool(true);
        op.compute(&ctx, &no_connections);

        assert!(op.last_message().contains("test"));
        assert!(op.last_message().contains("123"));
        // Passthrough
        assert!((op.outputs[0].value.as_float().unwrap() - 123.0).abs() < 0.001);
    }

    #[test]
    fn test_comment() {
        let mut op = CommentOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::String("This is a comment".to_string());
        op.compute(&ctx, &no_connections);
        assert_eq!(op.get_comment(), "This is a comment");
    }

    #[test]
    fn test_bookmark() {
        let mut op = BookmarkOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::String("MyBookmark".to_string());
        op.inputs[1].default = Value::Float(99.0);
        op.compute(&ctx, &no_connections);

        assert_eq!(op.name_str(), "MyBookmark");
        assert!((op.outputs[0].value.as_float().unwrap() - 99.0).abs() < 0.001);
    }
}
