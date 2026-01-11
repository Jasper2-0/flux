//! Serialization module for the operator system
//!
//! This module provides Rust-native serialization formats for projects,
//! symbols, graphs, and animations. The format is designed to be:
//!
//! - Human-readable (JSON-based)
//! - Version-aware for future migrations
//! - Clean separation of symbol definitions vs project data
//!
//! ## File Types
//!
//! | Extension | Purpose |
//! |-----------|---------|
//! | `.rproj` | Project configuration |
//! | `.rsym` | Symbol definition |
//! | `.rgraph` | Graph/composition |
//!
//! ## Example: Creating a Symbol File
//!
//! ```ignore
//! use flux_graph::serialization::{SymbolFile, SymbolDef, InputDef, OutputDef};
//!
//! let mut symbol = SymbolDef::new("MyEffect")
//!     .with_category("Effects")
//!     .with_description("A custom effect");
//!
//! symbol.add_input(InputDef::float("Intensity", 1.0).with_range(0.0, 2.0));
//! symbol.add_output(OutputDef::float("Result"));
//!
//! let file = SymbolFile::from_def(symbol);
//! // let json = flux_graph::serialization::io::save_symbol_str(&file).unwrap();
//! ```

pub mod animation;
pub mod error;
pub mod graph;
pub mod io;
pub mod library;
pub mod project;
pub mod symbol;
pub mod version;

// Re-export main types
pub use animation::{AnimationDef, CurveDef, ExtrapolationMode, InterpolationMode, KeyframeDef, TangentDef};
pub use error::{Result, SerializationError};
pub use graph::{
    GraphDef, GraphFile, InputOverride, InstanceOverride, PlaybackDef, PortUiOverride, ViewDef,
};
pub use io::{
    load_graph, load_graph_str, load_project, load_project_str, load_symbol, load_symbol_str,
    save_graph, save_graph_str, save_project, save_project_str, save_symbol, save_symbol_str,
    FileType,
};
pub use library::{LoadError, LoadResult, SymbolLibrary};
pub use project::{ProjectFile, ProjectMeta, ResourceConfig};
pub use symbol::{
    ChildDef, ConnectionDef, InputDef, InputUiMeta, InputValueDef, OutputDef, SymbolDef,
    SymbolFile, SymbolUiMeta,
};
pub use version::SchemaVersion;
