//! Flux Core - Foundation types for the Flux operator graph system
//!
//! This crate provides the core building blocks for creating operator graphs:
//!
//! - [`Id`] - Unique identifiers for nodes and ports
//! - [`Value`] / [`ValueType`] - Type-safe values that flow between operators
//! - [`InputPort`] / [`OutputPort`] - Port definitions for connecting operators
//! - [`EvalContext`] - Evaluation context containing timing, camera, and rendering state
//! - [`Operator`] - The trait that all operators implement
//! - [`DirtyFlag`] - Lazy evaluation tracking
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                        flux-core                             │
//! │  ┌─────────┐  ┌─────────────┐  ┌───────────────────────┐   │
//! │  │   Id    │  │   Value     │  │     EvalContext       │   │
//! │  │ (UUID)  │  │ (type-safe) │  │ (timing, camera, etc) │   │
//! │  └─────────┘  └─────────────┘  └───────────────────────┘   │
//! │  ┌─────────────────────────┐   ┌───────────────────────┐   │
//! │  │  InputPort / OutputPort │   │      Operator         │   │
//! │  │   (connection points)   │   │   (trait definition)  │   │
//! │  └─────────────────────────┘   └───────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```ignore
//! use flux_core::{Id, Value, EvalContext, InputPort, OutputPort};
//!
//! // Create a simple value
//! let value = Value::Float(42.0);
//!
//! // Create an evaluation context
//! let mut ctx = EvalContext::new();
//! ctx.advance(0.016); // Advance by ~60fps
//!
//! // Create ports
//! let input = InputPort::float("amplitude", 1.0);
//! let output = OutputPort::float("result");
//! ```

pub mod context;
pub mod dirty_flag;
pub mod error;
pub mod id;
pub mod operator;
pub mod operator_meta;
pub mod port;
pub mod value;

// Re-export commonly used types at crate root
pub use context::{
    CallContext, EvalContext, GizmoVisibility, Mat4, TransformGizmoMode, MAT4_IDENTITY,
};
pub use dirty_flag::{
    advance_invalidation_frame, current_invalidation_frame, reset_invalidation_frame, DirtyFlag,
    DirtyFlagSet, DirtyFlagTrigger,
};
pub use error::{EvalResult, OperatorError, OperatorResult};
pub use id::Id;
pub use operator::{InputResolver, Operator};
pub use operator_meta::{
    category_colors, EffectivePortMeta, OperatorMeta, PinShape, PortMeta, PortOverride,
};
pub use port::{InputPort, OutputPort, OutputTypeRule, TriggerInput, TriggerOutput, TypeConstraint};
pub use value::{Color, Gradient, GradientStop, Matrix4, TypeCategory, Value, ValueType};
