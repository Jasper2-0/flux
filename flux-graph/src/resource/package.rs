//! Resource package definitions

use std::path::{Path, PathBuf};

use super::{ResourceEntry, ResourceType};

/// A collection of related resources (like a project folder)
#[derive(Debug, Clone)]
pub struct ResourcePackage {
    /// Display name
    pub name: String,
    /// Root path for this package
    pub path: PathBuf,
    /// Optional description
    pub description: Option<String>,
    /// Optional version string
    pub version: Option<String>,
    /// Whether this package is read-only
    pub is_read_only: bool,
    /// Resources in this package
    resources: Vec<ResourceEntry>,
}

impl ResourcePackage {
    /// Create a new resource package
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            description: None,
            version: None,
            is_read_only: false,
            resources: Vec::new(),
        }
    }

    /// Create a read-only package (for built-in/system resources)
    pub fn read_only(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            description: None,
            version: None,
            is_read_only: true,
            resources: Vec::new(),
        }
    }

    /// Add a resource to the package
    pub fn add_resource(&mut self, resource: ResourceEntry) {
        self.resources.push(resource);
    }

    /// Get the number of resources
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    /// Get all resources of a specific type
    pub fn resources_of_type(&self, resource_type: ResourceType) -> impl Iterator<Item = &ResourceEntry> {
        self.resources
            .iter()
            .filter(move |r| r.resource_type == resource_type)
    }

    /// Get all resources
    pub fn resources(&self) -> &[ResourceEntry] {
        &self.resources
    }

    /// Find a resource by relative path
    pub fn find_resource(&self, relative_path: &str) -> Option<&ResourceEntry> {
        self.resources
            .iter()
            .find(|r| r.relative_path == relative_path)
    }

    /// Resolve a relative path to an absolute path
    pub fn resolve_path(&self, relative_path: &str) -> PathBuf {
        self.path.join(relative_path)
    }

    /// Get the root path
    pub fn root_path(&self) -> &Path {
        &self.path
    }

    /// Check if package is empty
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
}
