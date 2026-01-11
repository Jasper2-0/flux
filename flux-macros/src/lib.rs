//! Flux Macros - Procedural macros for the Flux operator graph system
//!
//! This crate provides derive macros for implementing the `Operator` and `OperatorMeta` traits.
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
//!         let a = self.get_a(get_input);
//!         let b = self.get_b(get_input);
//!         self.set_result(if b != 0.0 { a / b } else { 0.0 });
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
            let default_val = f.default_value
                .as_ref()
                .map(|d| {
                    syn::parse_str::<Expr>(d)
                        .unwrap_or_else(|_| syn::parse_str::<Expr>("0.0").unwrap())
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

    // Generate getter methods for inputs
    let input_getters: Vec<_> = input_fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let getter_name = format_ident!("get_{}", f.name);
            let field_type = &f.ty;
            let as_method = get_as_method(field_type);
            let default_val = get_default_for_type(field_type);
            quote! {
                /// Returns the value from the connected input or the default value.
                pub fn #getter_name(&self, get_input: &dyn Fn(Id, usize) -> Value) -> #field_type {
                    match self._inputs[#i].connection {
                        Some((node_id, output_idx)) => get_input(node_id, output_idx).#as_method().unwrap_or(#default_val),
                        None => self._inputs[#i].default.#as_method().unwrap_or(#default_val),
                    }
                }
            }
        })
        .collect();

    // Generate setter methods for outputs
    let output_setters: Vec<_> = output_fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let setter_name = format_ident!("set_{}", f.name);
            let field_type = &f.ty;
            let set_method = get_set_method(field_type);
            quote! {
                /// Sets the output value.
                pub fn #setter_name(&mut self, value: #field_type) {
                    self._outputs[#i].#set_method(value);
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

    let expanded = quote! {
        impl #name {
            /// Creates a new instance with default values.
            pub fn new() -> Self {
                Self {
                    _id: Id::new(),
                    _inputs: vec![#(#input_inits),*],
                    _outputs: vec![#(#output_inits),*],
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
                self._id
            }

            fn name(&self) -> &'static str {
                #operator_name
            }

            fn inputs(&self) -> &[InputPort] {
                &self._inputs
            }

            fn inputs_mut(&mut self) -> &mut [InputPort] {
                &mut self._inputs
            }

            fn outputs(&self) -> &[OutputPort] {
                &self._outputs
            }

            fn outputs_mut(&mut self) -> &mut [OutputPort] {
                &mut self._outputs
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

fn get_port_constructor(ty: &Type) -> proc_macro2::TokenStream {
    let type_str = quote!(#ty).to_string();
    match type_str.as_str() {
        "f32" => quote!(float),
        "i32" => quote!(int),
        "bool" => quote!(bool),
        _ => quote!(float),
    }
}

fn get_output_constructor(ty: &Type) -> proc_macro2::TokenStream {
    let type_str = quote!(#ty).to_string();
    match type_str.as_str() {
        "f32" => quote!(float),
        "i32" => quote!(int),
        "bool" => quote!(bool),
        _ => quote!(float),
    }
}

fn get_as_method(ty: &Type) -> proc_macro2::TokenStream {
    let type_str = quote!(#ty).to_string();
    match type_str.as_str() {
        "f32" => quote!(as_float),
        "i32" => quote!(as_int),
        "bool" => quote!(as_bool),
        _ => quote!(as_float),
    }
}

fn get_set_method(ty: &Type) -> proc_macro2::TokenStream {
    let type_str = quote!(#ty).to_string();
    match type_str.as_str() {
        "f32" => quote!(set_float),
        "i32" => quote!(set_int),
        "bool" => quote!(set_bool),
        _ => quote!(set_float),
    }
}

fn get_default_for_type(ty: &Type) -> Expr {
    let type_str = quote!(#ty).to_string();
    let default_str = match type_str.as_str() {
        "f32" => "0.0",
        "i32" => "0",
        "bool" => "false",
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
