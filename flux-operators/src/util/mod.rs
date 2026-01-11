//! Utility operators (6 total)
//! - Print, Passthrough, Comment
//! - Bookmark, TypeOf, IsNull

use crate::registry::OperatorRegistry;

mod debug;

pub use debug::*;

pub fn register_all(registry: &OperatorRegistry) {
    debug::register(registry);
}
