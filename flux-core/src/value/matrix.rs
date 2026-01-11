//! 4x4 transformation matrix

use serde::{Deserialize, Serialize};

/// 4x4 transformation matrix (column-major order)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Matrix4(pub [[f32; 4]; 4]);

impl Matrix4 {
    /// Identity matrix
    pub const IDENTITY: Self = Self([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    /// Zero matrix
    pub const ZERO: Self = Self([[0.0; 4]; 4]);

    /// Create a translation matrix
    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        Self([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [x, y, z, 1.0],
        ])
    }

    /// Create a uniform scale matrix
    pub fn scale_uniform(s: f32) -> Self {
        Self::scale(s, s, s)
    }

    /// Create a non-uniform scale matrix
    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Self([
            [x, 0.0, 0.0, 0.0],
            [0.0, y, 0.0, 0.0],
            [0.0, 0.0, z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Create a rotation matrix around the X axis (angle in radians)
    pub fn rotation_x(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, c, s, 0.0],
            [0.0, -s, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Create a rotation matrix around the Y axis (angle in radians)
    pub fn rotation_y(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self([
            [c, 0.0, -s, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [s, 0.0, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Create a rotation matrix around the Z axis (angle in radians)
    pub fn rotation_z(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self([
            [c, s, 0.0, 0.0],
            [-s, c, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Multiply two matrices
    pub fn mul(&self, other: &Self) -> Self {
        let mut result = Self::ZERO;
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result.0[i][j] += self.0[i][k] * other.0[k][j];
                }
            }
        }
        result
    }

    /// Transform a 3D point (applies translation)
    pub fn transform_point(&self, p: [f32; 3]) -> [f32; 3] {
        let w = self.0[0][3] * p[0] + self.0[1][3] * p[1] + self.0[2][3] * p[2] + self.0[3][3];
        [
            (self.0[0][0] * p[0] + self.0[1][0] * p[1] + self.0[2][0] * p[2] + self.0[3][0]) / w,
            (self.0[0][1] * p[0] + self.0[1][1] * p[1] + self.0[2][1] * p[2] + self.0[3][1]) / w,
            (self.0[0][2] * p[0] + self.0[1][2] * p[1] + self.0[2][2] * p[2] + self.0[3][2]) / w,
        ]
    }

    /// Transform a 3D vector (ignores translation)
    pub fn transform_vector(&self, v: [f32; 3]) -> [f32; 3] {
        [
            self.0[0][0] * v[0] + self.0[1][0] * v[1] + self.0[2][0] * v[2],
            self.0[0][1] * v[0] + self.0[1][1] * v[1] + self.0[2][1] * v[2],
            self.0[0][2] * v[0] + self.0[1][2] * v[1] + self.0[2][2] * v[2],
        ]
    }

    /// Get the translation component
    pub fn get_translation(&self) -> [f32; 3] {
        [self.0[3][0], self.0[3][1], self.0[3][2]]
    }

    /// Convert to flat array (column-major)
    pub fn to_array(&self) -> [f32; 16] {
        let mut arr = [0.0; 16];
        for i in 0..4 {
            for j in 0..4 {
                arr[i * 4 + j] = self.0[i][j];
            }
        }
        arr
    }
}

impl Default for Matrix4 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl std::ops::Mul for Matrix4 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::mul(&self, &rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix4_identity() {
        let m = Matrix4::IDENTITY;
        let point = [1.0, 2.0, 3.0];
        let result = m.transform_point(point);

        assert_eq!(result, point);
    }

    #[test]
    fn test_matrix4_translation() {
        let m = Matrix4::translation(10.0, 20.0, 30.0);
        let point = [1.0, 2.0, 3.0];
        let result = m.transform_point(point);

        assert_eq!(result, [11.0, 22.0, 33.0]);
    }

    #[test]
    fn test_matrix4_scale() {
        let m = Matrix4::scale(2.0, 3.0, 4.0);
        let point = [1.0, 1.0, 1.0];
        let result = m.transform_point(point);

        assert_eq!(result, [2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_matrix4_multiply() {
        let t = Matrix4::translation(10.0, 0.0, 0.0);
        let s = Matrix4::scale(2.0, 2.0, 2.0);
        let combined = s * t;

        let point = [1.0, 0.0, 0.0];
        let result = combined.transform_point(point);

        // Scale then translate: (1*2) + 10 = 12
        assert!((result[0] - 12.0).abs() < 0.01);
    }
}
