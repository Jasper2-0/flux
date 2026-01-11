//! Time and animation operators (10 total)

use crate::registry::OperatorRegistry;

mod clock;
mod oscillators;

pub use clock::*;
pub use oscillators::*;

pub fn register_all(registry: &OperatorRegistry) {
    clock::register(registry);
    oscillators::register(registry);
}
