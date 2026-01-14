//! Demo 9: Operator Registry
//!
//! This example demonstrates the operator registry:
//! - Listing all registered operators
//! - Getting operator metadata (name, category, description)
//! - Dynamic operator creation by name
//!
//! Run with: `cargo run --example 09_operator_registry`

use flux_operators::create_default_registry;

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 9: Operator Registry              ║");
    println!("╚════════════════════════════════════════╝\n");

    let registry = create_default_registry();

    println!("Registered operators ({}):", registry.len());
    for name in registry.list_names() {
        if let Some(meta) = registry
            .get_type_id(name)
            .and_then(|id| registry.get_meta(id))
        {
            println!("  - {} [{}]: {}", meta.name, meta.category, meta.description);
        }
    }

    // Create operators dynamically from registry
    println!("\nDynamic operator creation:");
    if let Some(add_op) = registry.create_by_name("Add") {
        println!(
            "  Created '{}' operator with {} inputs",
            add_op.name(),
            add_op.inputs().len()
        );
    }
    if let Some(sine_op) = registry.create_by_name("SineWave") {
        println!(
            "  Created '{}' operator with {} inputs",
            sine_op.name(),
            sine_op.inputs().len()
        );
    }
}
