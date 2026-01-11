//! String operators (8 total)
//! - StringConcat, StringFormat, StringLength, SubString
//! - StringSplit, FloatToString, IntToString, StringContains

use crate::registry::OperatorRegistry;

mod string_ops;

pub use string_ops::*;

pub fn register_all(registry: &OperatorRegistry) {
    string_ops::register(registry);
}
