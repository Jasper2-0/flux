//! Symbol file schema (.rsym)
//!
//! Symbols are reusable operator definitions that can be instantiated
//! in graphs. They define inputs, outputs, children, and connections.

use serde::{Deserialize, Serialize};

use flux_core::value::{Value, ValueType};
use flux_core::Id;

use super::animation::AnimationDef;
use super::version::SchemaVersion;

/// Symbol file schema (.rsym)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolFile {
    /// Schema version
    pub version: SchemaVersion,
    /// Symbol definition
    pub symbol: SymbolDef,
}

impl SymbolFile {
    /// Create a new symbol file
    pub fn new(name: &str) -> Self {
        Self {
            version: SchemaVersion::CURRENT,
            symbol: SymbolDef::new(name),
        }
    }

    /// Create from an existing symbol definition
    pub fn from_def(symbol: SymbolDef) -> Self {
        Self {
            version: SchemaVersion::CURRENT,
            symbol,
        }
    }
}

/// Symbol definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDef {
    /// Unique symbol identifier
    pub id: Id,
    /// Symbol name
    pub name: String,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Category for organization (e.g., "Math", "Color", "Transform")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Tags for searching/filtering
    #[serde(default)]
    pub tags: Vec<String>,

    /// Input definitions
    #[serde(default)]
    pub inputs: Vec<InputDef>,
    /// Output definitions
    #[serde(default)]
    pub outputs: Vec<OutputDef>,
    /// Child operators
    #[serde(default)]
    pub children: Vec<ChildDef>,
    /// Internal connections
    #[serde(default)]
    pub connections: Vec<ConnectionDef>,
    /// Animation data
    #[serde(default)]
    pub animations: Vec<AnimationDef>,

    /// UI metadata
    #[serde(default)]
    pub ui: SymbolUiMeta,
}

impl SymbolDef {
    /// Create a new symbol definition
    pub fn new(name: &str) -> Self {
        Self {
            id: Id::new(),
            name: name.to_string(),
            description: None,
            category: None,
            tags: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            children: Vec::new(),
            connections: Vec::new(),
            animations: Vec::new(),
            ui: SymbolUiMeta::default(),
        }
    }

    /// Create with a specific ID
    pub fn with_id(id: Id, name: &str) -> Self {
        let mut def = Self::new(name);
        def.id = id;
        def
    }

    /// Builder: set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    /// Builder: set category
    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    /// Builder: add a tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Add an input definition
    pub fn add_input(&mut self, input: InputDef) -> &mut Self {
        self.inputs.push(input);
        self
    }

    /// Add an output definition
    pub fn add_output(&mut self, output: OutputDef) -> &mut Self {
        self.outputs.push(output);
        self
    }

    /// Add a child operator
    pub fn add_child(&mut self, child: ChildDef) -> &mut Self {
        self.children.push(child);
        self
    }

    /// Add a connection
    pub fn add_connection(&mut self, connection: ConnectionDef) -> &mut Self {
        self.connections.push(connection);
        self
    }
}

/// Input slot definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDef {
    /// Unique input identifier
    pub id: Id,
    /// Input name
    pub name: String,
    /// Value type
    pub value_type: ValueType,
    /// Default value
    pub default: Value,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether this accepts multiple connections
    #[serde(default)]
    pub is_multi_input: bool,
    /// UI metadata
    #[serde(default)]
    pub ui: InputUiMeta,
}

impl InputDef {
    /// Create a new input definition
    pub fn new(name: &str, value_type: ValueType, default: Value) -> Self {
        Self {
            id: Id::new(),
            name: name.to_string(),
            value_type,
            default,
            description: None,
            is_multi_input: false,
            ui: InputUiMeta::default(),
        }
    }

    /// Create a float input
    pub fn float(name: &str, default: f32) -> Self {
        Self::new(name, ValueType::Float, Value::Float(default))
    }

    /// Create an int input
    pub fn int(name: &str, default: i32) -> Self {
        Self::new(name, ValueType::Int, Value::Int(default))
    }

    /// Create a bool input
    pub fn bool(name: &str, default: bool) -> Self {
        Self::new(name, ValueType::Bool, Value::Bool(default))
    }

    /// Create a vec3 input
    pub fn vec3(name: &str, default: [f32; 3]) -> Self {
        Self::new(name, ValueType::Vec3, Value::Vec3(default))
    }

    /// Create a color input
    pub fn color(name: &str, default: flux_core::value::Color) -> Self {
        Self::new(name, ValueType::Color, Value::Color(default))
    }

    /// Builder: set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    /// Builder: make multi-input
    pub fn multi_input(mut self) -> Self {
        self.is_multi_input = true;
        self
    }

    /// Builder: set UI range
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.ui.min = Some(min);
        self.ui.max = Some(max);
        self
    }
}

/// Output slot definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputDef {
    /// Unique output identifier
    pub id: Id,
    /// Output name
    pub name: String,
    /// Value type
    pub value_type: ValueType,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl OutputDef {
    /// Create a new output definition
    pub fn new(name: &str, value_type: ValueType) -> Self {
        Self {
            id: Id::new(),
            name: name.to_string(),
            value_type,
            description: None,
        }
    }

    /// Create a float output
    pub fn float(name: &str) -> Self {
        Self::new(name, ValueType::Float)
    }

    /// Create a vec3 output
    pub fn vec3(name: &str) -> Self {
        Self::new(name, ValueType::Vec3)
    }

    /// Create a color output
    pub fn color(name: &str) -> Self {
        Self::new(name, ValueType::Color)
    }
}

/// Child operator instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildDef {
    /// Unique instance identifier
    pub id: Id,
    /// Reference to the symbol type (ID or "builtin:name")
    pub symbol_ref: String,
    /// Optional display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Input value overrides
    #[serde(default)]
    pub input_values: Vec<InputValueDef>,
    /// UI position
    #[serde(default)]
    pub position: [f32; 2],
    /// Whether this child is bypassed
    #[serde(default)]
    pub is_bypassed: bool,
    /// Whether this child is disabled
    #[serde(default)]
    pub is_disabled: bool,
}

impl ChildDef {
    /// Create a new child definition
    pub fn new(symbol_ref: &str) -> Self {
        Self {
            id: Id::new(),
            symbol_ref: symbol_ref.to_string(),
            name: None,
            input_values: Vec::new(),
            position: [0.0, 0.0],
            is_bypassed: false,
            is_disabled: false,
        }
    }

    /// Create with a specific ID
    pub fn with_id(id: Id, symbol_ref: &str) -> Self {
        let mut child = Self::new(symbol_ref);
        child.id = id;
        child
    }

    /// Reference a builtin operator
    pub fn builtin(name: &str) -> Self {
        Self::new(&format!("builtin:{}", name))
    }

    /// Builder: set display name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Builder: set position
    pub fn at_position(mut self, x: f32, y: f32) -> Self {
        self.position = [x, y];
        self
    }

    /// Builder: set an input value
    pub fn with_input(mut self, input_id: Id, value: Value) -> Self {
        self.input_values.push(InputValueDef { input_id, value });
        self
    }
}

/// Input value override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputValueDef {
    /// Input slot ID
    pub input_id: Id,
    /// Override value
    pub value: Value,
}

/// Connection between operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDef {
    /// Source child ID (or special value for symbol inputs)
    pub source_child: Id,
    /// Source output index
    pub source_output: usize,
    /// Target child ID (or special value for symbol outputs)
    pub target_child: Id,
    /// Target input index
    pub target_input: usize,
}

impl ConnectionDef {
    /// Create a new connection
    pub fn new(source_child: Id, source_output: usize, target_child: Id, target_input: usize) -> Self {
        Self {
            source_child,
            source_output,
            target_child,
            target_input,
        }
    }
}

/// Symbol UI metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SymbolUiMeta {
    /// Display color (hex format, e.g., "#FF6B6B")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Icon name or path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Default node width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
}

/// Input UI metadata (for editor widgets)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputUiMeta {
    /// Minimum value for numeric inputs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Maximum value for numeric inputs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Step size for numeric inputs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    /// UI widget type hint ("slider", "dial", "color_picker", etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::value::Color;

    #[test]
    fn test_symbol_def_new() {
        let symbol = SymbolDef::new("TestSymbol");
        assert_eq!(symbol.name, "TestSymbol");
        assert!(symbol.inputs.is_empty());
        assert!(symbol.outputs.is_empty());
    }

    #[test]
    fn test_symbol_def_builder() {
        let symbol = SymbolDef::new("MyEffect")
            .with_description("A cool effect")
            .with_category("Effects")
            .with_tag("visual")
            .with_tag("color");

        assert_eq!(symbol.description, Some("A cool effect".to_string()));
        assert_eq!(symbol.category, Some("Effects".to_string()));
        assert_eq!(symbol.tags.len(), 2);
    }

    #[test]
    fn test_input_def_float() {
        let input = InputDef::float("Amplitude", 1.0)
            .with_description("Wave amplitude")
            .with_range(0.0, 10.0);

        assert_eq!(input.name, "Amplitude");
        assert_eq!(input.value_type, ValueType::Float);
        assert_eq!(input.ui.min, Some(0.0));
        assert_eq!(input.ui.max, Some(10.0));
    }

    #[test]
    fn test_child_def_builtin() {
        let child = ChildDef::builtin("add")
            .with_name("AddNumbers")
            .at_position(100.0, 50.0);

        assert_eq!(child.symbol_ref, "builtin:add");
        assert_eq!(child.name, Some("AddNumbers".to_string()));
        assert_eq!(child.position, [100.0, 50.0]);
    }

    #[test]
    fn test_symbol_file_serialize() {
        let mut symbol = SymbolDef::new("ColorPulse")
            .with_category("Effects");

        symbol.add_input(
            InputDef::color("BaseColor", Color::RED)
                .with_description("Base color for the effect"),
        );
        symbol.add_input(InputDef::float("Intensity", 1.0).with_range(0.0, 2.0));
        symbol.add_output(OutputDef::color("Result"));

        let file = SymbolFile::from_def(symbol);
        let json = serde_json::to_string_pretty(&file).unwrap();

        assert!(json.contains("ColorPulse"));
        assert!(json.contains("Effects"));
        assert!(json.contains("BaseColor"));

        // Roundtrip
        let restored: SymbolFile = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.symbol.name, "ColorPulse");
        assert_eq!(restored.symbol.inputs.len(), 2);
        assert_eq!(restored.symbol.outputs.len(), 1);
    }

    #[test]
    fn test_connection_def() {
        let source_id = Id::new();
        let target_id = Id::new();
        let conn = ConnectionDef::new(source_id, 0, target_id, 1);

        assert_eq!(conn.source_child, source_id);
        assert_eq!(conn.source_output, 0);
        assert_eq!(conn.target_child, target_id);
        assert_eq!(conn.target_input, 1);
    }
}
