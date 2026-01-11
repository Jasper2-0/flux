//! Bypass system for operators
//!
//! Bypass allows an operator to be "skipped" by passing its input directly
//! to its output. This is useful for temporarily disabling an operator's
//! effect while maintaining graph connectivity.
//!
//! Not all types support bypass - only types where direct pass-through
//! makes semantic sense (like Float, Vec3, etc.).

use serde::{Deserialize, Serialize};

use flux_core::port::{InputPort, OutputPort};
use flux_core::value::{Value, ValueType};

/// Types that support bypass (direct pass-through)
///
/// Only certain types can be bypassed because bypass requires that
/// passing the input directly to the output makes sense semantically.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BypassableType {
    /// Single float value
    Float,
    /// 2D vector
    Vec2,
    /// 3D vector
    Vec3,
    /// 4D vector
    Vec4,
    /// String value
    String,
    /// Integer value
    Int,
    // GPU types would go here in a full implementation:
    // Command,
    // Texture2D,
    // Buffer,
}

impl BypassableType {
    /// Convert from ValueType to BypassableType if supported
    pub fn from_value_type(vt: ValueType) -> Option<Self> {
        match vt {
            ValueType::Float => Some(Self::Float),
            ValueType::Int => Some(Self::Int),
            ValueType::Vec2 => Some(Self::Vec2),
            ValueType::Vec3 => Some(Self::Vec3),
            ValueType::Vec4 => Some(Self::Vec4),
            ValueType::String => Some(Self::String),
            // Bool is not bypassable - it doesn't make sense semantically
            ValueType::Bool => None,
            // New types - not currently bypassable
            ValueType::Color => None,
            ValueType::Gradient => None,
            ValueType::Matrix4 => None,
            ValueType::FloatList => None,
            ValueType::IntList => None,
            ValueType::Vec3List => None,
        }
    }

    /// Convert to ValueType
    pub fn to_value_type(self) -> ValueType {
        match self {
            Self::Float => ValueType::Float,
            Self::Int => ValueType::Int,
            Self::Vec2 => ValueType::Vec2,
            Self::Vec3 => ValueType::Vec3,
            Self::Vec4 => ValueType::Vec4,
            Self::String => ValueType::String,
        }
    }

    /// Check if a ValueType is bypassable
    pub fn is_bypassable(vt: ValueType) -> bool {
        Self::from_value_type(vt).is_some()
    }
}

/// Information about bypass capability of an operator
#[derive(Clone, Debug, Default)]
pub struct BypassInfo {
    /// Whether the operator can be bypassed
    pub can_bypass: bool,
    /// The input index to use for bypass (first matching input)
    pub bypass_input_index: Option<usize>,
    /// The output index to use for bypass (first matching output)
    pub bypass_output_index: Option<usize>,
    /// The type used for bypass
    pub bypass_type: Option<BypassableType>,
    /// All possible bypass pairs (input_idx, output_idx, type)
    pub bypass_pairs: Vec<(usize, usize, BypassableType)>,
}

impl BypassInfo {
    /// Check if bypass is possible
    pub fn is_bypassable(&self) -> bool {
        self.can_bypass
    }

    /// Get the primary bypass pair
    pub fn primary_pair(&self) -> Option<(usize, usize)> {
        match (self.bypass_input_index, self.bypass_output_index) {
            (Some(i), Some(o)) => Some((i, o)),
            _ => None,
        }
    }
}

/// Check if an operator can be bypassed based on its input/output slots
///
/// An operator can be bypassed if it has at least one input and one output
/// of the same bypassable type.
pub fn check_bypassable(inputs: &[InputPort], outputs: &[OutputPort]) -> BypassInfo {
    let mut info = BypassInfo::default();

    // Find all matching input/output pairs
    for (input_idx, input) in inputs.iter().enumerate() {
        if let Some(bypass_type) = BypassableType::from_value_type(input.value_type) {
            for (output_idx, output) in outputs.iter().enumerate() {
                if BypassableType::from_value_type(output.value_type) == Some(bypass_type) {
                    info.bypass_pairs.push((input_idx, output_idx, bypass_type));

                    // Set primary bypass pair (first found)
                    if !info.can_bypass {
                        info.can_bypass = true;
                        info.bypass_input_index = Some(input_idx);
                        info.bypass_output_index = Some(output_idx);
                        info.bypass_type = Some(bypass_type);
                    }
                }
            }
        }
    }

    info
}

/// Check if an operator can be bypassed (simple boolean check)
pub fn is_bypassable(inputs: &[InputPort], outputs: &[OutputPort]) -> bool {
    check_bypassable(inputs, outputs).can_bypass
}

/// Get the bypass value from an input slot
///
/// This returns the value that should be passed through when bypassed.
pub fn get_bypass_value(input: &InputPort) -> Value {
    // When bypassed, we pass through the default value
    // In a connected graph, this would be the connected input's value
    input.default.clone()
}

/// Bypass configuration for an operator instance
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BypassState {
    /// Whether bypass is currently enabled
    pub enabled: bool,
    /// Which input to use for bypass
    pub input_index: usize,
    /// Which output to use for bypass
    pub output_index: usize,
}

impl BypassState {
    /// Create a new bypass state
    pub fn new(input_index: usize, output_index: usize) -> Self {
        Self {
            enabled: false,
            input_index,
            output_index,
        }
    }

    /// Create from bypass info (using primary pair)
    pub fn from_info(info: &BypassInfo) -> Option<Self> {
        info.primary_pair().map(|(i, o)| Self::new(i, o))
    }

    /// Enable bypass
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable bypass
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Toggle bypass
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

/// Trait for operators that support bypass
pub trait Bypassable {
    /// Check if this operator can be bypassed
    fn can_bypass(&self) -> bool;

    /// Get bypass information
    fn bypass_info(&self) -> BypassInfo;

    /// Check if bypass is currently enabled
    fn is_bypassed(&self) -> bool;

    /// Set bypass state
    fn set_bypassed(&mut self, bypassed: bool);

    /// Get the bypass value (input value to pass through)
    fn get_bypass_value(&self) -> Option<Value>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bypassable_type_conversion() {
        assert_eq!(
            BypassableType::from_value_type(ValueType::Float),
            Some(BypassableType::Float)
        );
        assert_eq!(
            BypassableType::from_value_type(ValueType::Vec3),
            Some(BypassableType::Vec3)
        );
        assert_eq!(BypassableType::from_value_type(ValueType::Bool), None);

        assert_eq!(BypassableType::Float.to_value_type(), ValueType::Float);
    }

    #[test]
    fn test_is_bypassable() {
        assert!(BypassableType::is_bypassable(ValueType::Float));
        assert!(BypassableType::is_bypassable(ValueType::Vec3));
        assert!(BypassableType::is_bypassable(ValueType::String));
        assert!(!BypassableType::is_bypassable(ValueType::Bool));
    }

    #[test]
    fn test_check_bypassable_matching() {
        let inputs = vec![InputPort::float("A", 0.0), InputPort::float("B", 0.0)];
        let outputs = vec![OutputPort::float("Result")];

        let info = check_bypassable(&inputs, &outputs);

        assert!(info.can_bypass);
        assert_eq!(info.bypass_input_index, Some(0));
        assert_eq!(info.bypass_output_index, Some(0));
        assert_eq!(info.bypass_type, Some(BypassableType::Float));
        assert_eq!(info.bypass_pairs.len(), 2); // Both inputs match the output
    }

    #[test]
    fn test_check_bypassable_no_match() {
        let inputs = vec![InputPort::float("A", 0.0)];
        let outputs = vec![OutputPort::bool("Result")];

        let info = check_bypassable(&inputs, &outputs);

        assert!(!info.can_bypass);
        assert!(info.bypass_pairs.is_empty());
    }

    #[test]
    fn test_check_bypassable_vec3() {
        let inputs = vec![InputPort::vec3("Position", [0.0, 0.0, 0.0])];
        let outputs = vec![OutputPort::vec3("Result")];

        let info = check_bypassable(&inputs, &outputs);

        assert!(info.can_bypass);
        assert_eq!(info.bypass_type, Some(BypassableType::Vec3));
    }

    #[test]
    fn test_bypass_state() {
        let mut state = BypassState::new(0, 0);

        assert!(!state.enabled);

        state.enable();
        assert!(state.enabled);

        state.toggle();
        assert!(!state.enabled);

        state.toggle();
        assert!(state.enabled);
    }

    #[test]
    fn test_bypass_info_primary_pair() {
        let inputs = vec![
            InputPort::bool("Condition", false),
            InputPort::float("Value", 1.0),
        ];
        let outputs = vec![OutputPort::float("Result")];

        let info = check_bypassable(&inputs, &outputs);

        // Should find the float pair (index 1 -> 0)
        assert!(info.can_bypass);
        assert_eq!(info.primary_pair(), Some((1, 0)));
    }
}
