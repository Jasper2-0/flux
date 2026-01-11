use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::Curve;
use flux_core::id::Id;
use flux_core::value::Value;

/// Target for an animation curve
///
/// Specifies which input slot on which operator should receive
/// the animated value.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnimationTarget {
    /// The operator node ID
    pub node_id: Id,
    /// The input slot index on the operator
    pub input_index: usize,
}

impl AnimationTarget {
    pub fn new(node_id: Id, input_index: usize) -> Self {
        Self { node_id, input_index }
    }
}

/// A curve binding associates a curve with a target slot
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurveBinding {
    /// The animation curve
    pub curve: Curve,
    /// The target slot to animate
    pub target: AnimationTarget,
    /// Whether this binding is enabled
    pub enabled: bool,
}

impl CurveBinding {
    pub fn new(curve: Curve, target: AnimationTarget) -> Self {
        Self {
            curve,
            target,
            enabled: true,
        }
    }

    /// Sample the curve at the given time
    pub fn sample(&mut self, time: f64) -> f64 {
        if self.enabled {
            self.curve.sample(time)
        } else {
            0.0
        }
    }
}

/// Playback state for the animator
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

/// Loop mode for animation playback
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopMode {
    /// Play once and stop
    #[default]
    Once,
    /// Loop back to start when finished
    Loop,
    /// Ping-pong between start and end
    PingPong,
}

/// The Animator manages animation curves and their playback
///
/// It stores multiple curves, each bound to a specific input slot,
/// and provides methods for sampling all curves at a given time,
/// as well as playback controls.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Animator {
    /// All curve bindings managed by this animator
    bindings: Vec<CurveBinding>,
    /// Index by target for quick lookup
    #[serde(skip)]
    target_index: HashMap<AnimationTarget, usize>,
    /// Current playback time (in bars or seconds)
    current_time: f64,
    /// Playback state
    state: PlaybackState,
    /// Loop mode
    loop_mode: LoopMode,
    /// Playback speed multiplier
    speed: f64,
    /// Start time for playback range
    start_time: f64,
    /// End time for playback range
    end_time: f64,
}

impl Animator {
    /// Create a new animator
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
            target_index: HashMap::new(),
            current_time: 0.0,
            state: PlaybackState::Stopped,
            loop_mode: LoopMode::Once,
            speed: 1.0,
            start_time: 0.0,
            end_time: 1.0,
        }
    }

    /// Create an animator with a specific time range
    pub fn with_range(start: f64, end: f64) -> Self {
        let mut animator = Self::new();
        animator.start_time = start;
        animator.end_time = end;
        animator
    }

    /// Add a curve binding
    pub fn add_binding(&mut self, binding: CurveBinding) {
        let idx = self.bindings.len();
        self.target_index.insert(binding.target.clone(), idx);
        self.bindings.push(binding);
    }

    /// Add a curve for a specific target
    pub fn add_curve(&mut self, curve: Curve, node_id: Id, input_index: usize) {
        let target = AnimationTarget::new(node_id, input_index);
        self.add_binding(CurveBinding::new(curve, target));
    }

    /// Remove a curve binding by target
    pub fn remove_curve(&mut self, node_id: Id, input_index: usize) -> Option<CurveBinding> {
        let target = AnimationTarget::new(node_id, input_index);
        if let Some(idx) = self.target_index.remove(&target) {
            let binding = self.bindings.remove(idx);
            // Rebuild index
            self.rebuild_target_index();
            Some(binding)
        } else {
            None
        }
    }

    /// Get a curve binding by target
    pub fn get_binding(&self, node_id: Id, input_index: usize) -> Option<&CurveBinding> {
        let target = AnimationTarget::new(node_id, input_index);
        self.target_index.get(&target).map(|&idx| &self.bindings[idx])
    }

    /// Get a mutable curve binding by target
    pub fn get_binding_mut(&mut self, node_id: Id, input_index: usize) -> Option<&mut CurveBinding> {
        let target = AnimationTarget::new(node_id, input_index);
        if let Some(&idx) = self.target_index.get(&target) {
            Some(&mut self.bindings[idx])
        } else {
            None
        }
    }

    /// Get all bindings
    pub fn bindings(&self) -> &[CurveBinding] {
        &self.bindings
    }

    /// Get the number of curve bindings
    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    /// Rebuild the target index after modifications
    fn rebuild_target_index(&mut self) {
        self.target_index.clear();
        for (idx, binding) in self.bindings.iter().enumerate() {
            self.target_index.insert(binding.target.clone(), idx);
        }
    }

    // ========== Playback Controls ==========

    /// Get the current playback time
    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    /// Set the current time directly
    pub fn set_time(&mut self, time: f64) {
        self.current_time = time.clamp(self.start_time, self.end_time);
    }

    /// Get the playback state
    pub fn state(&self) -> PlaybackState {
        self.state
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    /// Start playback
    pub fn play(&mut self) {
        self.state = PlaybackState::Playing;
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
    }

    /// Stop playback and reset to start
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.current_time = self.start_time;
    }

    /// Toggle between play and pause
    pub fn toggle_playback(&mut self) {
        match self.state {
            PlaybackState::Playing => self.pause(),
            PlaybackState::Paused | PlaybackState::Stopped => self.play(),
        }
    }

    /// Set the loop mode
    pub fn set_loop_mode(&mut self, mode: LoopMode) {
        self.loop_mode = mode;
    }

    /// Get the loop mode
    pub fn loop_mode(&self) -> LoopMode {
        self.loop_mode
    }

    /// Set the playback speed
    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed.max(0.0);
    }

    /// Get the playback speed
    pub fn speed(&self) -> f64 {
        self.speed
    }

    /// Set the playback range
    pub fn set_range(&mut self, start: f64, end: f64) {
        self.start_time = start;
        self.end_time = end.max(start);
    }

    /// Get the playback range
    pub fn range(&self) -> (f64, f64) {
        (self.start_time, self.end_time)
    }

    /// Advance the playback time by delta (in seconds or bars)
    pub fn advance(&mut self, delta: f64) {
        if self.state != PlaybackState::Playing {
            return;
        }

        let adjusted_delta = delta * self.speed;
        self.current_time += adjusted_delta;

        // Handle looping
        let duration = self.end_time - self.start_time;
        if duration <= 0.0 {
            return;
        }

        match self.loop_mode {
            LoopMode::Once => {
                if self.current_time >= self.end_time {
                    self.current_time = self.end_time;
                    self.state = PlaybackState::Stopped;
                }
            }
            LoopMode::Loop => {
                while self.current_time >= self.end_time {
                    self.current_time -= duration;
                }
            }
            LoopMode::PingPong => {
                // Count how many times we've crossed the boundary
                let cycles = ((self.current_time - self.start_time) / duration).floor() as i32;
                let is_reversed = cycles % 2 != 0;

                // Get position within current cycle
                let pos_in_cycle = (self.current_time - self.start_time) % duration;

                self.current_time = if is_reversed {
                    self.end_time - pos_in_cycle
                } else {
                    self.start_time + pos_in_cycle
                };
            }
        }
    }

    // ========== Sampling ==========

    /// Sample all curves at the current time and return values by target
    pub fn sample_all(&mut self) -> Vec<(AnimationTarget, f64)> {
        let time = self.current_time;
        self.bindings
            .iter_mut()
            .filter(|b| b.enabled)
            .map(|b| (b.target.clone(), b.curve.sample(time)))
            .collect()
    }

    /// Sample all curves at a specific time
    pub fn sample_all_at(&mut self, time: f64) -> Vec<(AnimationTarget, f64)> {
        self.bindings
            .iter_mut()
            .filter(|b| b.enabled)
            .map(|b| (b.target.clone(), b.curve.sample(time)))
            .collect()
    }

    /// Sample a specific curve at the current time
    pub fn sample(&mut self, node_id: Id, input_index: usize) -> Option<f64> {
        let time = self.current_time;
        self.get_binding_mut(node_id, input_index)
            .filter(|b| b.enabled)
            .map(|b| b.curve.sample(time))
    }

    /// Sample a specific curve at a given time
    pub fn sample_at(&mut self, node_id: Id, input_index: usize, time: f64) -> Option<f64> {
        self.get_binding_mut(node_id, input_index)
            .filter(|b| b.enabled)
            .map(|b| b.curve.sample(time))
    }

    /// Get sampled values as a map of target -> Value
    pub fn get_animated_values(&mut self) -> HashMap<AnimationTarget, Value> {
        let time = self.current_time;
        self.bindings
            .iter_mut()
            .filter(|b| b.enabled)
            .map(|b| (b.target.clone(), Value::Float(b.curve.sample(time) as f32)))
            .collect()
    }
}

/// Builder for creating animators with curves
pub struct AnimatorBuilder {
    animator: Animator,
}

impl AnimatorBuilder {
    pub fn new() -> Self {
        Self {
            animator: Animator::new(),
        }
    }

    pub fn range(mut self, start: f64, end: f64) -> Self {
        self.animator.set_range(start, end);
        self
    }

    pub fn loop_mode(mut self, mode: LoopMode) -> Self {
        self.animator.set_loop_mode(mode);
        self
    }

    pub fn speed(mut self, speed: f64) -> Self {
        self.animator.set_speed(speed);
        self
    }

    pub fn curve(mut self, curve: Curve, node_id: Id, input_index: usize) -> Self {
        self.animator.add_curve(curve, node_id, input_index);
        self
    }

    pub fn binding(mut self, binding: CurveBinding) -> Self {
        self.animator.add_binding(binding);
        self
    }

    pub fn build(self) -> Animator {
        self.animator
    }
}

impl Default for AnimatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::CurveBuilder;

    fn make_test_node_id() -> Id {
        Id::new()
    }

    #[test]
    fn test_animator_basic() {
        let node_id = make_test_node_id();
        let mut animator = Animator::new();

        let curve = CurveBuilder::new()
            .keyframe(0.0, 0.0)
            .keyframe(1.0, 10.0)
            .build();

        animator.add_curve(curve, node_id, 0);

        assert_eq!(animator.binding_count(), 1);
        assert!(animator.get_binding(node_id, 0).is_some());
    }

    #[test]
    fn test_animator_sampling() {
        let node_id = make_test_node_id();
        let mut animator = Animator::new();

        let curve = CurveBuilder::new()
            .keyframe(0.0, 0.0)
            .keyframe(1.0, 10.0)
            .build();

        animator.add_curve(curve, node_id, 0);
        animator.set_time(0.5);

        let value = animator.sample(node_id, 0).unwrap();
        assert_eq!(value, 5.0);
    }

    #[test]
    fn test_animator_playback() {
        let mut animator = Animator::with_range(0.0, 2.0);

        assert_eq!(animator.state(), PlaybackState::Stopped);
        assert_eq!(animator.current_time(), 0.0);

        animator.play();
        assert!(animator.is_playing());

        animator.advance(0.5);
        assert_eq!(animator.current_time(), 0.5);

        animator.advance(0.5);
        assert_eq!(animator.current_time(), 1.0);

        animator.pause();
        assert_eq!(animator.state(), PlaybackState::Paused);

        animator.advance(0.5);
        assert_eq!(animator.current_time(), 1.0); // No change when paused

        animator.stop();
        assert_eq!(animator.current_time(), 0.0);
    }

    #[test]
    fn test_animator_loop() {
        let mut animator = Animator::with_range(0.0, 1.0);
        animator.set_loop_mode(LoopMode::Loop);
        animator.play();

        animator.advance(0.5);
        assert_eq!(animator.current_time(), 0.5);

        animator.advance(0.75);
        assert!((animator.current_time() - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_animator_once() {
        let mut animator = Animator::with_range(0.0, 1.0);
        animator.set_loop_mode(LoopMode::Once);
        animator.play();

        animator.advance(1.5);
        assert_eq!(animator.current_time(), 1.0);
        assert_eq!(animator.state(), PlaybackState::Stopped);
    }

    #[test]
    fn test_animator_speed() {
        let mut animator = Animator::with_range(0.0, 2.0);
        animator.set_speed(2.0);
        animator.play();

        animator.advance(0.5);
        assert_eq!(animator.current_time(), 1.0); // 0.5 * 2.0 = 1.0
    }

    #[test]
    fn test_animator_builder() {
        let node_id = make_test_node_id();
        let curve = CurveBuilder::new()
            .keyframe(0.0, 0.0)
            .keyframe(1.0, 10.0)
            .build();

        let animator = AnimatorBuilder::new()
            .range(0.0, 2.0)
            .loop_mode(LoopMode::Loop)
            .speed(1.5)
            .curve(curve, node_id, 0)
            .build();

        assert_eq!(animator.range(), (0.0, 2.0));
        assert_eq!(animator.loop_mode(), LoopMode::Loop);
        assert_eq!(animator.speed(), 1.5);
        assert_eq!(animator.binding_count(), 1);
    }

    #[test]
    fn test_sample_all() {
        let node1 = make_test_node_id();
        let node2 = make_test_node_id();

        let mut animator = Animator::new();

        animator.add_curve(
            CurveBuilder::new()
                .keyframe(0.0, 0.0)
                .keyframe(1.0, 100.0)
                .build(),
            node1,
            0,
        );

        animator.add_curve(
            CurveBuilder::new()
                .keyframe(0.0, 10.0)
                .keyframe(1.0, 20.0)
                .build(),
            node2,
            1,
        );

        animator.set_time(0.5);
        let values = animator.sample_all();

        assert_eq!(values.len(), 2);
    }
}
