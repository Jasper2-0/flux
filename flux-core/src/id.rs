//! Unique identifiers for the Flux system

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier using UUID v4
///
/// Used to identify symbols, instances, slots, and other entities
/// throughout the operator system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id(Uuid);

impl Id {
    /// Create a new random UUID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Parse from string (e.g., "550e8400-e29b-41d4-a716-446655440000")
    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Check if this is the nil UUID
    pub fn is_nil(&self) -> bool {
        self.0.is_nil()
    }

    /// The nil/empty UUID (all zeros)
    pub const NIL: Self = Self(Uuid::nil());
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for Id {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_new_is_unique() {
        let id1 = Id::new();
        let id2 = Id::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_id_parse_roundtrip() {
        let original = "550e8400-e29b-41d4-a716-446655440000";
        let id = Id::parse(original).unwrap();
        let formatted = id.to_string();
        assert_eq!(formatted, original);
    }

    #[test]
    fn test_id_parse_invalid() {
        assert!(Id::parse("not-a-uuid").is_err());
        assert!(Id::parse("").is_err());
    }

    #[test]
    fn test_id_nil() {
        assert!(Id::NIL.is_nil());
        assert!(!Id::new().is_nil());
    }

    #[test]
    fn test_id_serialize() {
        let id = Id::parse("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"550e8400-e29b-41d4-a716-446655440000\"");

        let deserialized: Id = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = Id::from(uuid);
        assert_eq!(id.as_uuid(), &uuid);
    }
}
