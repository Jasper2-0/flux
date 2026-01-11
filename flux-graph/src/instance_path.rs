use serde::{Deserialize, Serialize};

use flux_core::id::Id;

/// Path through the instance hierarchy
///
/// When operators are nested (composite operators containing subgraphs),
/// this path identifies a specific instance by tracking the chain of
/// parent-child relationships from the root.
///
/// For example, if we have:
/// - RootGraph
///   - CompositeOp (id: A)
///     - SubComposite (id: B)
///       - TargetOp (id: C)
///
/// The path to TargetOp would be: [A, B, C]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstancePath {
    segments: Vec<Id>,
}

impl InstancePath {
    /// Create a path with a single root node
    pub fn root(id: Id) -> Self {
        Self { segments: vec![id] }
    }

    /// Create an empty path (represents the root graph itself)
    pub fn empty() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Create a path from a vector of IDs
    pub fn from_segments(segments: Vec<Id>) -> Self {
        Self { segments }
    }

    /// Get the depth of this path (number of segments)
    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    /// Check if this is an empty/root path
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Get the last segment (the immediate instance ID)
    pub fn leaf(&self) -> Option<Id> {
        self.segments.last().copied()
    }

    /// Get the first segment (the topmost ancestor)
    pub fn root_id(&self) -> Option<Id> {
        self.segments.first().copied()
    }

    /// Create a child path by appending a new ID
    pub fn child(&self, child_id: Id) -> Self {
        let mut segments = self.segments.clone();
        segments.push(child_id);
        Self { segments }
    }

    /// Get the parent path (removing the last segment)
    pub fn parent(&self) -> Option<Self> {
        if self.segments.len() > 1 {
            Some(Self {
                segments: self.segments[..self.segments.len() - 1].to_vec(),
            })
        } else {
            None
        }
    }

    /// Check if this path is an ancestor of another path
    pub fn is_ancestor_of(&self, other: &InstancePath) -> bool {
        if self.segments.len() >= other.segments.len() {
            return false;
        }
        self.segments == other.segments[..self.segments.len()]
    }

    /// Check if this path is a descendant of another path
    pub fn is_descendant_of(&self, other: &InstancePath) -> bool {
        other.is_ancestor_of(self)
    }

    /// Get the relative path from an ancestor to this path
    pub fn relative_to(&self, ancestor: &InstancePath) -> Option<Self> {
        if !self.is_descendant_of(ancestor) && self != ancestor {
            return None;
        }
        Some(Self {
            segments: self.segments[ancestor.segments.len()..].to_vec(),
        })
    }

    /// Get the common ancestor of two paths
    pub fn common_ancestor(&self, other: &InstancePath) -> Self {
        let mut common = Vec::new();
        for (a, b) in self.segments.iter().zip(other.segments.iter()) {
            if a == b {
                common.push(*a);
            } else {
                break;
            }
        }
        Self { segments: common }
    }

    /// Iterate over the segments
    pub fn iter(&self) -> impl Iterator<Item = &Id> {
        self.segments.iter()
    }

    /// Get the segments as a slice
    pub fn segments(&self) -> &[Id] {
        &self.segments
    }
}

impl std::fmt::Display for InstancePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.segments.is_empty() {
            write!(f, "<root>")
        } else {
            let parts: Vec<String> = self.segments.iter().map(|id| format!("{}", id)).collect();
            write!(f, "{}", parts.join("/"))
        }
    }
}

impl From<Id> for InstancePath {
    fn from(id: Id) -> Self {
        Self::root(id)
    }
}

impl FromIterator<Id> for InstancePath {
    fn from_iter<T: IntoIterator<Item = Id>>(iter: T) -> Self {
        Self {
            segments: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_hierarchy() {
        let a = Id::new();
        let b = Id::new();
        let c = Id::new();

        let root = InstancePath::root(a);
        let child = root.child(b);
        let grandchild = child.child(c);

        assert_eq!(root.depth(), 1);
        assert_eq!(child.depth(), 2);
        assert_eq!(grandchild.depth(), 3);

        assert!(root.is_ancestor_of(&child));
        assert!(root.is_ancestor_of(&grandchild));
        assert!(child.is_ancestor_of(&grandchild));
        assert!(!grandchild.is_ancestor_of(&root));

        assert_eq!(child.parent(), Some(root.clone()));
        assert_eq!(grandchild.parent(), Some(child.clone()));
    }

    #[test]
    fn test_relative_path() {
        let a = Id::new();
        let b = Id::new();
        let c = Id::new();

        let root = InstancePath::root(a);
        let grandchild = root.child(b).child(c);

        let relative = grandchild.relative_to(&root);
        assert!(relative.is_some());
        assert_eq!(relative.unwrap().depth(), 2);
    }

    #[test]
    fn test_common_ancestor() {
        let a = Id::new();
        let b = Id::new();
        let c = Id::new();
        let d = Id::new();

        let path1 = InstancePath::from_segments(vec![a, b, c]);
        let path2 = InstancePath::from_segments(vec![a, b, d]);

        let common = path1.common_ancestor(&path2);
        assert_eq!(common.depth(), 2);
        assert_eq!(common.segments(), &[a, b]);
    }
}
