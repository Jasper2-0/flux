//! Value types for the Flux operator graph system
//!
//! This module contains the core value types used throughout the graph:
//! - [`Value`] - The main enum representing all possible values
//! - [`ValueType`] - Type identifiers for compile-time and runtime checks
//! - [`Color`] - RGBA color with HSV conversion
//! - [`Gradient`] - Color gradient with stops
//! - [`Matrix4`] - 4x4 transformation matrix

mod color;
mod gradient;
mod matrix;

pub use color::Color;
pub use gradient::{Gradient, GradientStop};
pub use matrix::Matrix4;

use serde::{Deserialize, Serialize};
use std::fmt;

/// All possible value types in the graph
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    // Primitives
    Float(f32),
    Int(i32),
    Bool(bool),

    // Vectors
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),

    // Text
    String(String),

    // Complex types
    Color(Color),
    Gradient(Gradient),
    Matrix4(Matrix4),

    // Collections
    FloatList(Vec<f32>),
    IntList(Vec<i32>),
    Vec3List(Vec<[f32; 3]>),
}

impl Value {
    /// Get the type of this value
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Float(_) => ValueType::Float,
            Value::Int(_) => ValueType::Int,
            Value::Bool(_) => ValueType::Bool,
            Value::Vec2(_) => ValueType::Vec2,
            Value::Vec3(_) => ValueType::Vec3,
            Value::Vec4(_) => ValueType::Vec4,
            Value::String(_) => ValueType::String,
            Value::Color(_) => ValueType::Color,
            Value::Gradient(_) => ValueType::Gradient,
            Value::Matrix4(_) => ValueType::Matrix4,
            Value::FloatList(_) => ValueType::FloatList,
            Value::IntList(_) => ValueType::IntList,
            Value::Vec3List(_) => ValueType::Vec3List,
        }
    }

    // ========== Primitive Accessors ==========

    /// Try to get as f32
    pub fn as_float(&self) -> Option<f32> {
        match self {
            Value::Float(v) => Some(*v),
            Value::Int(v) => Some(*v as f32),
            _ => None,
        }
    }

    /// Try to get as i32
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::Int(v) => Some(*v),
            Value::Float(v) => Some(*v as i32),
            _ => None,
        }
    }

    /// Try to get as bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            _ => None,
        }
    }

    // ========== Vector Accessors ==========

    /// Try to get as Vec2
    pub fn as_vec2(&self) -> Option<[f32; 2]> {
        match self {
            Value::Vec2(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to get as Vec3
    pub fn as_vec3(&self) -> Option<[f32; 3]> {
        match self {
            Value::Vec3(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to get as Vec4
    pub fn as_vec4(&self) -> Option<[f32; 4]> {
        match self {
            Value::Vec4(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to get as String
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }

    // ========== Complex Type Accessors ==========

    /// Try to get as Color
    pub fn as_color(&self) -> Option<Color> {
        match self {
            Value::Color(c) => Some(*c),
            Value::Vec4(v) => Some(Color::from_array(*v)),
            _ => None,
        }
    }

    /// Try to get as Gradient
    pub fn as_gradient(&self) -> Option<&Gradient> {
        match self {
            Value::Gradient(g) => Some(g),
            _ => None,
        }
    }

    /// Try to get as Matrix4
    pub fn as_matrix4(&self) -> Option<Matrix4> {
        match self {
            Value::Matrix4(m) => Some(*m),
            _ => None,
        }
    }

    // ========== List Accessors ==========

    /// Try to get as float list
    pub fn as_float_list(&self) -> Option<&[f32]> {
        match self {
            Value::FloatList(v) => Some(v),
            _ => None,
        }
    }

    /// Try to get as int list
    pub fn as_int_list(&self) -> Option<&[i32]> {
        match self {
            Value::IntList(v) => Some(v),
            _ => None,
        }
    }

    /// Try to get as vec3 list
    pub fn as_vec3_list(&self) -> Option<&[[f32; 3]]> {
        match self {
            Value::Vec3List(v) => Some(v),
            _ => None,
        }
    }

    // ========== Type Coercion ==========

    /// Attempt to coerce this value to the target type
    pub fn coerce_to(&self, target: ValueType) -> Option<Value> {
        // Identity - same type
        if self.value_type() == target {
            return Some(self.clone());
        }

        match (self, target) {
            // Numeric conversions
            (Value::Int(i), ValueType::Float) => Some(Value::Float(*i as f32)),
            (Value::Float(f), ValueType::Int) => Some(Value::Int(*f as i32)),
            (Value::Bool(b), ValueType::Int) => Some(Value::Int(if *b { 1 } else { 0 })),
            (Value::Bool(b), ValueType::Float) => Some(Value::Float(if *b { 1.0 } else { 0.0 })),
            (Value::Int(i), ValueType::Bool) => Some(Value::Bool(*i != 0)),
            (Value::Float(f), ValueType::Bool) => Some(Value::Bool(*f != 0.0)),

            // Vec4 <-> Color
            (Value::Vec4(v), ValueType::Color) => Some(Value::Color(Color::from_array(*v))),
            (Value::Color(c), ValueType::Vec4) => Some(Value::Vec4(c.to_array())),

            // Vec3 -> Vec4 (with w = 1.0)
            (Value::Vec3(v), ValueType::Vec4) => Some(Value::Vec4([v[0], v[1], v[2], 1.0])),
            // Vec3 -> Color (with a = 1.0)
            (Value::Vec3(v), ValueType::Color) => {
                Some(Value::Color(Color::rgba(v[0], v[1], v[2], 1.0)))
            }

            // Vec4 -> Vec3 (drop w)
            (Value::Vec4(v), ValueType::Vec3) => Some(Value::Vec3([v[0], v[1], v[2]])),
            // Color -> Vec3 (drop a)
            (Value::Color(c), ValueType::Vec3) => Some(Value::Vec3([c.r, c.g, c.b])),

            // Float -> Vec2/Vec3/Vec4 (broadcast)
            (Value::Float(f), ValueType::Vec2) => Some(Value::Vec2([*f, *f])),
            (Value::Float(f), ValueType::Vec3) => Some(Value::Vec3([*f, *f, *f])),
            (Value::Float(f), ValueType::Vec4) => Some(Value::Vec4([*f, *f, *f, *f])),
            (Value::Float(f), ValueType::Color) => Some(Value::Color(Color::rgba(*f, *f, *f, 1.0))),

            // String conversions
            (Value::Int(i), ValueType::String) => Some(Value::String(i.to_string())),
            (Value::Float(f), ValueType::String) => Some(Value::String(f.to_string())),
            (Value::Bool(b), ValueType::String) => Some(Value::String(b.to_string())),

            // No valid conversion
            _ => None,
        }
    }

    /// Check if this value can be coerced to the target type
    pub fn can_coerce_to(&self, target: ValueType) -> bool {
        self.value_type() == target || self.coerce_to(target).is_some()
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Float(0.0)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Float(v) => write!(f, "{}", v),
            Value::Int(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Vec2(v) => write!(f, "[{}, {}]", v[0], v[1]),
            Value::Vec3(v) => write!(f, "[{}, {}, {}]", v[0], v[1], v[2]),
            Value::Vec4(v) => write!(f, "[{}, {}, {}, {}]", v[0], v[1], v[2], v[3]),
            Value::String(v) => write!(f, "\"{}\"", v),
            Value::Color(c) => write!(f, "{}", c),
            Value::Gradient(g) => write!(f, "Gradient({} stops)", g.stops.len()),
            Value::Matrix4(_) => write!(f, "Matrix4"),
            Value::FloatList(v) => write!(f, "FloatList[{}]", v.len()),
            Value::IntList(v) => write!(f, "IntList[{}]", v.len()),
            Value::Vec3List(v) => write!(f, "Vec3List[{}]", v.len()),
        }
    }
}

// ========== From implementations ==========

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Float(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Int(v)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<[f32; 2]> for Value {
    fn from(v: [f32; 2]) -> Self {
        Value::Vec2(v)
    }
}

impl From<[f32; 3]> for Value {
    fn from(v: [f32; 3]) -> Self {
        Value::Vec3(v)
    }
}

impl From<[f32; 4]> for Value {
    fn from(v: [f32; 4]) -> Self {
        Value::Vec4(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
    }
}

impl From<Color> for Value {
    fn from(c: Color) -> Self {
        Value::Color(c)
    }
}

impl From<Gradient> for Value {
    fn from(g: Gradient) -> Self {
        Value::Gradient(g)
    }
}

impl From<Matrix4> for Value {
    fn from(m: Matrix4) -> Self {
        Value::Matrix4(m)
    }
}

/// Type identifier for compile-time and runtime type checking
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValueType {
    Float,
    Int,
    Bool,
    Vec2,
    Vec3,
    Vec4,
    String,
    Color,
    Gradient,
    Matrix4,
    FloatList,
    IntList,
    Vec3List,
}

/// Type categories for polymorphic inputs.
///
/// Type categories allow operators to accept multiple related types at an input.
/// For example, a math operator might accept any `Numeric` type (Float or Int),
/// or a vector operation might accept any `Vector` type (Vec2, Vec3, Vec4).
///
/// # Example
///
/// ```
/// use flux_core::value::{ValueType, TypeCategory};
///
/// // Check if Float is numeric
/// assert!(ValueType::Float.is_in_category(TypeCategory::Numeric));
/// assert!(ValueType::Int.is_in_category(TypeCategory::Numeric));
///
/// // Check vector types
/// assert!(ValueType::Vec3.is_in_category(TypeCategory::Vector));
/// assert!(!ValueType::Float.is_in_category(TypeCategory::Vector));
///
/// // Any matches everything
/// assert!(ValueType::String.is_in_category(TypeCategory::Any));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeCategory {
    /// Numeric types: Float, Int
    Numeric,
    /// Vector types: Vec2, Vec3, Vec4
    Vector,
    /// Color-compatible types: Color, Vec4, Vec3 (RGB)
    ColorLike,
    /// List types: FloatList, IntList, Vec3List
    List,
    /// Matrix types: Matrix4
    Matrix,
    /// Any type (accepts all)
    Any,
}

impl ValueType {
    /// Get a default value for this type
    pub fn default_value(&self) -> Value {
        match self {
            ValueType::Float => Value::Float(0.0),
            ValueType::Int => Value::Int(0),
            ValueType::Bool => Value::Bool(false),
            ValueType::Vec2 => Value::Vec2([0.0, 0.0]),
            ValueType::Vec3 => Value::Vec3([0.0, 0.0, 0.0]),
            ValueType::Vec4 => Value::Vec4([0.0, 0.0, 0.0, 0.0]),
            ValueType::String => Value::String(String::new()),
            ValueType::Color => Value::Color(Color::WHITE),
            ValueType::Gradient => Value::Gradient(Gradient::new()),
            ValueType::Matrix4 => Value::Matrix4(Matrix4::IDENTITY),
            ValueType::FloatList => Value::FloatList(Vec::new()),
            ValueType::IntList => Value::IntList(Vec::new()),
            ValueType::Vec3List => Value::Vec3List(Vec::new()),
        }
    }

    /// Check if this type can be coerced to the target type
    pub fn can_coerce_to(&self, target: ValueType) -> bool {
        if *self == target {
            return true;
        }

        matches!(
            (*self, target),
            // Numeric
            (ValueType::Int, ValueType::Float)
                | (ValueType::Float, ValueType::Int)
                | (ValueType::Bool, ValueType::Int)
                | (ValueType::Bool, ValueType::Float)
                | (ValueType::Int, ValueType::Bool)
                | (ValueType::Float, ValueType::Bool)
                // Vec/Color conversions
                | (ValueType::Vec4, ValueType::Color)
                | (ValueType::Color, ValueType::Vec4)
                | (ValueType::Vec3, ValueType::Vec4)
                | (ValueType::Vec3, ValueType::Color)
                | (ValueType::Vec4, ValueType::Vec3)
                | (ValueType::Color, ValueType::Vec3)
                // Float broadcast
                | (ValueType::Float, ValueType::Vec2)
                | (ValueType::Float, ValueType::Vec3)
                | (ValueType::Float, ValueType::Vec4)
                | (ValueType::Float, ValueType::Color)
                // To string
                | (ValueType::Int, ValueType::String)
                | (ValueType::Float, ValueType::String)
                | (ValueType::Bool, ValueType::String)
        )
    }

    /// Check if this type belongs to a category.
    ///
    /// Type categories enable polymorphic inputs that can accept multiple
    /// related types. For example, a math operator might accept any `Numeric`
    /// type (Float or Int).
    ///
    /// # Example
    ///
    /// ```
    /// use flux_core::value::{ValueType, TypeCategory};
    ///
    /// assert!(ValueType::Float.is_in_category(TypeCategory::Numeric));
    /// assert!(ValueType::Vec3.is_in_category(TypeCategory::Vector));
    /// assert!(ValueType::Color.is_in_category(TypeCategory::ColorLike));
    /// ```
    pub fn is_in_category(&self, category: TypeCategory) -> bool {
        match category {
            TypeCategory::Numeric => matches!(self, Self::Float | Self::Int),
            TypeCategory::Vector => matches!(self, Self::Vec2 | Self::Vec3 | Self::Vec4),
            TypeCategory::ColorLike => matches!(self, Self::Color | Self::Vec4 | Self::Vec3),
            TypeCategory::List => matches!(self, Self::FloatList | Self::IntList | Self::Vec3List),
            TypeCategory::Matrix => matches!(self, Self::Matrix4),
            TypeCategory::Any => true,
        }
    }

    /// Get all categories this type belongs to.
    ///
    /// Returns a list of all categories that would return `true` for
    /// `is_in_category()` (excluding `Any` which always matches).
    ///
    /// # Example
    ///
    /// ```
    /// use flux_core::value::{ValueType, TypeCategory};
    ///
    /// let categories = ValueType::Vec4.categories();
    /// assert!(categories.contains(&TypeCategory::Vector));
    /// assert!(categories.contains(&TypeCategory::ColorLike));
    /// ```
    pub fn categories(&self) -> Vec<TypeCategory> {
        let mut cats = Vec::new();

        if self.is_in_category(TypeCategory::Numeric) {
            cats.push(TypeCategory::Numeric);
        }
        if self.is_in_category(TypeCategory::Vector) {
            cats.push(TypeCategory::Vector);
        }
        if self.is_in_category(TypeCategory::ColorLike) {
            cats.push(TypeCategory::ColorLike);
        }
        if self.is_in_category(TypeCategory::List) {
            cats.push(TypeCategory::List);
        }
        if self.is_in_category(TypeCategory::Matrix) {
            cats.push(TypeCategory::Matrix);
        }

        cats
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Float => write!(f, "Float"),
            ValueType::Int => write!(f, "Int"),
            ValueType::Bool => write!(f, "Bool"),
            ValueType::Vec2 => write!(f, "Vec2"),
            ValueType::Vec3 => write!(f, "Vec3"),
            ValueType::Vec4 => write!(f, "Vec4"),
            ValueType::String => write!(f, "String"),
            ValueType::Color => write!(f, "Color"),
            ValueType::Gradient => write!(f, "Gradient"),
            ValueType::Matrix4 => write!(f, "Matrix4"),
            ValueType::FloatList => write!(f, "FloatList"),
            ValueType::IntList => write!(f, "IntList"),
            ValueType::Vec3List => write!(f, "Vec3List"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coerce_int_to_float() {
        let v = Value::Int(42);
        let result = v.coerce_to(ValueType::Float);
        assert_eq!(result, Some(Value::Float(42.0)));
    }

    #[test]
    fn test_coerce_float_to_vec3() {
        let v = Value::Float(1.5);
        let result = v.coerce_to(ValueType::Vec3);
        assert_eq!(result, Some(Value::Vec3([1.5, 1.5, 1.5])));
    }

    #[test]
    fn test_coerce_vec4_to_color() {
        let v = Value::Vec4([1.0, 0.5, 0.25, 0.8]);
        let result = v.coerce_to(ValueType::Color);

        if let Some(Value::Color(c)) = result {
            assert_eq!(c.r, 1.0);
            assert_eq!(c.g, 0.5);
            assert_eq!(c.b, 0.25);
            assert_eq!(c.a, 0.8);
        } else {
            panic!("Expected Color");
        }
    }

    #[test]
    fn test_coerce_color_to_vec4() {
        let v = Value::Color(Color::rgba(1.0, 0.5, 0.25, 0.8));
        let result = v.coerce_to(ValueType::Vec4);
        assert_eq!(result, Some(Value::Vec4([1.0, 0.5, 0.25, 0.8])));
    }

    #[test]
    fn test_coerce_incompatible() {
        let v = Value::String("test".into());
        assert!(v.coerce_to(ValueType::Vec3).is_none());
    }

    #[test]
    fn test_can_coerce_to() {
        assert!(Value::Float(1.0).can_coerce_to(ValueType::Vec3));
        assert!(Value::Vec4([0.0; 4]).can_coerce_to(ValueType::Color));
        assert!(!Value::String("x".into()).can_coerce_to(ValueType::Int));
    }

    #[test]
    fn test_value_type_can_coerce() {
        assert!(ValueType::Float.can_coerce_to(ValueType::Vec3));
        assert!(ValueType::Int.can_coerce_to(ValueType::Float));
        assert!(!ValueType::Gradient.can_coerce_to(ValueType::Float));
    }

    // =========================================================================
    // TypeCategory Tests
    // =========================================================================

    #[test]
    fn test_numeric_category() {
        // Float and Int are numeric
        assert!(ValueType::Float.is_in_category(TypeCategory::Numeric));
        assert!(ValueType::Int.is_in_category(TypeCategory::Numeric));

        // Other types are not numeric
        assert!(!ValueType::Bool.is_in_category(TypeCategory::Numeric));
        assert!(!ValueType::Vec3.is_in_category(TypeCategory::Numeric));
        assert!(!ValueType::String.is_in_category(TypeCategory::Numeric));
    }

    #[test]
    fn test_vector_category() {
        // Vec2, Vec3, Vec4 are vectors
        assert!(ValueType::Vec2.is_in_category(TypeCategory::Vector));
        assert!(ValueType::Vec3.is_in_category(TypeCategory::Vector));
        assert!(ValueType::Vec4.is_in_category(TypeCategory::Vector));

        // Other types are not vectors
        assert!(!ValueType::Float.is_in_category(TypeCategory::Vector));
        assert!(!ValueType::Color.is_in_category(TypeCategory::Vector));
    }

    #[test]
    fn test_color_like_category() {
        // Color, Vec4, Vec3 are color-like (can represent colors)
        assert!(ValueType::Color.is_in_category(TypeCategory::ColorLike));
        assert!(ValueType::Vec4.is_in_category(TypeCategory::ColorLike));
        assert!(ValueType::Vec3.is_in_category(TypeCategory::ColorLike));

        // Other types are not color-like
        assert!(!ValueType::Vec2.is_in_category(TypeCategory::ColorLike));
        assert!(!ValueType::Float.is_in_category(TypeCategory::ColorLike));
    }

    #[test]
    fn test_list_category() {
        assert!(ValueType::FloatList.is_in_category(TypeCategory::List));
        assert!(ValueType::IntList.is_in_category(TypeCategory::List));
        assert!(ValueType::Vec3List.is_in_category(TypeCategory::List));

        assert!(!ValueType::Float.is_in_category(TypeCategory::List));
    }

    #[test]
    fn test_matrix_category() {
        assert!(ValueType::Matrix4.is_in_category(TypeCategory::Matrix));
        assert!(!ValueType::Vec4.is_in_category(TypeCategory::Matrix));
    }

    #[test]
    fn test_any_category() {
        // Any matches everything
        assert!(ValueType::Float.is_in_category(TypeCategory::Any));
        assert!(ValueType::String.is_in_category(TypeCategory::Any));
        assert!(ValueType::Gradient.is_in_category(TypeCategory::Any));
    }

    #[test]
    fn test_categories_method() {
        // Float is only numeric
        let float_cats = ValueType::Float.categories();
        assert_eq!(float_cats.len(), 1);
        assert!(float_cats.contains(&TypeCategory::Numeric));

        // Vec4 is both vector and color-like
        let vec4_cats = ValueType::Vec4.categories();
        assert_eq!(vec4_cats.len(), 2);
        assert!(vec4_cats.contains(&TypeCategory::Vector));
        assert!(vec4_cats.contains(&TypeCategory::ColorLike));

        // Vec3 is also both vector and color-like
        let vec3_cats = ValueType::Vec3.categories();
        assert_eq!(vec3_cats.len(), 2);
        assert!(vec3_cats.contains(&TypeCategory::Vector));
        assert!(vec3_cats.contains(&TypeCategory::ColorLike));

        // Color is only color-like
        let color_cats = ValueType::Color.categories();
        assert_eq!(color_cats.len(), 1);
        assert!(color_cats.contains(&TypeCategory::ColorLike));

        // String has no categories (besides Any which we don't include)
        let string_cats = ValueType::String.categories();
        assert!(string_cats.is_empty());
    }
}
