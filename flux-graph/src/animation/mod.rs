//! Animation system for keyframe-based animation of operator inputs.
//!
//! This module provides:
//! - `Keyframe` - A single point in time with a value and interpolation settings
//! - `Curve` - A collection of keyframes that can be sampled at any time
//! - `Animator` - Manages animation curves for operator inputs
//! - `CurveBuilder` - Builder pattern for creating curves
//! - `AnimatorBuilder` - Builder pattern for creating animators
//!
//! # Example
//!
//! ```ignore
//! use flux_graph::animation::{CurveBuilder, AnimatorBuilder, LoopMode};
//! use flux_core::Id;
//!
//! // Create a simple animation curve
//! let curve = CurveBuilder::named("opacity")
//!     .keyframe(0.0, 0.0)
//!     .keyframe(1.0, 1.0)
//!     .keyframe(2.0, 0.5)
//!     .build();
//!
//! // Create an animator with the curve bound to a node's input
//! let node_id = Id::new();
//! let mut animator = AnimatorBuilder::new()
//!     .range(0.0, 2.0)
//!     .loop_mode(LoopMode::Loop)
//!     .curve(curve, node_id, 0)
//!     .build();
//!
//! // Sample the animation at different times
//! animator.set_time(0.5);
//! let value = animator.sample(node_id, 0);
//! ```

mod animator;
mod curve;
mod interpolation;
mod keyframe;

pub use animator::{AnimationTarget, Animator, AnimatorBuilder, CurveBinding, LoopMode, PlaybackState};
pub use curve::{Curve, CurveBuilder};
pub use interpolation::Interpolation;
pub use keyframe::Keyframe;
