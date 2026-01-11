//! Built-in operator implementations
//!
//! This module contains the basic operators that come with the system:
//! - [`ConstantOp`] - Outputs a constant value
//! - [`AddOp`] / [`MultiplyOp`] - Basic arithmetic
//! - [`SineWaveOp`] - Time-based sine wave generator
//! - [`SumOp`] - Variadic sum of multiple inputs
//! - [`CompareOp`] - Comparison operations
//! - [`Vec3ComposeOp`] - Vector composition
//! - [`ScopeOp`] - Waveform visualization

mod arithmetic;
mod compare;
mod compose;
mod constant;
mod scope;
mod sum;
mod wave;

pub use arithmetic::{AddOp, MultiplyOp};
pub use compare::{CompareMode, CompareOp};
pub use compose::Vec3ComposeOp;
pub use constant::ConstantOp;
pub use scope::ScopeOp;
pub use sum::SumOp;
pub use wave::SineWaveOp;
