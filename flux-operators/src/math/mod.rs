//! Math operators (35+ total)
//!
//! - Arithmetic (14): Add, Subtract, Multiply, Divide, Modulo, Pow, Sqrt, Log, Abs, Negate, Floor, Ceil, Round, Truncate
//!   All arithmetic operators are polymorphic and work with Float, Int, Vec2, Vec3, Vec4, and Color.
//! - Comparison (5): Min, Max, Clamp, Sign, Step - all polymorphic
//! - Interpolation (5): Lerp, SmoothStep (polymorphic), Remap, InverseLerp, MapRange
//! - Trigonometry (6): Sin, Cos (polymorphic), Tan, Atan2, DegreesToRadians, RadiansToDegrees
//! - Random/Noise (4): Random, PerlinNoise, PerlinNoise3D, Hash

mod arithmetic;
mod comparison;
mod interpolation;
mod random;
mod trig;

pub use arithmetic::*;
pub use comparison::*;
pub use interpolation::*;
pub use random::*;
pub use trig::*;

use crate::registry::OperatorRegistry;

/// Register all math operators
pub fn register_all(registry: &OperatorRegistry) {
    arithmetic::register(registry);
    comparison::register(registry);
    interpolation::register(registry);
    trig::register(registry);
    random::register(registry);
}
