//! Schema versioning for file format migrations
//!
//! All serialized files include a version number to support forward-compatible
//! migrations as the format evolves.

use serde::{Deserialize, Serialize};

/// Schema version for file format
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// Major version - breaking changes
    pub major: u32,
    /// Minor version - backwards-compatible additions
    pub minor: u32,
}

impl SchemaVersion {
    /// Current schema version
    pub const CURRENT: Self = Self { major: 1, minor: 0 };

    /// Create a new version
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// Check if this version is compatible with the expected major version
    pub fn is_compatible(&self, expected_major: u32) -> bool {
        self.major == expected_major
    }

    /// Check if this version is newer than another
    pub fn is_newer_than(&self, other: &SchemaVersion) -> bool {
        self.major > other.major || (self.major == other.major && self.minor > other.minor)
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::CURRENT
    }
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_current() {
        let v = SchemaVersion::CURRENT;
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
    }

    #[test]
    fn test_version_compatible() {
        let v = SchemaVersion::new(1, 5);
        assert!(v.is_compatible(1));
        assert!(!v.is_compatible(2));
    }

    #[test]
    fn test_version_newer() {
        let v1 = SchemaVersion::new(1, 0);
        let v2 = SchemaVersion::new(1, 5);
        let v3 = SchemaVersion::new(2, 0);

        assert!(!v1.is_newer_than(&v2));
        assert!(v2.is_newer_than(&v1));
        assert!(v3.is_newer_than(&v2));
    }

    #[test]
    fn test_version_display() {
        let v = SchemaVersion::new(2, 3);
        assert_eq!(format!("{}", v), "2.3");
    }
}
