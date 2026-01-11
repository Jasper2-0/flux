//! Graph file schema (.rgraph)
//!
//! Graphs represent compositions - instances of symbols with specific
//! configuration and playback settings.

use serde::{Deserialize, Serialize};

use flux_core::value::Value;
use flux_core::Id;

use super::version::SchemaVersion;

/// Graph file schema (.rgraph)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphFile {
    /// Schema version
    pub version: SchemaVersion,
    /// Graph definition
    pub graph: GraphDef,
}

impl GraphFile {
    /// Create a new graph file
    pub fn new(name: &str, root_symbol: Id) -> Self {
        Self {
            version: SchemaVersion::CURRENT,
            graph: GraphDef::new(name, root_symbol),
        }
    }
}

/// Graph/composition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDef {
    /// Unique graph identifier
    pub id: Id,
    /// Graph name
    pub name: String,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Root symbol for this graph
    pub root_symbol: Id,

    /// Instance-specific overrides
    #[serde(default)]
    pub instance_overrides: Vec<InstanceOverride>,

    /// Playback settings
    #[serde(default)]
    pub playback: PlaybackDef,

    /// View/camera state (for 3D graphs)
    #[serde(default)]
    pub view: ViewDef,
}

impl GraphDef {
    /// Create a new graph definition
    pub fn new(name: &str, root_symbol: Id) -> Self {
        Self {
            id: Id::new(),
            name: name.to_string(),
            description: None,
            root_symbol,
            instance_overrides: Vec::new(),
            playback: PlaybackDef::default(),
            view: ViewDef::default(),
        }
    }

    /// Builder: set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    /// Add an instance override
    pub fn add_override(&mut self, override_def: InstanceOverride) -> &mut Self {
        self.instance_overrides.push(override_def);
        self
    }
}

/// Override for a specific instance in the graph hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceOverride {
    /// Path to the instance (e.g., "child1.child2.child3")
    pub path: String,
    /// Input value overrides
    #[serde(default)]
    pub inputs: Vec<InputOverride>,
    /// Port UI metadata overrides (ranges, labels, etc.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub port_ui_overrides: Vec<PortUiOverride>,
}

impl InstanceOverride {
    /// Create a new instance override
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            inputs: Vec::new(),
            port_ui_overrides: Vec::new(),
        }
    }

    /// Add an input override
    pub fn with_input(mut self, input_id: Id, value: Value) -> Self {
        self.inputs.push(InputOverride { input_id, value });
        self
    }

    /// Add a port UI override
    pub fn with_port_ui(mut self, port_ui: PortUiOverride) -> Self {
        self.port_ui_overrides.push(port_ui);
        self
    }
}

/// Input value override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputOverride {
    /// Input slot ID
    pub input_id: Id,
    /// Override value
    pub value: Value,
}

/// Per-instance UI metadata override for a port.
///
/// Used to customize parameter ranges, labels, etc. for specific instances
/// without modifying the underlying operator definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortUiOverride {
    /// Input port index
    pub port_index: usize,
    /// Custom UI range (None = use operator default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<(f32, f32)>,
    /// Custom display label (None = use operator default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Custom unit suffix (None = use operator default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Custom step size (None = auto)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f32>,
}

impl PortUiOverride {
    /// Create a new port UI override
    pub fn new(port_index: usize) -> Self {
        Self {
            port_index,
            range: None,
            label: None,
            unit: None,
            step: None,
        }
    }

    /// Builder: set custom range
    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.range = Some((min, max));
        self
    }

    /// Builder: set custom label
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Builder: set custom unit
    pub fn with_unit(mut self, unit: &str) -> Self {
        self.unit = Some(unit.to_string());
        self
    }

    /// Builder: set custom step
    pub fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Convert from runtime PortOverride
    pub fn from_port_override(port_index: usize, override_: &flux_core::PortOverride) -> Self {
        Self {
            port_index,
            range: override_.range,
            label: override_.label.clone(),
            unit: override_.unit.clone(),
            step: override_.step,
        }
    }

    /// Convert to runtime PortOverride
    pub fn to_port_override(&self) -> flux_core::PortOverride {
        flux_core::PortOverride {
            range: self.range,
            label: self.label.clone(),
            unit: self.unit.clone(),
            step: self.step,
        }
    }

    /// Returns true if all override fields are None
    pub fn is_empty(&self) -> bool {
        self.range.is_none() && self.label.is_none() && self.unit.is_none() && self.step.is_none()
    }
}

/// Playback settings for the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackDef {
    /// Beats per minute
    #[serde(default = "default_bpm")]
    pub bpm: f64,
    /// Start time in seconds
    #[serde(default)]
    pub start_time: f64,
    /// End time in seconds (0 = infinite)
    #[serde(default)]
    pub end_time: f64,
    /// Whether to loop playback
    #[serde(default)]
    pub loop_enabled: bool,
}

fn default_bpm() -> f64 {
    120.0
}

impl Default for PlaybackDef {
    fn default() -> Self {
        Self {
            bpm: 120.0,
            start_time: 0.0,
            end_time: 0.0,
            loop_enabled: true,
        }
    }
}

/// Camera/view settings for the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDef {
    /// Camera position
    #[serde(default = "default_camera_position")]
    pub camera_position: [f32; 3],
    /// Camera target (look-at point)
    #[serde(default = "default_camera_target")]
    pub camera_target: [f32; 3],
    /// Field of view in degrees
    #[serde(default = "default_fov")]
    pub fov: f32,
}

fn default_camera_position() -> [f32; 3] {
    [0.0, 0.0, 5.0]
}

fn default_camera_target() -> [f32; 3] {
    [0.0, 0.0, 0.0]
}

fn default_fov() -> f32 {
    60.0
}

impl Default for ViewDef {
    fn default() -> Self {
        Self {
            camera_position: default_camera_position(),
            camera_target: default_camera_target(),
            fov: default_fov(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_def_new() {
        let root_id = Id::new();
        let graph = GraphDef::new("Main Graph", root_id);

        assert_eq!(graph.name, "Main Graph");
        assert_eq!(graph.root_symbol, root_id);
        assert!(graph.instance_overrides.is_empty());
    }

    #[test]
    fn test_playback_def_default() {
        let playback = PlaybackDef::default();
        assert_eq!(playback.bpm, 120.0);
        assert!(playback.loop_enabled);
    }

    #[test]
    fn test_view_def_default() {
        let view = ViewDef::default();
        assert_eq!(view.camera_position, [0.0, 0.0, 5.0]);
        assert_eq!(view.fov, 60.0);
    }

    #[test]
    fn test_instance_override() {
        let input_id = Id::new();
        let override_def = InstanceOverride::new("effect1.colorizer")
            .with_input(input_id, Value::Float(0.5));

        assert_eq!(override_def.path, "effect1.colorizer");
        assert_eq!(override_def.inputs.len(), 1);
    }

    #[test]
    fn test_graph_file_serialize() {
        let root_id = Id::new();
        let mut graph = GraphDef::new("Test Graph", root_id)
            .with_description("A test composition");

        graph.playback.bpm = 140.0;
        graph.view.fov = 45.0;

        let file = GraphFile {
            version: SchemaVersion::CURRENT,
            graph,
        };

        let json = serde_json::to_string_pretty(&file).unwrap();
        assert!(json.contains("Test Graph"));
        assert!(json.contains("140"));
        assert!(json.contains("45"));

        let restored: GraphFile = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.graph.name, "Test Graph");
        assert_eq!(restored.graph.playback.bpm, 140.0);
    }

    #[test]
    fn test_port_ui_override() {
        let port_override = PortUiOverride::new(0)
            .with_range(0.5, 2.0)
            .with_label("Fine Freq")
            .with_unit("Hz");

        assert_eq!(port_override.port_index, 0);
        assert_eq!(port_override.range, Some((0.5, 2.0)));
        assert_eq!(port_override.label, Some("Fine Freq".to_string()));
        assert_eq!(port_override.unit, Some("Hz".to_string()));
        assert!(!port_override.is_empty());
    }

    #[test]
    fn test_port_ui_override_conversion() {
        let runtime = flux_core::PortOverride::new()
            .with_range(1.0, 10.0)
            .with_label("Custom");

        let serialized = PortUiOverride::from_port_override(2, &runtime);
        assert_eq!(serialized.port_index, 2);
        assert_eq!(serialized.range, Some((1.0, 10.0)));
        assert_eq!(serialized.label, Some("Custom".to_string()));

        let back = serialized.to_port_override();
        assert_eq!(back.range, runtime.range);
        assert_eq!(back.label, runtime.label);
    }

    #[test]
    fn test_instance_override_with_port_ui() {
        let input_id = Id::new();
        let override_def = InstanceOverride::new("oscillator1")
            .with_input(input_id, Value::Float(440.0))
            .with_port_ui(PortUiOverride::new(0).with_range(20.0, 2000.0).with_unit("Hz"));

        assert_eq!(override_def.path, "oscillator1");
        assert_eq!(override_def.inputs.len(), 1);
        assert_eq!(override_def.port_ui_overrides.len(), 1);
        assert_eq!(override_def.port_ui_overrides[0].port_index, 0);

        // Test serialization
        let json = serde_json::to_string_pretty(&override_def).unwrap();
        assert!(json.contains("port_ui_overrides"));
        assert!(json.contains("20.0"));
        assert!(json.contains("2000.0"));

        let restored: InstanceOverride = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.port_ui_overrides.len(), 1);
        assert_eq!(restored.port_ui_overrides[0].range, Some((20.0, 2000.0)));
    }

    #[test]
    fn test_port_ui_override_empty_skipped_in_serialization() {
        // Empty port_ui_overrides should be skipped in JSON
        let override_def = InstanceOverride::new("node1");
        let json = serde_json::to_string(&override_def).unwrap();
        assert!(!json.contains("port_ui_overrides"));
    }
}
