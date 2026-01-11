//! Serialization error types

use thiserror::Error;

/// Errors that can occur during serialization/deserialization
#[derive(Error, Debug)]
pub enum SerializationError {
    /// IO error reading/writing files
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing/serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Schema version mismatch
    #[error("Version mismatch: file is v{file_major}.{file_minor}, expected v{expected_major}.x")]
    VersionMismatch {
        file_major: u32,
        file_minor: u32,
        expected_major: u32,
    },

    /// Invalid file extension
    #[error("Invalid file extension: expected {expected}, got {actual}")]
    InvalidExtension { expected: String, actual: String },

    /// Symbol not found during loading
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    /// Invalid symbol reference
    #[error("Invalid symbol reference: {0}")]
    InvalidReference(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid value type
    #[error("Invalid value type: expected {expected}, got {actual}")]
    InvalidValueType { expected: String, actual: String },

    /// Migration error
    #[error("Migration error: {0}")]
    MigrationFailed(String),

    /// File too large to load
    #[error("File too large: {size} bytes exceeds maximum of {max_size} bytes")]
    FileTooLarge { size: u64, max_size: u64 },
}

/// Result type for serialization operations
pub type Result<T> = std::result::Result<T, SerializationError>;
