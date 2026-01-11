//! Vector operators (15 total)
//!
//! - Vec2 (5): Vec2Compose, Vec2Decompose, Vec2Add, Vec2Scale, Vec2Length
//! - Vec3 (7): Vec3Compose, Vec3Decompose, Vec3Add, Vec3Subtract, Vec3Scale, Vec3Normalize, Vec3Dot, Vec3Cross, Vec3Length, Vec3Distance
//! - Vec4 (3): Vec4Compose, Vec4Decompose, Vec3ToVec4

mod vec2;
mod vec3;
mod vec4;

pub use vec2::*;
pub use vec3::*;
pub use vec4::*;

use crate::registry::OperatorRegistry;

pub fn register_all(registry: &OperatorRegistry) {
    vec2::register(registry);
    vec3::register(registry);
    vec4::register(registry);
}
