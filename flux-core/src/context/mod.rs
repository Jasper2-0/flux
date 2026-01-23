//! Evaluation context for the operator graph system
//!
//! This module contains:
//! - [`EvalContext`] - The main context passed during operator evaluation
//! - [`CallContext`] - Context identifier for subroutine/loop caching
//! - [`GizmoVisibility`] / [`TransformGizmoMode`] - Gizmo settings
//! - [`FogParameters`] / [`PbrMaterial`] / [`PointLight`] - Rendering settings
//! - [`Camera`] / [`PerspectiveCamera`] - Camera abstraction

mod call_context;
mod camera;
mod rendering;
mod types;

pub use call_context::CallContext;
pub use camera::{Camera, PerspectiveCamera};
pub use rendering::{FogParameters, PbrMaterial, PointLight};
pub use types::{GizmoVisibility, Mat4, TransformGizmoMode, MAT4_IDENTITY};

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::value::Value;

// ============================================================================
// Extension Storage
// ============================================================================

/// Type-erased extension storage for domain-specific contexts.
///
/// Extensions allow external systems (GPU, audio, physics) to attach
/// their context to EvalContext without flux-core having dependencies
/// on those systems.
#[derive(Clone, Default)]
pub struct Extensions {
    inner: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Extensions {
    /// Create empty extension storage
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Get an extension by type
    pub fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|ext| ext.downcast_ref())
    }

    /// Set an extension, returning the previous value if any
    pub fn set<T: 'static + Send + Sync>(&mut self, value: T) -> Option<Arc<dyn Any + Send + Sync>> {
        self.inner.insert(TypeId::of::<T>(), Arc::new(value))
    }

    /// Set an extension from an Arc (avoids double-wrapping)
    pub fn set_arc<T: 'static + Send + Sync>(
        &mut self,
        value: Arc<T>,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        self.inner.insert(TypeId::of::<T>(), value)
    }

    /// Remove an extension by type
    pub fn remove<T: 'static + Send + Sync>(&mut self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.inner.remove(&TypeId::of::<T>())
    }

    /// Check if an extension exists
    pub fn contains<T: 'static + Send + Sync>(&self) -> bool {
        self.inner.contains_key(&TypeId::of::<T>())
    }

    /// Get the number of extensions
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if there are no extensions
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl std::fmt::Debug for Extensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extensions")
            .field("count", &self.inner.len())
            .finish()
    }
}

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

    // === Camera/Transform ===
    /// Camera to clip space transform (projection matrix)
    pub camera_to_clip: Mat4,
    /// World to camera transform (view matrix)
    pub world_to_camera: Mat4,
    /// Object to world transform (model matrix)
    pub object_to_world: Mat4,

    // === Rendering ===
    /// Fog parameters
    pub fog: FogParameters,
    /// Current PBR material
    pub pbr_material: PbrMaterial,
    /// Active point lights
    pub point_lights: Vec<PointLight>,
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

    // === Extensions ===
    /// Type-erased extensions for domain-specific contexts (GPU, audio, etc.)
    ///
    /// This allows external systems to attach their context without
    /// flux-core depending on those systems.
    pub extensions: Extensions,

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

            // Camera/Transform
            camera_to_clip: MAT4_IDENTITY,
            world_to_camera: MAT4_IDENTITY,
            object_to_world: MAT4_IDENTITY,

            // Rendering
            fog: FogParameters::new(),
            pbr_material: PbrMaterial::new(),
            point_lights: Vec::new(),
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

            // Extensions
            extensions: Extensions::new(),

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

    // === Camera Management ===

    /// Set camera matrices from a Camera implementation
    pub fn set_camera(&mut self, camera: &impl Camera) {
        self.world_to_camera = camera.get_view_matrix();
        self.camera_to_clip = camera.get_projection_matrix();
    }

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

    // === Lighting ===

    /// Add a point light to the context
    pub fn add_point_light(&mut self, light: PointLight) {
        self.point_lights.push(light);
    }

    /// Clear all point lights
    pub fn clear_lights(&mut self) {
        self.point_lights.clear();
    }

    // === Extensions ===

    /// Get an extension by type.
    ///
    /// # Example
    ///
    /// ```
    /// use flux_core::EvalContext;
    ///
    /// struct MyGpuContext { /* ... */ }
    ///
    /// let ctx = EvalContext::new();
    /// if let Some(gpu) = ctx.get_extension::<MyGpuContext>() {
    ///     // Use GPU context
    /// }
    /// ```
    pub fn get_extension<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.extensions.get::<T>()
    }

    /// Set an extension, taking ownership.
    ///
    /// # Example
    ///
    /// ```
    /// use flux_core::EvalContext;
    ///
    /// struct MyGpuContext { device_name: String }
    ///
    /// let mut ctx = EvalContext::new();
    /// ctx.set_extension(MyGpuContext {
    ///     device_name: "RTX 4090".to_string()
    /// });
    /// ```
    pub fn set_extension<T: 'static + Send + Sync>(&mut self, value: T) {
        self.extensions.set(value);
    }

    /// Set an extension from an Arc (avoids double-wrapping when you already have an Arc).
    pub fn set_extension_arc<T: 'static + Send + Sync>(&mut self, value: Arc<T>) {
        self.extensions.set_arc(value);
    }

    /// Remove an extension by type.
    pub fn remove_extension<T: 'static + Send + Sync>(&mut self) {
        self.extensions.remove::<T>();
    }

    /// Check if an extension of the given type exists.
    pub fn has_extension<T: 'static + Send + Sync>(&self) -> bool {
        self.extensions.contains::<T>()
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
    fn test_fog_parameters() {
        let fog = FogParameters::linear(10.0, 100.0, [0.5, 0.5, 0.5, 1.0]);
        assert!(fog.enabled);
        assert_eq!(fog.start, 10.0);
        assert_eq!(fog.end, 100.0);

        let exp_fog = FogParameters::exponential(0.02, [0.3, 0.3, 0.3, 1.0]);
        assert!(exp_fog.enabled);
        assert_eq!(exp_fog.density, 0.02);
    }

    #[test]
    fn test_pbr_material() {
        let metal = PbrMaterial::metal([0.8, 0.7, 0.1, 1.0], 0.3);
        assert_eq!(metal.metallic, 1.0);
        assert_eq!(metal.roughness, 0.3);

        let plastic = PbrMaterial::dielectric([1.0, 0.0, 0.0, 1.0], 0.8);
        assert_eq!(plastic.metallic, 0.0);
    }

    #[test]
    fn test_point_light() {
        let mut ctx = EvalContext::new();
        ctx.add_point_light(PointLight::new([0.0, 5.0, 0.0], [1.0, 1.0, 1.0], 2.0));
        ctx.add_point_light(PointLight::new([5.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0));
        assert_eq!(ctx.point_lights.len(), 2);

        ctx.clear_lights();
        assert!(ctx.point_lights.is_empty());
    }

    #[test]
    fn test_perspective_camera() {
        let camera = PerspectiveCamera::look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

        let view = camera.get_view_matrix();
        let proj = camera.get_projection_matrix();
        let pos = camera.get_position();

        assert_eq!(pos, [0.0, 0.0, 5.0]);
        // View matrix should be valid
        assert!(view[3][3] != 0.0);
        // Projection matrix should be valid
        assert!(proj[0][0] != 0.0);
    }

    #[test]
    fn test_set_camera() {
        let mut ctx = EvalContext::new();
        let camera = PerspectiveCamera::new();
        ctx.set_camera(&camera);

        // Matrices should be set from camera
        assert_ne!(ctx.world_to_camera, MAT4_IDENTITY);
        assert_ne!(ctx.camera_to_clip, MAT4_IDENTITY);
    }

    #[test]
    fn test_transform_gizmo_mode() {
        let mut ctx = EvalContext::new();
        assert_eq!(ctx.transform_gizmo_mode, TransformGizmoMode::None);

        ctx.transform_gizmo_mode = TransformGizmoMode::Move;
        assert_eq!(ctx.transform_gizmo_mode, TransformGizmoMode::Move);
    }

    // === Extension Tests ===

    #[test]
    fn test_extensions_basic() {
        // Test type for extensions
        struct TestGpuContext {
            device_name: String,
        }

        let mut ext = Extensions::new();
        assert!(ext.is_empty());
        assert_eq!(ext.len(), 0);

        // Set and get
        ext.set(TestGpuContext {
            device_name: "Test GPU".to_string(),
        });
        assert!(!ext.is_empty());
        assert_eq!(ext.len(), 1);
        assert!(ext.contains::<TestGpuContext>());

        let gpu = ext.get::<TestGpuContext>().unwrap();
        assert_eq!(gpu.device_name, "Test GPU");

        // Remove
        ext.remove::<TestGpuContext>();
        assert!(ext.is_empty());
        assert!(!ext.contains::<TestGpuContext>());
    }

    #[test]
    fn test_extensions_multiple_types() {
        struct TypeA(i32);
        struct TypeB(String);

        let mut ext = Extensions::new();
        ext.set(TypeA(42));
        ext.set(TypeB("hello".to_string()));

        assert_eq!(ext.len(), 2);
        assert_eq!(ext.get::<TypeA>().unwrap().0, 42);
        assert_eq!(ext.get::<TypeB>().unwrap().0, "hello");

        // Getting wrong type returns None
        assert!(ext.get::<i32>().is_none());
    }

    #[test]
    fn test_extensions_arc() {
        struct SharedResource {
            value: i32,
        }

        let mut ext = Extensions::new();
        let shared = Arc::new(SharedResource { value: 100 });
        ext.set_arc(shared.clone());

        let retrieved = ext.get::<SharedResource>().unwrap();
        assert_eq!(retrieved.value, 100);
    }

    #[test]
    fn test_eval_context_extensions() {
        struct TestAudioContext {
            sample_rate: u32,
        }

        struct TestGpuContext {
            device_id: u32,
        }

        let mut ctx = EvalContext::new();
        assert!(!ctx.has_extension::<TestAudioContext>());

        // Set extension
        ctx.set_extension(TestAudioContext { sample_rate: 48000 });
        assert!(ctx.has_extension::<TestAudioContext>());

        // Get extension
        let audio = ctx.get_extension::<TestAudioContext>().unwrap();
        assert_eq!(audio.sample_rate, 48000);

        // Multiple extensions
        ctx.set_extension(TestGpuContext { device_id: 1 });
        assert!(ctx.has_extension::<TestGpuContext>());
        assert!(ctx.has_extension::<TestAudioContext>());

        // Remove extension
        ctx.remove_extension::<TestAudioContext>();
        assert!(!ctx.has_extension::<TestAudioContext>());
        assert!(ctx.has_extension::<TestGpuContext>());
    }

    #[test]
    fn test_eval_context_extensions_clone() {
        struct TestContext {
            value: i32,
        }

        let mut ctx = EvalContext::new();
        ctx.set_extension(TestContext { value: 42 });

        // Clone should preserve extensions
        let cloned = ctx.clone();
        let ext = cloned.get_extension::<TestContext>().unwrap();
        assert_eq!(ext.value, 42);
    }
}
