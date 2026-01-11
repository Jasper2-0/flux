use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use flux_core::id::Id;
use flux_core::operator::Operator;
use flux_core::operator_meta::PortMeta;

/// Result of creating an operator: the operator and its input port metadata.
///
/// The `PortMeta` must be captured at creation time because `OperatorMeta` is not
/// object-safe and cannot be accessed through `Box<dyn Operator>`.
pub type OperatorWithMeta = (Box<dyn Operator>, Vec<Option<PortMeta>>);

/// Factory function type for creating operators with metadata capture.
pub type MetaCapturingFactory = Arc<dyn Fn() -> OperatorWithMeta + Send + Sync>;

/// Factory function type for creating operators with parameters and metadata capture.
pub type ParameterizedMetaFactory =
    Arc<dyn Fn(&OperatorParams) -> OperatorWithMeta + Send + Sync>;

/// Factory function type for creating operators (simple version, no metadata).
///
/// Deprecated: prefer `MetaCapturingFactory` for new code.
pub type SimpleOperatorFactory = Arc<dyn Fn() -> Box<dyn Operator> + Send + Sync>;

/// Factory function type for creating operators with parameters (no metadata).
///
/// Deprecated: prefer `ParameterizedMetaFactory` for new code.
pub type ParameterizedFactory = Arc<dyn Fn(&OperatorParams) -> Box<dyn Operator> + Send + Sync>;

/// Metadata about a registered operator type for dynamic creation
#[derive(Clone)]
pub struct RegistryEntry {
    pub type_id: Id,
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
}

/// Metadata about an operator parameter
#[derive(Debug, Clone)]
pub struct ParameterMeta {
    /// Parameter name used in OperatorParams
    pub name: &'static str,
    /// Type of the parameter
    pub param_type: ParameterType,
    /// Default value
    pub default: ParameterValue,
}

/// Type of an operator parameter
#[derive(Debug, Clone)]
pub enum ParameterType {
    /// Float with optional min/max range
    Float { min: Option<f32>, max: Option<f32> },
    /// Integer with optional min/max range
    Int { min: Option<i32>, max: Option<i32> },
    /// Boolean
    Bool,
    /// Enum with named variants
    Enum { variants: Vec<&'static str> },
}

/// Value for an operator parameter
#[derive(Debug, Clone)]
pub enum ParameterValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    Enum(&'static str),
}

impl ParameterValue {
    pub fn as_float(&self) -> Option<f32> {
        match self {
            ParameterValue::Float(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            ParameterValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ParameterValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_enum(&self) -> Option<&'static str> {
        match self {
            ParameterValue::Enum(v) => Some(v),
            _ => None,
        }
    }
}

/// Parameters for creating an operator
#[derive(Debug, Clone, Default)]
pub struct OperatorParams {
    values: HashMap<&'static str, ParameterValue>,
}

impl OperatorParams {
    /// Create a new empty parameter set
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a parameter value (builder pattern)
    pub fn set(mut self, name: &'static str, value: ParameterValue) -> Self {
        self.values.insert(name, value);
        self
    }

    /// Get a parameter value
    pub fn get(&self, name: &str) -> Option<&ParameterValue> {
        self.values.get(name)
    }

    /// Get a float parameter with fallback to default
    pub fn get_float(&self, name: &str, default: f32) -> f32 {
        self.values
            .get(name)
            .and_then(|v| v.as_float())
            .unwrap_or(default)
    }

    /// Get an int parameter with fallback to default
    pub fn get_int(&self, name: &str, default: i32) -> i32 {
        self.values
            .get(name)
            .and_then(|v| v.as_int())
            .unwrap_or(default)
    }

    /// Get a bool parameter with fallback to default
    pub fn get_bool(&self, name: &str, default: bool) -> bool {
        self.values
            .get(name)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    /// Get an enum parameter with fallback to default
    pub fn get_enum(&self, name: &str, default: &'static str) -> &'static str {
        self.values
            .get(name)
            .and_then(|v| v.as_enum())
            .unwrap_or(default)
    }
}

/// Extended metadata for operators with parameters
#[derive(Clone)]
pub struct ExtendedEntry {
    pub meta: RegistryEntry,
    /// Parameter metadata for operators that require configuration
    pub parameters: Vec<ParameterMeta>,
}

/// Registration entry for an operator
struct Registration {
    entry: ExtendedEntry,
    /// Factory that creates operator with captured PortMeta
    factory: MetaCapturingFactory,
    /// Optional factory that accepts parameters
    param_factory: Option<ParameterizedMetaFactory>,
}

/// Global registry of all operator types
///
/// The registry allows dynamic operator creation by name or type ID.
/// This enables loading graphs from JSON and instantiating the correct operator types.
///
/// # Examples
///
/// ```ignore
/// let registry = OperatorRegistry::new();
///
/// // Register a simple operator
/// registry.register(
///     RegistryEntry { type_id: Id::new(), name: "Add", category: "Math", description: "..." },
///     || Box::new(AddOp::new()),
/// );
///
/// // Register an operator with parameters
/// registry.register_with_params(
///     RegistryEntry { type_id: Id::new(), name: "Compare", category: "Logic", description: "..." },
///     || Box::new(CompareOp::new(CompareMode::Equal)),
///     |params| {
///         let mode = match params.get_enum("mode", "Equal") {
///             "LessThan" => CompareMode::LessThan,
///             "GreaterThan" => CompareMode::GreaterThan,
///             _ => CompareMode::Equal,
///         };
///         Box::new(CompareOp::new(mode))
///     },
///     vec![ParameterMeta {
///         name: "mode",
///         param_type: ParameterType::Enum { variants: vec!["Equal", "LessThan", "GreaterThan"] },
///         default: ParameterValue::Enum("Equal"),
///     }],
/// );
///
/// // Create operators
/// let add = registry.create_by_name("Add");
/// let compare = registry.create_with_params("Compare", OperatorParams::new()
///     .set("mode", ParameterValue::Enum("GreaterThan")));
///
/// // List by category
/// for (category, entries) in registry.by_category() {
///     println!("{}:", category);
///     for entry in entries {
///         println!("  - {}", entry.meta.name);
///     }
/// }
/// ```
pub struct OperatorRegistry {
    /// Registrations by type ID
    by_id: RwLock<HashMap<Id, Registration>>,
    /// Lookup by name for convenience
    by_name: RwLock<HashMap<&'static str, Id>>,
}

/// Backward-compatible type alias
pub type OperatorFactory = SimpleOperatorFactory;

impl OperatorRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            by_id: RwLock::new(HashMap::new()),
            by_name: RwLock::new(HashMap::new()),
        }
    }

    /// Register an operator type with the registry.
    ///
    /// The factory function must return `(Box<dyn Operator>, Vec<Option<PortMeta>>)`
    /// to capture input port metadata before boxing.
    ///
    /// Use the `capture!` helper macro or call `capture_meta()` to create the factory.
    pub fn register<F>(&self, meta: RegistryEntry, factory: F)
    where
        F: Fn() -> OperatorWithMeta + Send + Sync + 'static,
    {
        let type_id = meta.type_id;
        let name = meta.name;

        let registration = Registration {
            entry: ExtendedEntry {
                meta,
                parameters: Vec::new(),
            },
            factory: Arc::new(factory),
            param_factory: None,
        };

        self.by_id.write().unwrap().insert(type_id, registration);
        self.by_name.write().unwrap().insert(name, type_id);
    }

    /// Register an operator with parameter support.
    ///
    /// Both factory functions must return `(Box<dyn Operator>, Vec<Option<PortMeta>>)`
    /// to capture input port metadata before boxing.
    pub fn register_with_params<F, P>(
        &self,
        meta: RegistryEntry,
        factory: F,
        param_factory: P,
        parameters: Vec<ParameterMeta>,
    ) where
        F: Fn() -> OperatorWithMeta + Send + Sync + 'static,
        P: Fn(&OperatorParams) -> OperatorWithMeta + Send + Sync + 'static,
    {
        let type_id = meta.type_id;
        let name = meta.name;

        let registration = Registration {
            entry: ExtendedEntry { meta, parameters },
            factory: Arc::new(factory),
            param_factory: Some(Arc::new(param_factory)),
        };

        self.by_id.write().unwrap().insert(type_id, registration);
        self.by_name.write().unwrap().insert(name, type_id);
    }

    /// Register an operator using a simpler interface.
    ///
    /// Note: This doesn't capture PortMeta. Use `register()` with `capture_meta()`
    /// for operators that need input port metadata (ranges, labels, etc).
    pub fn register_simple<F>(&self, name: &'static str, factory: F)
    where
        F: Fn() -> Box<dyn Operator> + Send + Sync + 'static,
    {
        let meta = RegistryEntry {
            type_id: Id::new(),
            name,
            category: "Uncategorized",
            description: "",
        };
        // Wrap in metadata-capturing factory with empty metadata
        self.register(meta, move || (factory(), Vec::new()));
    }

    /// Create an operator instance by type ID with default parameters.
    ///
    /// Returns the operator without port metadata. For UI integration that needs
    /// port metadata (ranges, labels), use `create_with_meta_by_id()` instead.
    pub fn create_by_id(&self, type_id: Id) -> Option<Box<dyn Operator>> {
        self.create_with_meta_by_id(type_id).map(|(op, _)| op)
    }

    /// Create an operator instance by name with default parameters.
    ///
    /// Returns the operator without port metadata. For UI integration that needs
    /// port metadata (ranges, labels), use `create_with_meta_by_name()` instead.
    pub fn create_by_name(&self, name: &str) -> Option<Box<dyn Operator>> {
        let type_id = self.by_name.read().unwrap().get(name).copied()?;
        self.create_by_id(type_id)
    }

    /// Create an operator with captured port metadata by type ID.
    ///
    /// Returns `(operator, input_port_metadata)` for UI integration.
    pub fn create_with_meta_by_id(&self, type_id: Id) -> Option<OperatorWithMeta> {
        self.by_id
            .read()
            .unwrap()
            .get(&type_id)
            .map(|reg| (reg.factory)())
    }

    /// Create an operator with captured port metadata by name.
    ///
    /// Returns `(operator, input_port_metadata)` for UI integration.
    pub fn create_with_meta_by_name(&self, name: &str) -> Option<OperatorWithMeta> {
        let type_id = self.by_name.read().unwrap().get(name).copied()?;
        self.create_with_meta_by_id(type_id)
    }

    /// Create an operator instance by type ID with custom parameters.
    ///
    /// If the operator doesn't support parameters, uses the default factory.
    /// Returns the operator without port metadata.
    pub fn create_with_params_by_id(
        &self,
        type_id: Id,
        params: &OperatorParams,
    ) -> Option<Box<dyn Operator>> {
        self.create_with_meta_and_params_by_id(type_id, params)
            .map(|(op, _)| op)
    }

    /// Create an operator instance by name with custom parameters.
    ///
    /// If the operator doesn't support parameters, uses the default factory.
    /// Returns the operator without port metadata.
    pub fn create_with_params(
        &self,
        name: &str,
        params: &OperatorParams,
    ) -> Option<Box<dyn Operator>> {
        let type_id = self.by_name.read().unwrap().get(name).copied()?;
        self.create_with_params_by_id(type_id, params)
    }

    /// Create an operator with metadata by type ID with custom parameters.
    ///
    /// Returns `(operator, input_port_metadata)` for UI integration.
    pub fn create_with_meta_and_params_by_id(
        &self,
        type_id: Id,
        params: &OperatorParams,
    ) -> Option<OperatorWithMeta> {
        self.by_id.read().unwrap().get(&type_id).map(|reg| {
            if let Some(ref param_factory) = reg.param_factory {
                param_factory(params)
            } else {
                (reg.factory)()
            }
        })
    }

    /// Create an operator with metadata by name with custom parameters.
    ///
    /// Returns `(operator, input_port_metadata)` for UI integration.
    pub fn create_with_meta_and_params(
        &self,
        name: &str,
        params: &OperatorParams,
    ) -> Option<OperatorWithMeta> {
        let type_id = self.by_name.read().unwrap().get(name).copied()?;
        self.create_with_meta_and_params_by_id(type_id, params)
    }

    /// Get metadata for an operator type (basic version for backward compatibility)
    pub fn get_meta(&self, type_id: Id) -> Option<RegistryEntry> {
        self.by_id
            .read()
            .unwrap()
            .get(&type_id)
            .map(|reg| reg.entry.meta.clone())
    }

    /// Get extended metadata including parameters
    pub fn get_extended_meta(&self, type_id: Id) -> Option<ExtendedEntry> {
        self.by_id
            .read()
            .unwrap()
            .get(&type_id)
            .map(|reg| reg.entry.clone())
    }

    /// Get extended metadata by name
    pub fn get_extended_meta_by_name(&self, name: &str) -> Option<ExtendedEntry> {
        let type_id = self.by_name.read().unwrap().get(name).copied()?;
        self.get_extended_meta(type_id)
    }

    /// Get the type ID for an operator name
    pub fn get_type_id(&self, name: &str) -> Option<Id> {
        self.by_name.read().unwrap().get(name).copied()
    }

    /// List all registered operator names
    pub fn list_names(&self) -> Vec<&'static str> {
        self.by_name.read().unwrap().keys().copied().collect()
    }

    /// List all registered operators with their metadata
    pub fn list_all(&self) -> Vec<RegistryEntry> {
        self.by_id
            .read()
            .unwrap()
            .values()
            .map(|reg| reg.entry.meta.clone())
            .collect()
    }

    /// List all registered operators with extended metadata
    pub fn list_all_extended(&self) -> Vec<ExtendedEntry> {
        self.by_id
            .read()
            .unwrap()
            .values()
            .map(|reg| reg.entry.clone())
            .collect()
    }

    /// Get operators grouped by category
    ///
    /// Returns a HashMap where keys are category names and values are lists
    /// of operators in that category.
    pub fn by_category(&self) -> HashMap<&'static str, Vec<ExtendedEntry>> {
        let mut result: HashMap<&'static str, Vec<ExtendedEntry>> = HashMap::new();

        for reg in self.by_id.read().unwrap().values() {
            let category = reg.entry.meta.category;
            result.entry(category).or_default().push(reg.entry.clone());
        }

        // Sort entries within each category by name
        for entries in result.values_mut() {
            entries.sort_by_key(|e| e.meta.name);
        }

        result
    }

    /// Get all categories in alphabetical order
    pub fn categories(&self) -> Vec<&'static str> {
        let mut cats: Vec<&'static str> = self
            .by_id
            .read()
            .unwrap()
            .values()
            .map(|reg| reg.entry.meta.category)
            .collect();
        cats.sort();
        cats.dedup();
        cats
    }

    /// Get the number of registered operator types
    pub fn len(&self) -> usize {
        self.by_id.read().unwrap().len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for OperatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Captures `PortMeta` from an operator before boxing it.
///
/// This helper function creates an operator and extracts its input port metadata
/// while the concrete type is still available. This is necessary because `OperatorMeta`
/// cannot be accessed through a trait object (`Box<dyn Operator>`).
///
/// # Example
///
/// ```ignore
/// use flux_operators::{capture_meta, AddOp, OperatorRegistry, RegistryEntry};
///
/// let registry = OperatorRegistry::new();
/// registry.register(
///     RegistryEntry { ... },
///     || capture_meta(AddOp::new()),
/// );
/// ```
pub fn capture_meta<T>(op: T) -> OperatorWithMeta
where
    T: Operator + flux_core::OperatorMeta + 'static,
{
    let input_count = op.inputs().len();
    let meta: Vec<Option<PortMeta>> = (0..input_count).map(|i| op.input_meta(i)).collect();
    (Box::new(op), meta)
}

/// Boxes an operator without capturing `PortMeta`.
///
/// Use this for operators that don't implement `OperatorMeta`. The returned
/// metadata vector will contain `None` for all inputs.
///
/// Prefer `capture_meta` when the operator implements `OperatorMeta`.
pub fn capture_meta_simple<T>(op: T) -> OperatorWithMeta
where
    T: Operator + 'static,
{
    let input_count = op.inputs().len();
    let meta: Vec<Option<PortMeta>> = vec![None; input_count];
    (Box::new(op), meta)
}

/// Create a pre-populated registry with all built-in operators.
///
/// This registers all operators with captured `PortMeta` so that UI code can
/// access input port metadata (ranges, labels, units) without downcasting.
pub fn create_default_registry() -> OperatorRegistry {
    use crate::builtin::{AddOp, CompareMode, CompareOp, ConstantOp, MultiplyOp, ScopeOp, SineWaveOp};

    let registry = OperatorRegistry::new();

    // Register all operators from module-level register() functions
    crate::register_all_operators(&registry);

    // =========================================================================
    // Builtin operators with special registration
    // =========================================================================

    // Constant has a parameter (initial value)
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Constant",
            category: "Sources",
            description: "Outputs a constant float value",
        },
        || capture_meta(ConstantOp::new(0.0)),
    );

    // SineWave is in builtin, not time/oscillators
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "SineWave",
            category: "Oscillators",
            description: "Generates a sine wave based on time",
        },
        || capture_meta(SineWaveOp::new()),
    );

    // Add and Multiply are in builtin, not math/arithmetic
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Add",
            category: "Math",
            description: "Adds two values together",
        },
        || capture_meta(AddOp::new()),
    );

    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Multiply",
            category: "Math",
            description: "Multiplies two values together",
        },
        || capture_meta(MultiplyOp::new()),
    );

    // Compare has parameter-based construction (mode enum)
    registry.register_with_params(
        RegistryEntry {
            type_id: Id::new(),
            name: "Compare",
            category: "Logic",
            description: "Compares two values and outputs a boolean",
        },
        || capture_meta(CompareOp::new(CompareMode::Equal)),
        |params| {
            let mode = match params.get_enum("mode", "Equal") {
                "LessThan" => CompareMode::LessThan,
                "LessOrEqual" => CompareMode::LessOrEqual,
                "GreaterThan" => CompareMode::GreaterThan,
                "GreaterOrEqual" => CompareMode::GreaterOrEqual,
                "NotEqual" => CompareMode::NotEqual,
                _ => CompareMode::Equal,
            };
            capture_meta(CompareOp::new(mode))
        },
        vec![ParameterMeta {
            name: "mode",
            param_type: ParameterType::Enum {
                variants: vec![
                    "Equal",
                    "NotEqual",
                    "LessThan",
                    "LessOrEqual",
                    "GreaterThan",
                    "GreaterOrEqual",
                ],
            },
            default: ParameterValue::Enum("Equal"),
        }],
    );

    // Scope is in builtin, not elsewhere
    registry.register(
        RegistryEntry {
            type_id: Id::new(),
            name: "Scope",
            category: "Output",
            description: "Visualizes signal values over time",
        },
        || capture_meta(ScopeOp::new()),
    );

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_create_by_name() {
        let registry = create_default_registry();

        let add_op = registry.create_by_name("Add");
        assert!(add_op.is_some());
        assert_eq!(add_op.unwrap().name(), "Add");

        let unknown = registry.create_by_name("Unknown");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_registry_list_names() {
        let registry = create_default_registry();
        let names = registry.list_names();

        assert!(names.contains(&"Constant"));
        assert!(names.contains(&"Add"));
        assert!(names.contains(&"Multiply"));
        assert!(names.contains(&"SineWave"));
    }

    #[test]
    fn test_registry_by_category() {
        let registry = create_default_registry();
        let by_cat = registry.by_category();

        // Check Math category
        assert!(by_cat.contains_key("Math"));
        let math_ops: Vec<_> = by_cat["Math"].iter().map(|e| e.meta.name).collect();
        assert!(math_ops.contains(&"Add"));
        assert!(math_ops.contains(&"Multiply"));

        // Check categories list
        let cats = registry.categories();
        assert!(cats.contains(&"Math"));
        assert!(cats.contains(&"Logic"));
        assert!(cats.contains(&"Time"));
    }

    #[test]
    fn test_registry_create_with_params() {
        let registry = create_default_registry();

        // Create Compare with default params (Equal)
        let compare_default = registry.create_by_name("Compare").unwrap();
        assert_eq!(compare_default.name(), "Compare");

        // Create Compare with GreaterThan mode
        let params = OperatorParams::new().set("mode", ParameterValue::Enum("GreaterThan"));
        let compare_gt = registry.create_with_params("Compare", &params).unwrap();
        assert_eq!(compare_gt.name(), "Compare");

        // Verify parameter metadata is available
        let meta = registry.get_extended_meta_by_name("Compare").unwrap();
        assert_eq!(meta.parameters.len(), 1);
        assert_eq!(meta.parameters[0].name, "mode");
    }

    #[test]
    fn test_operator_params() {
        let params = OperatorParams::new()
            .set("float_val", ParameterValue::Float(1.5))
            .set("int_val", ParameterValue::Int(42))
            .set("bool_val", ParameterValue::Bool(true))
            .set("enum_val", ParameterValue::Enum("Option1"));

        assert_eq!(params.get_float("float_val", 0.0), 1.5);
        assert_eq!(params.get_float("missing", 0.0), 0.0);
        assert_eq!(params.get_int("int_val", 0), 42);
        assert!(params.get_bool("bool_val", false));
        assert_eq!(params.get_enum("enum_val", "Default"), "Option1");
    }
}
