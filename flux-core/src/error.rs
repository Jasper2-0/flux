//! Error types for the Flux operator system
//!
//! This module provides a comprehensive error type that covers all the ways
//! operator evaluation and graph operations can fail.

use thiserror::Error;

use crate::id::Id;
use crate::value::ValueType;

/// Main error type for operator operations
#[derive(Error, Debug, Clone)]
pub enum OperatorError {
    // === Input/Output Errors ===
    /// Input slot not found
    #[error("Input '{name}' not found on operator {operator_id}")]
    InputNotFound { operator_id: Id, name: String },

    /// Output slot not found
    #[error("Output '{name}' not found on operator {operator_id}")]
    OutputNotFound { operator_id: Id, name: String },

    /// Input index out of bounds
    #[error("Input index {index} out of bounds (operator has {count} inputs)")]
    InputIndexOutOfBounds { index: usize, count: usize },

    /// Output index out of bounds
    #[error("Output index {index} out of bounds (operator has {count} outputs)")]
    OutputIndexOutOfBounds { index: usize, count: usize },

    // === Type Errors ===
    /// Type mismatch between slots
    #[error("Type mismatch: expected {expected:?}, got {actual:?}")]
    TypeMismatch { expected: ValueType, actual: ValueType },

    /// Cannot coerce between types
    #[error("Cannot coerce from {from:?} to {to:?}")]
    CoercionFailed { from: ValueType, to: ValueType },

    /// Invalid value for the context
    #[error("Invalid value: {message}")]
    InvalidValue { message: String },

    // === Connection Errors ===
    /// Attempting to create a cycle in the graph
    #[error("Connection would create a cycle in the graph")]
    CycleDetected,

    /// Connection already exists
    #[error("Connection already exists from {source_id} to {target_id}")]
    ConnectionExists { source_id: Id, target_id: Id },

    /// Connection not found
    #[error("No connection found to input {input_index} on operator {operator_id}")]
    ConnectionNotFound { operator_id: Id, input_index: usize },

    /// Invalid connection (incompatible slots)
    #[error("Invalid connection: {reason}")]
    InvalidConnection { reason: String },

    // === Operator Errors ===
    /// Operator not found in graph/registry
    #[error("Operator {0} not found")]
    OperatorNotFound(Id),

    /// Operator is not initialized
    #[error("Operator {0} is not initialized")]
    NotInitialized(Id),

    /// Operator evaluation failed
    #[error("Evaluation failed for operator {operator_id}: {reason}")]
    EvaluationFailed { operator_id: Id, reason: String },

    /// Operator is bypassed but bypass not supported
    #[error("Operator {0} does not support bypass")]
    BypassNotSupported(Id),

    // === Symbol/Instance Errors ===
    /// Symbol not found in registry
    #[error("Symbol {0} not found")]
    SymbolNotFound(Id),

    /// Child not found in symbol
    #[error("Child {child_id} not found in symbol {symbol_id}")]
    ChildNotFound { symbol_id: Id, child_id: Id },

    /// Instance path invalid
    #[error("Invalid instance path: {path}")]
    InvalidInstancePath { path: String },

    // === Resource Errors ===
    /// Resource not found
    #[error("Resource not found: {path}")]
    ResourceNotFound { path: String },

    /// Resource load failed
    #[error("Failed to load resource '{path}': {reason}")]
    ResourceLoadFailed { path: String, reason: String },

    // === Serialization Errors ===
    /// JSON parsing failed
    #[error("JSON parse error: {message}")]
    JsonParseError { message: String },

    /// Invalid format
    #[error("Invalid format: {message}")]
    InvalidFormat { message: String },

    // === Generic Errors ===
    /// Internal error (should not happen in normal operation)
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

/// Convenience type alias for operator results
pub type OperatorResult<T> = Result<T, OperatorError>;

/// Result of evaluating an operator
#[derive(Clone, Debug, Default)]
pub struct EvalResult {
    /// Whether the operator was evaluated (false if skipped/cached)
    pub evaluated: bool,
    /// Whether the operator is currently bypassed
    pub bypassed: bool,
    /// Whether any outputs were updated
    pub outputs_changed: bool,
    /// Non-fatal warnings during evaluation
    pub warnings: Vec<String>,
    /// Time taken to evaluate (in microseconds)
    pub eval_time_us: u64,
}

impl EvalResult {
    /// Create a result indicating the operator was evaluated
    pub fn evaluated() -> Self {
        Self {
            evaluated: true,
            ..Default::default()
        }
    }

    /// Create a result indicating the operator was skipped (cached)
    pub fn skipped() -> Self {
        Self::default()
    }

    /// Create a result indicating the operator was bypassed
    pub fn bypassed() -> Self {
        Self {
            bypassed: true,
            ..Default::default()
        }
    }

    /// Add a warning to the result
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Mark that outputs changed
    pub fn with_outputs_changed(mut self) -> Self {
        self.outputs_changed = true;
        self
    }

    /// Set evaluation time
    pub fn with_time(mut self, time_us: u64) -> Self {
        self.eval_time_us = time_us;
        self
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl OperatorError {
    /// Create an input not found error
    pub fn input_not_found(operator_id: Id, name: impl Into<String>) -> Self {
        Self::InputNotFound {
            operator_id,
            name: name.into(),
        }
    }

    /// Create an output not found error
    pub fn output_not_found(operator_id: Id, name: impl Into<String>) -> Self {
        Self::OutputNotFound {
            operator_id,
            name: name.into(),
        }
    }

    /// Create a type mismatch error
    pub fn type_mismatch(expected: ValueType, actual: ValueType) -> Self {
        Self::TypeMismatch { expected, actual }
    }

    /// Create a coercion failed error
    pub fn coercion_failed(from: ValueType, to: ValueType) -> Self {
        Self::CoercionFailed { from, to }
    }

    /// Create an evaluation failed error
    pub fn evaluation_failed(operator_id: Id, reason: impl Into<String>) -> Self {
        Self::EvaluationFailed {
            operator_id,
            reason: reason.into(),
        }
    }

    /// Create a resource not found error
    pub fn resource_not_found(path: impl Into<String>) -> Self {
        Self::ResourceNotFound { path: path.into() }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = OperatorError::type_mismatch(ValueType::Float, ValueType::Int);
        assert!(err.to_string().contains("Type mismatch"));
        assert!(err.to_string().contains("Float"));
        assert!(err.to_string().contains("Int"));
    }

    #[test]
    fn test_error_input_not_found() {
        let id = Id::new();
        let err = OperatorError::input_not_found(id, "value");
        assert!(err.to_string().contains("Input 'value' not found"));
    }

    #[test]
    fn test_error_evaluation_failed() {
        let id = Id::new();
        let err = OperatorError::evaluation_failed(id, "division by zero");
        assert!(err.to_string().contains("division by zero"));
    }

    #[test]
    fn test_operator_result() {
        fn might_fail(succeed: bool) -> OperatorResult<i32> {
            if succeed {
                Ok(42)
            } else {
                Err(OperatorError::internal("intentional failure"))
            }
        }

        assert_eq!(might_fail(true).unwrap(), 42);
        assert!(might_fail(false).is_err());
    }

    #[test]
    fn test_error_clone() {
        let err = OperatorError::CycleDetected;
        let cloned = err.clone();
        assert!(matches!(cloned, OperatorError::CycleDetected));
    }

    #[test]
    fn test_eval_result_default() {
        let result = EvalResult::default();
        assert!(!result.evaluated);
        assert!(!result.bypassed);
        assert!(!result.outputs_changed);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_eval_result_evaluated() {
        let result = EvalResult::evaluated();
        assert!(result.evaluated);
        assert!(!result.bypassed);
    }

    #[test]
    fn test_eval_result_bypassed() {
        let result = EvalResult::bypassed();
        assert!(result.bypassed);
        assert!(!result.evaluated);
    }

    #[test]
    fn test_eval_result_with_warning() {
        let result = EvalResult::evaluated()
            .with_warning("minor issue")
            .with_warning("another issue");

        assert!(result.has_warnings());
        assert_eq!(result.warnings.len(), 2);
    }

    #[test]
    fn test_eval_result_builder_pattern() {
        let result = EvalResult::evaluated()
            .with_outputs_changed()
            .with_time(1500);

        assert!(result.evaluated);
        assert!(result.outputs_changed);
        assert_eq!(result.eval_time_us, 1500);
    }
}
