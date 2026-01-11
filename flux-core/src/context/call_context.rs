//! Call context for context-aware caching
//!
//! The call context distinguishes between different evaluation contexts when the
//! same operator appears in multiple subroutine calls or loop iterations.
//!
//! # Problem
//!
//! Consider a subroutine that contains a shared operator. When called twice with
//! different inputs, the operator's cached result from the first call should NOT
//! be reused for the second call.
//!
//! # Solution
//!
//! Each evaluation context carries a `CallContext` identifier. Cache keys combine
//! the node ID with the call context, ensuring separate cache entries for each
//! invocation context.
//!
//! # Example
//!
//! ```
//! use flux_core::context::CallContext;
//!
//! let root = CallContext::default();
//! let first_call = root.child(0);
//! let second_call = root.child(1);
//!
//! // Different contexts produce different identifiers
//! assert_ne!(first_call, second_call);
//!
//! // Nested contexts work too
//! let nested = first_call.child(0);
//! assert_ne!(nested, first_call);
//! assert_ne!(nested, second_call);
//! ```

/// A context identifier for distinguishing operator evaluations in different
/// subroutine calls or loop iterations.
///
/// The context is a simple wrapped integer that can derive child contexts via
/// the [`child`](Self::child) method. This creates a unique identifier for each
/// nesting level without requiring heap allocation.
///
/// This pattern is based on Werkkzeug4's `CallId` system, which uses context-aware
/// caching to correctly handle shared operators in subroutines and loops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CallContext(u32);

impl CallContext {
    /// Create a root context with the default value.
    pub const fn root() -> Self {
        Self(0)
    }

    /// Create a child context for a subroutine call or loop iteration.
    ///
    /// Each call with a different `index` produces a unique context. The
    /// implementation uses wrapping arithmetic to avoid overflow while still
    /// producing distinct values for reasonable nesting depths.
    ///
    /// # Arguments
    ///
    /// * `index` - The child index (e.g., loop iteration number or call site ID)
    ///
    /// # Example
    ///
    /// ```
    /// use flux_core::context::CallContext;
    ///
    /// let ctx = CallContext::root();
    ///
    /// // Loop iterations get unique contexts
    /// let iter_0 = ctx.child(0);
    /// let iter_1 = ctx.child(1);
    /// assert_ne!(iter_0, iter_1);
    ///
    /// // Nested loops create deeper contexts
    /// let nested_0_0 = iter_0.child(0);
    /// let nested_0_1 = iter_0.child(1);
    /// assert_ne!(nested_0_0, nested_0_1);
    /// ```
    #[inline]
    pub const fn child(&self, index: u32) -> Self {
        // Use a multiplicative hash to spread child contexts
        // This ensures that child(0) from different parents are distinct
        Self(self.0.wrapping_mul(31).wrapping_add(index).wrapping_add(1))
    }

    /// Get the raw context value (for debugging or serialization).
    #[inline]
    pub const fn raw(&self) -> u32 {
        self.0
    }

    /// Create a context from a raw value (for deserialization).
    #[inline]
    pub const fn from_raw(value: u32) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_context() {
        let ctx = CallContext::root();
        assert_eq!(ctx.raw(), 0);
    }

    #[test]
    fn test_default_is_root() {
        assert_eq!(CallContext::default(), CallContext::root());
    }

    #[test]
    fn test_child_contexts_are_unique() {
        let root = CallContext::root();
        let child_0 = root.child(0);
        let child_1 = root.child(1);
        let child_2 = root.child(2);

        assert_ne!(child_0, root);
        assert_ne!(child_0, child_1);
        assert_ne!(child_1, child_2);
    }

    #[test]
    fn test_nested_contexts_are_unique() {
        let root = CallContext::root();
        let child_0 = root.child(0);
        let child_1 = root.child(1);

        let nested_0_0 = child_0.child(0);
        let nested_0_1 = child_0.child(1);
        let nested_1_0 = child_1.child(0);

        // All nested contexts should be unique
        assert_ne!(nested_0_0, nested_0_1);
        assert_ne!(nested_0_0, nested_1_0);
        assert_ne!(nested_0_1, nested_1_0);

        // Nested contexts should differ from their parents
        assert_ne!(nested_0_0, child_0);
        assert_ne!(nested_1_0, child_1);
    }

    #[test]
    fn test_hash_is_stable() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let ctx = CallContext::root().child(5).child(10);

        let mut hasher1 = DefaultHasher::new();
        ctx.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        ctx.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_roundtrip_raw() {
        let original = CallContext::root().child(42).child(7);
        let raw = original.raw();
        let restored = CallContext::from_raw(raw);
        assert_eq!(original, restored);
    }

    #[test]
    fn test_many_siblings() {
        let root = CallContext::root();
        let mut seen = std::collections::HashSet::new();

        // Generate 1000 sibling contexts and ensure they're all unique
        for i in 0..1000 {
            let ctx = root.child(i);
            assert!(seen.insert(ctx), "Duplicate context at index {}", i);
        }
    }
}
