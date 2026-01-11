use serde::{Deserialize, Serialize};

/// Interpolation mode for keyframes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Interpolation {
    /// Step function - value jumps instantly at keyframe
    Constant,
    /// Linear interpolation between keyframes
    #[default]
    Linear,
    /// Cubic bezier spline interpolation
    Spline,
}

impl Interpolation {
    /// Interpolate between two values
    pub fn interpolate(&self, a: f64, b: f64, t: f64) -> f64 {
        match self {
            Interpolation::Constant => a,
            Interpolation::Linear => Self::lerp(a, b, t),
            Interpolation::Spline => {
                // For spline, we use smooth step as a simple approximation
                // Full bezier requires tangent information from keyframes
                let t = Self::smoothstep(t);
                Self::lerp(a, b, t)
            }
        }
    }

    /// Linear interpolation
    #[inline]
    pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }

    /// Smooth step function for easing
    #[inline]
    pub fn smoothstep(t: f64) -> f64 {
        t * t * (3.0 - 2.0 * t)
    }

    /// Cubic bezier interpolation with control points
    pub fn cubic_bezier(p0: f64, p1: f64, p2: f64, p3: f64, t: f64) -> f64 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        mt3 * p0 + 3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3 * p3
    }

    /// Hermite spline interpolation (used for smooth curves with tangents)
    pub fn hermite(p0: f64, m0: f64, p1: f64, m1: f64, t: f64) -> f64 {
        let t2 = t * t;
        let t3 = t2 * t;

        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + t;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;

        h00 * p0 + h10 * m0 + h01 * p1 + h11 * m1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_interpolation() {
        let interp = Interpolation::Constant;
        assert_eq!(interp.interpolate(0.0, 10.0, 0.0), 0.0);
        assert_eq!(interp.interpolate(0.0, 10.0, 0.5), 0.0);
        assert_eq!(interp.interpolate(0.0, 10.0, 0.99), 0.0);
    }

    #[test]
    fn test_linear_interpolation() {
        let interp = Interpolation::Linear;
        assert_eq!(interp.interpolate(0.0, 10.0, 0.0), 0.0);
        assert_eq!(interp.interpolate(0.0, 10.0, 0.5), 5.0);
        assert_eq!(interp.interpolate(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(Interpolation::lerp(0.0, 100.0, 0.25), 25.0);
        assert_eq!(Interpolation::lerp(-10.0, 10.0, 0.5), 0.0);
    }

    #[test]
    fn test_smoothstep() {
        // Smoothstep should be 0 at 0, 0.5 at 0.5, 1 at 1
        assert_eq!(Interpolation::smoothstep(0.0), 0.0);
        assert_eq!(Interpolation::smoothstep(0.5), 0.5);
        assert_eq!(Interpolation::smoothstep(1.0), 1.0);
    }
}
