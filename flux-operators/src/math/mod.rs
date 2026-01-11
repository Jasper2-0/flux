//! Math operators (35 total)
//!
//! - Basic Arithmetic (10): Add, Subtract, Multiply, Divide, Modulo, Pow, Sqrt, Log, Abs, Negate
//! - Comparison (6): Compare, Min, Max, Clamp, Sign, Step
//! - Interpolation (5): Lerp, SmoothStep, Remap, InverseLerp, MapRange
//! - Trigonometry (6): Sin, Cos, Tan, Atan2, DegreesToRadians, RadiansToDegrees
//! - Rounding (4): Floor, Ceil, Round, Truncate
//! - Random/Noise (4): Random, PerlinNoise, PerlinNoise3D, Hash

mod arithmetic;
mod comparison;
mod interpolation;
mod random;
mod rounding;
mod trig;

pub use arithmetic::*;
pub use comparison::*;
pub use interpolation::*;
pub use random::*;
pub use rounding::*;
pub use trig::*;

use crate::registry::OperatorRegistry;

/// Register all math operators
pub fn register_all(registry: &OperatorRegistry) {
    arithmetic::register(registry);
    comparison::register(registry);
    interpolation::register(registry);
    trig::register(registry);
    rounding::register(registry);
    random::register(registry);
}
