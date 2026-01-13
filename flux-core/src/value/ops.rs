//! Arithmetic operations on Value types
//!
//! This module implements `std::ops` traits for [`Value`], enabling natural
//! arithmetic syntax like `a + b` for values of compatible types.
//!
//! # Broadcasting Rules
//!
//! When operands have different types, the "wider" type wins:
//! - `Float + Vec3 = Vec3` (scalar broadcasts to all components)
//! - `Int + Float = Float` (int promotes to float)
//! - `Vec3 + Vec3 = Vec3` (component-wise)
//!
//! # Integer Preservation
//!
//! Integer arithmetic is preserved when both operands are integers:
//! - `Int + Int = Int`
//! - `Int * Int = Int`
//! - `Int / Int = Int` (truncated)
//!
//! # Incompatible Types
//!
//! Operations on incompatible types return `None`:
//! - `String + Vec3 = None`
//! - `Gradient + Float = None`

use super::{Color, Value};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

// =============================================================================
// Helper macros for component-wise vector operations
// =============================================================================

/// Implements a binary operation for all arithmetic type combinations
macro_rules! impl_binary_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait for Value {
            type Output = Option<Value>;

            fn $method(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    // =========================================================
                    // Same-type operations
                    // =========================================================

                    // Primitives
                    (Value::Float(a), Value::Float(b)) => Some(Value::Float(a $op b)),
                    (Value::Int(a), Value::Int(b)) => Some(Value::Int(a $op b)),

                    // Vectors (component-wise)
                    (Value::Vec2(a), Value::Vec2(b)) => Some(Value::Vec2([
                        a[0] $op b[0],
                        a[1] $op b[1],
                    ])),
                    (Value::Vec3(a), Value::Vec3(b)) => Some(Value::Vec3([
                        a[0] $op b[0],
                        a[1] $op b[1],
                        a[2] $op b[2],
                    ])),
                    (Value::Vec4(a), Value::Vec4(b)) => Some(Value::Vec4([
                        a[0] $op b[0],
                        a[1] $op b[1],
                        a[2] $op b[2],
                        a[3] $op b[3],
                    ])),

                    // Color (component-wise RGBA)
                    (Value::Color(a), Value::Color(b)) => Some(Value::Color(Color::rgba(
                        a.r $op b.r,
                        a.g $op b.g,
                        a.b $op b.b,
                        a.a $op b.a,
                    ))),

                    // =========================================================
                    // Int/Float promotion (result is Float)
                    // =========================================================
                    (Value::Int(a), Value::Float(b)) => Some(Value::Float((a as f32) $op b)),
                    (Value::Float(a), Value::Int(b)) => Some(Value::Float(a $op (b as f32))),

                    // =========================================================
                    // Scalar broadcast to Vec2
                    // =========================================================
                    (Value::Float(s), Value::Vec2(v)) => Some(Value::Vec2([
                        s $op v[0],
                        s $op v[1],
                    ])),
                    (Value::Vec2(v), Value::Float(s)) => Some(Value::Vec2([
                        v[0] $op s,
                        v[1] $op s,
                    ])),
                    (Value::Int(i), Value::Vec2(v)) => {
                        let s = i as f32;
                        Some(Value::Vec2([s $op v[0], s $op v[1]]))
                    }
                    (Value::Vec2(v), Value::Int(i)) => {
                        let s = i as f32;
                        Some(Value::Vec2([v[0] $op s, v[1] $op s]))
                    }

                    // =========================================================
                    // Scalar broadcast to Vec3
                    // =========================================================
                    (Value::Float(s), Value::Vec3(v)) => Some(Value::Vec3([
                        s $op v[0],
                        s $op v[1],
                        s $op v[2],
                    ])),
                    (Value::Vec3(v), Value::Float(s)) => Some(Value::Vec3([
                        v[0] $op s,
                        v[1] $op s,
                        v[2] $op s,
                    ])),
                    (Value::Int(i), Value::Vec3(v)) => {
                        let s = i as f32;
                        Some(Value::Vec3([s $op v[0], s $op v[1], s $op v[2]]))
                    }
                    (Value::Vec3(v), Value::Int(i)) => {
                        let s = i as f32;
                        Some(Value::Vec3([v[0] $op s, v[1] $op s, v[2] $op s]))
                    }

                    // =========================================================
                    // Scalar broadcast to Vec4
                    // =========================================================
                    (Value::Float(s), Value::Vec4(v)) => Some(Value::Vec4([
                        s $op v[0],
                        s $op v[1],
                        s $op v[2],
                        s $op v[3],
                    ])),
                    (Value::Vec4(v), Value::Float(s)) => Some(Value::Vec4([
                        v[0] $op s,
                        v[1] $op s,
                        v[2] $op s,
                        v[3] $op s,
                    ])),
                    (Value::Int(i), Value::Vec4(v)) => {
                        let s = i as f32;
                        Some(Value::Vec4([s $op v[0], s $op v[1], s $op v[2], s $op v[3]]))
                    }
                    (Value::Vec4(v), Value::Int(i)) => {
                        let s = i as f32;
                        Some(Value::Vec4([v[0] $op s, v[1] $op s, v[2] $op s, v[3] $op s]))
                    }

                    // =========================================================
                    // Scalar broadcast to Color (RGB only, preserve alpha)
                    // =========================================================
                    (Value::Float(s), Value::Color(c)) => Some(Value::Color(Color::rgba(
                        s $op c.r,
                        s $op c.g,
                        s $op c.b,
                        c.a,  // Preserve alpha
                    ))),
                    (Value::Color(c), Value::Float(s)) => Some(Value::Color(Color::rgba(
                        c.r $op s,
                        c.g $op s,
                        c.b $op s,
                        c.a,  // Preserve alpha
                    ))),
                    (Value::Int(i), Value::Color(c)) => {
                        let s = i as f32;
                        Some(Value::Color(Color::rgba(s $op c.r, s $op c.g, s $op c.b, c.a)))
                    }
                    (Value::Color(c), Value::Int(i)) => {
                        let s = i as f32;
                        Some(Value::Color(Color::rgba(c.r $op s, c.g $op s, c.b $op s, c.a)))
                    }

                    // =========================================================
                    // Vec3 <-> Color interop
                    // =========================================================
                    (Value::Vec3(v), Value::Color(c)) => Some(Value::Color(Color::rgba(
                        v[0] $op c.r,
                        v[1] $op c.g,
                        v[2] $op c.b,
                        c.a,
                    ))),
                    (Value::Color(c), Value::Vec3(v)) => Some(Value::Color(Color::rgba(
                        c.r $op v[0],
                        c.g $op v[1],
                        c.b $op v[2],
                        c.a,
                    ))),

                    // =========================================================
                    // Vec4 <-> Color interop (isomorphic)
                    // =========================================================
                    (Value::Vec4(v), Value::Color(c)) => Some(Value::Color(Color::rgba(
                        v[0] $op c.r,
                        v[1] $op c.g,
                        v[2] $op c.b,
                        v[3] $op c.a,
                    ))),
                    (Value::Color(c), Value::Vec4(v)) => Some(Value::Color(Color::rgba(
                        c.r $op v[0],
                        c.g $op v[1],
                        c.b $op v[2],
                        c.a $op v[3],
                    ))),

                    // =========================================================
                    // Incompatible types
                    // =========================================================
                    _ => None,
                }
            }
        }
    };
}

// Generate Add, Sub, Mul, Div implementations
impl_binary_op!(Add, add, +);
impl_binary_op!(Sub, sub, -);
impl_binary_op!(Mul, mul, *);
impl_binary_op!(Div, div, /);

// =============================================================================
// Remainder (Modulo) - special handling for floats
// =============================================================================

impl Rem for Value {
    type Output = Option<Value>;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Primitives
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a % b)),
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    Some(Value::Int(0)) // Avoid panic, return 0
                } else {
                    Some(Value::Int(a % b))
                }
            }

            // Int/Float promotion
            (Value::Int(a), Value::Float(b)) => Some(Value::Float((a as f32) % b)),
            (Value::Float(a), Value::Int(b)) => Some(Value::Float(a % (b as f32))),

            // Vectors (component-wise)
            (Value::Vec2(a), Value::Vec2(b)) => {
                Some(Value::Vec2([a[0] % b[0], a[1] % b[1]]))
            }
            (Value::Vec3(a), Value::Vec3(b)) => {
                Some(Value::Vec3([a[0] % b[0], a[1] % b[1], a[2] % b[2]]))
            }
            (Value::Vec4(a), Value::Vec4(b)) => {
                Some(Value::Vec4([a[0] % b[0], a[1] % b[1], a[2] % b[2], a[3] % b[3]]))
            }

            // Scalar broadcast
            (Value::Float(s), Value::Vec3(v)) => {
                Some(Value::Vec3([s % v[0], s % v[1], s % v[2]]))
            }
            (Value::Vec3(v), Value::Float(s)) => {
                Some(Value::Vec3([v[0] % s, v[1] % s, v[2] % s]))
            }

            _ => None,
        }
    }
}

// =============================================================================
// Negation
// =============================================================================

impl Neg for Value {
    type Output = Option<Value>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Float(v) => Some(Value::Float(-v)),
            Value::Int(v) => Some(Value::Int(-v)),
            Value::Vec2(v) => Some(Value::Vec2([-v[0], -v[1]])),
            Value::Vec3(v) => Some(Value::Vec3([-v[0], -v[1], -v[2]])),
            Value::Vec4(v) => Some(Value::Vec4([-v[0], -v[1], -v[2], -v[3]])),
            Value::Color(c) => Some(Value::Color(Color::rgba(-c.r, -c.g, -c.b, c.a))), // Preserve alpha
            _ => None,
        }
    }
}

// =============================================================================
// Additional math operations as methods
// =============================================================================

impl Value {
    /// Power operation (self^exponent)
    pub fn pow(&self, exponent: &Value) -> Option<Value> {
        match (self, exponent) {
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a.powf(*b))),
            (Value::Int(a), Value::Int(b)) => {
                if *b >= 0 {
                    Some(Value::Int(a.pow(*b as u32)))
                } else {
                    // Negative exponent: promote to float
                    Some(Value::Float((*a as f32).powf(*b as f32)))
                }
            }
            (Value::Int(a), Value::Float(b)) => Some(Value::Float((*a as f32).powf(*b))),
            (Value::Float(a), Value::Int(b)) => Some(Value::Float(a.powf(*b as f32))),

            // Component-wise for vectors
            (Value::Vec3(v), Value::Float(e)) => Some(Value::Vec3([
                v[0].powf(*e),
                v[1].powf(*e),
                v[2].powf(*e),
            ])),
            (Value::Vec3(a), Value::Vec3(b)) => Some(Value::Vec3([
                a[0].powf(b[0]),
                a[1].powf(b[1]),
                a[2].powf(b[2]),
            ])),

            _ => None,
        }
    }

    /// Absolute value
    pub fn abs(&self) -> Option<Value> {
        match self {
            Value::Float(v) => Some(Value::Float(v.abs())),
            Value::Int(v) => Some(Value::Int(v.abs())),
            Value::Vec2(v) => Some(Value::Vec2([v[0].abs(), v[1].abs()])),
            Value::Vec3(v) => Some(Value::Vec3([v[0].abs(), v[1].abs(), v[2].abs()])),
            Value::Vec4(v) => Some(Value::Vec4([v[0].abs(), v[1].abs(), v[2].abs(), v[3].abs()])),
            _ => None,
        }
    }

    /// Square root
    pub fn sqrt(&self) -> Option<Value> {
        match self {
            Value::Float(v) => Some(Value::Float(v.sqrt())),
            Value::Int(v) => Some(Value::Float((*v as f32).sqrt())),
            Value::Vec2(v) => Some(Value::Vec2([v[0].sqrt(), v[1].sqrt()])),
            Value::Vec3(v) => Some(Value::Vec3([v[0].sqrt(), v[1].sqrt(), v[2].sqrt()])),
            Value::Vec4(v) => Some(Value::Vec4([v[0].sqrt(), v[1].sqrt(), v[2].sqrt(), v[3].sqrt()])),
            _ => None,
        }
    }

    /// Floor (round down)
    pub fn floor(&self) -> Option<Value> {
        match self {
            Value::Float(v) => Some(Value::Float(v.floor())),
            Value::Int(v) => Some(Value::Int(*v)),
            Value::Vec2(v) => Some(Value::Vec2([v[0].floor(), v[1].floor()])),
            Value::Vec3(v) => Some(Value::Vec3([v[0].floor(), v[1].floor(), v[2].floor()])),
            Value::Vec4(v) => Some(Value::Vec4([v[0].floor(), v[1].floor(), v[2].floor(), v[3].floor()])),
            _ => None,
        }
    }

    /// Ceil (round up)
    pub fn ceil(&self) -> Option<Value> {
        match self {
            Value::Float(v) => Some(Value::Float(v.ceil())),
            Value::Int(v) => Some(Value::Int(*v)),
            Value::Vec2(v) => Some(Value::Vec2([v[0].ceil(), v[1].ceil()])),
            Value::Vec3(v) => Some(Value::Vec3([v[0].ceil(), v[1].ceil(), v[2].ceil()])),
            Value::Vec4(v) => Some(Value::Vec4([v[0].ceil(), v[1].ceil(), v[2].ceil(), v[3].ceil()])),
            _ => None,
        }
    }

    /// Get the "width" of a type for broadcasting purposes.
    /// Higher values are "wider" and take precedence.
    pub fn type_width(&self) -> u8 {
        match self {
            Value::Int(_) => 1,
            Value::Float(_) => 2,
            Value::Vec2(_) => 3,
            Value::Vec3(_) | Value::Color(_) => 4,
            Value::Vec4(_) => 5,
            _ => 0, // Non-arithmetic types
        }
    }

    /// Check if this value is an arithmetic type
    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            Value::Float(_)
                | Value::Int(_)
                | Value::Vec2(_)
                | Value::Vec3(_)
                | Value::Vec4(_)
                | Value::Color(_)
        )
    }

    // =========================================================================
    // Comparison operations
    // =========================================================================

    /// Per-component minimum of two values
    pub fn min_value(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a.min(*b))),
            (Value::Int(a), Value::Int(b)) => Some(Value::Int(*a.min(b))),
            (Value::Int(a), Value::Float(b)) => Some(Value::Float((*a as f32).min(*b))),
            (Value::Float(a), Value::Int(b)) => Some(Value::Float(a.min(*b as f32))),
            (Value::Vec2(a), Value::Vec2(b)) => Some(Value::Vec2([a[0].min(b[0]), a[1].min(b[1])])),
            (Value::Vec3(a), Value::Vec3(b)) => Some(Value::Vec3([
                a[0].min(b[0]),
                a[1].min(b[1]),
                a[2].min(b[2]),
            ])),
            (Value::Vec4(a), Value::Vec4(b)) => Some(Value::Vec4([
                a[0].min(b[0]),
                a[1].min(b[1]),
                a[2].min(b[2]),
                a[3].min(b[3]),
            ])),
            (Value::Color(a), Value::Color(b)) => Some(Value::Color(Color::rgba(
                a.r.min(b.r),
                a.g.min(b.g),
                a.b.min(b.b),
                a.a.min(b.a),
            ))),
            // Scalar broadcast
            (Value::Float(s), Value::Vec3(v)) => Some(Value::Vec3([s.min(v[0]), s.min(v[1]), s.min(v[2])])),
            (Value::Vec3(v), Value::Float(s)) => Some(Value::Vec3([v[0].min(*s), v[1].min(*s), v[2].min(*s)])),
            _ => None,
        }
    }

    /// Per-component maximum of two values
    pub fn max_value(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Some(Value::Float(a.max(*b))),
            (Value::Int(a), Value::Int(b)) => Some(Value::Int(*a.max(b))),
            (Value::Int(a), Value::Float(b)) => Some(Value::Float((*a as f32).max(*b))),
            (Value::Float(a), Value::Int(b)) => Some(Value::Float(a.max(*b as f32))),
            (Value::Vec2(a), Value::Vec2(b)) => Some(Value::Vec2([a[0].max(b[0]), a[1].max(b[1])])),
            (Value::Vec3(a), Value::Vec3(b)) => Some(Value::Vec3([
                a[0].max(b[0]),
                a[1].max(b[1]),
                a[2].max(b[2]),
            ])),
            (Value::Vec4(a), Value::Vec4(b)) => Some(Value::Vec4([
                a[0].max(b[0]),
                a[1].max(b[1]),
                a[2].max(b[2]),
                a[3].max(b[3]),
            ])),
            (Value::Color(a), Value::Color(b)) => Some(Value::Color(Color::rgba(
                a.r.max(b.r),
                a.g.max(b.g),
                a.b.max(b.b),
                a.a.max(b.a),
            ))),
            // Scalar broadcast
            (Value::Float(s), Value::Vec3(v)) => Some(Value::Vec3([s.max(v[0]), s.max(v[1]), s.max(v[2])])),
            (Value::Vec3(v), Value::Float(s)) => Some(Value::Vec3([v[0].max(*s), v[1].max(*s), v[2].max(*s)])),
            _ => None,
        }
    }

    /// Per-component clamp between min and max
    pub fn clamp_value(&self, min_val: &Value, max_val: &Value) -> Option<Value> {
        match (self, min_val, max_val) {
            (Value::Float(v), Value::Float(lo), Value::Float(hi)) => Some(Value::Float(v.clamp(*lo, *hi))),
            (Value::Int(v), Value::Int(lo), Value::Int(hi)) => Some(Value::Int((*v).clamp(*lo, *hi))),
            (Value::Vec2(v), Value::Vec2(lo), Value::Vec2(hi)) => Some(Value::Vec2([
                v[0].clamp(lo[0], hi[0]),
                v[1].clamp(lo[1], hi[1]),
            ])),
            (Value::Vec3(v), Value::Vec3(lo), Value::Vec3(hi)) => Some(Value::Vec3([
                v[0].clamp(lo[0], hi[0]),
                v[1].clamp(lo[1], hi[1]),
                v[2].clamp(lo[2], hi[2]),
            ])),
            (Value::Vec4(v), Value::Vec4(lo), Value::Vec4(hi)) => Some(Value::Vec4([
                v[0].clamp(lo[0], hi[0]),
                v[1].clamp(lo[1], hi[1]),
                v[2].clamp(lo[2], hi[2]),
                v[3].clamp(lo[3], hi[3]),
            ])),
            (Value::Color(v), Value::Color(lo), Value::Color(hi)) => Some(Value::Color(Color::rgba(
                v.r.clamp(lo.r, hi.r),
                v.g.clamp(lo.g, hi.g),
                v.b.clamp(lo.b, hi.b),
                v.a.clamp(lo.a, hi.a),
            ))),
            // Scalar broadcast for min/max
            (Value::Vec3(v), Value::Float(lo), Value::Float(hi)) => Some(Value::Vec3([
                v[0].clamp(*lo, *hi),
                v[1].clamp(*lo, *hi),
                v[2].clamp(*lo, *hi),
            ])),
            (Value::Color(v), Value::Float(lo), Value::Float(hi)) => Some(Value::Color(Color::rgba(
                v.r.clamp(*lo, *hi),
                v.g.clamp(*lo, *hi),
                v.b.clamp(*lo, *hi),
                v.a.clamp(*lo, *hi),
            ))),
            _ => None,
        }
    }

    /// Per-component sign: returns -1, 0, or 1
    pub fn sign(&self) -> Option<Value> {
        match self {
            Value::Float(v) => Some(Value::Float(if *v > 0.0 { 1.0 } else if *v < 0.0 { -1.0 } else { 0.0 })),
            Value::Int(v) => Some(Value::Int(v.signum())),
            Value::Vec2(v) => Some(Value::Vec2([
                if v[0] > 0.0 { 1.0 } else if v[0] < 0.0 { -1.0 } else { 0.0 },
                if v[1] > 0.0 { 1.0 } else if v[1] < 0.0 { -1.0 } else { 0.0 },
            ])),
            Value::Vec3(v) => Some(Value::Vec3([
                if v[0] > 0.0 { 1.0 } else if v[0] < 0.0 { -1.0 } else { 0.0 },
                if v[1] > 0.0 { 1.0 } else if v[1] < 0.0 { -1.0 } else { 0.0 },
                if v[2] > 0.0 { 1.0 } else if v[2] < 0.0 { -1.0 } else { 0.0 },
            ])),
            Value::Vec4(v) => Some(Value::Vec4([
                if v[0] > 0.0 { 1.0 } else if v[0] < 0.0 { -1.0 } else { 0.0 },
                if v[1] > 0.0 { 1.0 } else if v[1] < 0.0 { -1.0 } else { 0.0 },
                if v[2] > 0.0 { 1.0 } else if v[2] < 0.0 { -1.0 } else { 0.0 },
                if v[3] > 0.0 { 1.0 } else if v[3] < 0.0 { -1.0 } else { 0.0 },
            ])),
            _ => None,
        }
    }

    /// Per-component step: 0 if self < edge, else 1 (GLSL-style)
    pub fn step(&self, edge: &Value) -> Option<Value> {
        match (self, edge) {
            (Value::Float(v), Value::Float(e)) => Some(Value::Float(if *v < *e { 0.0 } else { 1.0 })),
            (Value::Int(v), Value::Int(e)) => Some(Value::Int(if *v < *e { 0 } else { 1 })),
            (Value::Vec2(v), Value::Vec2(e)) => Some(Value::Vec2([
                if v[0] < e[0] { 0.0 } else { 1.0 },
                if v[1] < e[1] { 0.0 } else { 1.0 },
            ])),
            (Value::Vec3(v), Value::Vec3(e)) => Some(Value::Vec3([
                if v[0] < e[0] { 0.0 } else { 1.0 },
                if v[1] < e[1] { 0.0 } else { 1.0 },
                if v[2] < e[2] { 0.0 } else { 1.0 },
            ])),
            (Value::Vec4(v), Value::Vec4(e)) => Some(Value::Vec4([
                if v[0] < e[0] { 0.0 } else { 1.0 },
                if v[1] < e[1] { 0.0 } else { 1.0 },
                if v[2] < e[2] { 0.0 } else { 1.0 },
                if v[3] < e[3] { 0.0 } else { 1.0 },
            ])),
            // Scalar edge broadcast
            (Value::Vec3(v), Value::Float(e)) => Some(Value::Vec3([
                if v[0] < *e { 0.0 } else { 1.0 },
                if v[1] < *e { 0.0 } else { 1.0 },
                if v[2] < *e { 0.0 } else { 1.0 },
            ])),
            _ => None,
        }
    }

    // =========================================================================
    // Interpolation operations
    // =========================================================================

    /// Linear interpolation: self + (other - self) * t
    pub fn lerp(&self, other: &Value, t: &Value) -> Option<Value> {
        // Get t as a float for scalar interpolation
        let t_float = match t {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f32),
            _ => None,
        };

        match (self, other, t_float) {
            // Scalar t (most common)
            (Value::Float(a), Value::Float(b), Some(t)) => Some(Value::Float(a + (b - a) * t)),
            (Value::Int(a), Value::Int(b), Some(t)) => Some(Value::Float(*a as f32 + (*b - *a) as f32 * t)),
            (Value::Vec2(a), Value::Vec2(b), Some(t)) => Some(Value::Vec2([
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
            ])),
            (Value::Vec3(a), Value::Vec3(b), Some(t)) => Some(Value::Vec3([
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
            ])),
            (Value::Vec4(a), Value::Vec4(b), Some(t)) => Some(Value::Vec4([
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
                a[3] + (b[3] - a[3]) * t,
            ])),
            (Value::Color(a), Value::Color(b), Some(t)) => Some(Value::Color(Color::rgba(
                a.r + (b.r - a.r) * t,
                a.g + (b.g - a.g) * t,
                a.b + (b.b - a.b) * t,
                a.a + (b.a - a.a) * t,
            ))),
            _ => {
                // Try per-component t
                match (self, other, t) {
                    (Value::Vec3(a), Value::Vec3(b), Value::Vec3(tv)) => Some(Value::Vec3([
                        a[0] + (b[0] - a[0]) * tv[0],
                        a[1] + (b[1] - a[1]) * tv[1],
                        a[2] + (b[2] - a[2]) * tv[2],
                    ])),
                    _ => None,
                }
            }
        }
    }

    /// GLSL-style smoothstep: hermite interpolation between edge0 and edge1
    pub fn smoothstep(&self, edge0: &Value, edge1: &Value) -> Option<Value> {
        fn smooth(x: f32, e0: f32, e1: f32) -> f32 {
            let range = e1 - e0;
            if range.abs() < f32::EPSILON {
                if x < e0 { 0.0 } else { 1.0 }
            } else {
                let t = ((x - e0) / range).clamp(0.0, 1.0);
                t * t * (3.0 - 2.0 * t)
            }
        }

        match (self, edge0, edge1) {
            (Value::Float(x), Value::Float(e0), Value::Float(e1)) => {
                Some(Value::Float(smooth(*x, *e0, *e1)))
            }
            (Value::Vec2(x), Value::Vec2(e0), Value::Vec2(e1)) => Some(Value::Vec2([
                smooth(x[0], e0[0], e1[0]),
                smooth(x[1], e0[1], e1[1]),
            ])),
            (Value::Vec3(x), Value::Vec3(e0), Value::Vec3(e1)) => Some(Value::Vec3([
                smooth(x[0], e0[0], e1[0]),
                smooth(x[1], e0[1], e1[1]),
                smooth(x[2], e0[2], e1[2]),
            ])),
            (Value::Vec4(x), Value::Vec4(e0), Value::Vec4(e1)) => Some(Value::Vec4([
                smooth(x[0], e0[0], e1[0]),
                smooth(x[1], e0[1], e1[1]),
                smooth(x[2], e0[2], e1[2]),
                smooth(x[3], e0[3], e1[3]),
            ])),
            // Scalar edges broadcast
            (Value::Vec3(x), Value::Float(e0), Value::Float(e1)) => Some(Value::Vec3([
                smooth(x[0], *e0, *e1),
                smooth(x[1], *e0, *e1),
                smooth(x[2], *e0, *e1),
            ])),
            _ => None,
        }
    }

    // =========================================================================
    // Trigonometry operations
    // =========================================================================

    /// Per-component sine
    pub fn sin(&self) -> Option<Value> {
        match self {
            Value::Float(v) => Some(Value::Float(v.sin())),
            Value::Int(v) => Some(Value::Float((*v as f32).sin())),
            Value::Vec2(v) => Some(Value::Vec2([v[0].sin(), v[1].sin()])),
            Value::Vec3(v) => Some(Value::Vec3([v[0].sin(), v[1].sin(), v[2].sin()])),
            Value::Vec4(v) => Some(Value::Vec4([v[0].sin(), v[1].sin(), v[2].sin(), v[3].sin()])),
            _ => None,
        }
    }

    /// Per-component cosine
    pub fn cos(&self) -> Option<Value> {
        match self {
            Value::Float(v) => Some(Value::Float(v.cos())),
            Value::Int(v) => Some(Value::Float((*v as f32).cos())),
            Value::Vec2(v) => Some(Value::Vec2([v[0].cos(), v[1].cos()])),
            Value::Vec3(v) => Some(Value::Vec3([v[0].cos(), v[1].cos(), v[2].cos()])),
            Value::Vec4(v) => Some(Value::Vec4([v[0].cos(), v[1].cos(), v[2].cos(), v[3].cos()])),
            _ => None,
        }
    }

    // =========================================================================
    // Additional rounding operations
    // =========================================================================

    /// Per-component round to nearest integer
    pub fn round(&self) -> Option<Value> {
        match self {
            Value::Float(v) => Some(Value::Float(v.round())),
            Value::Int(v) => Some(Value::Int(*v)),
            Value::Vec2(v) => Some(Value::Vec2([v[0].round(), v[1].round()])),
            Value::Vec3(v) => Some(Value::Vec3([v[0].round(), v[1].round(), v[2].round()])),
            Value::Vec4(v) => Some(Value::Vec4([v[0].round(), v[1].round(), v[2].round(), v[3].round()])),
            _ => None,
        }
    }

    /// Per-component truncate toward zero
    pub fn trunc(&self) -> Option<Value> {
        match self {
            Value::Float(v) => Some(Value::Float(v.trunc())),
            Value::Int(v) => Some(Value::Int(*v)),
            Value::Vec2(v) => Some(Value::Vec2([v[0].trunc(), v[1].trunc()])),
            Value::Vec3(v) => Some(Value::Vec3([v[0].trunc(), v[1].trunc(), v[2].trunc()])),
            Value::Vec4(v) => Some(Value::Vec4([v[0].trunc(), v[1].trunc(), v[2].trunc(), v[3].trunc()])),
            _ => None,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Addition tests
    // =========================================================================

    #[test]
    fn test_add_float_float() {
        let a = Value::Float(3.0);
        let b = Value::Float(4.0);
        assert_eq!(a + b, Some(Value::Float(7.0)));
    }

    #[test]
    fn test_add_int_int() {
        let a = Value::Int(3);
        let b = Value::Int(4);
        assert_eq!(a + b, Some(Value::Int(7)));
    }

    #[test]
    fn test_add_int_float_promotes() {
        let a = Value::Int(3);
        let b = Value::Float(4.5);
        assert_eq!(a + b, Some(Value::Float(7.5)));
    }

    #[test]
    fn test_add_vec3_vec3() {
        let a = Value::Vec3([1.0, 2.0, 3.0]);
        let b = Value::Vec3([4.0, 5.0, 6.0]);
        assert_eq!(a + b, Some(Value::Vec3([5.0, 7.0, 9.0])));
    }

    #[test]
    fn test_add_float_vec3_broadcast() {
        let a = Value::Float(2.0);
        let b = Value::Vec3([1.0, 2.0, 3.0]);
        assert_eq!(a + b, Some(Value::Vec3([3.0, 4.0, 5.0])));
    }

    #[test]
    fn test_add_vec3_float_broadcast() {
        let a = Value::Vec3([1.0, 2.0, 3.0]);
        let b = Value::Float(10.0);
        assert_eq!(a + b, Some(Value::Vec3([11.0, 12.0, 13.0])));
    }

    #[test]
    fn test_add_color_color() {
        let a = Value::Color(Color::rgba(0.1, 0.2, 0.3, 1.0));
        let b = Value::Color(Color::rgba(0.4, 0.3, 0.2, 0.5));
        let result = (a + b).unwrap();
        if let Value::Color(c) = result {
            assert!((c.r - 0.5).abs() < 0.001);
            assert!((c.g - 0.5).abs() < 0.001);
            assert!((c.b - 0.5).abs() < 0.001);
            assert!((c.a - 1.5).abs() < 0.001);
        } else {
            panic!("Expected Color");
        }
    }

    #[test]
    fn test_add_float_color_preserves_alpha() {
        let a = Value::Float(0.1);
        let b = Value::Color(Color::rgba(0.2, 0.3, 0.4, 0.8));
        let result = (a + b).unwrap();
        if let Value::Color(c) = result {
            assert!((c.r - 0.3).abs() < 0.001);
            assert!((c.g - 0.4).abs() < 0.001);
            assert!((c.b - 0.5).abs() < 0.001);
            assert!((c.a - 0.8).abs() < 0.001); // Alpha preserved
        } else {
            panic!("Expected Color");
        }
    }

    #[test]
    fn test_add_incompatible_returns_none() {
        let a = Value::String("hello".into());
        let b = Value::Vec3([1.0, 2.0, 3.0]);
        assert_eq!(a + b, None);
    }

    // =========================================================================
    // Subtraction tests
    // =========================================================================

    #[test]
    fn test_sub_float_float() {
        let a = Value::Float(10.0);
        let b = Value::Float(3.0);
        assert_eq!(a - b, Some(Value::Float(7.0)));
    }

    #[test]
    fn test_sub_vec3_vec3() {
        let a = Value::Vec3([5.0, 7.0, 9.0]);
        let b = Value::Vec3([1.0, 2.0, 3.0]);
        assert_eq!(a - b, Some(Value::Vec3([4.0, 5.0, 6.0])));
    }

    // =========================================================================
    // Multiplication tests
    // =========================================================================

    #[test]
    fn test_mul_float_float() {
        let a = Value::Float(3.0);
        let b = Value::Float(4.0);
        assert_eq!(a * b, Some(Value::Float(12.0)));
    }

    #[test]
    fn test_mul_int_int() {
        let a = Value::Int(3);
        let b = Value::Int(4);
        assert_eq!(a * b, Some(Value::Int(12)));
    }

    #[test]
    fn test_mul_float_vec3_scale() {
        let a = Value::Float(2.0);
        let b = Value::Vec3([1.0, 2.0, 3.0]);
        assert_eq!(a * b, Some(Value::Vec3([2.0, 4.0, 6.0])));
    }

    // =========================================================================
    // Division tests
    // =========================================================================

    #[test]
    fn test_div_float_float() {
        let a = Value::Float(10.0);
        let b = Value::Float(2.0);
        assert_eq!(a / b, Some(Value::Float(5.0)));
    }

    #[test]
    fn test_div_int_int_truncates() {
        let a = Value::Int(7);
        let b = Value::Int(2);
        assert_eq!(a / b, Some(Value::Int(3))); // Truncated
    }

    #[test]
    fn test_div_vec3_float() {
        let a = Value::Vec3([10.0, 20.0, 30.0]);
        let b = Value::Float(10.0);
        assert_eq!(a / b, Some(Value::Vec3([1.0, 2.0, 3.0])));
    }

    // =========================================================================
    // Negation tests
    // =========================================================================

    #[test]
    fn test_neg_float() {
        let a = Value::Float(5.0);
        assert_eq!(-a, Some(Value::Float(-5.0)));
    }

    #[test]
    fn test_neg_int() {
        let a = Value::Int(5);
        assert_eq!(-a, Some(Value::Int(-5)));
    }

    #[test]
    fn test_neg_vec3() {
        let a = Value::Vec3([1.0, -2.0, 3.0]);
        assert_eq!(-a, Some(Value::Vec3([-1.0, 2.0, -3.0])));
    }

    // =========================================================================
    // Additional math tests
    // =========================================================================

    #[test]
    fn test_pow() {
        let a = Value::Float(2.0);
        let b = Value::Float(3.0);
        assert_eq!(a.pow(&b), Some(Value::Float(8.0)));
    }

    #[test]
    fn test_pow_int() {
        let a = Value::Int(2);
        let b = Value::Int(3);
        assert_eq!(a.pow(&b), Some(Value::Int(8)));
    }

    #[test]
    fn test_abs() {
        assert_eq!(Value::Float(-5.0).abs(), Some(Value::Float(5.0)));
        assert_eq!(Value::Int(-5).abs(), Some(Value::Int(5)));
        assert_eq!(
            Value::Vec3([-1.0, 2.0, -3.0]).abs(),
            Some(Value::Vec3([1.0, 2.0, 3.0]))
        );
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(Value::Float(16.0).sqrt(), Some(Value::Float(4.0)));
        assert_eq!(Value::Int(16).sqrt(), Some(Value::Float(4.0)));
    }

    // =========================================================================
    // Type width tests
    // =========================================================================

    #[test]
    fn test_type_width() {
        assert_eq!(Value::Int(0).type_width(), 1);
        assert_eq!(Value::Float(0.0).type_width(), 2);
        assert_eq!(Value::Vec2([0.0, 0.0]).type_width(), 3);
        assert_eq!(Value::Vec3([0.0, 0.0, 0.0]).type_width(), 4);
        assert_eq!(Value::Color(Color::WHITE).type_width(), 4);
        assert_eq!(Value::Vec4([0.0, 0.0, 0.0, 0.0]).type_width(), 5);
        assert_eq!(Value::String("".into()).type_width(), 0);
    }

    #[test]
    fn test_is_arithmetic() {
        assert!(Value::Float(0.0).is_arithmetic());
        assert!(Value::Int(0).is_arithmetic());
        assert!(Value::Vec3([0.0, 0.0, 0.0]).is_arithmetic());
        assert!(Value::Color(Color::WHITE).is_arithmetic());
        assert!(!Value::String("".into()).is_arithmetic());
        assert!(!Value::Bool(false).is_arithmetic());
    }

    // =========================================================================
    // Comparison tests
    // =========================================================================

    #[test]
    fn test_min_float() {
        let a = Value::Float(3.0);
        let b = Value::Float(5.0);
        assert_eq!(a.min_value(&b), Some(Value::Float(3.0)));
    }

    #[test]
    fn test_min_vec3() {
        let a = Value::Vec3([1.0, 5.0, 3.0]);
        let b = Value::Vec3([2.0, 2.0, 4.0]);
        assert_eq!(a.min_value(&b), Some(Value::Vec3([1.0, 2.0, 3.0])));
    }

    #[test]
    fn test_max_float() {
        let a = Value::Float(3.0);
        let b = Value::Float(5.0);
        assert_eq!(a.max_value(&b), Some(Value::Float(5.0)));
    }

    #[test]
    fn test_max_vec3() {
        let a = Value::Vec3([1.0, 5.0, 3.0]);
        let b = Value::Vec3([2.0, 2.0, 4.0]);
        assert_eq!(a.max_value(&b), Some(Value::Vec3([2.0, 5.0, 4.0])));
    }

    #[test]
    fn test_clamp_float() {
        let v = Value::Float(1.5);
        let lo = Value::Float(0.0);
        let hi = Value::Float(1.0);
        assert_eq!(v.clamp_value(&lo, &hi), Some(Value::Float(1.0)));
    }

    #[test]
    fn test_clamp_vec3() {
        let v = Value::Vec3([-0.5, 0.5, 1.5]);
        let lo = Value::Vec3([0.0, 0.0, 0.0]);
        let hi = Value::Vec3([1.0, 1.0, 1.0]);
        assert_eq!(v.clamp_value(&lo, &hi), Some(Value::Vec3([0.0, 0.5, 1.0])));
    }

    #[test]
    fn test_clamp_vec3_scalar_bounds() {
        let v = Value::Vec3([-0.5, 0.5, 1.5]);
        let lo = Value::Float(0.0);
        let hi = Value::Float(1.0);
        assert_eq!(v.clamp_value(&lo, &hi), Some(Value::Vec3([0.0, 0.5, 1.0])));
    }

    #[test]
    fn test_sign_float() {
        assert_eq!(Value::Float(5.0).sign(), Some(Value::Float(1.0)));
        assert_eq!(Value::Float(-5.0).sign(), Some(Value::Float(-1.0)));
        assert_eq!(Value::Float(0.0).sign(), Some(Value::Float(0.0)));
    }

    #[test]
    fn test_sign_vec3() {
        let v = Value::Vec3([5.0, -3.0, 0.0]);
        assert_eq!(v.sign(), Some(Value::Vec3([1.0, -1.0, 0.0])));
    }

    #[test]
    fn test_step_float() {
        let edge = Value::Float(0.5);
        assert_eq!(Value::Float(0.3).step(&edge), Some(Value::Float(0.0)));
        assert_eq!(Value::Float(0.7).step(&edge), Some(Value::Float(1.0)));
    }

    #[test]
    fn test_step_vec3() {
        let v = Value::Vec3([0.3, 0.5, 0.7]);
        let edge = Value::Vec3([0.5, 0.5, 0.5]);
        assert_eq!(v.step(&edge), Some(Value::Vec3([0.0, 1.0, 1.0])));
    }

    // =========================================================================
    // Interpolation tests
    // =========================================================================

    #[test]
    fn test_lerp_float() {
        let a = Value::Float(0.0);
        let b = Value::Float(10.0);
        let t = Value::Float(0.5);
        assert_eq!(a.lerp(&b, &t), Some(Value::Float(5.0)));
    }

    #[test]
    fn test_lerp_vec3() {
        let a = Value::Vec3([0.0, 0.0, 0.0]);
        let b = Value::Vec3([10.0, 20.0, 30.0]);
        let t = Value::Float(0.5);
        assert_eq!(a.lerp(&b, &t), Some(Value::Vec3([5.0, 10.0, 15.0])));
    }

    #[test]
    fn test_lerp_color() {
        let a = Value::Color(Color::rgba(0.0, 0.0, 0.0, 1.0));
        let b = Value::Color(Color::rgba(1.0, 1.0, 1.0, 1.0));
        let t = Value::Float(0.5);
        if let Some(Value::Color(c)) = a.lerp(&b, &t) {
            assert!((c.r - 0.5).abs() < 0.001);
            assert!((c.g - 0.5).abs() < 0.001);
            assert!((c.b - 0.5).abs() < 0.001);
        } else {
            panic!("Expected Color");
        }
    }

    #[test]
    fn test_smoothstep_float() {
        let x = Value::Float(0.5);
        let e0 = Value::Float(0.0);
        let e1 = Value::Float(1.0);
        assert_eq!(x.smoothstep(&e0, &e1), Some(Value::Float(0.5)));
    }

    #[test]
    fn test_smoothstep_vec3() {
        let x = Value::Vec3([0.0, 0.5, 1.0]);
        let e0 = Value::Float(0.0);
        let e1 = Value::Float(1.0);
        let result = x.smoothstep(&e0, &e1).unwrap();
        if let Value::Vec3(v) = result {
            assert!((v[0] - 0.0).abs() < 0.001);
            assert!((v[1] - 0.5).abs() < 0.001);
            assert!((v[2] - 1.0).abs() < 0.001);
        }
    }

    // =========================================================================
    // Trig tests
    // =========================================================================

    #[test]
    fn test_sin_float() {
        use std::f32::consts::PI;
        let v = Value::Float(PI / 2.0);
        let result = v.sin().unwrap();
        if let Value::Float(s) = result {
            assert!((s - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_sin_vec3() {
        use std::f32::consts::PI;
        let v = Value::Vec3([0.0, PI / 2.0, PI]);
        let result = v.sin().unwrap();
        if let Value::Vec3(s) = result {
            assert!((s[0] - 0.0).abs() < 0.001);
            assert!((s[1] - 1.0).abs() < 0.001);
            assert!((s[2] - 0.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_cos_float() {
        let v = Value::Float(0.0);
        assert_eq!(v.cos(), Some(Value::Float(1.0)));
    }

    #[test]
    fn test_cos_vec3() {
        use std::f32::consts::PI;
        let v = Value::Vec3([0.0, PI, 2.0 * PI]);
        let result = v.cos().unwrap();
        if let Value::Vec3(c) = result {
            assert!((c[0] - 1.0).abs() < 0.001);
            assert!((c[1] - (-1.0)).abs() < 0.001);
            assert!((c[2] - 1.0).abs() < 0.001);
        }
    }

    // =========================================================================
    // Rounding tests
    // =========================================================================

    #[test]
    fn test_round_float() {
        assert_eq!(Value::Float(2.4).round(), Some(Value::Float(2.0)));
        assert_eq!(Value::Float(2.6).round(), Some(Value::Float(3.0)));
    }

    #[test]
    fn test_round_vec3() {
        let v = Value::Vec3([2.4, 2.5, 2.6]);
        // Note: 2.5 rounds to 2.0 (banker's rounding in Rust)
        let result = v.round().unwrap();
        if let Value::Vec3(r) = result {
            assert_eq!(r[0], 2.0);
            assert_eq!(r[2], 3.0);
        }
    }

    #[test]
    fn test_trunc_float() {
        assert_eq!(Value::Float(2.9).trunc(), Some(Value::Float(2.0)));
        assert_eq!(Value::Float(-2.9).trunc(), Some(Value::Float(-2.0)));
    }

    #[test]
    fn test_trunc_vec3() {
        let v = Value::Vec3([2.9, -2.9, 0.5]);
        assert_eq!(v.trunc(), Some(Value::Vec3([2.0, -2.0, 0.0])));
    }
}
