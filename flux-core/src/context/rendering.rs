//! Rendering parameters for the evaluation context

use serde::{Deserialize, Serialize};

/// Fog parameters for atmospheric rendering
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FogParameters {
    /// Whether fog is enabled
    pub enabled: bool,
    /// Fog color (RGBA)
    pub color: [f32; 4],
    /// Fog density (for exponential fog)
    pub density: f32,
    /// Start distance for linear fog
    pub start: f32,
    /// End distance for linear fog
    pub end: f32,
}

impl FogParameters {
    pub fn new() -> Self {
        Self {
            enabled: false,
            color: [0.5, 0.5, 0.5, 1.0],
            density: 0.01,
            start: 10.0,
            end: 100.0,
        }
    }

    /// Create linear fog with start/end distances
    pub fn linear(start: f32, end: f32, color: [f32; 4]) -> Self {
        Self {
            enabled: true,
            color,
            density: 0.0,
            start,
            end,
        }
    }

    /// Create exponential fog with density
    pub fn exponential(density: f32, color: [f32; 4]) -> Self {
        Self {
            enabled: true,
            color,
            density,
            start: 0.0,
            end: 0.0,
        }
    }
}

/// PBR (Physically Based Rendering) material settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PbrMaterial {
    /// Base color (RGBA)
    pub albedo: [f32; 4],
    /// Surface roughness (0.0 = smooth, 1.0 = rough)
    pub roughness: f32,
    /// Metallic factor (0.0 = dielectric, 1.0 = metal)
    pub metallic: f32,
    /// Emissive color (RGB)
    pub emissive: [f32; 3],
    /// Ambient occlusion factor
    pub ao: f32,
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            albedo: [1.0, 1.0, 1.0, 1.0],
            roughness: 0.5,
            metallic: 0.0,
            emissive: [0.0, 0.0, 0.0],
            ao: 1.0,
        }
    }
}

impl PbrMaterial {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a metallic material
    pub fn metal(albedo: [f32; 4], roughness: f32) -> Self {
        Self {
            albedo,
            roughness,
            metallic: 1.0,
            ..Default::default()
        }
    }

    /// Create a dielectric (non-metallic) material
    pub fn dielectric(albedo: [f32; 4], roughness: f32) -> Self {
        Self {
            albedo,
            roughness,
            metallic: 0.0,
            ..Default::default()
        }
    }
}

/// Point light definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointLight {
    /// World-space position
    pub position: [f32; 3],
    /// Light color (RGB)
    pub color: [f32; 3],
    /// Light intensity multiplier
    pub intensity: f32,
    /// Attenuation range
    pub range: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            range: 10.0,
        }
    }
}

impl PointLight {
    pub fn new(position: [f32; 3], color: [f32; 3], intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
            range: 10.0,
        }
    }
}
