//! String operators: Concat, Format, Length, SubString, Split, FloatToString, IntToString, Contains

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::OperatorMeta;
use flux_macros::Operator;
use crate::register_operators;
use crate::registry::OperatorRegistry;
use flux_core::port::{InputPort, OutputPort};

// ============================================================================
// StringConcat Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "StringConcat", category = "String", description = "Concatenate two strings")]
#[operator(category_color = [0.35, 0.50, 0.55, 1.0])]
#[allow(dead_code)]
pub struct StringConcatOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "A", default = "")]
    a: String,
    #[input(label = "B", default = "")]
    b: String,
    #[output(label = "Result")]
    result: String,
}

impl StringConcatOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let a = self.get_a(get_input);
        let b = self.get_b(get_input);
        self.set_result(&format!("{}{}", a, b));
    }
}

// ============================================================================
// StringFormat Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "StringFormat", category = "String", description = "Format string with value")]
#[operator(category_color = [0.35, 0.50, 0.55, 1.0])]
#[allow(dead_code)]
pub struct StringFormatOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "Format", default = "{}")]
    format: String,
    #[input(label = "Value", default = 0.0)]
    value: f32,
    #[output(label = "Result")]
    result: String,
}

impl StringFormatOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let format_str = self.get_format(get_input);
        let value = self.get_value(get_input);
        // Simple placeholder replacement (replaces first {} with value)
        self.set_result(&format_str.replacen("{}", &value.to_string(), 1));
    }
}

// ============================================================================
// StringLength Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "StringLength", category = "String", description = "Get string length")]
#[operator(category_color = [0.35, 0.50, 0.55, 1.0])]
#[allow(dead_code)]
pub struct StringLengthOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    #[input(label = "String", default = "")]
    string: String,
    #[output(label = "Length")]
    length: i32,
}

impl StringLengthOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let s = self.get_string(get_input);
        self.set_length(s.len() as i32);
    }
}

// ============================================================================
// SubString Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "SubString", category = "String", description = "Extract substring")]
#[operator(category_color = [0.35, 0.50, 0.55, 1.0])]
#[allow(dead_code)]
pub struct SubStringOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
    #[input(label = "String", default = "")]
    string: String,
    #[input(label = "Start", default = 0)]
    start: i32,
    #[input(label = "Length", default = -1)]
    length_input: i32,
    #[output(label = "Result")]
    result: String,
}

impl SubStringOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let s = self.get_string(get_input);
        let start = self.get_start(get_input).max(0) as usize;
        let length = self.get_length_input(get_input);

        let result = if start >= s.len() {
            String::new()
        } else if length < 0 {
            s[start..].to_string()
        } else {
            let end = (start + length as usize).min(s.len());
            s[start..end].to_string()
        };

        self.set_result(&result);
    }
}

// ============================================================================
// StringSplit Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "StringSplit", category = "String", description = "Split string by delimiter")]
#[operator(category_color = [0.35, 0.50, 0.55, 1.0])]
#[allow(dead_code)]
pub struct StringSplitOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
    #[input(label = "String", default = "")]
    string: String,
    #[input(label = "Delimiter", default = ",")]
    delimiter: String,
    #[input(label = "Index", default = 0)]
    index: i32,
    #[output(label = "Result")]
    result: String,
}

impl StringSplitOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let s = self.get_string(get_input);
        let delimiter = self.get_delimiter(get_input);
        let index = self.get_index(get_input).max(0) as usize;

        let parts: Vec<&str> = s.split(&delimiter).collect();
        let result = parts.get(index).copied().unwrap_or("");

        self.set_result(result);
    }
}

// ============================================================================
// FloatToString Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "FloatToString", category = "String", description = "Convert float to string")]
#[operator(category_color = [0.35, 0.50, 0.55, 1.0])]
#[allow(dead_code)]
pub struct FloatToStringOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "Value", default = 0.0)]
    value: f32,
    #[input(label = "Decimals", default = 2)]
    decimals: i32,
    #[output(label = "Result")]
    result: String,
}

impl FloatToStringOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = self.get_value(get_input);
        let decimals = self.get_decimals(get_input).clamp(0, 10) as usize;
        self.set_result(&format!("{:.1$}", value, decimals));
    }
}

// ============================================================================
// IntToString Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "IntToString", category = "String", description = "Convert integer to string")]
#[operator(category_color = [0.35, 0.50, 0.55, 1.0])]
#[allow(dead_code)]
pub struct IntToStringOp {
    id: Id,
    inputs: [InputPort; 1],
    outputs: [OutputPort; 1],
    #[input(label = "Value", default = 0)]
    value: i32,
    #[output(label = "Result")]
    result: String,
}

impl IntToStringOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = self.get_value(get_input);
        self.set_result(&value.to_string());
    }
}

// ============================================================================
// StringContains Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "StringContains", category = "String", description = "Check if string contains substring")]
#[operator(category_color = [0.35, 0.50, 0.55, 1.0])]
#[allow(dead_code)]
pub struct StringContainsOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
    #[input(label = "String", default = "")]
    string: String,
    #[input(label = "Search", default = "")]
    search: String,
    #[input(label = "CaseSensitive", default = true)]
    case_sensitive: bool,
    #[output(label = "Contains")]
    contains: bool,
}

impl StringContainsOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let string = self.get_string(get_input);
        let search = self.get_search(get_input);
        let case_sensitive = self.get_case_sensitive(get_input);

        let contains = if case_sensitive {
            string.contains(&search)
        } else {
            string.to_lowercase().contains(&search.to_lowercase())
        };

        self.set_contains(contains);
    }
}

// ============================================================================
// Registration
// ============================================================================

pub fn register(registry: &OperatorRegistry) {
    register_operators!(
        registry,
        StringConcatOp,
        StringFormatOp,
        StringLengthOp,
        SubStringOp,
        StringSplitOp,
        FloatToStringOp,
        IntToStringOp,
        StringContainsOp,
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
