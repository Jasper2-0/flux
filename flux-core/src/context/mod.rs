//! Evaluation context for the operator graph system
//!
//! This module contains:
//! - [`EvalContext`] - The main context passed during operator evaluation
//! - [`CallContext`] - Context identifier for subroutine/loop caching
//! - [`GizmoVisibility`] / [`TransformGizmoMode`] - Gizmo settings
//! - [`Mat4`] - 4x4 matrix type alias

mod call_context;
mod types;

pub use call_context::CallContext;
pub use types::{GizmoVisibility, Mat4, TransformGizmoMode, MAT4_IDENTITY};

use std::collections::HashMap;

use crate::value::Value;

// ============================================================================
// Evaluation Context
// ============================================================================

/// Full evaluation context passed during operator computation
#[derive(Clone, Debug)]
pub struct EvalContext {
    // === Timing ===
    /// Global time in seconds
    pub time: f64,
    /// Local time (may differ in nested compositions)
    pub local_time: f64,
    /// Local FX time for effects
    pub local_fx_time: f64,
    /// Delta time since last frame
    pub delta_time: f64,
    /// Current frame number
    pub frame: u64,

    // === Transform ===
    /// Camera to clip space transform (projection matrix)
    pub camera_to_clip: Mat4,
    /// World to camera transform (view matrix)
    pub world_to_camera: Mat4,
    /// Object to world transform (model matrix)
    pub object_to_world: Mat4,

    // === Display ===
    /// Background color (RGBA)
    pub background_color: [f32; 4],
    /// Foreground/text color (RGBA)
    pub foreground_color: [f32; 4],
    /// Render resolution (width, height)
    pub resolution: (u32, u32),

    // === Context Variables ===
    /// Boolean context variables
    pub bool_vars: HashMap<String, bool>,
    /// Integer context variables
    pub int_vars: HashMap<String, i32>,
    /// Float context variables
    pub float_vars: HashMap<String, f32>,
    /// String context variables
    pub string_vars: HashMap<String, String>,
    /// Generic object context variables
    pub object_vars: HashMap<String, Value>,

    // === Gizmos ===
    /// Current gizmo visibility setting
    pub show_gizmos: GizmoVisibility,
    /// Current transform gizmo mode
    pub transform_gizmo_mode: TransformGizmoMode,

    // === Call Context ===
    /// Context identifier for subroutine/loop caching.
    ///
    /// When the same operator is evaluated in different subroutine calls
    /// or loop iterations, this context ensures separate cache entries.
    pub call_context: CallContext,

    // === Internal ===
    /// Parent time for nested time contexts
    parent_time: Option<f64>,
}

impl EvalContext {
    pub fn new() -> Self {
        Self {
            // Timing
            time: 0.0,
            local_time: 0.0,
            local_fx_time: 0.0,
            delta_time: 0.0,
            frame: 0,

            // Transform
            camera_to_clip: MAT4_IDENTITY,
            world_to_camera: MAT4_IDENTITY,
            object_to_world: MAT4_IDENTITY,

            // Display
            background_color: [0.0, 0.0, 0.0, 1.0],
            foreground_color: [1.0, 1.0, 1.0, 1.0],
            resolution: (1920, 1080),

            // Context Variables
            bool_vars: HashMap::new(),
            int_vars: HashMap::new(),
            float_vars: HashMap::new(),
            string_vars: HashMap::new(),
            object_vars: HashMap::new(),

            // Gizmos
            show_gizmos: GizmoVisibility::default(),
            transform_gizmo_mode: TransformGizmoMode::default(),

            // Call Context
            call_context: CallContext::root(),

            // Internal
            parent_time: None,
        }
    }

    /// Reset context to default state
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    // === Time Management ===

    /// Advance time by dt seconds and increment frame
    pub fn advance(&mut self, dt: f64) {
        self.delta_time = dt;
        self.time += dt;
        self.local_time += dt;
        self.local_fx_time += dt;
        self.frame += 1;
    }

    /// Check if time has changed beyond a resolution threshold
    pub fn has_time_changed(&self, resolution: f64) -> bool {
        if let Some(parent) = self.parent_time {
            (self.time - parent).abs() > resolution
        } else {
            self.delta_time.abs() > resolution
        }
    }

    /// Create a child context with different local time
    pub fn with_local_time(&self, local_time: f64) -> Self {
        let mut ctx = self.clone();
        ctx.parent_time = Some(self.time);
        ctx.local_time = local_time;
        ctx
    }

    /// Create a child context for FX with separate time
    pub fn with_fx_time(&self, fx_time: f64) -> Self {
        let mut ctx = self.clone();
        ctx.local_fx_time = fx_time;
        ctx
    }

    /// Create a child context for a subroutine call or loop iteration.
    ///
    /// This creates a new context with a derived [`CallContext`] that ensures
    /// cache isolation for operators evaluated within this context.
    ///
    /// # Arguments
    ///
    /// * `index` - The child index (e.g., loop iteration number or call site ID)
    ///
    /// # Example
    ///
    /// ```
    /// use flux_core::EvalContext;
    ///
    /// let ctx = EvalContext::new();
    ///
    /// // Create contexts for loop iterations
    /// let iter_0 = ctx.with_call_context(0);
    /// let iter_1 = ctx.with_call_context(1);
    ///
    /// // Each iteration has a unique call context
    /// assert_ne!(iter_0.call_context, iter_1.call_context);
    /// ```
    pub fn with_call_context(&self, index: u32) -> Self {
        let mut ctx = self.clone();
        ctx.call_context = self.call_context.child(index);
        ctx
    }

    // === Transform Management ===

    /// Set to default camera (identity matrices)
    pub fn set_default_camera(&mut self) {
        self.world_to_camera = MAT4_IDENTITY;
        self.camera_to_clip = MAT4_IDENTITY;
    }

    /// Set the object transform matrix
    pub fn set_object_transform(&mut self, transform: Mat4) {
        self.object_to_world = transform;
    }

    // === Variable Accessors ===

    // Float variables
    pub fn set_float_var(&mut self, name: &str, value: f32) {
        self.float_vars.insert(name.to_string(), value);
    }

    pub fn get_float_var(&self, name: &str) -> Option<f32> {
        self.float_vars.get(name).copied()
    }

    pub fn get_float_var_or(&self, name: &str, default: f32) -> f32 {
        self.float_vars.get(name).copied().unwrap_or(default)
    }

    // Int variables
    pub fn set_int_var(&mut self, name: &str, value: i32) {
        self.int_vars.insert(name.to_string(), value);
    }

    pub fn get_int_var(&self, name: &str) -> Option<i32> {
        self.int_vars.get(name).copied()
    }

    pub fn get_int_var_or(&self, name: &str, default: i32) -> i32 {
        self.int_vars.get(name).copied().unwrap_or(default)
    }

    // Bool variables
    pub fn set_bool_var(&mut self, name: &str, value: bool) {
        self.bool_vars.insert(name.to_string(), value);
    }

    pub fn get_bool_var(&self, name: &str) -> Option<bool> {
        self.bool_vars.get(name).copied()
    }

    pub fn get_bool_var_or(&self, name: &str, default: bool) -> bool {
        self.bool_vars.get(name).copied().unwrap_or(default)
    }

    // String variables
    pub fn set_string_var(&mut self, name: &str, value: &str) {
        self.string_vars.insert(name.to_string(), value.to_string());
    }

    pub fn get_string_var(&self, name: &str) -> Option<&String> {
        self.string_vars.get(name)
    }

    pub fn get_string_var_or<'a>(&'a self, name: &str, default: &'a str) -> &'a str {
        self.string_vars
            .get(name)
            .map(|s| s.as_str())
            .unwrap_or(default)
    }

    // Object variables
    pub fn set_object_var(&mut self, name: &str, value: Value) {
        self.object_vars.insert(name.to_string(), value);
    }

    pub fn get_object_var(&self, name: &str) -> Option<&Value> {
        self.object_vars.get(name)
    }

    // === Gizmos ===

    /// Check if gizmos should be visible
    pub fn should_show_gizmos(&self, is_selected: bool) -> bool {
        match self.show_gizmos {
            GizmoVisibility::Off => false,
            GizmoVisibility::On => true,
            GizmoVisibility::IfSelected => is_selected,
            GizmoVisibility::Inherit => true, // Default to showing if no parent context
        }
    }
}

impl Default for EvalContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;

    #[test]
    fn test_eval_context_new() {
        let ctx = EvalContext::new();
        assert_eq!(ctx.time, 0.0);
        assert_eq!(ctx.frame, 0);
        assert_eq!(ctx.resolution, (1920, 1080));
    }

    #[test]
    fn test_eval_context_advance() {
        let mut ctx = EvalContext::new();
        ctx.advance(0.016);
        assert!((ctx.time - 0.016).abs() < 1e-10);
        assert!((ctx.delta_time - 0.016).abs() < 1e-10);
        assert_eq!(ctx.frame, 1);
    }

    #[test]
    fn test_eval_context_reset() {
        let mut ctx = EvalContext::new();
        ctx.advance(1.0);
        ctx.set_float_var("test", 42.0);
        ctx.reset();
        assert_eq!(ctx.time, 0.0);
        assert_eq!(ctx.frame, 0);
        assert!(ctx.float_vars.is_empty());
    }

    #[test]
    fn test_context_variables() {
        let mut ctx = EvalContext::new();

        // Float
        ctx.set_float_var("speed", 10.5);
        assert_eq!(ctx.get_float_var("speed"), Some(10.5));
        assert_eq!(ctx.get_float_var_or("missing", 0.0), 0.0);

        // Int
        ctx.set_int_var("count", 42);
        assert_eq!(ctx.get_int_var("count"), Some(42));
        assert_eq!(ctx.get_int_var_or("missing", -1), -1);

        // Bool
        ctx.set_bool_var("enabled", true);
        assert_eq!(ctx.get_bool_var("enabled"), Some(true));
        assert!(!ctx.get_bool_var_or("missing", false));

        // String
        ctx.set_string_var("name", "test");
        assert_eq!(ctx.get_string_var("name"), Some(&"test".to_string()));
        assert_eq!(ctx.get_string_var_or("missing", "default"), "default");

        // Object
        ctx.set_object_var("value", Value::Float(PI));
        assert_eq!(ctx.get_object_var("value"), Some(&Value::Float(PI)));
    }

    #[test]
    fn test_with_local_time() {
        let ctx = EvalContext::new();
        let child = ctx.with_local_time(5.0);
        assert_eq!(child.local_time, 5.0);
        assert_eq!(child.parent_time, Some(0.0));
    }

    #[test]
    fn test_has_time_changed() {
        let mut ctx = EvalContext::new();
        ctx.advance(0.1);
        assert!(ctx.has_time_changed(0.01));
        assert!(!ctx.has_time_changed(1.0));
    }

    #[test]
    fn test_gizmo_visibility() {
        let mut ctx = EvalContext::new();

        ctx.show_gizmos = GizmoVisibility::Off;
        assert!(!ctx.should_show_gizmos(true));
        assert!(!ctx.should_show_gizmos(false));

        ctx.show_gizmos = GizmoVisibility::On;
        assert!(ctx.should_show_gizmos(true));
        assert!(ctx.should_show_gizmos(false));

        ctx.show_gizmos = GizmoVisibility::IfSelected;
        assert!(ctx.should_show_gizmos(true));
        assert!(!ctx.should_show_gizmos(false));
    }

    #[test]
    fn test_transform_gizmo_mode() {
        let mut ctx = EvalContext::new();
        assert_eq!(ctx.transform_gizmo_mode, TransformGizmoMode::None);

        ctx.transform_gizmo_mode = TransformGizmoMode::Move;
        assert_eq!(ctx.transform_gizmo_mode, TransformGizmoMode::Move);
    }
}
