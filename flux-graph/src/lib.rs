//! Flux Graph - Graph execution and serialization
//!
//! This crate provides the graph execution engine, symbol system, and serialization.
//!
//! # Modules
//!
//! - [`graph`] - The main graph structure for connecting and evaluating operators
//! - [`associated`] - Associated graph wrapper for external ID management
//! - [`bypass`] - Bypass state management for disabled nodes
//! - [`composite`] - Composite operators (nested graphs)
//! - [`conversion`] - Type conversion operators (auto-inserted by graph)
//! - [`slot_ref`] - Slot references for input/output connections
//! - [`instance_path`] - Path tracking for nested operator instances
//! - [`symbol`] - Symbol table for operator definitions
//! - [`animation`] - Keyframe animation system
//! - [`serialization`] - Graph serialization to/from JSON
//! - [`resource`] - Resource management (textures, meshes, etc.)
//! - [`playback`] - Audio and timeline playback

pub mod animation;
pub mod associated;
pub mod bypass;
pub mod commands;
pub mod compiler;
pub mod composite;
pub mod conversion;
pub mod graph;
pub mod instance_path;
pub mod playback;
pub mod serialization;
pub mod slot_ref;
pub mod symbol;
pub mod undo;

// Re-export main types
pub use associated::{AssociatedGraph, NodeHandle};
pub use bypass::{Bypassable, BypassableType, BypassInfo, BypassState};
pub use commands::{
    AddNodeCommand, Command, ConnectCommand, DisconnectCommand, MacroCommand, RemoveNodeCommand,
    SetInputDefaultCommand,
};
pub use compiler::CompiledGraph;
pub use composite::CompositeOp;
pub use conversion::ConversionOp;
pub use graph::{Connection, Graph, GraphEvent, GraphStats};
pub use instance_path::InstancePath;
pub use slot_ref::SlotRef;
pub use undo::UndoRedoStack;
