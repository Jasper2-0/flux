//! Dirty flag system for lazy evaluation tracking
//!
//! This module provides the [`DirtyFlag`] type for tracking when values need recomputation,
//! along with [`DirtyFlagTrigger`] for configuring different invalidation behaviors.

use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::context::EvalContext;

// Global invalidation frame counter
// This is incremented each time a global invalidation occurs
static INVALIDATION_FRAME: AtomicU64 = AtomicU64::new(0);

/// Advance the global invalidation frame counter
/// Call this when you want to invalidate all `Always` or time-based dirty flags
pub fn advance_invalidation_frame() {
    INVALIDATION_FRAME.fetch_add(1, Ordering::SeqCst);
}

/// Get the current global invalidation frame
pub fn current_invalidation_frame() -> u64 {
    INVALIDATION_FRAME.load(Ordering::SeqCst)
}

/// Reset the global invalidation frame (mainly for testing)
pub fn reset_invalidation_frame() {
    INVALIDATION_FRAME.store(0, Ordering::SeqCst);
}

/// Conditions that trigger dirty state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirtyFlagTrigger {
    /// Never automatically dirty (only manual invalidation)
    None,
    /// Always dirty - recompute every evaluation
    Always,
    /// Dirty when animated (default - dirty when version changes)
    #[default]
    Animated,
    /// Dirty when time changes
    TimeChanged,
    /// Dirty when frame number changes
    FrameChanged,
}

/// Tracks whether a value needs recomputation
///
/// The dirty flag system supports multiple trigger modes:
/// - `None`: Only dirty when explicitly marked
/// - `Always`: Always needs recomputation
/// - `Animated`: Dirty when version changes (default)
/// - `TimeChanged`: Dirty when eval context time changes
/// - `FrameChanged`: Dirty when eval context frame changes
#[derive(Clone, Debug)]
pub struct DirtyFlag {
    /// Target version (incremented when value changes)
    target: u64,
    /// Last version we computed at
    reference: u64,
    /// Frame number when this was last invalidated
    invalidated_at_frame: u64,
    /// Last context time we evaluated at
    last_time: f64,
    /// Last context frame we evaluated at
    last_frame: u64,
    /// What triggers this flag to become dirty
    trigger: DirtyFlagTrigger,
}

impl DirtyFlag {
    /// Create a new dirty flag with default Animated trigger
    pub fn new() -> Self {
        Self {
            target: 1,
            reference: 0,
            invalidated_at_frame: 0,
            last_time: f64::NEG_INFINITY,
            last_frame: u64::MAX,
            trigger: DirtyFlagTrigger::Animated,
        }
    }

    /// Create a new dirty flag with a specific trigger
    pub fn with_trigger(trigger: DirtyFlagTrigger) -> Self {
        Self {
            target: 1,
            reference: 0,
            invalidated_at_frame: 0,
            last_time: f64::NEG_INFINITY,
            last_frame: u64::MAX,
            trigger,
        }
    }

    /// Get the current trigger mode
    pub fn trigger(&self) -> DirtyFlagTrigger {
        self.trigger
    }

    /// Set the trigger mode
    pub fn set_trigger(&mut self, trigger: DirtyFlagTrigger) {
        self.trigger = trigger;
    }

    /// Returns true if the value needs recomputation (basic check)
    pub fn is_dirty(&self) -> bool {
        self.reference < self.target
    }

    /// Check if dirty based on trigger and evaluation context
    ///
    /// This is the primary method to use during evaluation, as it
    /// takes into account the trigger mode and context state.
    pub fn is_dirty_for_context(&self, ctx: &EvalContext) -> bool {
        match self.trigger {
            DirtyFlagTrigger::None => {
                // Only dirty if explicitly marked
                self.reference < self.target
            }
            DirtyFlagTrigger::Always => {
                // Always needs recomputation
                true
            }
            DirtyFlagTrigger::Animated => {
                // Dirty if version changed OR if global invalidation occurred
                self.reference < self.target
                    || self.invalidated_at_frame < current_invalidation_frame()
            }
            DirtyFlagTrigger::TimeChanged => {
                // Dirty if time changed
                (self.last_time - ctx.time).abs() > 1e-10 || self.reference < self.target
            }
            DirtyFlagTrigger::FrameChanged => {
                // Dirty if frame changed
                self.last_frame != ctx.frame || self.reference < self.target
            }
        }
    }

    /// Mark the value as needing recomputation
    pub fn mark_dirty(&mut self) {
        self.target += 1;
    }

    /// Invalidate at the current global frame
    pub fn invalidate(&mut self) {
        self.target += 1;
        self.invalidated_at_frame = current_invalidation_frame();
    }

    /// Mark the value as up-to-date
    pub fn mark_clean(&mut self) {
        self.reference = self.target;
        self.invalidated_at_frame = current_invalidation_frame();
    }

    /// Mark clean with context (updates time/frame tracking)
    pub fn mark_clean_for_context(&mut self, ctx: &EvalContext) {
        self.reference = self.target;
        self.invalidated_at_frame = current_invalidation_frame();
        self.last_time = ctx.time;
        self.last_frame = ctx.frame;
    }

    /// Get the target version
    pub fn target(&self) -> u64 {
        self.target
    }

    /// Get the reference version
    pub fn reference_version(&self) -> u64 {
        self.reference
    }

    /// Get the frame this was invalidated at
    pub fn invalidated_at(&self) -> u64 {
        self.invalidated_at_frame
    }
}

impl Default for DirtyFlag {
    fn default() -> Self {
        Self::new()
    }
}

/// A collection of dirty flags that can be tracked together
#[derive(Clone, Debug, Default)]
pub struct DirtyFlagSet {
    flags: Vec<DirtyFlag>,
}

impl DirtyFlagSet {
    /// Create a new empty set
    pub fn new() -> Self {
        Self { flags: Vec::new() }
    }

    /// Add a flag to the set
    pub fn add(&mut self, flag: DirtyFlag) -> usize {
        let idx = self.flags.len();
        self.flags.push(flag);
        idx
    }

    /// Get a flag by index
    pub fn get(&self, index: usize) -> Option<&DirtyFlag> {
        self.flags.get(index)
    }

    /// Get a mutable flag by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut DirtyFlag> {
        self.flags.get_mut(index)
    }

    /// Check if any flag is dirty
    pub fn any_dirty(&self) -> bool {
        self.flags.iter().any(|f| f.is_dirty())
    }

    /// Check if any flag is dirty for context
    pub fn any_dirty_for_context(&self, ctx: &EvalContext) -> bool {
        self.flags.iter().any(|f| f.is_dirty_for_context(ctx))
    }

    /// Mark all flags as dirty
    pub fn mark_all_dirty(&mut self) {
        for flag in &mut self.flags {
            flag.mark_dirty();
        }
    }

    /// Mark all flags as clean
    pub fn mark_all_clean(&mut self) {
        for flag in &mut self.flags {
            flag.mark_clean();
        }
    }

    /// Mark all flags as clean for context
    pub fn mark_all_clean_for_context(&mut self, ctx: &EvalContext) {
        for flag in &mut self.flags {
            flag.mark_clean_for_context(ctx);
        }
    }

    /// Get the number of flags
    pub fn len(&self) -> usize {
        self.flags.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.flags.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_flag_basic() {
        let mut flag = DirtyFlag::new();

        // Initially dirty
        assert!(flag.is_dirty());

        // After cleaning, not dirty
        flag.mark_clean();
        assert!(!flag.is_dirty());

        // After marking dirty, dirty again
        flag.mark_dirty();
        assert!(flag.is_dirty());
    }

    #[test]
    fn test_dirty_flag_triggers() {
        reset_invalidation_frame();

        // None trigger - only explicit marks
        let mut flag_none = DirtyFlag::with_trigger(DirtyFlagTrigger::None);
        let ctx = EvalContext::new();
        flag_none.mark_clean_for_context(&ctx);
        assert!(!flag_none.is_dirty_for_context(&ctx));

        // Always trigger - always dirty
        let flag_always = DirtyFlag::with_trigger(DirtyFlagTrigger::Always);
        assert!(flag_always.is_dirty_for_context(&ctx));

        // Animated trigger - dirty on version change
        let mut flag_animated = DirtyFlag::with_trigger(DirtyFlagTrigger::Animated);
        flag_animated.mark_clean_for_context(&ctx);
        assert!(!flag_animated.is_dirty_for_context(&ctx));
        flag_animated.mark_dirty();
        assert!(flag_animated.is_dirty_for_context(&ctx));
    }

    #[test]
    fn test_time_changed_trigger() {
        let mut flag = DirtyFlag::with_trigger(DirtyFlagTrigger::TimeChanged);
        let mut ctx = EvalContext::new();

        flag.mark_clean_for_context(&ctx);
        assert!(!flag.is_dirty_for_context(&ctx));

        // Time changes - should be dirty
        ctx.time = 1.0;
        assert!(flag.is_dirty_for_context(&ctx));

        // Clean and same time - not dirty
        flag.mark_clean_for_context(&ctx);
        assert!(!flag.is_dirty_for_context(&ctx));
    }

    #[test]
    fn test_frame_changed_trigger() {
        let mut flag = DirtyFlag::with_trigger(DirtyFlagTrigger::FrameChanged);
        let mut ctx = EvalContext::new();

        flag.mark_clean_for_context(&ctx);
        assert!(!flag.is_dirty_for_context(&ctx));

        // Frame changes - should be dirty
        ctx.frame = 1;
        assert!(flag.is_dirty_for_context(&ctx));

        // Clean and same frame - not dirty
        flag.mark_clean_for_context(&ctx);
        assert!(!flag.is_dirty_for_context(&ctx));
    }

    #[test]
    fn test_global_invalidation() {
        reset_invalidation_frame();

        let mut flag = DirtyFlag::with_trigger(DirtyFlagTrigger::Animated);
        let ctx = EvalContext::new();

        flag.mark_clean_for_context(&ctx);
        assert!(!flag.is_dirty_for_context(&ctx));

        // Advance global frame - should become dirty
        advance_invalidation_frame();
        assert!(flag.is_dirty_for_context(&ctx));

        // Clean again - not dirty
        flag.mark_clean_for_context(&ctx);
        assert!(!flag.is_dirty_for_context(&ctx));
    }

    #[test]
    fn test_dirty_flag_set() {
        let mut set = DirtyFlagSet::new();
        let ctx = EvalContext::new();

        set.add(DirtyFlag::new());
        set.add(DirtyFlag::new());

        // Initially all dirty
        assert!(set.any_dirty());
        assert!(set.any_dirty_for_context(&ctx));

        // Clean all
        set.mark_all_clean_for_context(&ctx);
        assert!(!set.any_dirty());

        // Mark one dirty
        set.get_mut(0).unwrap().mark_dirty();
        assert!(set.any_dirty());
    }
}
