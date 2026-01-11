//! Visual metadata for operators
//!
//! This module provides traits and types for visual editor integration.
//! Operators can implement [`OperatorMeta`] to provide visual hints for
//! rendering in node editors like nodal.
//!
//! # Example
//!
//! ```ignore
//! use flux_core::{OperatorMeta, PortMeta, PinShape};
//!
//! impl OperatorMeta for MyOperator {
//!     fn category(&self) -> &'static str { "Math" }
//!
//!     fn input_meta(&self, index: usize) -> Option<PortMeta> {
//!         match index {
//!             0 => Some(PortMeta::new("A")),
//!             1 => Some(PortMeta::new("B")),
//!             _ => None,
//!         }
//!     }
//!
//!     fn output_meta(&self, index: usize) -> Option<PortMeta> {
//!         match index {
//!             0 => Some(PortMeta::new("Result").with_shape(PinShape::TriangleFilled)),
//!             _ => None,
//!         }
//!     }
//! }
//! ```

/// Visual metadata for operators.
///
/// Implement this trait alongside [`Operator`](crate::Operator) to provide
/// visual hints for node editors. All methods have sensible defaults, so you
/// only need to override what you want to customize.
pub trait OperatorMeta {
    /// Category for grouping in menus (e.g., "Math", "Sources", "Logic").
    ///
    /// Used to organize operators in add-node menus.
    fn category(&self) -> &'static str {
        "Uncategorized"
    }

    /// Category color as [R, G, B, A] in 0.0-1.0 range.
    ///
    /// Used to color-code nodes by category.
    fn category_color(&self) -> [f32; 4] {
        [0.5, 0.5, 0.5, 1.0]
    }

    /// Icon identifier (e.g., FontAwesome unicode or icon name).
    ///
    /// Optional icon to display in the node titlebar.
    fn icon(&self) -> Option<&'static str> {
        None
    }

    /// Short description for tooltips.
    fn description(&self) -> &'static str {
        ""
    }

    /// Input port metadata by index.
    ///
    /// Return `None` for indices beyond the operator's input count.
    fn input_meta(&self, _index: usize) -> Option<PortMeta> {
        None
    }

    /// Output port metadata by index.
    ///
    /// Return `None` for indices beyond the operator's output count.
    fn output_meta(&self, _index: usize) -> Option<PortMeta> {
        None
    }
}

/// Metadata for a single port (input or output).
#[derive(Debug, Clone)]
pub struct PortMeta {
    /// Display label for the port.
    pub label: &'static str,

    /// Visual shape hint for the pin.
    pub shape: PinShape,

    /// Optional color override [R, G, B, A].
    pub color: Option<[f32; 4]>,

    /// Value range for UI sliders [min, max].
    /// Used by inspectors to show appropriate controls.
    pub range: Option<(f32, f32)>,

    /// Unit suffix for display (e.g., "Hz", "ms", "rad").
    pub unit: Option<&'static str>,
}

impl PortMeta {
    /// Create a new PortMeta with just a label.
    pub const fn new(label: &'static str) -> Self {
        Self {
            label,
            shape: PinShape::CircleFilled,
            color: None,
            range: None,
            unit: None,
        }
    }

    /// Set the pin shape.
    pub const fn with_shape(mut self, shape: PinShape) -> Self {
        self.shape = shape;
        self
    }

    /// Set the color.
    pub const fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the value range for sliders.
    pub const fn with_range(mut self, min: f32, max: f32) -> Self {
        self.range = Some((min, max));
        self
    }

    /// Set the unit suffix.
    pub const fn with_unit(mut self, unit: &'static str) -> Self {
        self.unit = Some(unit);
        self
    }

    /// Returns true if this port represents a semantic parameter.
    ///
    /// Semantic parameters are inputs with meaningful names that should be displayed
    /// as labels (e.g., "Frequency", "Min", "Max") rather than showing their current value.
    ///
    /// The heuristic: if `range` or `unit` is set, the operator author has defined
    /// this as a parameter with specific meaning, so show the label. Otherwise,
    /// it's a generic signal input (like "A", "B") where showing the value is more useful.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // SineWave's Frequency input has range and unit → semantic → show "Frequency"
    /// PortMeta::new("Frequency").with_range(0.01, 100.0).with_unit("Hz")
    ///     .is_semantic_parameter() // true
    ///
    /// // Add's A input has no range/unit → signal → show the value "0"
    /// PortMeta::new("A")
    ///     .is_semantic_parameter() // false
    /// ```
    pub const fn is_semantic_parameter(&self) -> bool {
        self.range.is_some() || self.unit.is_some()
    }
}

impl Default for PortMeta {
    fn default() -> Self {
        Self::new("Port")
    }
}

/// Visual shape hint for pins in node editors.
///
/// These map to common shapes used in node-based editors.
/// The actual rendering is done by the visual layer (e.g., nodal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PinShape {
    /// Hollow circle (typically for unconnected optional inputs).
    Circle,
    /// Filled circle (default for most inputs).
    #[default]
    CircleFilled,
    /// Hollow triangle (typically for flow/execution ports).
    Triangle,
    /// Filled triangle (default for outputs).
    TriangleFilled,
    /// Hollow quad/diamond (typically for special types).
    Quad,
    /// Filled quad/diamond.
    QuadFilled,
}

/// Per-instance overrides for port UI behavior.
///
/// All fields are optional - `None` means "use `PortMeta` default".
/// Store these per-node in your graph to allow users to customize
/// parameter ranges, labels, etc. for individual instances.
///
/// # Example
///
/// ```ignore
/// // User wants fine control over frequency for this specific oscillator
/// let override_ = PortOverride::new()
///     .with_range(0.5, 2.0)  // Narrow from default 0-100 Hz
///     .with_label("Fine Freq");
/// ```
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortOverride {
    /// Custom UI range (None = use PortMeta default).
    pub range: Option<(f32, f32)>,

    /// Custom display label (None = use PortMeta default).
    pub label: Option<String>,

    /// Custom unit suffix (None = use PortMeta default).
    pub unit: Option<String>,

    /// Custom step size for UI controls (None = auto).
    pub step: Option<f32>,
}

impl PortOverride {
    /// Create an empty override (all fields None).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom range.
    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.range = Some((min, max));
        self
    }

    /// Set a custom label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set a custom unit suffix.
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Set a custom step size.
    pub fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Returns true if all fields are None (no overrides).
    pub fn is_empty(&self) -> bool {
        self.range.is_none()
            && self.label.is_none()
            && self.unit.is_none()
            && self.step.is_none()
    }
}

/// Resolved port metadata combining operator defaults and per-instance overrides.
///
/// Use this in UI code instead of querying `PortMeta` and `PortOverride` separately.
/// The fields are fully resolved - no Option chaining needed in rendering code.
///
/// # Example
///
/// ```ignore
/// let effective = EffectivePortMeta::from_meta(
///     operator.input_meta(0),
///     node.input_overrides.get(0),
/// );
///
/// // No need to check for overrides - already resolved
/// if let Some((min, max)) = effective.range {
///     ui.slider(&effective.label, min, max, &mut value);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct EffectivePortMeta {
    /// Resolved label (from override or PortMeta).
    pub label: String,

    /// Resolved range (from override or PortMeta, if any).
    pub range: Option<(f32, f32)>,

    /// Resolved unit suffix (from override or PortMeta, if any).
    pub unit: Option<String>,

    /// Step size from override (None = auto-calculate based on range).
    pub step: Option<f32>,

    /// Pin shape (from PortMeta - not overridable).
    pub shape: PinShape,

    /// Pin color override (from PortMeta - not overridable).
    pub color: Option<[f32; 4]>,
}

impl EffectivePortMeta {
    /// Create from PortMeta defaults, optionally applying per-instance overrides.
    ///
    /// # Arguments
    ///
    /// * `meta` - The operator's port metadata (None uses sensible defaults)
    /// * `override_` - Per-instance overrides (None means use all defaults)
    pub fn from_meta(meta: Option<PortMeta>, override_: Option<&PortOverride>) -> Self {
        let meta = meta.unwrap_or_default();
        let override_ = override_.cloned().unwrap_or_default();

        Self {
            label: override_.label.unwrap_or_else(|| meta.label.to_string()),
            range: override_.range.or(meta.range),
            unit: override_.unit.or_else(|| meta.unit.map(|s| s.to_string())),
            step: override_.step,
            shape: meta.shape,
            color: meta.color,
        }
    }
}

impl Default for EffectivePortMeta {
    fn default() -> Self {
        Self::from_meta(None, None)
    }
}

/// Standard category colors for common operator types.
///
/// These are optional conventions - operators can use any color.
pub mod category_colors {
    /// Sources (constants, time, inputs) - green
    pub const SOURCES: [f32; 4] = [0.25, 0.55, 0.35, 1.0];

    /// Time-related operators - teal/green
    pub const TIME: [f32; 4] = [0.20, 0.50, 0.45, 1.0];

    /// Math operations - blue
    pub const MATH: [f32; 4] = [0.35, 0.35, 0.55, 1.0];

    /// Oscillators and generators - purple
    pub const OSCILLATORS: [f32; 4] = [0.50, 0.35, 0.55, 1.0];

    /// Logic and control flow - orange
    pub const LOGIC: [f32; 4] = [0.55, 0.45, 0.25, 1.0];

    /// Range and interpolation - cyan
    pub const RANGE: [f32; 4] = [0.25, 0.50, 0.55, 1.0];

    /// Vectors and geometry - teal
    pub const VECTORS: [f32; 4] = [0.20, 0.55, 0.50, 1.0];

    /// Colors and gradients - pink
    pub const COLORS: [f32; 4] = [0.55, 0.35, 0.45, 1.0];

    /// Output/display nodes - red
    pub const OUTPUT: [f32; 4] = [0.55, 0.30, 0.30, 1.0];

    /// State and memory - yellow
    pub const STATE: [f32; 4] = [0.55, 0.50, 0.25, 1.0];

    /// Uncategorized - gray
    pub const UNCATEGORIZED: [f32; 4] = [0.50, 0.50, 0.50, 1.0];

    /// List operations - coral/salmon
    pub const LIST: [f32; 4] = [0.55, 0.40, 0.35, 1.0];

    /// Flow control - orange/amber
    pub const FLOW: [f32; 4] = [0.60, 0.45, 0.25, 1.0];

    /// Utility/Debug - gray-blue
    pub const UTIL: [f32; 4] = [0.45, 0.45, 0.50, 1.0];

    /// String operations - light blue/cyan
    pub const STRING: [f32; 4] = [0.35, 0.50, 0.55, 1.0];
}
