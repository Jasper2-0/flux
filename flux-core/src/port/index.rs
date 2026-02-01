//! Type-safe port index wrappers
//!
//! These newtypes provide compile-time safety by distinguishing between
//! input and output port indices. This prevents accidentally passing an
//! output index to a function expecting an input index.
//!
//! # Example
//!
//! ```
//! use flux_core::port::{InputIndex, OutputIndex};
//!
//! fn get_input_meta(index: InputIndex) {
//!     // Can only be called with InputIndex
//! }
//!
//! fn get_output_meta(index: OutputIndex) {
//!     // Can only be called with OutputIndex
//! }
//!
//! // Compile-time safety:
//! get_input_meta(InputIndex(0));   // OK
//! get_output_meta(OutputIndex(0)); // OK
//! // get_input_meta(OutputIndex(0));  // ERROR: type mismatch
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Type-safe wrapper for input port indices.
///
/// Use this instead of raw `usize` for input port indices to get compile-time
/// safety. The wrapper is zero-cost at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct InputIndex(pub usize);

impl InputIndex {
    /// Create a new input index.
    #[inline]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Get the raw index value.
    #[inline]
    pub const fn get(self) -> usize {
        self.0
    }
}

impl From<usize> for InputIndex {
    #[inline]
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl From<InputIndex> for usize {
    #[inline]
    fn from(index: InputIndex) -> Self {
        index.0
    }
}

impl fmt::Display for InputIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "input[{}]", self.0)
    }
}

/// Type-safe wrapper for output port indices.
///
/// Use this instead of raw `usize` for output port indices to get compile-time
/// safety. The wrapper is zero-cost at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct OutputIndex(pub usize);

impl OutputIndex {
    /// Create a new output index.
    #[inline]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Get the raw index value.
    #[inline]
    pub const fn get(self) -> usize {
        self.0
    }
}

impl From<usize> for OutputIndex {
    #[inline]
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl From<OutputIndex> for usize {
    #[inline]
    fn from(index: OutputIndex) -> Self {
        index.0
    }
}

impl fmt::Display for OutputIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "output[{}]", self.0)
    }
}

/// Type-safe wrapper for trigger input indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct TriggerInputIndex(pub usize);

impl TriggerInputIndex {
    #[inline]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn get(self) -> usize {
        self.0
    }
}

impl From<usize> for TriggerInputIndex {
    #[inline]
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl From<TriggerInputIndex> for usize {
    #[inline]
    fn from(index: TriggerInputIndex) -> Self {
        index.0
    }
}

impl fmt::Display for TriggerInputIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "trigger_in[{}]", self.0)
    }
}

/// Type-safe wrapper for trigger output indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct TriggerOutputIndex(pub usize);

impl TriggerOutputIndex {
    #[inline]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn get(self) -> usize {
        self.0
    }
}

impl From<usize> for TriggerOutputIndex {
    #[inline]
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl From<TriggerOutputIndex> for usize {
    #[inline]
    fn from(index: TriggerOutputIndex) -> Self {
        index.0
    }
}

impl fmt::Display for TriggerOutputIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "trigger_out[{}]", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_index_creation() {
        let idx = InputIndex::new(5);
        assert_eq!(idx.get(), 5);
        assert_eq!(idx.0, 5);
    }

    #[test]
    fn test_output_index_creation() {
        let idx = OutputIndex::new(3);
        assert_eq!(idx.get(), 3);
    }

    #[test]
    fn test_from_usize() {
        let input: InputIndex = 2.into();
        let output: OutputIndex = 4.into();
        assert_eq!(input.get(), 2);
        assert_eq!(output.get(), 4);
    }

    #[test]
    fn test_to_usize() {
        let input = InputIndex(7);
        let output = OutputIndex(9);
        let i: usize = input.into();
        let o: usize = output.into();
        assert_eq!(i, 7);
        assert_eq!(o, 9);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", InputIndex(3)), "input[3]");
        assert_eq!(format!("{}", OutputIndex(5)), "output[5]");
    }

    #[test]
    fn test_comparison() {
        assert!(InputIndex(1) < InputIndex(2));
        assert!(OutputIndex(5) > OutputIndex(3));
        assert_eq!(InputIndex(0), InputIndex(0));
    }

    #[test]
    fn test_zero_cost() {
        // Verify the types have the same size as usize
        assert_eq!(std::mem::size_of::<InputIndex>(), std::mem::size_of::<usize>());
        assert_eq!(std::mem::size_of::<OutputIndex>(), std::mem::size_of::<usize>());
    }
}
