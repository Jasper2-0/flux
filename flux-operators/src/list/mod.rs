//! List operators (33 total)
//!
//! ## Polymorphic (work with any list type)
//! - ListLength, ListGet, ListSlice, ListConcat
//! - ListReverse, ListFirst, ListLast
//!
//! ## FloatList-specific
//! - FloatList, ListSum, ListAverage, ListMin, ListMax
//! - ListMap, ListFilter
//!
//! ## IntList-specific
//! - IntList, IntListSum, IntListMin, IntListMax, IntListRange
//!
//! ## Vec3List-specific
//! - Vec3List, Vec3ListNormalize, Vec3ListCentroid, Vec3ListBounds
//!
//! ## ColorList-specific
//! - ColorList, ColorListSample, ColorListBlend
//!
//! ## Conversions
//! - IntListToFloatList, FloatListToIntList
//! - Vec3ListFlatten, FloatListToVec3List
//! - ColorListToVec4List, Vec4ListToColorList

use crate::registry::OperatorRegistry;

mod list_ops;
mod int_list_ops;
mod vec3_list_ops;
mod color_list_ops;
mod conversions;

pub use list_ops::*;
pub use int_list_ops::*;
pub use vec3_list_ops::*;
pub use color_list_ops::*;
pub use conversions::*;

pub fn register_all(registry: &OperatorRegistry) {
    list_ops::register(registry);
    int_list_ops::register(registry);
    vec3_list_ops::register(registry);
    color_list_ops::register(registry);
    conversions::register(registry);
}
