//! Flux Operators - Operator implementations for the Flux graph system
//!
//! This crate provides all the built-in operators for creating computational graphs.
//! Operators are organized by category:
//!
//! - [`builtin`] - Core operators (Constant, Add, Multiply, SineWave, etc.)
//! - [`math`] - Mathematical operations (arithmetic, trig, interpolation, etc.)
//! - [`logic`] - Boolean and integer logic
//! - [`vector`] - Vec2, Vec3, Vec4 operations
//! - [`color`] - Color manipulation
//! - [`time`] - Time-based operations (clocks, oscillators)
//! - [`flow`] - Control flow (state, context, conditionals)
//! - [`string`] - String manipulation
//! - [`list`] - List operations
//! - [`util`] - Utility operators (debug, etc.)
//!
//! # Registry
//!
//! The [`OperatorRegistry`] provides dynamic operator creation by name or type ID.
//! Use [`create_default_registry`] to get a registry with all built-in operators.
//!
//! # Derive Macro
//!
//! The `Operator` derive macro simplifies creating new operators:
//!
//! ```ignore
//! use flux_macros::Operator;
//! use flux_core::{Id, InputPort, OutputPort, EvalContext, Operator, OperatorMeta, Value};
//!
//! #[derive(Operator)]
//! #[operator(name = "MyAdd", category = "Math", description = "Adds two numbers")]
//! #[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
//! struct MyAddOp {
//!     _id: Id,
//!     _inputs: Vec<InputPort>,
//!     _outputs: Vec<OutputPort>,
//!     #[input(label = "A", default = 0.0)]
//!     a: f32,
//!     #[input(label = "B", default = 0.0)]
//!     b: f32,
//!     #[output(label = "Sum")]
//!     sum: f32,
//! }
//!
//! impl MyAddOp {
//!     fn compute_impl(&mut self, _ctx: &EvalContext, get_input: &dyn Fn(Id, usize) -> Value) {
//!         let a = self.get_a(get_input);
//!         let b = self.get_b(get_input);
//!         self.set_sum(a + b);
//!     }
//! }
//! ```

#![allow(ambiguous_glob_reexports)]

// Re-export the derive macros
pub use flux_macros::Operator;
pub use flux_macros::OperatorMeta as DeriveOperatorMeta;

pub mod builtin;
pub mod color;
pub mod flow;
pub mod list;
pub mod logic;
pub mod math;
pub mod registry;
pub mod string;
pub mod time;
pub mod util;
pub mod vector;

// Re-export builtin operators at the crate root
pub use builtin::*;

// Re-export all category operators
pub use color::*;
pub use flow::*;
pub use list::*;
pub use logic::*;
pub use math::*;
pub use string::*;
pub use time::*;
pub use util::*;
pub use vector::*;

// Re-export registry types
pub use registry::{
    capture_meta, capture_meta_simple, create_default_registry, ExtendedEntry,
    MetaCapturingFactory, OperatorFactory, OperatorParams, OperatorRegistry, OperatorWithMeta,
    ParameterMeta, ParameterizedMetaFactory, ParameterType, ParameterValue, RegistryEntry,
};

/// Register all operators with the given registry
pub fn register_all_operators(registry: &OperatorRegistry) {
    math::register_all(registry);
    logic::register_all(registry);
    vector::register_all(registry);
    color::register_all(registry);
    time::register_all(registry);
    flow::register_all(registry);
    string::register_all(registry);
    list::register_all(registry);
    util::register_all(registry);
}

#[cfg(test)]
mod derive_macro_tests {
    use flux_core::{
        EvalContext, Id, InputPort, InputResolver, Operator, OperatorMeta, OutputPort, PinShape,
        Value,
    };
    use flux_macros::Operator;

    /// A test operator created with the derive macro.
    /// This demonstrates the full attribute syntax.
    #[derive(Operator)]
    #[operator(name = "TestMult", category = "Math", description = "Multiplies two numbers")]
    #[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
    #[allow(dead_code)] // Marker fields are intentionally unused at runtime
    struct TestMultOp {
        _id: Id,
        _inputs: Vec<InputPort>,
        _outputs: Vec<OutputPort>,
        #[input(label = "A", default = 1.0)]
        a: f32,
        #[input(label = "B", default = 1.0, range = (0.0, 10.0), unit = "x")]
        b: f32,
        #[output(label = "Product")]
        product: f32,
    }

    impl TestMultOp {
        fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
            let a = self.get_a(get_input);
            let b = self.get_b(get_input);
            self.set_product(a * b);
        }
    }

    #[test]
    fn test_derive_operator_trait() {
        let op = TestMultOp::new();

        // Test Operator trait methods
        assert_eq!(op.name(), "TestMult");
        assert_eq!(op.inputs().len(), 2);
        assert_eq!(op.outputs().len(), 1);

        // Check input defaults (InputPort uses `name` field, not `label`)
        assert_eq!(op.inputs()[0].name, "A");
        assert_eq!(op.inputs()[0].default.as_float(), Some(1.0));
        assert_eq!(op.inputs()[1].name, "B");
        assert_eq!(op.inputs()[1].default.as_float(), Some(1.0));

        // Check output
        assert_eq!(op.outputs()[0].name, "Product");
    }

    #[test]
    fn test_derive_operator_meta() {
        let op = TestMultOp::new();

        // Test OperatorMeta trait methods
        assert_eq!(op.category(), "Math");
        assert_eq!(op.description(), "Multiplies two numbers");
        assert_eq!(op.category_color(), [0.35, 0.35, 0.55, 1.0]);

        // Test input meta
        let input_a = op.input_meta(0).unwrap();
        assert_eq!(input_a.label, "A");
        assert_eq!(input_a.shape, PinShape::CircleFilled);

        let input_b = op.input_meta(1).unwrap();
        assert_eq!(input_b.label, "B");
        assert_eq!(input_b.range, Some((0.0, 10.0)));
        assert_eq!(input_b.unit, Some("x"));

        // Test output meta
        let output = op.output_meta(0).unwrap();
        assert_eq!(output.label, "Product");
        assert_eq!(output.shape, PinShape::TriangleFilled);
    }

    #[test]
    fn test_derive_compute() {
        let mut op = TestMultOp::new();

        // Set input defaults
        op.inputs_mut()[0].default = Value::Float(3.0);
        op.inputs_mut()[1].default = Value::Float(4.0);

        // Compute
        let ctx = EvalContext::new();
        let get_input = |_: Id, _: usize| Value::Float(0.0);
        op.compute(&ctx, &get_input);

        // Check result
        assert_eq!(op.outputs()[0].value.as_float(), Some(12.0));
    }
}
