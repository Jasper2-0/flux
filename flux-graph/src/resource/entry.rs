//! Resource entry definitions

use std::collections::HashMap;
use std::path::Path;

use super::ResourceType;

/// A single resource entry
#[derive(Debug, Clone)]
pub struct ResourceEntry {
    /// Display name
    pub name: String,
    /// File path (relative to package root)
    pub relative_path: String,
    /// Type of resource
    pub resource_type: ResourceType,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
    /// File size in bytes (if known)
    pub size_bytes: Option<u64>,
}

impl ResourceEntry {
    /// Create a new resource entry with name and path
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        let relative_path = path.into();
        let resource_type = ResourceType::from_path(Path::new(&relative_path));
        Self {
            name: name.into(),
            relative_path,
            resource_type,
            metadata: HashMap::new(),
            size_bytes: None,
        }
    }

    /// Create from just a path, deriving name from filename
    pub fn from_path(path: impl Into<String>) -> Self {
        let relative_path = path.into();
        let name = Path::new(&relative_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let resource_type = ResourceType::from_path(Path::new(&relative_path));
        Self {
            name,
            relative_path,
            resource_type,
            metadata: HashMap::new(),
            size_bytes: None,
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set file size
    pub fn with_size(mut self, size: u64) -> Self {
        self.size_bytes = Some(size);
        self
    }
}
