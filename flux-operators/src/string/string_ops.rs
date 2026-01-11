//! String operators: Concat, Format, Length, SubString, Split, FloatToString, IntToString, Contains

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

fn get_bool(input: &InputPort, get_input: InputResolver) -> bool {
    match input.connection {
        Some((node_id, output_idx)) => get_input(node_id, output_idx).as_bool().unwrap_or(false),
        None => input.default.as_bool().unwrap_or(false),
    }
}

// ============================================================================
// StringConcat Operator
// ============================================================================

pub struct StringConcatOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl StringConcatOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::string("A", ""),
                InputPort::string("B", ""),
            ],
            outputs: [OutputPort::string("Result")],
        }
    }
}

impl Default for StringConcatOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for StringConcatOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "StringConcat" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = get_string(&self.inputs[0], get_input);
        let b = get_string(&self.inputs[1], get_input);
        self.outputs[0].set_string(&format!("{}{}", a, b));
    }
}

impl OperatorMeta for StringConcatOp {
    fn category(&self) -> &'static str { "String" }
    fn category_color(&self) -> [f32; 4] { category_colors::STRING }
    fn description(&self) -> &'static str { "Concatenate two strings" }
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
// StringFormat Operator
// ============================================================================

pub struct StringFormatOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl StringFormatOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::string("Format", "{}"),
                InputPort::float("Value", 0.0),
            ],
            outputs: [OutputPort::string("Result")],
        }
    }
}

impl Default for StringFormatOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for StringFormatOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "StringFormat" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let format_str = get_string(&self.inputs[0], get_input);
        let value = get_float(&self.inputs[1], get_input);
        // Simple placeholder replacement (replaces first {} with value)
        let result = format_str.replacen("{}", &value.to_string(), 1);
        self.outputs[0].set_string(&result);
    }
}

impl OperatorMeta for StringFormatOp {
    fn category(&self) -> &'static str { "String" }
    fn category_color(&self) -> [f32; 4] { category_colors::STRING }
    fn description(&self) -> &'static str { "Format string with value" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Format")),
            1 => Some(PortMeta::new("Value")),
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
// StringLength Operator
// ============================================================================

pub struct StringLengthOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl StringLengthOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::string("String", "")],
            outputs: [OutputPort::int("Length")],
        }
    }
}

impl Default for StringLengthOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for StringLengthOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "StringLength" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let s = get_string(&self.inputs[0], get_input);
        self.outputs[0].set_int(s.len() as i32);
    }
}

impl OperatorMeta for StringLengthOp {
    fn category(&self) -> &'static str { "String" }
    fn category_color(&self) -> [f32; 4] { category_colors::STRING }
    fn description(&self) -> &'static str { "Get string length" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("String")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Length").with_shape(PinShape::TriangleFilled)),
            _ => None,
        }
    }
}

// ============================================================================
// SubString Operator
// ============================================================================

pub struct SubStringOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl SubStringOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::string("String", ""),
                InputPort::int("Start", 0),
                InputPort::int("Length", -1), // -1 means to end
            ],
            outputs: [OutputPort::string("Result")],
        }
    }
}

impl Default for SubStringOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for SubStringOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "SubString" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let s = get_string(&self.inputs[0], get_input);
        let start = get_int(&self.inputs[1], get_input).max(0) as usize;
        let length = get_int(&self.inputs[2], get_input);

        let result = if start >= s.len() {
            String::new()
        } else if length < 0 {
            s[start..].to_string()
        } else {
            let end = (start + length as usize).min(s.len());
            s[start..end].to_string()
        };

        self.outputs[0].set_string(&result);
    }
}

impl OperatorMeta for SubStringOp {
    fn category(&self) -> &'static str { "String" }
    fn category_color(&self) -> [f32; 4] { category_colors::STRING }
    fn description(&self) -> &'static str { "Extract substring" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("String")),
            1 => Some(PortMeta::new("Start")),
            2 => Some(PortMeta::new("Length")),
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
// StringSplit Operator
// ============================================================================

pub struct StringSplitOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl StringSplitOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::string("String", ""),
                InputPort::string("Delimiter", ","),
                InputPort::int("Index", 0),
            ],
            outputs: [OutputPort::string("Result")],
        }
    }
}

impl Default for StringSplitOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for StringSplitOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "StringSplit" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let s = get_string(&self.inputs[0], get_input);
        let delimiter = get_string(&self.inputs[1], get_input);
        let index = get_int(&self.inputs[2], get_input).max(0) as usize;

        let parts: Vec<&str> = s.split(&delimiter).collect();
        let result = parts.get(index).copied().unwrap_or("");

        self.outputs[0].set_string(result);
    }
}

impl OperatorMeta for StringSplitOp {
    fn category(&self) -> &'static str { "String" }
    fn category_color(&self) -> [f32; 4] { category_colors::STRING }
    fn description(&self) -> &'static str { "Split string by delimiter" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("String")),
            1 => Some(PortMeta::new("Delimiter")),
            2 => Some(PortMeta::new("Index")),
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
// FloatToString Operator
// ============================================================================

pub struct FloatToStringOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
}

impl FloatToStringOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::float("Value", 0.0),
                InputPort::int("Decimals", 2),
            ],
            outputs: [OutputPort::string("Result")],
        }
    }
}

impl Default for FloatToStringOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for FloatToStringOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "FloatToString" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_float(&self.inputs[0], get_input);
        let decimals = get_int(&self.inputs[1], get_input).clamp(0, 10) as usize;
        let result = format!("{:.1$}", value, decimals);
        self.outputs[0].set_string(&result);
    }
}

impl OperatorMeta for FloatToStringOp {
    fn category(&self) -> &'static str { "String" }
    fn category_color(&self) -> [f32; 4] { category_colors::STRING }
    fn description(&self) -> &'static str { "Convert float to string" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Value")),
            1 => Some(PortMeta::new("Decimals")),
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
// IntToString Operator
// ============================================================================

pub struct IntToStringOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
}

impl IntToStringOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [InputPort::int("Value", 0)],
            outputs: [OutputPort::string("Result")],
        }
    }
}

impl Default for IntToStringOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for IntToStringOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "IntToString" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = get_int(&self.inputs[0], get_input);
        self.outputs[0].set_string(&value.to_string());
    }
}

impl OperatorMeta for IntToStringOp {
    fn category(&self) -> &'static str { "String" }
    fn category_color(&self) -> [f32; 4] { category_colors::STRING }
    fn description(&self) -> &'static str { "Convert integer to string" }
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
// StringContains Operator
// ============================================================================

pub struct StringContainsOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
}

impl StringContainsOp {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            inputs: [
                InputPort::string("String", ""),
                InputPort::string("Search", ""),
                InputPort::bool("CaseSensitive", true),
            ],
            outputs: [OutputPort::bool("Contains")],
        }
    }
}

impl Default for StringContainsOp {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator for StringContainsOp {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn id(&self) -> Id { self.id }
    fn name(&self) -> &'static str { "StringContains" }
    fn inputs(&self) -> &[InputPort] { &self.inputs }
    fn inputs_mut(&mut self) -> &mut [InputPort] { &mut self.inputs }
    fn outputs(&self) -> &[OutputPort] { &self.outputs }
    fn outputs_mut(&mut self) -> &mut [OutputPort] { &mut self.outputs }

    fn compute(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let string = get_string(&self.inputs[0], get_input);
        let search = get_string(&self.inputs[1], get_input);
        let case_sensitive = get_bool(&self.inputs[2], get_input);

        let contains = if case_sensitive {
            string.contains(&search)
        } else {
            string.to_lowercase().contains(&search.to_lowercase())
        };

        self.outputs[0].set_bool(contains);
    }
}

impl OperatorMeta for StringContainsOp {
    fn category(&self) -> &'static str { "String" }
    fn category_color(&self) -> [f32; 4] { category_colors::STRING }
    fn description(&self) -> &'static str { "Check if string contains substring" }
    fn input_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("String")),
            1 => Some(PortMeta::new("Search")),
            2 => Some(PortMeta::new("CaseSensitive")),
            _ => None,
        }
    }
    fn output_meta(&self, index: usize) -> Option<PortMeta> {
        match index {
            0 => Some(PortMeta::new("Contains").with_shape(PinShape::TriangleFilled)),
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
            name: "StringConcat",
            category: "String",
            description: "Concatenate two strings",
        },
        || capture_meta(StringConcatOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "StringFormat",
            category: "String",
            description: "Format string with value",
        },
        || capture_meta(StringFormatOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "StringLength",
            category: "String",
            description: "Get string length",
        },
        || capture_meta(StringLengthOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "SubString",
            category: "String",
            description: "Extract substring",
        },
        || capture_meta(SubStringOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "StringSplit",
            category: "String",
            description: "Split string by delimiter",
        },
        || capture_meta(StringSplitOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "FloatToString",
            category: "String",
            description: "Convert float to string",
        },
        || capture_meta(FloatToStringOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "IntToString",
            category: "String",
            description: "Convert integer to string",
        },
        || capture_meta(IntToStringOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "StringContains",
            category: "String",
            description: "Check if string contains substring",
        },
        || capture_meta(StringContainsOp::new()),
    );
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;
    use flux_core::Value;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Float(0.0)
    }

    #[test]
    fn test_string_concat() {
        let mut op = StringConcatOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::String("Hello ".to_string());
        op.inputs[1].default = Value::String("World".to_string());
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_string(), Some("Hello World"));
    }

    #[test]
    fn test_string_format() {
        let mut op = StringFormatOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::String("Value: {}".to_string());
        op.inputs[1].default = Value::Float(42.5);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_string(), Some("Value: 42.5"));
    }

    #[test]
    fn test_string_length() {
        let mut op = StringLengthOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::String("Hello".to_string());
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_int(), Some(5));
    }

    #[test]
    fn test_substring() {
        let mut op = SubStringOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::String("Hello World".to_string());
        op.inputs[1].default = Value::Int(6);
        op.inputs[2].default = Value::Int(5);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_string(), Some("World"));

        // Test to end
        op.inputs[2].default = Value::Int(-1);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_string(), Some("World"));
    }

    #[test]
    fn test_string_split() {
        let mut op = StringSplitOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::String("a,b,c".to_string());
        op.inputs[1].default = Value::String(",".to_string());
        op.inputs[2].default = Value::Int(1);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_string(), Some("b"));
    }

    #[test]
    fn test_float_to_string() {
        let mut op = FloatToStringOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Float(PI);
        op.inputs[1].default = Value::Int(2);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_string(), Some("3.14"));
    }

    #[test]
    fn test_int_to_string() {
        let mut op = IntToStringOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Int(42);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_string(), Some("42"));
    }

    #[test]
    fn test_string_contains() {
        let mut op = StringContainsOp::new();
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::String("Hello World".to_string());
        op.inputs[1].default = Value::String("World".to_string());
        op.inputs[2].default = Value::Bool(true);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(true));

        // Case insensitive
        op.inputs[1].default = Value::String("WORLD".to_string());
        op.inputs[2].default = Value::Bool(false);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(true));

        // Case sensitive - should fail
        op.inputs[2].default = Value::Bool(true);
        op.compute(&ctx, &no_connections);
        assert_eq!(op.outputs[0].value.as_bool(), Some(false));
    }
}
