//! Core Operator trait definition
//!
//! This module defines the [`Operator`] trait that all graph nodes must implement.
//! The trait is object-safe to allow heterogeneous collections of operators.
//!
//! # Downcasting
//!
//! The `Operator` trait extends `Downcast` from the `downcast-rs` crate, which
//! provides automatic `as_any()` implementations. You can downcast a `&dyn Operator`
//! to a concrete type using:
//!
//! ```ignore
//! use flux_core::Operator;
//!
//! fn process(op: &dyn Operator) {
//!     if let Some(sin_op) = op.downcast_ref::<SinOp>() {
//!         // Work with the concrete SinOp type
//!     }
//! }
//! ```

use downcast_rs::{impl_downcast, Downcast};

use crate::context::EvalContext;
use crate::id::Id;
use crate::operator_settings::OperatorSettings;
use crate::operator_visuals::OperatorVisuals;
use crate::port::{InputPort, OutputPort, TriggerInput, TriggerOutput};
use crate::value::Value;

/// Function type for resolving input values from connected nodes
pub type InputResolver<'a> = &'a dyn Fn(Id, usize) -> Value;

/// Core trait for all operators (object-safe)
///
/// This is the fundamental building block of the operator graph system.
/// Each operator can have multiple inputs and outputs, and performs
/// computation during the `compute` phase.
///
/// # Downcasting
///
/// The trait automatically provides downcasting via the `downcast-rs` crate.
/// Use `downcast_ref::<T>()` or `downcast_mut::<T>()` on trait objects.
///
/// # Example
///
/// ```ignore
/// struct MyOperator {
///     id: Id,
///     inputs: Vec<InputPort>,
///     outputs: Vec<OutputPort>,
/// }
///
/// impl Operator for MyOperator {
///     fn id(&self) -> Id { self.id }
///     fn name(&self) -> &'static str { "MyOperator" }
///     // ... implement other methods
/// }
/// ```
pub trait Operator: Downcast {

    /// Unique instance ID
    fn id(&self) -> Id;

    /// Human-readable name
    fn name(&self) -> &'static str;

    /// Get input slots
    fn inputs(&self) -> &[InputPort];
    fn inputs_mut(&mut self) -> &mut [InputPort];

    /// Get output slots
    fn outputs(&self) -> &[OutputPort];
    fn outputs_mut(&mut self) -> &mut [OutputPort];

    /// Compute outputs from inputs.
    /// The `get_input_value` function resolves connected inputs by (node_id, output_index).
    fn compute(&mut self, ctx: &EvalContext, get_input_value: InputResolver);

    /// Returns true if this operator is time-varying (depends on ctx.time).
    /// Time-varying operators are always recomputed.
    fn is_time_varying(&self) -> bool {
        false
    }

    /// Returns true if this operator can operate in-place on its inputs.
    ///
    /// When true, the graph evaluator may pass ownership of input values to
    /// this operator instead of cloning them. This is an optimization for
    /// operators that transform data (e.g., scale mesh, adjust brightness)
    /// without needing to preserve the original.
    ///
    /// # Requirements
    ///
    /// Operators returning `true` must:
    /// - Not rely on input values being preserved after computation
    /// - Be able to handle both owned and cloned inputs gracefully
    ///
    /// # Default
    ///
    /// Returns `false` by default, meaning inputs are always cloned.
    ///
    /// # Example
    ///
    /// ```ignore
    /// impl Operator for ScaleOp {
    ///     fn can_operate_in_place(&self) -> bool {
    ///         true // We just multiply values, don't need to preserve input
    ///     }
    ///     // ... other methods
    /// }
    /// ```
    fn can_operate_in_place(&self) -> bool {
        false
    }

    // =========================================================================
    // Trigger ports (optional push-based execution)
    // =========================================================================

    /// Get trigger input ports.
    ///
    /// Trigger inputs receive signals from upstream operators to initiate
    /// push-based execution. Unlike value inputs, triggers don't carry data -
    /// they simply signal "execute now".
    ///
    /// # Default
    ///
    /// Returns an empty slice. Override if your operator has trigger inputs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// struct FrameCounter {
    ///     trigger_inputs: Vec<TriggerInput>,  // "OnFrame" trigger
    ///     count: u64,
    /// }
    ///
    /// impl Operator for FrameCounter {
    ///     fn trigger_inputs(&self) -> &[TriggerInput] {
    ///         &self.trigger_inputs
    ///     }
    ///     // ...
    /// }
    /// ```
    fn trigger_inputs(&self) -> &[TriggerInput] {
        &[]
    }

    /// Get mutable trigger input ports for connection management.
    fn trigger_inputs_mut(&mut self) -> &mut [TriggerInput] {
        &mut []
    }

    /// Get trigger output ports.
    ///
    /// Trigger outputs emit signals to downstream operators. When fired,
    /// all connected trigger inputs receive the signal.
    ///
    /// # Default
    ///
    /// Returns an empty slice. Override if your operator has trigger outputs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// struct MainLoop {
    ///     trigger_outputs: Vec<TriggerOutput>,  // "OnFrame", "OnInit"
    /// }
    ///
    /// impl Operator for MainLoop {
    ///     fn trigger_outputs(&self) -> &[TriggerOutput] {
    ///         &self.trigger_outputs
    ///     }
    ///     // ...
    /// }
    /// ```
    fn trigger_outputs(&self) -> &[TriggerOutput] {
        &[]
    }

    /// Get mutable trigger output ports for connection management.
    fn trigger_outputs_mut(&mut self) -> &mut [TriggerOutput] {
        &mut []
    }

    /// Called when a trigger input receives a signal.
    ///
    /// This is the push-based counterpart to `compute()`. While `compute()`
    /// is called during pull-based evaluation, `on_triggered()` is called
    /// immediately when an upstream trigger output fires.
    ///
    /// # Arguments
    ///
    /// * `trigger_index` - Index of the trigger input that fired
    /// * `ctx` - Evaluation context with timing information
    /// * `get_input_value` - Function to resolve connected value inputs
    ///
    /// # Returns
    ///
    /// Indices of trigger outputs to fire, if any. This enables trigger
    /// chains where one operator's trigger causes downstream triggers.
    ///
    /// # Default
    ///
    /// Returns an empty vec (no triggers fired). Override if your operator
    /// needs to respond to trigger signals.
    ///
    /// # Example
    ///
    /// ```ignore
    /// impl Operator for FrameCounter {
    ///     fn on_triggered(
    ///         &mut self,
    ///         trigger_index: usize,
    ///         ctx: &EvalContext,
    ///         _get_input: InputResolver,
    ///     ) -> Vec<usize> {
    ///         if trigger_index == 0 {  // OnFrame trigger
    ///             self.count += 1;
    ///             self.outputs[0].set(Value::Int(self.count as i64));
    ///             vec![0]  // Fire "Done" trigger
    ///         } else {
    ///             vec![]
    ///         }
    ///     }
    /// }
    /// ```
    fn on_triggered(
        &mut self,
        _trigger_index: usize,
        _ctx: &EvalContext,
        _get_input_value: InputResolver,
    ) -> Vec<usize> {
        Vec::new()
    }

    // =========================================================================
    // Settings and Visualization (optional interfaces)
    // =========================================================================

    /// Get operator settings (editable parameters beyond input defaults).
    ///
    /// Override this method and return `Some(self)` if your operator implements
    /// [`OperatorSettings`] to expose editable parameters like comparison modes,
    /// waveform types, etc.
    ///
    /// # Example
    ///
    /// ```ignore
    /// impl Operator for CompareOp {
    ///     fn settings(&self) -> Option<&dyn OperatorSettings> {
    ///         Some(self)
    ///     }
    ///     // ... other methods
    /// }
    /// ```
    fn settings(&self) -> Option<&dyn OperatorSettings> {
        None
    }

    /// Get mutable operator settings for editing parameters.
    ///
    /// Override this method alongside `settings()` if your operator implements
    /// [`OperatorSettings`].
    fn settings_mut(&mut self) -> Option<&mut dyn OperatorSettings> {
        None
    }

    /// Get visualization data from this operator.
    ///
    /// Override this method and return `Some(self)` if your operator implements
    /// [`OperatorVisuals`] to expose visualization data like waveforms,
    /// histograms, etc.
    ///
    /// # Example
    ///
    /// ```ignore
    /// impl Operator for ScopeOp {
    ///     fn visuals(&self) -> Option<&dyn OperatorVisuals> {
    ///         Some(self)
    ///     }
    ///     // ... other methods
    /// }
    /// ```
    fn visuals(&self) -> Option<&dyn OperatorVisuals> {
        None
    }
}

// Enable downcasting for boxed Operator trait objects.
// This provides methods like `downcast_ref::<T>()` and `downcast_mut::<T>()`
// directly on `&dyn Operator` and `Box<dyn Operator>`.
impl_downcast!(Operator);
