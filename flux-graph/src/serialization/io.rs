//! File I/O operations for serialization
//!
//! Load and save functions for project, symbol, and graph files.

use std::fs;
use std::path::Path;

use super::error::{Result, SerializationError};
use super::graph::GraphFile;
use super::project::ProjectFile;
use super::symbol::SymbolFile;
use super::version::SchemaVersion;

/// Maximum file size allowed for loading (50 MB)
const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;

// ============================================================================
// Version Validation
// ============================================================================

fn validate_version(version: &SchemaVersion, expected_major: u32) -> Result<()> {
    if version.major != expected_major {
        return Err(SerializationError::VersionMismatch {
            file_major: version.major,
            file_minor: version.minor,
            expected_major,
        });
    }
    Ok(())
}

/// Check file size before loading to prevent memory exhaustion
fn check_file_size(path: impl AsRef<Path>) -> Result<()> {
    let metadata = fs::metadata(path.as_ref())?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(SerializationError::FileTooLarge {
            size: metadata.len(),
            max_size: MAX_FILE_SIZE,
        });
    }
    Ok(())
}

// ============================================================================
// Project Files (.rproj)
// ============================================================================

/// Load a project file
pub fn load_project(path: impl AsRef<Path>) -> Result<ProjectFile> {
    check_file_size(&path)?;
    let content = fs::read_to_string(path)?;
    let project: ProjectFile = serde_json::from_str(&content)?;
    validate_version(&project.version, 1)?;
    Ok(project)
}

/// Save a project file
pub fn save_project(project: &ProjectFile, path: impl AsRef<Path>) -> Result<()> {
    let content = serde_json::to_string_pretty(project)?;
    fs::write(path, content)?;
    Ok(())
}

/// Load a project file from a JSON string
pub fn load_project_str(json: &str) -> Result<ProjectFile> {
    let project: ProjectFile = serde_json::from_str(json)?;
    validate_version(&project.version, 1)?;
    Ok(project)
}

/// Serialize a project file to JSON string
pub fn save_project_str(project: &ProjectFile) -> Result<String> {
    Ok(serde_json::to_string_pretty(project)?)
}

// ============================================================================
// Symbol Files (.rsym)
// ============================================================================

/// Load a symbol file
pub fn load_symbol(path: impl AsRef<Path>) -> Result<SymbolFile> {
    check_file_size(&path)?;
    let content = fs::read_to_string(path)?;
    let symbol: SymbolFile = serde_json::from_str(&content)?;
    validate_version(&symbol.version, 1)?;
    Ok(symbol)
}

/// Save a symbol file
pub fn save_symbol(symbol: &SymbolFile, path: impl AsRef<Path>) -> Result<()> {
    let content = serde_json::to_string_pretty(symbol)?;
    fs::write(path, content)?;
    Ok(())
}

/// Load a symbol file from a JSON string
pub fn load_symbol_str(json: &str) -> Result<SymbolFile> {
    let symbol: SymbolFile = serde_json::from_str(json)?;
    validate_version(&symbol.version, 1)?;
    Ok(symbol)
}

/// Serialize a symbol file to JSON string
pub fn save_symbol_str(symbol: &SymbolFile) -> Result<String> {
    Ok(serde_json::to_string_pretty(symbol)?)
}

// ============================================================================
// Graph Files (.rgraph)
// ============================================================================

/// Load a graph file
pub fn load_graph(path: impl AsRef<Path>) -> Result<GraphFile> {
    check_file_size(&path)?;
    let content = fs::read_to_string(path)?;
    let graph: GraphFile = serde_json::from_str(&content)?;
    validate_version(&graph.version, 1)?;
    Ok(graph)
}

/// Save a graph file
pub fn save_graph(graph: &GraphFile, path: impl AsRef<Path>) -> Result<()> {
    let content = serde_json::to_string_pretty(graph)?;
    fs::write(path, content)?;
    Ok(())
}

/// Load a graph file from a JSON string
pub fn load_graph_str(json: &str) -> Result<GraphFile> {
    let graph: GraphFile = serde_json::from_str(json)?;
    validate_version(&graph.version, 1)?;
    Ok(graph)
}

/// Serialize a graph file to JSON string
pub fn save_graph_str(graph: &GraphFile) -> Result<String> {
    Ok(serde_json::to_string_pretty(graph)?)
}

// ============================================================================
// Auto-detect File Type
// ============================================================================

/// File type based on extension
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Project,
    Symbol,
    Graph,
    Unknown,
}

impl FileType {
    /// Detect file type from path extension
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        match path.as_ref().extension().and_then(|e| e.to_str()) {
            Some("rproj") => Self::Project,
            Some("rsym") => Self::Symbol,
            Some("rgraph") => Self::Graph,
            _ => Self::Unknown,
        }
    }

    /// Get the expected extension for this file type
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Project => "rproj",
            Self::Symbol => "rsym",
            Self::Graph => "rgraph",
            Self::Unknown => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::Id;

    #[test]
    fn test_project_roundtrip_str() {
        let project = ProjectFile::new("Test Project");
        let json = save_project_str(&project).unwrap();
        let restored = load_project_str(&json).unwrap();
        assert_eq!(restored.project.name, "Test Project");
    }

    #[test]
    fn test_symbol_roundtrip_str() {
        let symbol = SymbolFile::new("TestSymbol");
        let json = save_symbol_str(&symbol).unwrap();
        let restored = load_symbol_str(&json).unwrap();
        assert_eq!(restored.symbol.name, "TestSymbol");
    }

    #[test]
    fn test_graph_roundtrip_str() {
        let root_id = Id::new();
        let graph = GraphFile::new("Main", root_id);
        let json = save_graph_str(&graph).unwrap();
        let restored = load_graph_str(&json).unwrap();
        assert_eq!(restored.graph.name, "Main");
    }

    #[test]
    fn test_file_type_detection() {
        assert_eq!(FileType::from_path("test.rproj"), FileType::Project);
        assert_eq!(FileType::from_path("test.rsym"), FileType::Symbol);
        assert_eq!(FileType::from_path("test.rgraph"), FileType::Graph);
        assert_eq!(FileType::from_path("test.txt"), FileType::Unknown);
    }

    #[test]
    fn test_version_validation() {
        // Invalid version should fail
        let json = r#"{
            "version": { "major": 99, "minor": 0 },
            "project": {
                "id": "00000000-0000-0000-0000-000000000000",
                "name": "Test"
            },
            "main_graph": "main.rgraph"
        }"#;

        let result = load_project_str(json);
        assert!(matches!(result, Err(SerializationError::VersionMismatch { .. })));
    }
}
