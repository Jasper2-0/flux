//! Operator visualization data
//!
//! This module provides traits and types for operators that expose internal
//! state for custom rendering in UIs. Unlike operator settings (which are
//! editable), visualization data is read-only.
//!
//! # Example
//!
//! ```ignore
//! use flux_core::{OperatorVisuals, VisualData};
//!
//! impl OperatorVisuals for ScopeOp {
//!     fn visual_data(&self) -> VisualData {
//!         VisualData::Waveform {
//!             samples: self.buffer.iter().copied().collect(),
//!             range: self.value_range(),
//!         }
//!     }
//! }
//! ```

/// Read-only visualization data from an operator.
///
/// This enum represents different kinds of data that operators can expose
/// for custom rendering. The data is read-only - the operator's state
/// cannot be modified through this.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum VisualData {
    /// No visualization data available.
    ///
    /// This is the default for most operators.
    #[default]
    None,

    /// Waveform display data (for oscilloscope-like displays).
    ///
    /// Used by operators like `ScopeOp` that maintain a ring buffer
    /// of recent values for visualization.
    Waveform {
        /// Sample values in chronological order.
        samples: Vec<f32>,
        /// Value range (min, max) for auto-scaling the display.
        range: (f32, f32),
    },
    // Future variants could include:
    // - Histogram { bins: Vec<f32>, range: (f32, f32) }
    // - XYPlot { x: Vec<f32>, y: Vec<f32> }
    // - Spectrum { magnitudes: Vec<f32>, frequencies: Vec<f32> }
}

impl VisualData {
    /// Returns true if this is `VisualData::None`.
    pub fn is_none(&self) -> bool {
        matches!(self, VisualData::None)
    }

    /// Returns the waveform data if this is a `Waveform` variant.
    pub fn as_waveform(&self) -> Option<(&[f32], (f32, f32))> {
        match self {
            VisualData::Waveform { samples, range } => Some((samples.as_slice(), *range)),
            _ => None,
        }
    }
}


/// Trait for operators that provide visualization data.
///
/// Implement this trait for operators that maintain internal state
/// suitable for custom rendering (e.g., oscilloscope displays,
/// histograms, spectrograms).
///
/// # Default Implementation
///
/// The default implementation returns `VisualData::None`, which is
/// correct for most operators.
///
/// # Example
///
/// ```ignore
/// impl OperatorVisuals for ScopeOp {
///     fn visual_data(&self) -> VisualData {
///         VisualData::Waveform {
///             samples: self.buffer.iter().copied().collect(),
///             range: self.value_range(),
///         }
///     }
/// }
/// ```
pub trait OperatorVisuals {
    /// Get visualization data for custom rendering.
    ///
    /// Returns `VisualData::None` for most operators.
    fn visual_data(&self) -> VisualData {
        VisualData::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_data_none() {
        let data = VisualData::None;
        assert!(data.is_none());
        assert!(data.as_waveform().is_none());
    }

    #[test]
    fn test_visual_data_waveform() {
        let data = VisualData::Waveform {
            samples: vec![0.0, 0.5, 1.0, 0.5, 0.0],
            range: (-1.0, 1.0),
        };
        assert!(!data.is_none());

        let (samples, range) = data.as_waveform().unwrap();
        assert_eq!(samples.len(), 5);
        assert_eq!(range, (-1.0, 1.0));
    }

    #[test]
    fn test_default() {
        let data = VisualData::default();
        assert!(data.is_none());
    }
}
