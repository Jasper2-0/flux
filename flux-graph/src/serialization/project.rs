//! Project file schema (.rproj)
//!
//! The project file is the root configuration for a visual programming project.
//! It defines metadata, resource paths, and the main graph entry point.

use serde::{Deserialize, Serialize};

use flux_core::Id;

use super::version::SchemaVersion;

/// Project file schema (.rproj)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    /// Schema version for migrations
    pub version: SchemaVersion,
    /// Project metadata
    pub project: ProjectMeta,
    /// Resource directory configuration
    #[serde(default)]
    pub resources: ResourceConfig,
    /// Symbol library paths (relative to project root)
    #[serde(default)]
    pub symbol_paths: Vec<String>,
    /// Main graph entry point (relative to project root)
    pub main_graph: String,
}

impl ProjectFile {
    /// Create a new project file with defaults
    pub fn new(name: &str) -> Self {
        Self {
            version: SchemaVersion::CURRENT,
            project: ProjectMeta::new(name),
            resources: ResourceConfig::default(),
            symbol_paths: vec!["symbols".into()],
            main_graph: "graphs/main.rgraph".into(),
        }
    }
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    /// Unique project identifier
    pub id: Id,
    /// Project name
    pub name: String,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Creation timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// Last modified timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
}

impl ProjectMeta {
    /// Create new project metadata
    pub fn new(name: &str) -> Self {
        Self {
            id: Id::new(),
            name: name.to_string(),
            description: None,
            author: None,
            created: None,
            modified: None,
        }
    }

    /// Builder: set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    /// Builder: set author
    pub fn with_author(mut self, author: &str) -> Self {
        self.author = Some(author.to_string());
        self
    }
}

/// Resource directory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Texture search directories
    #[serde(default = "default_texture_dirs")]
    pub texture_dirs: Vec<String>,
    /// Audio search directories
    #[serde(default = "default_audio_dirs")]
    pub audio_dirs: Vec<String>,
    /// 3D model search directories
    #[serde(default = "default_model_dirs")]
    pub model_dirs: Vec<String>,
    /// Shader search directories
    #[serde(default = "default_shader_dirs")]
    pub shader_dirs: Vec<String>,
}

fn default_texture_dirs() -> Vec<String> {
    vec!["resources/textures".into()]
}

fn default_audio_dirs() -> Vec<String> {
    vec!["resources/audio".into()]
}

fn default_model_dirs() -> Vec<String> {
    vec!["resources/models".into()]
}

fn default_shader_dirs() -> Vec<String> {
    vec!["resources/shaders".into()]
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            texture_dirs: default_texture_dirs(),
            audio_dirs: default_audio_dirs(),
            model_dirs: default_model_dirs(),
            shader_dirs: default_shader_dirs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_file_new() {
        let project = ProjectFile::new("Test Project");
        assert_eq!(project.project.name, "Test Project");
        assert_eq!(project.version, SchemaVersion::CURRENT);
        assert!(!project.symbol_paths.is_empty());
    }

    #[test]
    fn test_project_meta_builder() {
        let meta = ProjectMeta::new("My Project")
            .with_description("A test project")
            .with_author("Test Author");

        assert_eq!(meta.name, "My Project");
        assert_eq!(meta.description, Some("A test project".to_string()));
        assert_eq!(meta.author, Some("Test Author".to_string()));
    }

    #[test]
    fn test_resource_config_default() {
        let config = ResourceConfig::default();
        assert!(config.texture_dirs.contains(&"resources/textures".to_string()));
        assert!(config.audio_dirs.contains(&"resources/audio".to_string()));
    }

    #[test]
    fn test_project_file_serialize() {
        let project = ProjectFile::new("Serialize Test");
        let json = serde_json::to_string_pretty(&project).unwrap();
        assert!(json.contains("Serialize Test"));
        assert!(json.contains("version"));

        let restored: ProjectFile = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.project.name, "Serialize Test");
    }
}
