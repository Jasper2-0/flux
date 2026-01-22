//! Resource management system
//!
//! This module provides a hierarchical resource management system with:
//! - [`ResourceType`] - Classification of resource files by type
//! - [`ResourceEntry`] - Individual resource metadata
//! - [`ResourcePackage`] - Collections of related resources
//! - [`ResourceManager`] - Central manager for all resource packages

mod entry;
mod package;
mod types;

pub use entry::ResourceEntry;
pub use package::ResourcePackage;
pub use types::ResourceType;

use std::path::PathBuf;

use flux_core::id::Id;

/// Central resource manager that handles multiple packages
#[derive(Debug, Default)]
pub struct ResourceManager {
    /// Packages with their IDs
    packages: Vec<(Id, ResourcePackage)>,
    /// Search paths for resolving relative resource references
    search_paths: Vec<PathBuf>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a package and return its ID
    pub fn add_package(&mut self, package: ResourcePackage) -> Id {
        let id = Id::new();
        self.packages.push((id, package));
        id
    }

    /// Get the number of packages
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }

    /// Get total resource count across all packages
    pub fn total_resource_count(&self) -> usize {
        self.packages.iter().map(|(_, p)| p.resource_count()).sum()
    }

    /// Get all package names
    pub fn package_names(&self) -> impl Iterator<Item = &str> {
        self.packages.iter().map(|(_, p)| p.name.as_str())
    }

    /// Get a package by name
    pub fn get_package_by_name(&self, name: &str) -> Option<&ResourcePackage> {
        self.packages
            .iter()
            .find(|(_, p)| p.name == name)
            .map(|(_, p)| p)
    }

    /// Get a package by ID
    pub fn get_package(&self, id: Id) -> Option<&ResourcePackage> {
        self.packages
            .iter()
            .find(|(pid, _)| *pid == id)
            .map(|(_, p)| p)
    }

    /// Get a mutable package by ID
    pub fn get_package_mut(&mut self, id: Id) -> Option<&mut ResourcePackage> {
        self.packages
            .iter_mut()
            .find(|(pid, _)| *pid == id)
            .map(|(_, p)| p)
    }

    /// Find a resource by relative path across all packages
    pub fn find_resource(&self, relative_path: &str) -> Option<(&ResourcePackage, &ResourceEntry)> {
        for (_, pkg) in &self.packages {
            if let Some(entry) = pkg.find_resource(relative_path) {
                return Some((pkg, entry));
            }
        }
        None
    }

    /// Find all resources of a specific type across all packages
    pub fn find_resources_of_type(&self, resource_type: ResourceType) -> Vec<(&ResourcePackage, &ResourceEntry)> {
        let mut results = Vec::new();
        for (_, pkg) in &self.packages {
            for entry in pkg.resources_of_type(resource_type) {
                results.push((pkg, entry));
            }
        }
        results
    }

    /// Add a search path for resource resolution
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Get all search paths
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }

    /// List all packages
    pub fn packages(&self) -> impl Iterator<Item = &ResourcePackage> {
        self.packages.iter().map(|(_, p)| p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type_from_extension() {
        assert_eq!(ResourceType::from_extension("flux"), ResourceType::Symbol);
        assert_eq!(ResourceType::from_extension("png"), ResourceType::Image);
        assert_eq!(ResourceType::from_extension("PNG"), ResourceType::Image);
        assert_eq!(ResourceType::from_extension("mp4"), ResourceType::Video);
        assert_eq!(ResourceType::from_extension("wav"), ResourceType::Audio);
        assert_eq!(ResourceType::from_extension("ttf"), ResourceType::Font);
        assert_eq!(ResourceType::from_extension("obj"), ResourceType::Model3D);
        assert_eq!(ResourceType::from_extension("hlsl"), ResourceType::Shader);
        assert_eq!(ResourceType::from_extension("json"), ResourceType::Data);
        assert_eq!(ResourceType::from_extension("xyz"), ResourceType::Unknown);
    }

    #[test]
    fn test_resource_type_extensions() {
        assert!(ResourceType::Image.extensions().contains(&"png"));
        assert!(ResourceType::Video.extensions().contains(&"mp4"));
        assert!(ResourceType::Audio.extensions().contains(&"wav"));
        assert!(ResourceType::Symbol.extensions().contains(&"flux"));
    }

    #[test]
    fn test_resource_entry() {
        let entry = ResourceEntry::new("Wood Texture", "textures/wood.png")
            .with_metadata("author", "Artist")
            .with_size(1024);

        assert_eq!(entry.name, "Wood Texture");
        assert_eq!(entry.resource_type, ResourceType::Image);
        assert_eq!(entry.metadata.get("author"), Some(&"Artist".to_string()));
        assert_eq!(entry.size_bytes, Some(1024));
    }

    #[test]
    fn test_resource_entry_from_path() {
        let entry = ResourceEntry::from_path("sounds/explosion.wav");
        assert_eq!(entry.name, "explosion");
        assert_eq!(entry.resource_type, ResourceType::Audio);
    }

    #[test]
    fn test_resource_package() {
        let mut pkg = ResourcePackage::new("Test Package", "/projects/test");

        pkg.add_resource(ResourceEntry::from_path("img1.png"));
        pkg.add_resource(ResourceEntry::from_path("img2.jpg"));
        pkg.add_resource(ResourceEntry::from_path("snd1.wav"));

        assert_eq!(pkg.resource_count(), 3);
        assert_eq!(pkg.resources_of_type(ResourceType::Image).count(), 2);
        assert_eq!(pkg.resources_of_type(ResourceType::Audio).count(), 1);

        assert!(pkg.find_resource("img1.png").is_some());
        assert!(pkg.find_resource("nonexistent").is_none());
    }

    #[test]
    fn test_resource_package_read_only() {
        let pkg = ResourcePackage::read_only("Core", "/app/core");
        assert!(pkg.is_read_only);

        let pkg2 = ResourcePackage::new("User", "/user/project");
        assert!(!pkg2.is_read_only);
    }

    #[test]
    fn test_resource_manager() {
        let mut manager = ResourceManager::new();

        let mut pkg1 = ResourcePackage::new("Package 1", "/pkg1");
        pkg1.add_resource(ResourceEntry::from_path("tex1.png"));

        let mut pkg2 = ResourcePackage::new("Package 2", "/pkg2");
        pkg2.add_resource(ResourceEntry::from_path("tex2.png"));

        manager.add_package(pkg1);
        manager.add_package(pkg2);

        assert_eq!(manager.package_count(), 2);
        assert_eq!(manager.total_resource_count(), 2);

        // Can find resources across packages
        assert!(manager.find_resource("tex1.png").is_some());
        assert!(manager.find_resource("tex2.png").is_some());
        assert!(manager.find_resource("nonexistent").is_none());
    }

    #[test]
    fn test_find_resources_of_type() {
        let mut manager = ResourceManager::new();

        let mut pkg1 = ResourcePackage::new("Package 1", "/pkg1");
        pkg1.add_resource(ResourceEntry::from_path("tex1.png"));
        pkg1.add_resource(ResourceEntry::from_path("snd1.wav"));

        let mut pkg2 = ResourcePackage::new("Package 2", "/pkg2");
        pkg2.add_resource(ResourceEntry::from_path("tex2.png"));

        manager.add_package(pkg1);
        manager.add_package(pkg2);

        let images = manager.find_resources_of_type(ResourceType::Image);
        assert_eq!(images.len(), 2);

        let sounds = manager.find_resources_of_type(ResourceType::Audio);
        assert_eq!(sounds.len(), 1);
    }

    #[test]
    fn test_search_paths() {
        let mut manager = ResourceManager::new();
        manager.add_search_path(PathBuf::from("/textures"));
        manager.add_search_path(PathBuf::from("/sounds"));

        assert_eq!(manager.search_paths().len(), 2);
    }

    #[test]
    fn test_package_names() {
        let mut manager = ResourceManager::new();
        manager.add_package(ResourcePackage::new("Alpha", "/alpha"));
        manager.add_package(ResourcePackage::new("Beta", "/beta"));

        let names: Vec<_> = manager.package_names().collect();
        assert!(names.contains(&"Alpha"));
        assert!(names.contains(&"Beta"));
    }
}
