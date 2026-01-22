//! Operator settings for editable parameters
//!
//! This module provides traits and types for exposing operator-level settings
//! to UIs in a type-safe, generic way. Unlike port defaults (handled by
//! `Operator::inputs()`), these are operator-specific settings like comparison
//! modes, waveform types, etc.
//!
//! # Example
//!
//! ```ignore
//! use flux_core::{OperatorSettings, ParamDef, ParamValue};
//!
//! impl OperatorSettings for CompareOp {
//!     fn params(&self) -> Vec<ParamDef> {
//!         vec![ParamDef {
//!             name: "mode",
//!             label: "Comparison Mode",
//!             value: ParamValue::Enum {
//!                 index: self.mode as usize,
//!                 options: &["==", "!=", "<", "<=", ">", ">="],
//!             },
//!         }]
//!     }
//!
//!     fn set_param(&mut self, name: &str, value: ParamValue) -> bool {
//!         if name == "mode" {
//!             if let ParamValue::Enum { index, .. } = value {
//!                 self.mode = CompareMode::from_index(index);
//!                 return true;
//!             }
//!         }
//!         false
//!     }
//! }
//! ```

/// Value types for operator parameters.
///
/// These represent the different kinds of editable settings an operator
/// can expose to the UI.
///
/// Note: When setting enum parameters, only the `index` field is used.
/// The `options` field is provided for convenience when constructing values
/// for UI display but is ignored when setting values.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    /// Float parameter (rendered as slider or input field).
    Float(f32),

    /// Integer parameter.
    Int(i32),

    /// Boolean parameter (rendered as checkbox or toggle).
    Bool(bool),

    /// Enum parameter with named variants (rendered as combo box/dropdown).
    ///
    /// When setting a parameter, only `index` is used. The `options` field
    /// is for UI display purposes and can be empty when just setting a value.
    Enum {
        /// Current variant index (0-based).
        index: usize,
        /// Available variant display names for UI (can be empty when setting).
        options: Vec<&'static str>,
    },
}

impl ParamValue {
    /// Create an enum value for setting a parameter (options not needed).
    pub fn enum_index(index: usize) -> Self {
        ParamValue::Enum { index, options: vec![] }
    }
}

impl ParamValue {
    /// Get as float, if this is a Float variant.
    pub fn as_float(&self) -> Option<f32> {
        match self {
            ParamValue::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as int, if this is an Int variant.
    pub fn as_int(&self) -> Option<i32> {
        match self {
            ParamValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as bool, if this is a Bool variant.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ParamValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as enum index, if this is an Enum variant.
    pub fn as_enum_index(&self) -> Option<usize> {
        match self {
            ParamValue::Enum { index, .. } => Some(*index),
            _ => None,
        }
    }
}

/// Definition of a single editable parameter on an operator.
///
/// This provides enough information for a UI to render an appropriate
/// control for editing the parameter.
#[derive(Debug, Clone)]
pub struct ParamDef {
    /// Parameter name (machine identifier, e.g., "mode", "waveform").
    ///
    /// Used when calling `set_param()` to identify which parameter to update.
    pub name: &'static str,

    /// Display label for UI (human-readable, e.g., "Comparison Mode").
    pub label: &'static str,

    /// Current value of the parameter.
    pub value: ParamValue,
}

impl ParamDef {
    /// Create a new float parameter definition.
    pub fn float(name: &'static str, label: &'static str, value: f32) -> Self {
        Self {
            name,
            label,
            value: ParamValue::Float(value),
        }
    }

    /// Create a new int parameter definition.
    pub fn int(name: &'static str, label: &'static str, value: i32) -> Self {
        Self {
            name,
            label,
            value: ParamValue::Int(value),
        }
    }

    /// Create a new bool parameter definition.
    pub fn bool(name: &'static str, label: &'static str, value: bool) -> Self {
        Self {
            name,
            label,
            value: ParamValue::Bool(value),
        }
    }

    /// Create a new enum parameter definition.
    pub fn enumeration(
        name: &'static str,
        label: &'static str,
        index: usize,
        options: &[&'static str],
    ) -> Self {
        Self {
            name,
            label,
            value: ParamValue::Enum { index, options: options.to_vec() },
        }
    }
}

/// Trait for operators with editable settings beyond input defaults.
///
/// Input port defaults (A=0, B=0) are handled by `Operator::inputs()`.
/// This trait is for *operator-level* settings like comparison mode,
/// waveform type, etc.
///
/// # Default Implementation
///
/// The default implementation returns no parameters, which is correct for
/// most operators that only have input port defaults.
///
/// # Example
///
/// ```ignore
/// impl OperatorSettings for CompareOp {
///     fn params(&self) -> Vec<ParamDef> {
///         vec![ParamDef::enumeration(
///             "mode",
///             "Comparison Mode",
///             self.mode as usize,
///             &["==", "!=", "<", "<=", ">", ">="],
///         )]
///     }
///
///     fn set_param(&mut self, name: &str, value: ParamValue) -> bool {
///         if name == "mode" {
///             if let ParamValue::Enum { index, .. } = value {
///                 self.mode = CompareMode::from_index(index)?;
///                 return true;
///             }
///         }
///         false
///     }
/// }
/// ```
pub trait OperatorSettings {
    /// Returns all editable parameters.
    ///
    /// The default implementation returns an empty vector (no settings).
    fn params(&self) -> Vec<ParamDef> {
        vec![]
    }

    /// Set a parameter by name.
    ///
    /// Returns `true` if the parameter was found and successfully set,
    /// `false` if the parameter doesn't exist or the value type is wrong.
    fn set_param(&mut self, name: &str, value: ParamValue) -> bool {
        // Silence unused variable warnings in default impl
        let _ = (name, value);
        false
    }
}

#[cfg(test)]
mod tests {
    use std::{f32::consts::PI};

    use super::*;

    #[test]
    fn test_param_value_accessors() {
        let f = ParamValue::Float(PI);
        assert_eq!(f.as_float(), Some(PI));
        assert_eq!(f.as_int(), None);

        let i = ParamValue::Int(42);
        assert_eq!(i.as_int(), Some(42));

        let b = ParamValue::Bool(true);
        assert_eq!(b.as_bool(), Some(true));

        let e = ParamValue::Enum {
            index: 2,
            options: vec!["a", "b", "c"],
        };
        assert_eq!(e.as_enum_index(), Some(2));
    }

    #[test]
    fn test_param_def_constructors() {
        let f = ParamDef::float("freq", "Frequency", 440.0);
        assert_eq!(f.name, "freq");
        assert_eq!(f.label, "Frequency");
        assert!(matches!(f.value, ParamValue::Float(440.0)));

        let e = ParamDef::enumeration("mode", "Mode", 1, &["A", "B", "C"]);
        match &e.value {
            ParamValue::Enum { index, options } => {
                assert_eq!(*index, 1);
                assert_eq!(options.as_slice(), &["A", "B", "C"]);
            }
            _ => panic!("Expected Enum variant"),
        }
    }
}
