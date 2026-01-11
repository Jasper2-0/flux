//! Animation data schema
//!
//! Defines the serialization format for animation curves, keyframes,
//! and associated metadata.

use serde::{Deserialize, Serialize};

use flux_core::Id;

/// Animation definition for a single input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationDef {
    /// Target child ID
    pub target_child: Id,
    /// Target input index
    pub target_input: usize,
    /// Animation curve
    pub curve: CurveDef,
}

impl AnimationDef {
    /// Create a new animation definition
    pub fn new(target_child: Id, target_input: usize) -> Self {
        Self {
            target_child,
            target_input,
            curve: CurveDef::new(),
        }
    }

    /// Add a keyframe to the curve
    pub fn add_keyframe(&mut self, keyframe: KeyframeDef) -> &mut Self {
        self.curve.add_keyframe(keyframe);
        self
    }
}

/// Animation curve definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveDef {
    /// Keyframes in time order
    pub keyframes: Vec<KeyframeDef>,
    /// Behavior before first keyframe
    #[serde(default)]
    pub pre_behavior: ExtrapolationMode,
    /// Behavior after last keyframe
    #[serde(default)]
    pub post_behavior: ExtrapolationMode,
}

impl CurveDef {
    /// Create an empty curve
    pub fn new() -> Self {
        Self {
            keyframes: Vec::new(),
            pre_behavior: ExtrapolationMode::default(),
            post_behavior: ExtrapolationMode::default(),
        }
    }

    /// Add a keyframe, maintaining time order
    pub fn add_keyframe(&mut self, keyframe: KeyframeDef) {
        self.keyframes.push(keyframe);
        self.keyframes.sort_by(|a, b| {
            a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Get the time range of this curve
    pub fn time_range(&self) -> Option<(f64, f64)> {
        if self.keyframes.is_empty() {
            return None;
        }
        Some((
            self.keyframes.first().unwrap().time,
            self.keyframes.last().unwrap().time,
        ))
    }
}

impl Default for CurveDef {
    fn default() -> Self {
        Self::new()
    }
}

/// A keyframe in an animation curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeDef {
    /// Time position in seconds
    pub time: f64,
    /// Value at this keyframe
    pub value: f64,
    /// Interpolation mode to next keyframe
    #[serde(default)]
    pub interpolation: InterpolationMode,
    /// In tangent for cubic interpolation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_tangent: Option<TangentDef>,
    /// Out tangent for cubic interpolation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_tangent: Option<TangentDef>,
}

impl KeyframeDef {
    /// Create a keyframe at the given time and value
    pub fn new(time: f64, value: f64) -> Self {
        Self {
            time,
            value,
            interpolation: InterpolationMode::default(),
            in_tangent: None,
            out_tangent: None,
        }
    }

    /// Builder: set interpolation mode
    pub fn with_interpolation(mut self, mode: InterpolationMode) -> Self {
        self.interpolation = mode;
        self
    }

    /// Builder: set tangents for bezier interpolation
    pub fn with_tangents(mut self, in_value: f64, out_value: f64) -> Self {
        self.in_tangent = Some(TangentDef::new(in_value, 1.0));
        self.out_tangent = Some(TangentDef::new(out_value, 1.0));
        self
    }

    /// Builder: set tangents with weights
    pub fn with_weighted_tangents(
        mut self,
        in_value: f64,
        in_weight: f64,
        out_value: f64,
        out_weight: f64,
    ) -> Self {
        self.in_tangent = Some(TangentDef::new(in_value, in_weight));
        self.out_tangent = Some(TangentDef::new(out_value, out_weight));
        self
    }
}

/// Tangent definition for bezier interpolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TangentDef {
    /// Tangent slope value
    pub value: f64,
    /// Tangent weight (for weighted bezier)
    #[serde(default = "default_weight")]
    pub weight: f64,
}

fn default_weight() -> f64 {
    1.0
}

impl TangentDef {
    /// Create a new tangent
    pub fn new(value: f64, weight: f64) -> Self {
        Self { value, weight }
    }

    /// Create a flat (zero slope) tangent
    pub fn flat() -> Self {
        Self { value: 0.0, weight: 1.0 }
    }
}

/// Interpolation mode between keyframes
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterpolationMode {
    /// Constant (step) - value jumps at keyframe
    Constant,
    /// Linear interpolation
    #[default]
    Linear,
    /// Bezier/cubic interpolation with tangents
    Bezier,
    /// Smooth interpolation (auto-computed tangents)
    Smooth,
}

/// Extrapolation mode for values outside keyframe range
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtrapolationMode {
    /// Hold the boundary value
    #[default]
    Constant,
    /// Continue the slope at the boundary
    Linear,
    /// Repeat the curve
    Cycle,
    /// Repeat with offset (cumulative)
    CycleOffset,
    /// Ping-pong (reverse at boundaries)
    Oscillate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_def() {
        let kf = KeyframeDef::new(1.0, 42.0);
        assert_eq!(kf.time, 1.0);
        assert_eq!(kf.value, 42.0);
        assert_eq!(kf.interpolation, InterpolationMode::Linear);
    }

    #[test]
    fn test_keyframe_with_tangents() {
        let kf = KeyframeDef::new(0.0, 0.0)
            .with_interpolation(InterpolationMode::Bezier)
            .with_tangents(-1.0, 1.0);

        assert!(kf.in_tangent.is_some());
        assert!(kf.out_tangent.is_some());
        assert_eq!(kf.in_tangent.unwrap().value, -1.0);
        assert_eq!(kf.out_tangent.unwrap().value, 1.0);
    }

    #[test]
    fn test_curve_def() {
        let mut curve = CurveDef::new();
        curve.add_keyframe(KeyframeDef::new(1.0, 100.0));
        curve.add_keyframe(KeyframeDef::new(0.0, 0.0));
        curve.add_keyframe(KeyframeDef::new(2.0, 50.0));

        // Should be sorted by time
        assert_eq!(curve.keyframes[0].time, 0.0);
        assert_eq!(curve.keyframes[1].time, 1.0);
        assert_eq!(curve.keyframes[2].time, 2.0);

        assert_eq!(curve.time_range(), Some((0.0, 2.0)));
    }

    #[test]
    fn test_animation_def() {
        let child_id = Id::new();
        let mut anim = AnimationDef::new(child_id, 0);
        anim.add_keyframe(KeyframeDef::new(0.0, 0.0));
        anim.add_keyframe(KeyframeDef::new(1.0, 1.0));

        assert_eq!(anim.target_child, child_id);
        assert_eq!(anim.curve.keyframes.len(), 2);
    }

    #[test]
    fn test_animation_serialize() {
        let child_id = Id::new();
        let mut anim = AnimationDef::new(child_id, 0);
        anim.curve.add_keyframe(KeyframeDef::new(0.0, 0.0));
        anim.curve.add_keyframe(
            KeyframeDef::new(1.0, 1.0)
                .with_interpolation(InterpolationMode::Bezier)
                .with_tangents(0.5, 0.5),
        );
        anim.curve.post_behavior = ExtrapolationMode::Cycle;

        let json = serde_json::to_string_pretty(&anim).unwrap();
        assert!(json.contains("keyframes"));
        assert!(json.contains("Bezier"));
        assert!(json.contains("Cycle"));

        let restored: AnimationDef = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.curve.keyframes.len(), 2);
        assert_eq!(restored.curve.post_behavior, ExtrapolationMode::Cycle);
    }
}
