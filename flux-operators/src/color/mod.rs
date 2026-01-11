//! Color operators (8 total)

use crate::registry::OperatorRegistry;

mod color_ops;

pub use color_ops::*;

pub fn register_all(registry: &OperatorRegistry) {
    color_ops::register(registry);
}
