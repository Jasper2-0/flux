use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use flux_core::id::Id;

use super::Symbol;

/// Registry for managing Symbol definitions
///
/// The registry stores all symbol definitions and provides lookup
/// by ID or name. It's thread-safe for concurrent access.
#[derive(Default)]
pub struct SymbolRegistry {
    /// Symbols indexed by ID
    by_id: RwLock<HashMap<Id, Arc<Symbol>>>,
    /// Symbol IDs indexed by name
    by_name: RwLock<HashMap<String, Id>>,
}

impl SymbolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a symbol
    pub fn register(&self, symbol: Symbol) -> Id {
        let id = symbol.id;
        let name = symbol.name.clone();
        let symbol = Arc::new(symbol);

        self.by_id.write().unwrap().insert(id, symbol);
        self.by_name.write().unwrap().insert(name, id);

        id
    }

    /// Unregister a symbol
    pub fn unregister(&self, id: Id) -> Option<Arc<Symbol>> {
        let symbol = self.by_id.write().unwrap().remove(&id)?;
        self.by_name.write().unwrap().remove(&symbol.name);
        Some(symbol)
    }

    /// Get a symbol by ID
    pub fn get(&self, id: Id) -> Option<Arc<Symbol>> {
        self.by_id.read().unwrap().get(&id).cloned()
    }

    /// Get a symbol by name
    pub fn get_by_name(&self, name: &str) -> Option<Arc<Symbol>> {
        let id = self.by_name.read().unwrap().get(name).copied()?;
        self.get(id)
    }

    /// Get symbol ID by name
    pub fn get_id(&self, name: &str) -> Option<Id> {
        self.by_name.read().unwrap().get(name).copied()
    }

    /// Check if a symbol exists
    pub fn contains(&self, id: Id) -> bool {
        self.by_id.read().unwrap().contains_key(&id)
    }

    /// Check if a symbol exists by name
    pub fn contains_name(&self, name: &str) -> bool {
        self.by_name.read().unwrap().contains_key(name)
    }

    /// Get all symbol IDs
    pub fn ids(&self) -> Vec<Id> {
        self.by_id.read().unwrap().keys().copied().collect()
    }

    /// Get all symbol names
    pub fn names(&self) -> Vec<String> {
        self.by_name.read().unwrap().keys().cloned().collect()
    }

    /// Get the number of registered symbols
    pub fn len(&self) -> usize {
        self.by_id.read().unwrap().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.by_id.read().unwrap().is_empty()
    }

    /// Get symbols by category
    pub fn get_by_category(&self, category: &str) -> Vec<Arc<Symbol>> {
        self.by_id
            .read()
            .unwrap()
            .values()
            .filter(|s| s.category.as_deref() == Some(category))
            .cloned()
            .collect()
    }

    /// Get all categories
    pub fn categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .by_id
            .read()
            .unwrap()
            .values()
            .filter_map(|s| s.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }

    /// Clear all symbols
    pub fn clear(&self) {
        self.by_id.write().unwrap().clear();
        self.by_name.write().unwrap().clear();
    }
}

impl std::fmt::Debug for SymbolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolRegistry")
            .field("count", &self.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::{InputDefinition, OutputDefinition};

    fn make_test_symbol(name: &str, category: &str) -> Symbol {
        let mut symbol = Symbol::new(name).with_category(category);
        symbol.add_input(InputDefinition::float("Input", 0.0));
        symbol.add_output(OutputDefinition::float("Output"));
        symbol
    }

    #[test]
    fn test_registry_register() {
        let registry = SymbolRegistry::new();

        let symbol = make_test_symbol("Add", "Math");
        let id = registry.register(symbol);

        assert!(registry.contains(id));
        assert!(registry.contains_name("Add"));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_lookup() {
        let registry = SymbolRegistry::new();

        let symbol = make_test_symbol("Multiply", "Math");
        let id = registry.register(symbol);

        let found = registry.get(id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Multiply");

        let found_by_name = registry.get_by_name("Multiply");
        assert!(found_by_name.is_some());
        assert_eq!(found_by_name.unwrap().id, id);
    }

    #[test]
    fn test_registry_unregister() {
        let registry = SymbolRegistry::new();

        let symbol = make_test_symbol("Test", "Test");
        let id = registry.register(symbol);

        assert!(registry.contains(id));

        let removed = registry.unregister(id);
        assert!(removed.is_some());
        assert!(!registry.contains(id));
        assert!(!registry.contains_name("Test"));
    }

    #[test]
    fn test_registry_categories() {
        let registry = SymbolRegistry::new();

        registry.register(make_test_symbol("Add", "Math"));
        registry.register(make_test_symbol("Sub", "Math"));
        registry.register(make_test_symbol("And", "Logic"));

        let categories = registry.categories();
        assert_eq!(categories.len(), 2);
        assert!(categories.contains(&"Math".to_string()));
        assert!(categories.contains(&"Logic".to_string()));

        let math_symbols = registry.get_by_category("Math");
        assert_eq!(math_symbols.len(), 2);
    }

    #[test]
    fn test_registry_names_and_ids() {
        let registry = SymbolRegistry::new();

        registry.register(make_test_symbol("A", "Test"));
        registry.register(make_test_symbol("B", "Test"));
        registry.register(make_test_symbol("C", "Test"));

        let names = registry.names();
        assert_eq!(names.len(), 3);

        let ids = registry.ids();
        assert_eq!(ids.len(), 3);
    }
}
