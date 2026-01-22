//! Resource type definitions

use std::path::Path;

/// Types of resources that can be managed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Symbol,
    Image,
    Video,
    Audio,
    Font,
    Model3D,
    Shader,
    Data,
    Unknown,
}

impl ResourceType {
    /// Determine resource type from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "flux" => ResourceType::Symbol,
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tga" | "webp" | "hdr" => ResourceType::Image,
            "mp4" | "mov" | "avi" | "mkv" | "webm" => ResourceType::Video,
            "mp3" | "wav" | "ogg" | "flac" | "aac" => ResourceType::Audio,
            "ttf" | "otf" | "woff" | "woff2" => ResourceType::Font,
            "obj" | "fbx" | "gltf" | "glb" => ResourceType::Model3D,
            "hlsl" | "glsl" | "vert" | "frag" | "comp" => ResourceType::Shader,
            "json" | "xml" | "csv" | "txt" => ResourceType::Data,
            _ => ResourceType::Unknown,
        }
    }

    /// Get common file extensions for this resource type
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            ResourceType::Symbol => &["flux"],
            ResourceType::Image => &["png", "jpg", "jpeg", "gif", "bmp", "tga", "webp", "hdr"],
            ResourceType::Video => &["mp4", "mov", "avi", "mkv", "webm"],
            ResourceType::Audio => &["mp3", "wav", "ogg", "flac", "aac"],
            ResourceType::Font => &["ttf", "otf", "woff", "woff2"],
            ResourceType::Model3D => &["obj", "fbx", "gltf", "glb"],
            ResourceType::Shader => &["hlsl", "glsl", "vert", "frag", "comp"],
            ResourceType::Data => &["json", "xml", "csv", "txt"],
            ResourceType::Unknown => &[],
        }
    }

    /// Determine resource type from a path
    pub fn from_path(path: &Path) -> Self {
        path.extension()
            .and_then(|e| e.to_str())
            .map(Self::from_extension)
            .unwrap_or(ResourceType::Unknown)
    }
}
