//! Transformation matrices
//!
//! This module provides:
//! - [`Matrix3`] - 3×3 matrix for normal/direction transforms
//! - [`Matrix4`] - 4×4 matrix for full affine transforms

use serde::{Deserialize, Serialize};

// ============================================================================
// Matrix3 - 3×3 transformation matrix
// ============================================================================

/// 3×3 transformation matrix (column-major order)
///
/// Used for:
/// - Normal transformation (inverse-transpose of upper-left 3×3 of Matrix4)
/// - Direction vector transformation (rotation + scale, no translation)
/// - Rotation matrices
///
/// # Column-Major Layout
///
/// The matrix is stored in column-major order for GPU compatibility:
/// ```text
/// [ col0[0]  col1[0]  col2[0] ]   [ m[0][0]  m[1][0]  m[2][0] ]
/// [ col0[1]  col1[1]  col2[1] ] = [ m[0][1]  m[1][1]  m[2][1] ]
/// [ col0[2]  col1[2]  col2[2] ]   [ m[0][2]  m[1][2]  m[2][2] ]
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Matrix3(pub [[f32; 3]; 3]);

impl Matrix3 {
    /// Identity matrix
    pub const IDENTITY: Self = Self([
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ]);

    /// Zero matrix
    pub const ZERO: Self = Self([[0.0; 3]; 3]);

    /// Create an identity matrix
    #[inline]
    pub fn identity() -> Self {
        Self::IDENTITY
    }

    /// Create a scale matrix
    pub fn from_scale(x: f32, y: f32, z: f32) -> Self {
        Self([
            [x, 0.0, 0.0],
            [0.0, y, 0.0],
            [0.0, 0.0, z],
        ])
    }

    /// Create a uniform scale matrix
    pub fn from_scale_uniform(s: f32) -> Self {
        Self::from_scale(s, s, s)
    }

    /// Create a rotation matrix around the X axis (angle in radians)
    pub fn rotation_x(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self([
            [1.0, 0.0, 0.0],
            [0.0, c, s],
            [0.0, -s, c],
        ])
    }

    /// Create a rotation matrix around the Y axis (angle in radians)
    pub fn rotation_y(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self([
            [c, 0.0, -s],
            [0.0, 1.0, 0.0],
            [s, 0.0, c],
        ])
    }

    /// Create a rotation matrix around the Z axis (angle in radians)
    pub fn rotation_z(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self([
            [c, s, 0.0],
            [-s, c, 0.0],
            [0.0, 0.0, 1.0],
        ])
    }

    /// Create a rotation matrix from a quaternion [x, y, z, w]
    ///
    /// Assumes the quaternion is normalized.
    pub fn from_rotation(quat: [f32; 4]) -> Self {
        let [x, y, z, w] = quat;

        let xx = x * x;
        let yy = y * y;
        let zz = z * z;
        let xy = x * y;
        let xz = x * z;
        let yz = y * z;
        let wx = w * x;
        let wy = w * y;
        let wz = w * z;

        Self([
            [1.0 - 2.0 * (yy + zz), 2.0 * (xy + wz), 2.0 * (xz - wy)],
            [2.0 * (xy - wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz + wx)],
            [2.0 * (xz + wy), 2.0 * (yz - wx), 1.0 - 2.0 * (xx + yy)],
        ])
    }

    /// Multiply two matrices
    pub fn mul(&self, other: &Self) -> Self {
        let mut result = Self::ZERO;
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result.0[i][j] += self.0[i][k] * other.0[k][j];
                }
            }
        }
        result
    }

    /// Transpose the matrix
    pub fn transpose(&self) -> Self {
        Self([
            [self.0[0][0], self.0[1][0], self.0[2][0]],
            [self.0[0][1], self.0[1][1], self.0[2][1]],
            [self.0[0][2], self.0[1][2], self.0[2][2]],
        ])
    }

    /// Calculate the determinant
    pub fn determinant(&self) -> f32 {
        let m = &self.0;
        m[0][0] * (m[1][1] * m[2][2] - m[2][1] * m[1][2])
            - m[1][0] * (m[0][1] * m[2][2] - m[2][1] * m[0][2])
            + m[2][0] * (m[0][1] * m[1][2] - m[1][1] * m[0][2])
    }

    /// Calculate the inverse matrix
    ///
    /// Returns `None` if the matrix is singular (determinant is zero).
    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < 1e-10 {
            return None;
        }

        let inv_det = 1.0 / det;
        let m = &self.0;

        // Calculate adjugate matrix (cofactor matrix transposed)
        Some(Self([
            [
                (m[1][1] * m[2][2] - m[2][1] * m[1][2]) * inv_det,
                (m[2][1] * m[0][2] - m[0][1] * m[2][2]) * inv_det,
                (m[0][1] * m[1][2] - m[1][1] * m[0][2]) * inv_det,
            ],
            [
                (m[2][0] * m[1][2] - m[1][0] * m[2][2]) * inv_det,
                (m[0][0] * m[2][2] - m[2][0] * m[0][2]) * inv_det,
                (m[1][0] * m[0][2] - m[0][0] * m[1][2]) * inv_det,
            ],
            [
                (m[1][0] * m[2][1] - m[2][0] * m[1][1]) * inv_det,
                (m[2][0] * m[0][1] - m[0][0] * m[2][1]) * inv_det,
                (m[0][0] * m[1][1] - m[1][0] * m[0][1]) * inv_det,
            ],
        ]))
    }

    /// Transform a 3D vector
    pub fn transform_vector(&self, v: [f32; 3]) -> [f32; 3] {
        [
            self.0[0][0] * v[0] + self.0[1][0] * v[1] + self.0[2][0] * v[2],
            self.0[0][1] * v[0] + self.0[1][1] * v[1] + self.0[2][1] * v[2],
            self.0[0][2] * v[0] + self.0[1][2] * v[1] + self.0[2][2] * v[2],
        ]
    }

    /// Convert to flat array (column-major order)
    pub fn to_array(&self) -> [f32; 9] {
        [
            self.0[0][0], self.0[0][1], self.0[0][2],
            self.0[1][0], self.0[1][1], self.0[1][2],
            self.0[2][0], self.0[2][1], self.0[2][2],
        ]
    }

    /// Embed into a 4×4 matrix with identity translation
    ///
    /// The Matrix3 becomes the upper-left 3×3 of the Matrix4,
    /// with the translation set to zero and w = 1.
    pub fn to_mat4(&self) -> Matrix4 {
        Matrix4([
            [self.0[0][0], self.0[0][1], self.0[0][2], 0.0],
            [self.0[1][0], self.0[1][1], self.0[1][2], 0.0],
            [self.0[2][0], self.0[2][1], self.0[2][2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}

impl Default for Matrix3 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl std::ops::Mul for Matrix3 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::mul(&self, &rhs)
    }
}

// ============================================================================
// Matrix4 - 4×4 transformation matrix
// ============================================================================

/// 4×4 transformation matrix (column-major order)
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

    /// Extract the upper-left 3×3 matrix (rotation + scale)
    ///
    /// This extracts the linear transformation part, discarding translation.
    pub fn to_mat3(&self) -> Matrix3 {
        Matrix3([
            [self.0[0][0], self.0[0][1], self.0[0][2]],
            [self.0[1][0], self.0[1][1], self.0[1][2]],
            [self.0[2][0], self.0[2][1], self.0[2][2]],
        ])
    }

    /// Compute the normal matrix (inverse-transpose of upper-left 3×3)
    ///
    /// Normals require special transformation to remain perpendicular to surfaces
    /// under non-uniform scaling. This returns the correct transformation matrix.
    ///
    /// Returns `None` if the upper-left 3×3 is singular (cannot be inverted).
    pub fn normal_matrix(&self) -> Option<Matrix3> {
        self.to_mat3().inverse().map(|inv| inv.transpose())
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

    // =========================================================================
    // Matrix4 Tests
    // =========================================================================

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

    // =========================================================================
    // Matrix3 Tests
    // =========================================================================

    #[test]
    fn test_matrix3_identity() {
        let m = Matrix3::IDENTITY;
        let v = [1.0, 2.0, 3.0];
        let result = m.transform_vector(v);
        assert_eq!(result, v);
    }

    #[test]
    fn test_matrix3_scale() {
        let m = Matrix3::from_scale(2.0, 3.0, 4.0);
        let v = [1.0, 1.0, 1.0];
        let result = m.transform_vector(v);
        assert_eq!(result, [2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_matrix3_multiply() {
        let a = Matrix3::from_scale(2.0, 2.0, 2.0);
        let b = Matrix3::from_scale(3.0, 3.0, 3.0);
        let c = a * b;

        let v = [1.0, 1.0, 1.0];
        let result = c.transform_vector(v);
        assert_eq!(result, [6.0, 6.0, 6.0]);
    }

    #[test]
    fn test_matrix3_transpose() {
        let m = Matrix3([
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
            [7.0, 8.0, 9.0],
        ]);
        let t = m.transpose();

        assert_eq!(t.0[0], [1.0, 4.0, 7.0]);
        assert_eq!(t.0[1], [2.0, 5.0, 8.0]);
        assert_eq!(t.0[2], [3.0, 6.0, 9.0]);
    }

    #[test]
    fn test_matrix3_determinant() {
        // Identity matrix has determinant 1
        assert!((Matrix3::IDENTITY.determinant() - 1.0).abs() < 1e-6);

        // Scale matrix determinant = product of scales
        let scale = Matrix3::from_scale(2.0, 3.0, 4.0);
        assert!((scale.determinant() - 24.0).abs() < 1e-6);

        // Zero matrix has determinant 0
        assert!(Matrix3::ZERO.determinant().abs() < 1e-10);
    }

    #[test]
    fn test_matrix3_inverse() {
        // Identity inverse is identity
        let inv = Matrix3::IDENTITY.inverse().unwrap();
        assert_eq!(inv, Matrix3::IDENTITY);

        // Scale inverse
        let scale = Matrix3::from_scale(2.0, 4.0, 8.0);
        let inv = scale.inverse().unwrap();
        let expected = Matrix3::from_scale(0.5, 0.25, 0.125);
        for i in 0..3 {
            for j in 0..3 {
                assert!((inv.0[i][j] - expected.0[i][j]).abs() < 1e-6);
            }
        }

        // Singular matrix returns None
        assert!(Matrix3::ZERO.inverse().is_none());
    }

    #[test]
    fn test_matrix3_inverse_roundtrip() {
        let m = Matrix3::from_scale(2.0, 3.0, 4.0);
        let inv = m.inverse().unwrap();
        let result = m * inv;

        // Should be close to identity
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((result.0[i][j] - expected).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn test_matrix3_rotation_z() {
        use std::f32::consts::FRAC_PI_2;

        let m = Matrix3::rotation_z(FRAC_PI_2); // 90 degrees
        let v = [1.0, 0.0, 0.0];
        let result = m.transform_vector(v);

        // Rotating [1,0,0] 90° around Z should give [0,1,0]
        assert!(result[0].abs() < 1e-6);
        assert!((result[1] - 1.0).abs() < 1e-6);
        assert!(result[2].abs() < 1e-6);
    }

    #[test]
    fn test_matrix3_from_quaternion() {
        // Identity quaternion [0, 0, 0, 1] should give identity matrix
        let m = Matrix3::from_rotation([0.0, 0.0, 0.0, 1.0]);
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((m.0[i][j] - expected).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn test_matrix3_to_array() {
        let m = Matrix3([
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
            [7.0, 8.0, 9.0],
        ]);
        let arr = m.to_array();

        // Column-major: col0, col1, col2
        assert_eq!(arr, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
    }

    // =========================================================================
    // Matrix3 <-> Matrix4 Conversion Tests
    // =========================================================================

    #[test]
    fn test_matrix4_to_mat3() {
        let m4 = Matrix4::scale(2.0, 3.0, 4.0);
        let m3 = m4.to_mat3();

        // Should extract the scale portion
        let v = [1.0, 1.0, 1.0];
        let result = m3.transform_vector(v);
        assert_eq!(result, [2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_matrix4_to_mat3_ignores_translation() {
        let m4 = Matrix4::translation(100.0, 200.0, 300.0);
        let m3 = m4.to_mat3();

        // Translation should be ignored, resulting in identity
        assert_eq!(m3, Matrix3::IDENTITY);
    }

    #[test]
    fn test_matrix3_to_mat4() {
        let m3 = Matrix3::from_scale(2.0, 3.0, 4.0);
        let m4 = m3.to_mat4();

        // Should embed as upper-left 3x3 with zero translation
        let point = [1.0, 1.0, 1.0];
        let result = m4.transform_point(point);
        assert_eq!(result, [2.0, 3.0, 4.0]);

        // Translation column should be zero
        assert_eq!(m4.get_translation(), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_matrix3_to_mat4_roundtrip() {
        let original = Matrix3::from_scale(2.0, 3.0, 4.0);
        let m4 = original.to_mat4();
        let recovered = m4.to_mat3();

        assert_eq!(original, recovered);
    }

    #[test]
    fn test_normal_matrix() {
        // For uniform scale, normal matrix should be the inverse scale
        let m4 = Matrix4::scale(2.0, 2.0, 2.0);
        let normal = m4.normal_matrix().unwrap();

        // Inverse-transpose of uniform scale is inverse scale
        let v = [1.0, 0.0, 0.0];
        let result = normal.transform_vector(v);
        assert!((result[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_normal_matrix_non_uniform_scale() {
        // Non-uniform scale requires proper normal transformation
        let m4 = Matrix4::scale(2.0, 1.0, 1.0);
        let normal = m4.normal_matrix().unwrap();

        // Normal along X should be scaled by 1/2
        let n = [1.0, 0.0, 0.0];
        let result = normal.transform_vector(n);
        assert!((result[0] - 0.5).abs() < 1e-6);
        assert!(result[1].abs() < 1e-6);
        assert!(result[2].abs() < 1e-6);
    }

    #[test]
    fn test_normal_matrix_rotation_only() {
        // For rotation-only matrices, normal matrix equals the rotation
        let m4 = Matrix4::rotation_z(std::f32::consts::FRAC_PI_4);
        let normal = m4.normal_matrix().unwrap();
        let m3 = m4.to_mat3();

        // Inverse-transpose of rotation = rotation itself
        for i in 0..3 {
            for j in 0..3 {
                assert!((normal.0[i][j] - m3.0[i][j]).abs() < 1e-6);
            }
        }
    }
}
