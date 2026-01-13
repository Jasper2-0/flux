//! Type constraint system for polymorphic ports
//!
//! This module provides:
//! - [`TypeConstraint`] - Defines what types an input port accepts
//! - [`OutputTypeRule`] - Defines how an output port's type is determined

use crate::value::{TypeCategory, ValueType};

/// Defines what types an input port will accept
///
/// TypeConstraints enable polymorphic operators by allowing ports to accept
/// multiple types rather than a single fixed type.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeConstraint {
    /// Accept only this exact type
    Exact(ValueType),

    /// Accept any type in this category (e.g., Arithmetic, Numeric, Vector)
    Category(TypeCategory),

    /// Accept any of these specific types
    OneOf(Vec<ValueType>),

    /// Must match the type of another input (by index)
    /// Used for binary operators where both inputs should be same type
    SameAsInput(usize),

    /// Accept any type
    Any,
}

impl TypeConstraint {
    /// Check if a value type satisfies this constraint
    pub fn accepts(&self, value_type: ValueType) -> bool {
        match self {
            TypeConstraint::Exact(expected) => value_type == *expected,
            TypeConstraint::Category(category) => value_type.is_in_category(*category),
            TypeConstraint::OneOf(types) => types.contains(&value_type),
            TypeConstraint::SameAsInput(_) => {
                // This requires context to evaluate properly
                // Return true here; actual validation happens at connection time
                true
            }
            TypeConstraint::Any => true,
        }
    }

    /// Check if constraint accepts the type, with additional context for SameAsInput
    pub fn accepts_with_context(
        &self,
        value_type: ValueType,
        other_input_types: &[Option<ValueType>],
    ) -> bool {
        match self {
            TypeConstraint::SameAsInput(idx) => {
                if let Some(Some(other_type)) = other_input_types.get(*idx) {
                    // Must match the other input's type
                    value_type == *other_type
                } else {
                    // Other input not connected, accept anything
                    true
                }
            }
            _ => self.accepts(value_type),
        }
    }

    /// Get the default type for this constraint (used when no connection)
    pub fn default_type(&self) -> ValueType {
        match self {
            TypeConstraint::Exact(t) => *t,
            TypeConstraint::Category(cat) => match cat {
                TypeCategory::Numeric => ValueType::Float,
                TypeCategory::Vector => ValueType::Vec3,
                TypeCategory::ColorLike => ValueType::Color,
                TypeCategory::List => ValueType::FloatList,
                TypeCategory::Matrix => ValueType::Matrix4,
                TypeCategory::Arithmetic => ValueType::Float,
                TypeCategory::Any => ValueType::Float,
            },
            TypeConstraint::OneOf(types) => types.first().copied().unwrap_or(ValueType::Float),
            TypeConstraint::SameAsInput(_) => ValueType::Float, // Resolved at runtime
            TypeConstraint::Any => ValueType::Float,
        }
    }

    // Convenience constructors

    /// Create a constraint for exact type match
    pub fn exact(value_type: ValueType) -> Self {
        TypeConstraint::Exact(value_type)
    }

    /// Create a constraint for arithmetic types (Float, Int, Vec2, Vec3, Vec4, Color)
    pub fn arithmetic() -> Self {
        TypeConstraint::Category(TypeCategory::Arithmetic)
    }

    /// Create a constraint for numeric types (Float, Int)
    pub fn numeric() -> Self {
        TypeConstraint::Category(TypeCategory::Numeric)
    }

    /// Create a constraint for vector types (Vec2, Vec3, Vec4)
    pub fn vector() -> Self {
        TypeConstraint::Category(TypeCategory::Vector)
    }

    /// Create a constraint for color-like types (Vec3, Vec4, Color)
    pub fn color_like() -> Self {
        TypeConstraint::Category(TypeCategory::ColorLike)
    }

    /// Create a constraint that matches another input's type
    pub fn same_as(input_index: usize) -> Self {
        TypeConstraint::SameAsInput(input_index)
    }

    /// Create a constraint that accepts any type
    pub fn any() -> Self {
        TypeConstraint::Any
    }
}

impl Default for TypeConstraint {
    fn default() -> Self {
        TypeConstraint::Any
    }
}

/// Defines how an output port's type is determined
///
/// For polymorphic operators, the output type often depends on the input types.
#[derive(Clone, Debug, PartialEq)]
pub enum OutputTypeRule {
    /// Output is always this fixed type
    Fixed(ValueType),

    /// Output type matches the specified input (by index)
    SameAsInput(usize),

    /// Output type is the "wider" of multiple inputs (for broadcasting)
    /// Vec3 is wider than Float, Float is wider than Int
    Wider(Vec<usize>),

    /// Custom rule (type resolved dynamically)
    /// Used when output type depends on complex logic
    Dynamic,
}

impl OutputTypeRule {
    /// Resolve the output type given the connected input types
    pub fn resolve(&self, input_types: &[Option<ValueType>]) -> ValueType {
        match self {
            OutputTypeRule::Fixed(t) => *t,

            OutputTypeRule::SameAsInput(idx) => input_types
                .get(*idx)
                .and_then(|t| *t)
                .unwrap_or(ValueType::Float),

            OutputTypeRule::Wider(indices) => {
                let types: Vec<ValueType> = indices
                    .iter()
                    .filter_map(|idx| input_types.get(*idx).and_then(|t| *t))
                    .collect();

                if types.is_empty() {
                    ValueType::Float
                } else {
                    Self::find_wider_type(&types)
                }
            }

            OutputTypeRule::Dynamic => ValueType::Float, // Must be resolved elsewhere
        }
    }

    /// Find the "wider" type among a set of types (for broadcasting)
    ///
    /// Width hierarchy (highest to lowest):
    /// - Vec4 / Color (4 components)
    /// - Vec3 (3 components)
    /// - Vec2 (2 components)
    /// - Float (1 component, floating point)
    /// - Int (1 component, integer)
    fn find_wider_type(types: &[ValueType]) -> ValueType {
        let mut widest = types[0];
        let mut widest_width = Self::type_width(widest);

        for &t in &types[1..] {
            let w = Self::type_width(t);
            if w > widest_width {
                widest = t;
                widest_width = w;
            }
        }

        widest
    }

    /// Get the "width" of a type for comparison
    /// Higher width = wider type
    fn type_width(value_type: ValueType) -> u8 {
        match value_type {
            ValueType::Int => 1,
            ValueType::Float => 2,
            ValueType::Vec2 => 3,
            ValueType::Vec3 => 4,
            ValueType::Vec4 | ValueType::Color => 5,
            // Non-arithmetic types default to Float width
            _ => 2,
        }
    }

    // Convenience constructors

    /// Create a rule for fixed output type
    pub fn fixed(value_type: ValueType) -> Self {
        OutputTypeRule::Fixed(value_type)
    }

    /// Create a rule matching the first input
    pub fn same_as_first() -> Self {
        OutputTypeRule::SameAsInput(0)
    }

    /// Create a rule matching a specific input
    pub fn same_as(input_index: usize) -> Self {
        OutputTypeRule::SameAsInput(input_index)
    }

    /// Create a rule that uses the wider of the first two inputs
    pub fn wider_of_first_two() -> Self {
        OutputTypeRule::Wider(vec![0, 1])
    }

    /// Create a rule for dynamic type resolution
    pub fn dynamic() -> Self {
        OutputTypeRule::Dynamic
    }
}

impl Default for OutputTypeRule {
    fn default() -> Self {
        OutputTypeRule::Dynamic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_constraint() {
        let constraint = TypeConstraint::exact(ValueType::Float);
        assert!(constraint.accepts(ValueType::Float));
        assert!(!constraint.accepts(ValueType::Int));
        assert!(!constraint.accepts(ValueType::Vec3));
    }

    #[test]
    fn test_arithmetic_constraint() {
        let constraint = TypeConstraint::arithmetic();
        assert!(constraint.accepts(ValueType::Float));
        assert!(constraint.accepts(ValueType::Int));
        assert!(constraint.accepts(ValueType::Vec2));
        assert!(constraint.accepts(ValueType::Vec3));
        assert!(constraint.accepts(ValueType::Vec4));
        assert!(constraint.accepts(ValueType::Color));
        assert!(!constraint.accepts(ValueType::String));
        assert!(!constraint.accepts(ValueType::Bool));
    }

    #[test]
    fn test_numeric_constraint() {
        let constraint = TypeConstraint::numeric();
        assert!(constraint.accepts(ValueType::Float));
        assert!(constraint.accepts(ValueType::Int));
        assert!(!constraint.accepts(ValueType::Vec3));
    }

    #[test]
    fn test_one_of_constraint() {
        let constraint = TypeConstraint::OneOf(vec![ValueType::Float, ValueType::Vec3]);
        assert!(constraint.accepts(ValueType::Float));
        assert!(constraint.accepts(ValueType::Vec3));
        assert!(!constraint.accepts(ValueType::Int));
        assert!(!constraint.accepts(ValueType::Vec2));
    }

    #[test]
    fn test_same_as_input_with_context() {
        let constraint = TypeConstraint::same_as(0);

        // When input 0 is Float, only accept Float
        let context = vec![Some(ValueType::Float)];
        assert!(constraint.accepts_with_context(ValueType::Float, &context));
        assert!(!constraint.accepts_with_context(ValueType::Vec3, &context));

        // When input 0 is Vec3, only accept Vec3
        let context = vec![Some(ValueType::Vec3)];
        assert!(constraint.accepts_with_context(ValueType::Vec3, &context));
        assert!(!constraint.accepts_with_context(ValueType::Float, &context));

        // When input 0 not connected, accept anything
        let context = vec![None];
        assert!(constraint.accepts_with_context(ValueType::Float, &context));
        assert!(constraint.accepts_with_context(ValueType::Vec3, &context));
    }

    #[test]
    fn test_output_type_rule_fixed() {
        let rule = OutputTypeRule::fixed(ValueType::Float);
        assert_eq!(rule.resolve(&[]), ValueType::Float);
        assert_eq!(
            rule.resolve(&[Some(ValueType::Vec3)]),
            ValueType::Float
        );
    }

    #[test]
    fn test_output_type_rule_same_as_input() {
        let rule = OutputTypeRule::same_as_first();

        assert_eq!(
            rule.resolve(&[Some(ValueType::Vec3)]),
            ValueType::Vec3
        );
        assert_eq!(
            rule.resolve(&[Some(ValueType::Float)]),
            ValueType::Float
        );
        // No input connected
        assert_eq!(rule.resolve(&[None]), ValueType::Float);
    }

    #[test]
    fn test_output_type_rule_wider() {
        let rule = OutputTypeRule::wider_of_first_two();

        // Float + Vec3 = Vec3 (Vec3 is wider)
        assert_eq!(
            rule.resolve(&[Some(ValueType::Float), Some(ValueType::Vec3)]),
            ValueType::Vec3
        );

        // Int + Float = Float (Float is wider)
        assert_eq!(
            rule.resolve(&[Some(ValueType::Int), Some(ValueType::Float)]),
            ValueType::Float
        );

        // Vec2 + Vec4 = Vec4 (Vec4 is wider)
        assert_eq!(
            rule.resolve(&[Some(ValueType::Vec2), Some(ValueType::Vec4)]),
            ValueType::Vec4
        );

        // Same types
        assert_eq!(
            rule.resolve(&[Some(ValueType::Vec3), Some(ValueType::Vec3)]),
            ValueType::Vec3
        );

        // One input missing
        assert_eq!(
            rule.resolve(&[Some(ValueType::Vec3), None]),
            ValueType::Vec3
        );

        // Both missing
        assert_eq!(rule.resolve(&[None, None]), ValueType::Float);
    }

    #[test]
    fn test_constraint_default_types() {
        assert_eq!(
            TypeConstraint::arithmetic().default_type(),
            ValueType::Float
        );
        assert_eq!(
            TypeConstraint::numeric().default_type(),
            ValueType::Float
        );
        assert_eq!(TypeConstraint::vector().default_type(), ValueType::Vec3);
        assert_eq!(
            TypeConstraint::color_like().default_type(),
            ValueType::Color
        );
    }
}
