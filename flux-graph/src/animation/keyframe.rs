use serde::{Deserialize, Serialize};

use super::Interpolation;

/// A single keyframe in an animation curve
///
/// Keyframes define the value at a specific point in time, along with
/// interpolation settings for how to transition to/from adjacent keyframes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    /// Time position (in bars or seconds, depending on context)
    pub time: f64,
    /// Value at this keyframe
    pub value: f64,
    /// Interpolation mode for incoming curve (from previous keyframe)
    pub in_type: Interpolation,
    /// Interpolation mode for outgoing curve (to next keyframe)
    pub out_type: Interpolation,
    /// Tangent for incoming spline (used when in_type is Spline)
    pub in_tangent: f64,
    /// Tangent for outgoing spline (used when out_type is Spline)
    pub out_tangent: f64,
}

impl Keyframe {
    /// Create a new keyframe with default linear interpolation
    pub fn new(time: f64, value: f64) -> Self {
        Self {
            time,
            value,
            in_type: Interpolation::Linear,
            out_type: Interpolation::Linear,
            in_tangent: 0.0,
            out_tangent: 0.0,
        }
    }

    /// Create a keyframe with constant (step) interpolation
    pub fn constant(time: f64, value: f64) -> Self {
        Self {
            time,
            value,
            in_type: Interpolation::Constant,
            out_type: Interpolation::Constant,
            in_tangent: 0.0,
            out_tangent: 0.0,
        }
    }

    /// Create a keyframe with spline interpolation
    pub fn spline(time: f64, value: f64, in_tangent: f64, out_tangent: f64) -> Self {
        Self {
            time,
            value,
            in_type: Interpolation::Spline,
            out_type: Interpolation::Spline,
            in_tangent,
            out_tangent,
        }
    }

    /// Create a keyframe with linear interpolation
    pub fn linear(time: f64, value: f64) -> Self {
        Self::new(time, value)
    }

    /// Set interpolation modes
    pub fn with_interpolation(mut self, in_type: Interpolation, out_type: Interpolation) -> Self {
        self.in_type = in_type;
        self.out_type = out_type;
        self
    }

    /// Set tangents for spline interpolation
    pub fn with_tangents(mut self, in_tangent: f64, out_tangent: f64) -> Self {
        self.in_tangent = in_tangent;
        self.out_tangent = out_tangent;
        self
    }

    /// Check if this keyframe uses spline interpolation
    pub fn uses_spline(&self) -> bool {
        self.in_type == Interpolation::Spline || self.out_type == Interpolation::Spline
    }

    /// Auto-calculate tangent based on neighboring keyframes
    /// Uses Catmull-Rom style tangent calculation
    pub fn auto_tangent(prev: Option<&Keyframe>, current: &Keyframe, next: Option<&Keyframe>) -> f64 {
        match (prev, next) {
            (Some(p), Some(n)) => {
                // Average slope from prev to next
                let dt = n.time - p.time;
                if dt.abs() < 1e-10 {
                    0.0
                } else {
                    (n.value - p.value) / dt
                }
            }
            (Some(p), None) => {
                // Only previous: use slope to current
                let dt = current.time - p.time;
                if dt.abs() < 1e-10 {
                    0.0
                } else {
                    (current.value - p.value) / dt
                }
            }
            (None, Some(n)) => {
                // Only next: use slope from current
                let dt = n.time - current.time;
                if dt.abs() < 1e-10 {
                    0.0
                } else {
                    (n.value - current.value) / dt
                }
            }
            (None, None) => 0.0,
        }
    }
}

impl Default for Keyframe {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_constructors() {
        let k1 = Keyframe::new(1.0, 5.0);
        assert_eq!(k1.time, 1.0);
        assert_eq!(k1.value, 5.0);
        assert_eq!(k1.in_type, Interpolation::Linear);

        let k2 = Keyframe::constant(2.0, 10.0);
        assert_eq!(k2.in_type, Interpolation::Constant);
        assert_eq!(k2.out_type, Interpolation::Constant);

        let k3 = Keyframe::spline(3.0, 15.0, 0.5, -0.5);
        assert_eq!(k3.in_tangent, 0.5);
        assert_eq!(k3.out_tangent, -0.5);
    }

    #[test]
    fn test_auto_tangent() {
        let k1 = Keyframe::new(0.0, 0.0);
        let k2 = Keyframe::new(1.0, 10.0);
        let k3 = Keyframe::new(2.0, 20.0);

        // Linear progression: tangent should be 10
        let tangent = Keyframe::auto_tangent(Some(&k1), &k2, Some(&k3));
        assert!((tangent - 10.0).abs() < 0.001);
    }
}
