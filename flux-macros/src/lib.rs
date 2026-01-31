//! Flux Macros - Procedural macros for the Flux operator graph system
//!
//! This crate provides derive macros for implementing the `Operator` and `OperatorMeta` traits.
//!
//! # Supported Types
//!
//! The macro supports these input/output types:
//! - `f32` - Float values
//! - `i32` - Integer values
//! - `bool` - Boolean values
//! - `String` - String values
//! - `[f32; 2]` - Vec2 values
//! - `[f32; 3]` - Vec3 values
//! - `[f32; 4]` - Vec4 values
//! - `Color` - Color values (from flux_core::value::Color)
//!
//! # Simple Example
//!
//! For operators with Vec-based ports (generated `new()` constructor):
//!
//! ```ignore
//! use flux_macros::Operator;
//! use flux_core::{Id, InputPort, OutputPort, EvalContext, Operator, OperatorMeta, Value};
//!
//! #[derive(Operator)]
//! #[operator(name = "Divide", category = "Math", description = "Divides A by B")]
//! #[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
//! struct DivideOp {
//!     _id: Id,
//!     _inputs: Vec<InputPort>,
//!     _outputs: Vec<OutputPort>,
//!     #[input(label = "A", default = 0.0)]
//!     a: f32,
//!     #[input(label = "B", default = 1.0)]
//!     b: f32,
//!     #[output(label = "Result")]
//!     result: f32,
//! }
//!
//! impl DivideOp {
//!     fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
//!         // Generated getters use InputPort::resolve_* methods internally
//!         let a = self.get_a(get_input);
//!         let b = self.get_b(get_input);
//!         self.set_result(if b != 0.0 { a / b } else { 0.0 });
//!     }
//! }
//! ```
//!
//! # Vec3 Example
//!
//! ```ignore
//! #[derive(Operator)]
//! #[operator(name = "Vec3Scale", category = "Vector", description = "Scales a Vec3")]
//! struct Vec3ScaleOp {
//!     _id: Id,
//!     _inputs: Vec<InputPort>,
//!     _outputs: Vec<OutputPort>,
//!     #[input(label = "Vector", default = [0.0, 0.0, 0.0])]
//!     vector: [f32; 3],
//!     #[input(label = "Scale", default = 1.0)]
//!     scale: f32,
//!     #[output(label = "Scaled")]
//!     scaled: [f32; 3],
//! }
//!
//! impl Vec3ScaleOp {
//!     fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
//!         let v = self.get_vector(get_input);  // Returns [f32; 3]
//!         let s = self.get_scale(get_input);   // Returns f32
//!         self.set_scaled([v[0] * s, v[1] * s, v[2] * s]);
//!     }
//! }
//! ```
//!
//! # OperatorMeta Derive Only
//!
//! For existing operators that already implement `Operator`, use `OperatorMeta` derive:
//!
//! ```ignore
//! use flux_macros::OperatorMeta;
//!
//! #[derive(OperatorMeta)]
//! #[meta(category = "Math", description = "Adds two numbers")]
//! #[meta(category_color = [0.35, 0.35, 0.55, 1.0])]
//! #[input_meta(0, label = "A")]
//! #[input_meta(1, label = "B")]
//! #[output_meta(0, label = "Sum", shape = "TriangleFilled")]
//! struct AddOp { /* ... */ }
//! ```

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Expr, Fields, Type};

/// Detected storage pattern for inputs/outputs
#[derive(Clone, Copy, Debug, PartialEq)]
enum StoragePattern {
    /// Vec-based: `_inputs: Vec<InputPort>` (legacy pattern)
    Vec,
    /// Array-based: `inputs: [InputPort; N]` (preferred pattern)
    Array,
}

/// Information about the port storage fields
struct PortStorageInfo {
    pattern: StoragePattern,
    inputs_field: String,
    outputs_field: String,
    id_field: String,
}

impl Default for PortStorageInfo {
    fn default() -> Self {
        Self {
            pattern: StoragePattern::Vec,
            inputs_field: "_inputs".to_string(),
            outputs_field: "_outputs".to_string(),
            id_field: "_id".to_string(),
        }
    }
}

/// Derive macro for implementing both `Operator` and `OperatorMeta` traits.
///
/// This is a convenience macro for new operators. For existing operators,
/// use `#[derive(OperatorMeta)]` instead.
///
/// See crate-level documentation for usage examples.
#[proc_macro_derive(Operator, attributes(operator, input, output))]
pub fn derive_operator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Parse struct-level attributes
    let operator_name = get_operator_attr(&input.attrs, "name").unwrap_or_else(|| name.to_string());
    let category = get_operator_attr(&input.attrs, "category").unwrap_or_else(|| "Uncategorized".to_string());
    let description = get_operator_attr(&input.attrs, "description").unwrap_or_default();
    let icon = get_operator_attr(&input.attrs, "icon");
    let category_color = get_color_attr(&input.attrs).unwrap_or([0.5, 0.5, 0.5, 1.0]);

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Operator derive only supports structs with named fields"),
        },
        _ => panic!("Operator derive only supports structs"),
    };

    // Detect the storage pattern (array vs Vec) by examining field types
    let mut storage_info = PortStorageInfo::default();
    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let type_str = quote!(#field.ty).to_string();

        // Check for array pattern: `inputs: [InputPort; N]`
        if field_name == "inputs" && type_str.contains("[InputPort") {
            storage_info.pattern = StoragePattern::Array;
            storage_info.inputs_field = "inputs".to_string();
        } else if field_name == "outputs" && type_str.contains("[OutputPort") {
            storage_info.outputs_field = "outputs".to_string();
        } else if field_name == "id" && type_str.contains("Id") {
            storage_info.id_field = "id".to_string();
        }
        // Check for Vec pattern: `_inputs: Vec<InputPort>`
        else if field_name == "_inputs" && type_str.contains("Vec") {
            storage_info.pattern = StoragePattern::Vec;
            storage_info.inputs_field = "_inputs".to_string();
        } else if field_name == "_outputs" && type_str.contains("Vec") {
            storage_info.outputs_field = "_outputs".to_string();
        } else if field_name == "_id" {
            storage_info.id_field = "_id".to_string();
        }
    }

    let mut input_fields: Vec<InputFieldInfo> = Vec::new();
    let mut output_fields: Vec<OutputFieldInfo> = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        if has_attr(&field.attrs, "input") {
            let label = get_attr_value(&field.attrs, "input", "label")
                .unwrap_or_else(|| capitalize(&field_name.to_string()));
            let default_value = get_attr_value(&field.attrs, "input", "default");
            let range = get_range_attr(&field.attrs, "input");
            let unit = get_attr_value(&field.attrs, "input", "unit");
            let shape = get_attr_value(&field.attrs, "input", "shape")
                .unwrap_or_else(|| "CircleFilled".to_string());

            input_fields.push(InputFieldInfo {
                name: field_name.clone(),
                ty: field_type.clone(),
                label,
                default_value,
                range,
                unit,
                shape,
            });
        } else if has_attr(&field.attrs, "output") {
            let label = get_attr_value(&field.attrs, "output", "label")
                .unwrap_or_else(|| capitalize(&field_name.to_string()));
            let unit = get_attr_value(&field.attrs, "output", "unit");
            let shape = get_attr_value(&field.attrs, "output", "shape")
                .unwrap_or_else(|| "TriangleFilled".to_string());

            output_fields.push(OutputFieldInfo {
                name: field_name.clone(),
                ty: field_type.clone(),
                label,
                unit,
                shape,
            });
        }
    }

    // Generate input port initialization
    let input_inits: Vec<_> = input_fields
        .iter()
        .map(|f| {
            let type_str = normalize_type(&f.ty);
            let default_val = f.default_value
                .as_ref()
                .map(|d| {
                    // For String type, wrap the value in quotes since parse_kv strips them
                    if type_str == "String" {
                        let quoted = format!("\"{}\"", d);
                        syn::parse_str::<Expr>(&quoted)
                            .unwrap_or_else(|_| syn::parse_str::<Expr>("\"\"").unwrap())
                    } else {
                        syn::parse_str::<Expr>(d)
                            .unwrap_or_else(|_| syn::parse_str::<Expr>("0.0").unwrap())
                    }
                })
                .unwrap_or_else(|| get_default_for_type(&f.ty));
            let label = &f.label;
            let port_ctor = get_port_constructor(&f.ty);
            quote! {
                InputPort::#port_ctor(#label, #default_val)
            }
        })
        .collect();

    // Generate output port initialization
    let output_inits: Vec<_> = output_fields
        .iter()
        .map(|f| {
            let label = &f.label;
            let port_ctor = get_output_constructor(&f.ty);
            quote! {
                OutputPort::#port_ctor(#label)
            }
        })
        .collect();

    // Generate getter methods for inputs using the new resolve_* methods
    let inputs_field_ident = format_ident!("{}", storage_info.inputs_field);
    let input_getters: Vec<_> = input_fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let getter_name = format_ident!("get_{}", f.name);
            let field_type = &f.ty;
            let resolve_method = get_resolve_method(field_type);
            let inputs_field = &inputs_field_ident;
            quote! {
                /// Returns the value from the connected input or the default value.
                #[inline]
                pub fn #getter_name(&self, get_input: &dyn Fn(Id, usize) -> flux_core::Value) -> #field_type {
                    self.#inputs_field[#i].#resolve_method(get_input)
                }
            }
        })
        .collect();

    // Generate setter methods for outputs
    let outputs_field_ident = format_ident!("{}", storage_info.outputs_field);
    let output_setters: Vec<_> = output_fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let setter_name = format_ident!("set_{}", f.name);
            let field_type = &f.ty;
            let type_str = normalize_type(field_type);
            let outputs_field = &outputs_field_ident;

            // Handle special cases for String and Color
            match type_str.as_str() {
                "String" => {
                    quote! {
                        /// Sets the output string value.
                        #[inline]
                        pub fn #setter_name(&mut self, value: &str) {
                            self.#outputs_field[#i].set_string(value);
                        }
                    }
                }
                "Color" => {
                    quote! {
                        /// Sets the output color value.
                        #[inline]
                        pub fn #setter_name(&mut self, color: Color) {
                            self.#outputs_field[#i].set_color(color.r, color.g, color.b, color.a);
                        }
                    }
                }
                _ => {
                    let set_method = get_set_method(field_type);
                    quote! {
                        /// Sets the output value.
                        #[inline]
                        pub fn #setter_name(&mut self, value: #field_type) {
                            self.#outputs_field[#i].#set_method(value);
                        }
                    }
                }
            }
        })
        .collect();

    // Generate field initialization for the marker fields to their default values
    let input_field_inits: Vec<_> = input_fields
        .iter()
        .map(|f| {
            let name = &f.name;
            let default_val = get_default_for_type(&f.ty);
            quote! { #name: #default_val }
        })
        .collect();

    let output_field_inits: Vec<_> = output_fields
        .iter()
        .map(|f| {
            let name = &f.name;
            let default_val = get_default_for_type(&f.ty);
            quote! { #name: #default_val }
        })
        .collect();

    // Generate OperatorMeta input_meta match arms
    let input_meta_arms: Vec<_> = input_fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let label = &f.label;
            let shape_ident = format_ident!("{}", f.shape);

            let mut builder = quote! {
                flux_core::PortMeta::new(#label).with_shape(flux_core::PinShape::#shape_ident)
            };

            if let Some((min, max)) = &f.range {
                let min_val: f32 = min.parse().unwrap_or(0.0);
                let max_val: f32 = max.parse().unwrap_or(1.0);
                builder = quote! { #builder.with_range(#min_val, #max_val) };
            }

            if let Some(unit) = &f.unit {
                builder = quote! { #builder.with_unit(#unit) };
            }

            quote! {
                #i => Some(#builder),
            }
        })
        .collect();

    // Generate OperatorMeta output_meta match arms
    let output_meta_arms: Vec<_> = output_fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let label = &f.label;
            let shape_ident = format_ident!("{}", f.shape);

            let mut builder = quote! {
                flux_core::PortMeta::new(#label).with_shape(flux_core::PinShape::#shape_ident)
            };

            if let Some(unit) = &f.unit {
                builder = quote! { #builder.with_unit(#unit) };
            }

            quote! {
                #i => Some(#builder),
            }
        })
        .collect();

    // Generate icon method
    let icon_impl = if let Some(icon_str) = icon {
        quote! {
            fn icon(&self) -> Option<&'static str> {
                Some(#icon_str)
            }
        }
    } else {
        quote! {}
    };

    // Category color array
    let [r, g, b, a] = category_color;

    // Generate field identifiers based on storage pattern
    let id_field_ident = format_ident!("{}", storage_info.id_field);
    let inputs_field_ident2 = format_ident!("{}", storage_info.inputs_field);
    let outputs_field_ident2 = format_ident!("{}", storage_info.outputs_field);

    // Generate initialization based on storage pattern
    let (inputs_init, outputs_init) = match storage_info.pattern {
        StoragePattern::Array => (
            quote! { [#(#input_inits),*] },
            quote! { [#(#output_inits),*] },
        ),
        StoragePattern::Vec => (
            quote! { vec![#(#input_inits),*] },
            quote! { vec![#(#output_inits),*] },
        ),
    };

    let expanded = quote! {
        impl #name {
            /// Creates a new instance with default values.
            pub fn new() -> Self {
                Self {
                    #id_field_ident: Id::new(),
                    #inputs_field_ident2: #inputs_init,
                    #outputs_field_ident2: #outputs_init,
                    #(#input_field_inits,)*
                    #(#output_field_inits,)*
                }
            }

            #(#input_getters)*
            #(#output_setters)*
        }

        impl Default for #name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl Operator for #name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn id(&self) -> Id {
                self.#id_field_ident
            }

            fn name(&self) -> &'static str {
                #operator_name
            }

            fn inputs(&self) -> &[InputPort] {
                &self.#inputs_field_ident2
            }

            fn inputs_mut(&mut self) -> &mut [InputPort] {
                &mut self.#inputs_field_ident2
            }

            fn outputs(&self) -> &[OutputPort] {
                &self.#outputs_field_ident2
            }

            fn outputs_mut(&mut self) -> &mut [OutputPort] {
                &mut self.#outputs_field_ident2
            }

            fn compute(&mut self, ctx: &EvalContext, get_input: InputResolver) {
                self.compute_impl(ctx, get_input);
            }
        }

        impl OperatorMeta for #name {
            fn category(&self) -> &'static str {
                #category
            }

            fn category_color(&self) -> [f32; 4] {
                [#r, #g, #b, #a]
            }

            fn description(&self) -> &'static str {
                #description
            }

            #icon_impl

            fn input_meta(&self, index: usize) -> Option<flux_core::PortMeta> {
                match index {
                    #(#input_meta_arms)*
                    _ => None,
                }
            }

            fn output_meta(&self, index: usize) -> Option<flux_core::PortMeta> {
                match index {
                    #(#output_meta_arms)*
                    _ => None,
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for implementing only the `OperatorMeta` trait.
///
/// Use this for existing operators that already implement `Operator`.
/// Port metadata is specified using struct-level attributes.
///
/// # Example
///
/// ```ignore
/// #[derive(OperatorMeta)]
/// #[meta(category = "Math", description = "Adds two numbers")]
/// #[meta(category_color = [0.35, 0.35, 0.55, 1.0])]
/// #[input_meta(0, label = "A")]
/// #[input_meta(1, label = "B")]
/// #[output_meta(0, label = "Sum", shape = "TriangleFilled")]
/// struct AddOp { /* ... */ }
/// ```
#[proc_macro_derive(OperatorMeta, attributes(meta, input_meta, output_meta))]
pub fn derive_operator_meta(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Parse struct-level attributes
    let category = get_attr_value(&input.attrs, "meta", "category")
        .unwrap_or_else(|| "Uncategorized".to_string());
    let description = get_attr_value(&input.attrs, "meta", "description")
        .unwrap_or_default();
    let icon = get_attr_value(&input.attrs, "meta", "icon");
    let category_color = get_meta_color_attr(&input.attrs).unwrap_or([0.5, 0.5, 0.5, 1.0]);

    // Parse input_meta and output_meta attributes
    let input_metas = parse_port_meta_attrs(&input.attrs, "input_meta");
    let output_metas = parse_port_meta_attrs(&input.attrs, "output_meta");

    // Generate input_meta match arms
    let input_meta_arms: Vec<_> = input_metas
        .iter()
        .map(|pm| {
            let index = pm.index;
            let label = &pm.label;
            let shape_ident = format_ident!("{}", pm.shape);

            let mut builder = quote! {
                flux_core::PortMeta::new(#label).with_shape(flux_core::PinShape::#shape_ident)
            };

            if let Some((min, max)) = &pm.range {
                builder = quote! { #builder.with_range(#min, #max) };
            }

            if let Some(unit) = &pm.unit {
                builder = quote! { #builder.with_unit(#unit) };
            }

            quote! {
                #index => Some(#builder),
            }
        })
        .collect();

    // Generate output_meta match arms
    let output_meta_arms: Vec<_> = output_metas
        .iter()
        .map(|pm| {
            let index = pm.index;
            let label = &pm.label;
            let shape_ident = format_ident!("{}", pm.shape);

            let mut builder = quote! {
                flux_core::PortMeta::new(#label).with_shape(flux_core::PinShape::#shape_ident)
            };

            if let Some(unit) = &pm.unit {
                builder = quote! { #builder.with_unit(#unit) };
            }

            quote! {
                #index => Some(#builder),
            }
        })
        .collect();

    // Generate icon method
    let icon_impl = if let Some(icon_str) = icon {
        quote! {
            fn icon(&self) -> Option<&'static str> {
                Some(#icon_str)
            }
        }
    } else {
        quote! {}
    };

    let [r, g, b, a] = category_color;

    let expanded = quote! {
        impl OperatorMeta for #name {
            fn category(&self) -> &'static str {
                #category
            }

            fn category_color(&self) -> [f32; 4] {
                [#r, #g, #b, #a]
            }

            fn description(&self) -> &'static str {
                #description
            }

            #icon_impl

            fn input_meta(&self, index: usize) -> Option<flux_core::PortMeta> {
                match index {
                    #(#input_meta_arms)*
                    _ => None,
                }
            }

            fn output_meta(&self, index: usize) -> Option<flux_core::PortMeta> {
                match index {
                    #(#output_meta_arms)*
                    _ => None,
                }
            }
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Helper structs
// ============================================================================

struct InputFieldInfo {
    name: proc_macro2::Ident,
    ty: Type,
    label: String,
    default_value: Option<String>,
    range: Option<(String, String)>,
    unit: Option<String>,
    shape: String,
}

impl Clone for InputFieldInfo {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            ty: self.ty.clone(),
            label: self.label.clone(),
            default_value: self.default_value.clone(),
            range: self.range.clone(),
            unit: self.unit.clone(),
            shape: self.shape.clone(),
        }
    }
}

struct OutputFieldInfo {
    name: proc_macro2::Ident,
    ty: Type,
    label: String,
    unit: Option<String>,
    shape: String,
}

struct PortMetaInfo {
    index: usize,
    label: String,
    shape: String,
    range: Option<(f32, f32)>,
    unit: Option<String>,
}

// ============================================================================
// Attribute parsing helpers
// ============================================================================

fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|a| a.path().is_ident(name))
}

fn get_operator_attr(attrs: &[Attribute], key: &str) -> Option<String> {
    get_attr_value(attrs, "operator", key)
}

fn get_attr_value(attrs: &[Attribute], attr_name: &str, key: &str) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident(attr_name) {
            if let Ok(meta) = attr.meta.require_list() {
                let tokens = meta.tokens.to_string();
                if let Some(value) = parse_kv(&tokens, key) {
                    return Some(value);
                }
            }
        }
    }
    None
}

fn parse_kv(tokens: &str, key: &str) -> Option<String> {
    let pattern = format!("{} =", key);
    let mut pos = tokens.find(&pattern)?;
    pos += pattern.len();

    let rest = tokens[pos..].trim_start();

    if let Some(stripped) = rest.strip_prefix('"') {
        let end = stripped.find('"')?;
        return Some(stripped[..end].to_string());
    }

    if rest.starts_with('(') {
        let end = rest.find(')')?;
        return Some(rest[..=end].to_string());
    }

    if rest.starts_with('[') {
        let end = rest.find(']')?;
        return Some(rest[..=end].to_string());
    }

    let end = rest.find(',').unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

fn get_range_attr(attrs: &[Attribute], attr_name: &str) -> Option<(String, String)> {
    let range_str = get_attr_value(attrs, attr_name, "range")?;
    let inner = range_str.trim_start_matches('(').trim_end_matches(')');
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() == 2 {
        Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
    } else {
        None
    }
}

fn get_color_attr(attrs: &[Attribute]) -> Option<[f32; 4]> {
    let color_str = get_attr_value(attrs, "operator", "category_color")?;
    parse_color_array(&color_str)
}

fn get_meta_color_attr(attrs: &[Attribute]) -> Option<[f32; 4]> {
    let color_str = get_attr_value(attrs, "meta", "category_color")?;
    parse_color_array(&color_str)
}

fn parse_color_array(color_str: &str) -> Option<[f32; 4]> {
    let inner = color_str.trim_start_matches('[').trim_end_matches(']');
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() == 4 {
        let r: f32 = parts[0].trim().parse().ok()?;
        let g: f32 = parts[1].trim().parse().ok()?;
        let b: f32 = parts[2].trim().parse().ok()?;
        let a: f32 = parts[3].trim().parse().ok()?;
        Some([r, g, b, a])
    } else {
        None
    }
}

fn parse_port_meta_attrs(attrs: &[Attribute], attr_name: &str) -> Vec<PortMetaInfo> {
    let mut result = Vec::new();

    for attr in attrs {
        if attr.path().is_ident(attr_name) {
            if let Ok(meta) = attr.meta.require_list() {
                let tokens = meta.tokens.to_string();

                // Parse: index, label = "...", shape = "...", etc.
                let parts: Vec<&str> = tokens.splitn(2, ',').collect();
                if parts.is_empty() {
                    continue;
                }

                let index: usize = parts[0].trim().parse().unwrap_or(0);
                let rest = if parts.len() > 1 { parts[1] } else { "" };

                let label = parse_kv(rest, "label").unwrap_or_else(|| format!("Port {}", index));
                let shape = parse_kv(rest, "shape").unwrap_or_else(|| {
                    if attr_name == "output_meta" {
                        "TriangleFilled".to_string()
                    } else {
                        "CircleFilled".to_string()
                    }
                });
                let unit = parse_kv(rest, "unit");
                let range = parse_kv(rest, "range").and_then(|r| {
                    let inner = r.trim_start_matches('(').trim_end_matches(')');
                    let parts: Vec<&str> = inner.split(',').collect();
                    if parts.len() == 2 {
                        let min: f32 = parts[0].trim().parse().ok()?;
                        let max: f32 = parts[1].trim().parse().ok()?;
                        Some((min, max))
                    } else {
                        None
                    }
                });

                result.push(PortMetaInfo {
                    index,
                    label,
                    shape,
                    range,
                    unit,
                });
            }
        }
    }

    result
}

// ============================================================================
// Type helpers
// ============================================================================

/// Normalize type string by removing spaces for consistent matching
fn normalize_type(ty: &Type) -> String {
    quote!(#ty).to_string().replace(' ', "")
}

fn get_port_constructor(ty: &Type) -> proc_macro2::TokenStream {
    let type_str = normalize_type(ty);
    match type_str.as_str() {
        "f32" => quote!(float),
        "i32" => quote!(int),
        "bool" => quote!(bool),
        "String" => quote!(string),
        "[f32;2]" => quote!(vec2),
        "[f32;3]" => quote!(vec3),
        "[f32;4]" => quote!(vec4),
        "Color" => quote!(color),
        _ => quote!(float),
    }
}

fn get_output_constructor(ty: &Type) -> proc_macro2::TokenStream {
    let type_str = normalize_type(ty);
    match type_str.as_str() {
        "f32" => quote!(float),
        "i32" => quote!(int),
        "bool" => quote!(bool),
        "String" => quote!(string),
        "[f32;2]" => quote!(vec2),
        "[f32;3]" => quote!(vec3),
        "[f32;4]" => quote!(vec4),
        "Color" => quote!(color),
        _ => quote!(float),
    }
}

/// Get the resolve_* method name for a type
fn get_resolve_method(ty: &Type) -> proc_macro2::TokenStream {
    let type_str = normalize_type(ty);
    match type_str.as_str() {
        "f32" => quote!(resolve_float),
        "i32" => quote!(resolve_int),
        "bool" => quote!(resolve_bool),
        "String" => quote!(resolve_string),
        "[f32;2]" => quote!(resolve_vec2),
        "[f32;3]" => quote!(resolve_vec3),
        "[f32;4]" => quote!(resolve_vec4),
        "Color" => quote!(resolve_color),
        _ => quote!(resolve_float),
    }
}

fn get_set_method(ty: &Type) -> proc_macro2::TokenStream {
    let type_str = normalize_type(ty);
    match type_str.as_str() {
        "f32" => quote!(set_float),
        "i32" => quote!(set_int),
        "bool" => quote!(set_bool),
        "String" => quote!(set_string),
        "[f32;2]" => quote!(set_vec2),
        "[f32;3]" => quote!(set_vec3),
        "[f32;4]" => quote!(set_vec4),
        "Color" => quote!(set_color),
        _ => quote!(set_float),
    }
}

fn get_default_for_type(ty: &Type) -> Expr {
    let type_str = normalize_type(ty);
    let default_str = match type_str.as_str() {
        "f32" => "0.0",
        "i32" => "0",
        "bool" => "false",
        "String" => "String::new()",
        "[f32;2]" => "[0.0, 0.0]",
        "[f32;3]" => "[0.0, 0.0, 0.0]",
        "[f32;4]" => "[0.0, 0.0, 0.0, 0.0]",
        "Color" => "Color::BLACK",
        _ => "0.0",
    };
    syn::parse_str::<Expr>(default_str).unwrap()
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(chars).collect(),
    }
}
