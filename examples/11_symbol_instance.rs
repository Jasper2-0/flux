//! Demo 11: Symbol/Instance System
//!
//! This example demonstrates the Symbol/Instance pattern:
//! - Creating Symbols (operator blueprints/definitions)
//! - Defining inputs and outputs with metadata
//! - Creating Instances from Symbols
//! - Parent symbols with child references
//! - Instance lifecycle (initialize, activate, bypass)
//!
//! Run with: `cargo run --example 11_symbol_instance`

use flux_core::Id;
use flux_graph::symbol::{InputDefinition, OutputDefinition, Symbol, SymbolChild};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 11: Symbol/Instance System        ║");
    println!("╚════════════════════════════════════════╝\n");

    // Create a Symbol (operator definition/blueprint)
    let mut add_symbol = Symbol::new("Add")
        .with_description("Adds two values")
        .with_category("Math")
        .bypassable();

    // Define inputs and outputs
    add_symbol.add_input(InputDefinition::float("A", 0.0).with_description("First operand"));
    add_symbol.add_input(InputDefinition::float("B", 0.0).with_description("Second operand"));
    add_symbol.add_output(OutputDefinition::float("Result").with_description("Sum of A and B"));

    println!("Created Symbol: {}", add_symbol.name);
    println!("  Description: {:?}", add_symbol.description);
    println!("  Category: {:?}", add_symbol.category);
    println!("  Inputs: {}", add_symbol.input_definitions.len());
    println!("  Outputs: {}", add_symbol.output_definitions.len());
    println!("  Bypassable: {}", add_symbol.is_bypassable);

    // Create an Instance from the Symbol
    let instance = add_symbol.create_instance();

    println!("\nCreated Instance from Symbol:");
    println!("  Instance ID: {:?}", instance.id);
    println!("  Symbol ID: {:?}", instance.symbol_id);
    println!("  Input slots: {}", instance.inputs.len());
    println!("  Output slots: {}", instance.outputs.len());
    println!("  Initialized: {}", instance.is_initialized());

    // Create a parent symbol with children
    let mut parent_symbol = Symbol::new("CompositeAdd")
        .with_description("Demonstrates nested symbols")
        .with_category("Composite");

    parent_symbol.add_input(InputDefinition::float("X", 0.0));
    parent_symbol.add_input(InputDefinition::float("Y", 0.0));
    parent_symbol.add_input(InputDefinition::float("Z", 0.0));
    parent_symbol.add_output(OutputDefinition::float("Sum"));

    // Add child operators (referencing other symbols)
    let add1_child = SymbolChild::named(Id::new(), add_symbol.id, "Add1");
    let add2_child = SymbolChild::named(Id::new(), add_symbol.id, "Add2");

    let _add1_id = parent_symbol.add_child(add1_child);
    let _add2_id = parent_symbol.add_child(add2_child);

    println!("\nCreated Parent Symbol: {}", parent_symbol.name);
    println!("  Children: {}", parent_symbol.child_count());
    for child_id in parent_symbol.child_ids() {
        if let Some(child) = parent_symbol.get_child(child_id) {
            println!(
                "    - {} (refs symbol {:?})",
                child.display_name(),
                child.symbol_id
            );
        }
    }

    // Create an instance of the parent
    let mut parent_instance = parent_symbol.create_instance();
    parent_instance.initialize();
    parent_instance.set_active(true);

    println!("\nParent Instance Status:");
    println!("  Initialized: {}", parent_instance.is_initialized());
    println!("  Active: {}", parent_instance.is_active());
    println!("  Bypassed: {}", parent_instance.is_bypassed());

    // Demonstrate bypass
    parent_instance.set_bypass(true);
    println!("  After bypass: {}", parent_instance.is_bypassed());
}
