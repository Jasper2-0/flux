//! Logic and Integer operators (12 total)
//!
//! - Boolean (6): And, Or, Not, Xor, All, Any
//! - Integer (6): IntAdd, IntMultiply, IntDivide, IntModulo, IntClamp, IntToFloat

mod boolean;
mod integer;

pub use boolean::*;
pub use integer::*;

use crate::registry::OperatorRegistry;

pub fn register_all(registry: &OperatorRegistry) {
    boolean::register(registry);
    integer::register(registry);
}
