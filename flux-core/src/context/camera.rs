//! Camera trait and implementations

use super::Mat4;

/// Trait for camera implementations
pub trait Camera {
    /// Get the view matrix (world-to-camera transform)
    fn get_view_matrix(&self) -> Mat4;
    /// Get the projection matrix (camera-to-clip transform)
    fn get_projection_matrix(&self) -> Mat4;
    /// Get the camera's world-space position
    fn get_position(&self) -> [f32; 3];
}

/// Simple perspective camera implementation
#[derive(Clone, Debug)]
pub struct PerspectiveCamera {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub fov_y: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 5.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            fov_y: std::f32::consts::FRAC_PI_4, // 45 degrees
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl PerspectiveCamera {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a look-at camera
    pub fn look_at(position: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Self {
        Self {
            position,
            target,
            up,
            ..Default::default()
        }
    }
}

impl Camera for PerspectiveCamera {
    fn get_view_matrix(&self) -> Mat4 {
        // Simplified look-at matrix calculation
        let f = normalize(sub(self.target, self.position));
        let s = normalize(cross(f, self.up));
        let u = cross(s, f);

        [
            [s[0], u[0], -f[0], 0.0],
            [s[1], u[1], -f[1], 0.0],
            [s[2], u[2], -f[2], 0.0],
            [
                -dot(s, self.position),
                -dot(u, self.position),
                dot(f, self.position),
                1.0,
            ],
        ]
    }

    fn get_projection_matrix(&self) -> Mat4 {
        let f = 1.0 / (self.fov_y / 2.0).tan();
        let nf = 1.0 / (self.near - self.far);

        [
            [f / self.aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (self.far + self.near) * nf, -1.0],
            [0.0, 0.0, 2.0 * self.far * self.near * nf, 0.0],
        ]
    }

    fn get_position(&self) -> [f32; 3] {
        self.position
    }
}

// Helper math functions for camera
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}
