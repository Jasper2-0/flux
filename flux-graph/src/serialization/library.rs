//! Symbol library for managing and caching symbol definitions
//!
//! The library loads symbols from disk and provides lookup by ID or name.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use flux_core::Id;

use super::symbol::{SymbolDef, SymbolFile};
use super::io;

/// Error encountered while loading a symbol file
#[derive(Debug)]
pub struct LoadError {
    /// Path that failed to load
    pub path: PathBuf,
    /// Error message
    pub message: String,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.message)
    }
}

/// Result of loading symbols, including both successes and failures
#[derive(Debug)]
pub struct LoadResult {
    /// Number of successfully loaded symbols
    pub loaded: usize,
    /// Errors encountered during loading
    pub errors: Vec<LoadError>,
}

/// Manages loading and caching of symbol definitions
pub struct SymbolLibrary {
    /// Loaded symbols by ID
    symbols: HashMap<Id, SymbolFile>,
    /// Symbol ID lookup by name
    name_index: HashMap<String, Id>,
    /// Search paths for symbol files
    search_paths: Vec<PathBuf>,
    /// Built-in symbols (always available)
    builtins: HashMap<Id, SymbolFile>,
}

impl SymbolLibrary {
    /// Create a new empty symbol library
    pub fn new() -> Self {
        let mut lib = Self {
            symbols: HashMap::new(),
            name_index: HashMap::new(),
            search_paths: Vec::new(),
            builtins: HashMap::new(),
        };
        lib.register_builtins();
        lib
    }

    /// Add a search path for symbol files
    ///
    /// The path is canonicalized to prevent path traversal attacks.
    /// Duplicate paths are ignored.
    ///
    /// Returns `true` if the path was added, `false` if it was a duplicate
    /// or couldn't be canonicalized.
    pub fn add_search_path(&mut self, path: impl AsRef<Path>) -> bool {
        // Canonicalize path to resolve symlinks and relative paths
        let canonical = match std::fs::canonicalize(path.as_ref()) {
            Ok(p) => p,
            Err(_) => {
                // If path doesn't exist yet, just use the provided path
                // but convert to absolute to prevent traversal
                if path.as_ref().is_absolute() {
                    path.as_ref().to_path_buf()
                } else {
                    // Reject relative paths that don't exist
                    return false;
                }
            }
        };

        // Check for duplicates
        if self.search_paths.contains(&canonical) {
            return false;
        }

        self.search_paths.push(canonical);
        true
    }

    /// Load all symbols from search paths
    ///
    /// Returns a `LoadResult` containing both the count of successfully loaded
    /// symbols and any errors encountered during loading.
    pub fn load_all(&mut self) -> LoadResult {
        let mut result = LoadResult {
            loaded: 0,
            errors: Vec::new(),
        };
        for path in self.search_paths.clone() {
            let dir_result = self.load_from_directory(&path);
            result.loaded += dir_result.loaded;
            result.errors.extend(dir_result.errors);
        }
        result
    }

    /// Load symbols from a directory (recursive)
    fn load_from_directory(&mut self, dir: &Path) -> LoadResult {
        let mut result = LoadResult {
            loaded: 0,
            errors: Vec::new(),
        };

        if !dir.exists() {
            return result;
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                result.errors.push(LoadError {
                    path: dir.to_path_buf(),
                    message: format!("Failed to read directory: {}", e),
                });
                return result;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    result.errors.push(LoadError {
                        path: dir.to_path_buf(),
                        message: format!("Failed to read entry: {}", e),
                    });
                    continue;
                }
            };

            let path = entry.path();

            if path.is_dir() {
                let sub_result = self.load_from_directory(&path);
                result.loaded += sub_result.loaded;
                result.errors.extend(sub_result.errors);
            } else if path.extension().map(|e| e == "rsym").unwrap_or(false) {
                match io::load_symbol(&path) {
                    Ok(symbol) => {
                        self.register(symbol);
                        result.loaded += 1;
                    }
                    Err(e) => {
                        result.errors.push(LoadError {
                            path: path.clone(),
                            message: e.to_string(),
                        });
                    }
                }
            }
        }

        result
    }

    /// Register a symbol
    pub fn register(&mut self, symbol: SymbolFile) {
        let id = symbol.symbol.id;
        let name = symbol.symbol.name.clone();
        self.name_index.insert(name, id);
        self.symbols.insert(id, symbol);
    }

    /// Unregister a symbol by ID
    pub fn unregister(&mut self, id: Id) -> Option<SymbolFile> {
        if let Some(symbol) = self.symbols.remove(&id) {
            self.name_index.remove(&symbol.symbol.name);
            Some(symbol)
        } else {
            None
        }
    }

    /// Get a symbol by ID
    pub fn get(&self, id: Id) -> Option<&SymbolFile> {
        self.symbols.get(&id).or_else(|| self.builtins.get(&id))
    }

    /// Get a symbol definition by ID
    pub fn get_def(&self, id: Id) -> Option<&SymbolDef> {
        self.get(id).map(|f| &f.symbol)
    }

    /// Get a symbol by name
    pub fn get_by_name(&self, name: &str) -> Option<&SymbolFile> {
        // Check for builtin prefix
        if let Some(builtin_name) = name.strip_prefix("builtin:") {
            return self
                .builtins
                .values()
                .find(|s| s.symbol.name == builtin_name);
        }

        self.name_index.get(name).and_then(|id| self.get(*id))
    }

    /// Get ID by name
    pub fn get_id_by_name(&self, name: &str) -> Option<Id> {
        if let Some(builtin_name) = name.strip_prefix("builtin:") {
            return self
                .builtins
                .values()
                .find(|s| s.symbol.name == builtin_name)
                .map(|s| s.symbol.id);
        }
        self.name_index.get(name).copied()
    }

    /// Check if a symbol exists
    pub fn contains(&self, id: Id) -> bool {
        self.symbols.contains_key(&id) || self.builtins.contains_key(&id)
    }

    /// Check if a symbol name exists
    pub fn contains_name(&self, name: &str) -> bool {
        self.get_by_name(name).is_some()
    }

    /// List all available symbols
    pub fn list(&self) -> Vec<&SymbolDef> {
        self.symbols
            .values()
            .chain(self.builtins.values())
            .map(|f| &f.symbol)
            .collect()
    }

    /// List symbols by category
    pub fn list_by_category(&self, category: &str) -> Vec<&SymbolDef> {
        self.list()
            .into_iter()
            .filter(|s| s.category.as_deref() == Some(category))
            .collect()
    }

    /// List all categories
    pub fn categories(&self) -> Vec<String> {
        let mut categories: Vec<_> = self
            .list()
            .into_iter()
            .filter_map(|s| s.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }

    /// Search symbols by tag
    pub fn search_by_tag(&self, tag: &str) -> Vec<&SymbolDef> {
        self.list()
            .into_iter()
            .filter(|s| s.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Search symbols by name (partial match)
    pub fn search(&self, query: &str) -> Vec<&SymbolDef> {
        let query_lower = query.to_lowercase();
        self.list()
            .into_iter()
            .filter(|s| s.name.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Number of loaded symbols (excluding builtins)
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if library is empty (excluding builtins)
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Number of builtin symbols
    pub fn builtin_count(&self) -> usize {
        self.builtins.len()
    }

    /// Register built-in operator symbols
    fn register_builtins(&mut self) {
        use flux_core::value::{Color, ValueType};
        use super::symbol::{InputDef, OutputDef};

        // Add - adds two floats
        let mut add = SymbolDef::new("add")
            .with_category("Math")
            .with_description("Adds two values");
        add.add_input(InputDef::float("A", 0.0));
        add.add_input(InputDef::float("B", 0.0));
        add.add_output(OutputDef::float("Result"));
        self.register_builtin(add);

        // Multiply - multiplies two floats
        let mut multiply = SymbolDef::new("multiply")
            .with_category("Math")
            .with_description("Multiplies two values");
        multiply.add_input(InputDef::float("A", 0.0));
        multiply.add_input(InputDef::float("B", 1.0));
        multiply.add_output(OutputDef::float("Result"));
        self.register_builtin(multiply);

        // Constant - outputs a constant float value
        let mut constant = SymbolDef::new("constant")
            .with_category("Math")
            .with_description("Outputs a constant value");
        constant.add_input(InputDef::float("Value", 0.0));
        constant.add_output(OutputDef::float("Output"));
        self.register_builtin(constant);

        // SineWave - time-based sine wave
        let mut sine = SymbolDef::new("sine_wave")
            .with_category("Animation")
            .with_description("Generates a sine wave based on time");
        sine.add_input(InputDef::float("Frequency", 1.0).with_range(0.0, 100.0));
        sine.add_input(InputDef::float("Amplitude", 1.0));
        sine.add_input(InputDef::float("Phase", 0.0));
        sine.add_output(OutputDef::float("Value"));
        self.register_builtin(sine);

        // LerpColor - interpolate between colors
        let mut lerp_color = SymbolDef::new("lerp_color")
            .with_category("Color")
            .with_description("Linearly interpolates between two colors");
        lerp_color.add_input(InputDef::color("A", Color::BLACK));
        lerp_color.add_input(InputDef::color("B", Color::WHITE));
        lerp_color.add_input(InputDef::float("T", 0.5).with_range(0.0, 1.0));
        lerp_color.add_output(OutputDef::color("Result"));
        self.register_builtin(lerp_color);

        // Vec3Compose - create Vec3 from components
        let mut vec3_compose = SymbolDef::new("vec3_compose")
            .with_category("Vector")
            .with_description("Creates a Vec3 from X, Y, Z components");
        vec3_compose.add_input(InputDef::float("X", 0.0));
        vec3_compose.add_input(InputDef::float("Y", 0.0));
        vec3_compose.add_input(InputDef::float("Z", 0.0));
        vec3_compose.add_output(OutputDef::vec3("Vector"));
        self.register_builtin(vec3_compose);

        // Compare - compares two values
        let mut compare = SymbolDef::new("compare")
            .with_category("Logic")
            .with_description("Compares two values");
        compare.add_input(InputDef::float("A", 0.0));
        compare.add_input(InputDef::float("B", 0.0));
        compare.add_output(OutputDef::new("Result", ValueType::Bool));
        self.register_builtin(compare);

        // Sum - sums multiple inputs
        let mut sum = SymbolDef::new("sum")
            .with_category("Math")
            .with_description("Sums multiple input values");
        sum.add_input(InputDef::float("Values", 0.0).multi_input());
        sum.add_output(OutputDef::float("Sum"));
        self.register_builtin(sum);
    }

    /// Register a builtin symbol
    fn register_builtin(&mut self, def: SymbolDef) {
        let id = def.id;
        let file = SymbolFile::from_def(def);
        self.builtins.insert(id, file);
    }
}

impl Default for SymbolLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_new() {
        let lib = SymbolLibrary::new();
        assert!(lib.is_empty());
        assert!(lib.builtin_count() > 0);
    }

    #[test]
    fn test_library_builtins() {
        let lib = SymbolLibrary::new();

        // Should find builtin by prefixed name
        assert!(lib.get_by_name("builtin:add").is_some());
        assert!(lib.get_by_name("builtin:multiply").is_some());
        assert!(lib.get_by_name("builtin:sine_wave").is_some());
        assert!(lib.get_by_name("builtin:lerp_color").is_some());

        // Should not find non-existent builtins
        assert!(lib.get_by_name("builtin:nonexistent").is_none());
    }

    #[test]
    fn test_library_register() {
        let mut lib = SymbolLibrary::new();
        let symbol = SymbolFile::new("CustomSymbol");

        lib.register(symbol);

        assert!(lib.contains_name("CustomSymbol"));
        assert_eq!(lib.len(), 1);
    }

    #[test]
    fn test_library_categories() {
        let lib = SymbolLibrary::new();
        let categories = lib.categories();

        assert!(categories.contains(&"Math".to_string()));
        assert!(categories.contains(&"Color".to_string()));
    }

    #[test]
    fn test_library_list_by_category() {
        let lib = SymbolLibrary::new();
        let math_symbols = lib.list_by_category("Math");

        assert!(!math_symbols.is_empty());
        assert!(math_symbols.iter().any(|s| s.name == "add"));
    }

    #[test]
    fn test_library_search() {
        let lib = SymbolLibrary::new();
        let results = lib.search("sine");

        assert!(!results.is_empty());
        assert!(results.iter().any(|s| s.name.contains("sine")));
    }

    #[test]
    fn test_library_unregister() {
        let mut lib = SymbolLibrary::new();
        let symbol = SymbolFile::new("ToRemove");
        let id = symbol.symbol.id;

        lib.register(symbol);
        assert!(lib.contains(id));

        lib.unregister(id);
        assert!(!lib.contains(id));
    }
}
