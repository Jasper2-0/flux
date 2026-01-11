use serde::{Deserialize, Serialize};

use super::{Interpolation, Keyframe};

/// An animation curve containing keyframes
///
/// The curve stores a sorted list of keyframes and provides methods for
/// sampling values at any point in time. Keyframes are automatically
/// sorted by time when sampling.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Curve {
    /// The keyframes in this curve
    keyframes: Vec<Keyframe>,
    /// Whether the keyframes are currently sorted by time
    sorted: bool,
    /// Optional name for the curve
    #[serde(default)]
    pub name: Option<String>,
}

impl Curve {
    /// Create a new empty curve
    pub fn new() -> Self {
        Self {
            keyframes: Vec::new(),
            sorted: true,
            name: None,
        }
    }

    /// Create a curve with a name
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            keyframes: Vec::new(),
            sorted: true,
            name: Some(name.into()),
        }
    }

    /// Create a curve from a list of keyframes
    pub fn from_keyframes(keyframes: Vec<Keyframe>) -> Self {
        Self {
            keyframes,
            sorted: false,
            name: None,
        }
    }

    /// Add a keyframe to the curve
    pub fn add_keyframe(&mut self, keyframe: Keyframe) {
        self.keyframes.push(keyframe);
        self.sorted = false;
    }

    /// Add a simple linear keyframe at the given time and value
    pub fn add(&mut self, time: f64, value: f64) {
        self.add_keyframe(Keyframe::new(time, value));
    }

    /// Add a constant (step) keyframe
    pub fn add_constant(&mut self, time: f64, value: f64) {
        self.add_keyframe(Keyframe::constant(time, value));
    }

    /// Add a spline keyframe with tangents
    pub fn add_spline(&mut self, time: f64, value: f64, in_tangent: f64, out_tangent: f64) {
        self.add_keyframe(Keyframe::spline(time, value, in_tangent, out_tangent));
    }

    /// Remove keyframe at the specified time (if exists)
    pub fn remove_keyframe_at(&mut self, time: f64) -> Option<Keyframe> {
        self.ensure_sorted();
        if let Some(idx) = self.keyframes.iter().position(|k| (k.time - time).abs() < 1e-10) {
            Some(self.keyframes.remove(idx))
        } else {
            None
        }
    }

    /// Get the keyframe at a specific time (if one exists)
    pub fn get_keyframe(&self, time: f64) -> Option<&Keyframe> {
        self.keyframes.iter().find(|k| (k.time - time).abs() < 1e-10)
    }

    /// Get a mutable reference to a keyframe at a specific time
    pub fn get_keyframe_mut(&mut self, time: f64) -> Option<&mut Keyframe> {
        self.keyframes.iter_mut().find(|k| (k.time - time).abs() < 1e-10)
    }

    /// Get all keyframes (unsorted)
    pub fn keyframes(&self) -> &[Keyframe] {
        &self.keyframes
    }

    /// Get the number of keyframes
    pub fn len(&self) -> usize {
        self.keyframes.len()
    }

    /// Check if the curve has no keyframes
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }

    /// Clear all keyframes
    pub fn clear(&mut self) {
        self.keyframes.clear();
        self.sorted = true;
    }

    /// Get the time range of this curve (min_time, max_time)
    pub fn time_range(&mut self) -> Option<(f64, f64)> {
        if self.keyframes.is_empty() {
            return None;
        }
        self.ensure_sorted();
        let first = self.keyframes.first().unwrap();
        let last = self.keyframes.last().unwrap();
        Some((first.time, last.time))
    }

    /// Get the value range of this curve (min_value, max_value)
    pub fn value_range(&self) -> Option<(f64, f64)> {
        if self.keyframes.is_empty() {
            return None;
        }
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for k in &self.keyframes {
            min = min.min(k.value);
            max = max.max(k.value);
        }
        Some((min, max))
    }

    /// Ensure keyframes are sorted by time
    fn ensure_sorted(&mut self) {
        if !self.sorted {
            self.keyframes.sort_by(|a, b| {
                a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal)
            });
            self.sorted = true;
        }
    }

    /// Sample the curve at a given time
    ///
    /// Returns the interpolated value at the specified time.
    /// - Before first keyframe: returns first keyframe value
    /// - After last keyframe: returns last keyframe value
    /// - Between keyframes: interpolates based on keyframe settings
    pub fn sample(&mut self, time: f64) -> f64 {
        if self.keyframes.is_empty() {
            return 0.0;
        }

        self.ensure_sorted();

        // Handle edge cases
        let first = &self.keyframes[0];
        if time <= first.time {
            return first.value;
        }

        let last = &self.keyframes[self.keyframes.len() - 1];
        if time >= last.time {
            return last.value;
        }

        // Find the two keyframes to interpolate between
        let (k0, k1) = self.find_surrounding_keyframes(time);

        // Calculate interpolation parameter
        let dt = k1.time - k0.time;
        let t = if dt.abs() < 1e-10 {
            0.0
        } else {
            (time - k0.time) / dt
        };

        // Interpolate based on the outgoing type of k0 and incoming type of k1
        self.interpolate_between(k0, k1, t)
    }

    /// Find the two keyframes surrounding the given time
    /// Assumes keyframes are sorted and time is within range
    fn find_surrounding_keyframes(&self, time: f64) -> (&Keyframe, &Keyframe) {
        for i in 0..self.keyframes.len() - 1 {
            if time >= self.keyframes[i].time && time < self.keyframes[i + 1].time {
                return (&self.keyframes[i], &self.keyframes[i + 1]);
            }
        }
        // Shouldn't reach here if time is properly within range
        let len = self.keyframes.len();
        (&self.keyframes[len - 2], &self.keyframes[len - 1])
    }

    /// Interpolate between two keyframes
    fn interpolate_between(&self, k0: &Keyframe, k1: &Keyframe, t: f64) -> f64 {
        // Use the outgoing interpolation of the first keyframe
        match k0.out_type {
            Interpolation::Constant => k0.value,
            Interpolation::Linear => Interpolation::lerp(k0.value, k1.value, t),
            Interpolation::Spline => {
                // Use Hermite interpolation with tangents
                let dt = k1.time - k0.time;
                let m0 = k0.out_tangent * dt;
                let m1 = k1.in_tangent * dt;
                Interpolation::hermite(k0.value, m0, k1.value, m1, t)
            }
        }
    }

    /// Sample multiple points along the curve (for visualization)
    pub fn sample_range(&mut self, start: f64, end: f64, num_samples: usize) -> Vec<(f64, f64)> {
        if num_samples < 2 {
            return vec![(start, self.sample(start))];
        }

        let step = (end - start) / (num_samples - 1) as f64;
        (0..num_samples)
            .map(|i| {
                let t = start + step * i as f64;
                (t, self.sample(t))
            })
            .collect()
    }

    /// Auto-calculate tangents for all spline keyframes using Catmull-Rom
    pub fn auto_tangents(&mut self) {
        self.ensure_sorted();

        let len = self.keyframes.len();
        if len < 2 {
            return;
        }

        // Calculate tangents for each keyframe
        let tangents: Vec<f64> = (0..len)
            .map(|i| {
                let prev = if i > 0 { Some(&self.keyframes[i - 1]) } else { None };
                let next = if i < len - 1 { Some(&self.keyframes[i + 1]) } else { None };
                Keyframe::auto_tangent(prev, &self.keyframes[i], next)
            })
            .collect();

        // Apply tangents to keyframes
        for (i, tangent) in tangents.into_iter().enumerate() {
            if self.keyframes[i].uses_spline() {
                self.keyframes[i].in_tangent = tangent;
                self.keyframes[i].out_tangent = tangent;
            }
        }
    }
}

/// Builder pattern for creating curves
pub struct CurveBuilder {
    curve: Curve,
}

impl CurveBuilder {
    pub fn new() -> Self {
        Self {
            curve: Curve::new(),
        }
    }

    pub fn named(name: impl Into<String>) -> Self {
        Self {
            curve: Curve::named(name),
        }
    }

    pub fn keyframe(mut self, time: f64, value: f64) -> Self {
        self.curve.add(time, value);
        self
    }

    pub fn constant(mut self, time: f64, value: f64) -> Self {
        self.curve.add_constant(time, value);
        self
    }

    pub fn spline(mut self, time: f64, value: f64, in_tangent: f64, out_tangent: f64) -> Self {
        self.curve.add_spline(time, value, in_tangent, out_tangent);
        self
    }

    pub fn auto_tangents(mut self) -> Self {
        self.curve.auto_tangents();
        self
    }

    pub fn build(self) -> Curve {
        self.curve
    }
}

impl Default for CurveBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_curve() {
        let mut curve = Curve::new();
        assert!(curve.is_empty());
        assert_eq!(curve.sample(0.0), 0.0);
        assert_eq!(curve.sample(1.0), 0.0);
    }

    #[test]
    fn test_single_keyframe() {
        let mut curve = Curve::new();
        curve.add(1.0, 5.0);

        assert_eq!(curve.sample(0.0), 5.0); // Before
        assert_eq!(curve.sample(1.0), 5.0); // At
        assert_eq!(curve.sample(2.0), 5.0); // After
    }

    #[test]
    fn test_linear_interpolation() {
        let mut curve = Curve::new();
        curve.add(0.0, 0.0);
        curve.add(1.0, 10.0);

        assert_eq!(curve.sample(0.0), 0.0);
        assert_eq!(curve.sample(0.5), 5.0);
        assert_eq!(curve.sample(1.0), 10.0);
    }

    #[test]
    fn test_constant_interpolation() {
        let mut curve = Curve::new();
        curve.add_constant(0.0, 0.0);
        curve.add_constant(1.0, 10.0);

        assert_eq!(curve.sample(0.0), 0.0);
        assert_eq!(curve.sample(0.5), 0.0); // Step function stays at 0
        assert_eq!(curve.sample(0.99), 0.0);
        assert_eq!(curve.sample(1.0), 10.0);
    }

    #[test]
    fn test_multiple_keyframes() {
        let mut curve = Curve::new();
        curve.add(0.0, 0.0);
        curve.add(1.0, 10.0);
        curve.add(2.0, 5.0);
        curve.add(3.0, 15.0);

        assert_eq!(curve.sample(0.0), 0.0);
        assert_eq!(curve.sample(0.5), 5.0);
        assert_eq!(curve.sample(1.0), 10.0);
        assert_eq!(curve.sample(1.5), 7.5);
        assert_eq!(curve.sample(2.0), 5.0);
        assert_eq!(curve.sample(2.5), 10.0);
        assert_eq!(curve.sample(3.0), 15.0);
    }

    #[test]
    fn test_time_range() {
        let mut curve = Curve::new();
        curve.add(1.0, 0.0);
        curve.add(5.0, 10.0);
        curve.add(3.0, 5.0); // Out of order

        let range = curve.time_range().unwrap();
        assert_eq!(range, (1.0, 5.0));
    }

    #[test]
    fn test_value_range() {
        let mut curve = Curve::new();
        curve.add(0.0, -5.0);
        curve.add(1.0, 10.0);
        curve.add(2.0, 3.0);

        let range = curve.value_range().unwrap();
        assert_eq!(range, (-5.0, 10.0));
    }

    #[test]
    fn test_builder() {
        let mut curve = CurveBuilder::named("test")
            .keyframe(0.0, 0.0)
            .keyframe(1.0, 10.0)
            .keyframe(2.0, 5.0)
            .build();

        assert_eq!(curve.name, Some("test".to_string()));
        assert_eq!(curve.len(), 3);
        assert_eq!(curve.sample(0.5), 5.0);
    }

    #[test]
    fn test_sample_range() {
        let mut curve = Curve::new();
        curve.add(0.0, 0.0);
        curve.add(1.0, 10.0);

        let samples = curve.sample_range(0.0, 1.0, 5);
        assert_eq!(samples.len(), 5);
        assert_eq!(samples[0], (0.0, 0.0));
        assert_eq!(samples[2], (0.5, 5.0));
        assert_eq!(samples[4], (1.0, 10.0));
    }

    #[test]
    fn test_remove_keyframe() {
        let mut curve = Curve::new();
        curve.add(0.0, 0.0);
        curve.add(1.0, 10.0);
        curve.add(2.0, 20.0);

        let removed = curve.remove_keyframe_at(1.0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().value, 10.0);
        assert_eq!(curve.len(), 2);
    }
}
