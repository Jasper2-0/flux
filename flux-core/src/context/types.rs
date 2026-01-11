//! Gizmo and transform types for the evaluation context

use serde::{Deserialize, Serialize};

/// Gizmo visibility modes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GizmoVisibility {
    /// Inherit visibility from parent context
    #[default]
    Inherit,
    /// Gizmos are always hidden
    Off,
    /// Gizmos are always visible
    On,
    /// Gizmos are visible only when object is selected
    IfSelected,
}

/// Transform gizmo modes for 3D manipulation
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TransformGizmoMode {
    /// No transform gizmo active
    #[default]
    None,
    /// Selection mode
    Select,
    /// Move/translate mode
    Move,
    /// Rotation mode
    Rotate,
    /// Scale mode
    Scale,
}

/// 4x4 transformation matrix (column-major order)
pub type Mat4 = [[f32; 4]; 4];

/// Identity matrix constant
pub const MAT4_IDENTITY: Mat4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];
