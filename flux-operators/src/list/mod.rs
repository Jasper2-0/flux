//! List operators (8 total)
//! - FloatList, ListLength, ListGet, ListSum
//! - ListAverage, ListMin, ListMax, ListMap

use crate::registry::OperatorRegistry;

mod list_ops;

pub use list_ops::*;

pub fn register_all(registry: &OperatorRegistry) {
    list_ops::register(registry);
}
