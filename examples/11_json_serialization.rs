//! Demo 7: JSON Serialization
//!
//! This example demonstrates symbol serialization with the modern format:
//! - Creating SymbolDef, ChildDef structures
//! - JSON serialization and deserialization with versioning
//! - Round-trip validation
//!
//! Run with: `cargo run --example 07_json_serialization`

use flux_graph::serialization::{ChildDef, ConnectionDef, InputDef, OutputDef, SymbolDef, SymbolFile};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 7: JSON Serialization             ║");
    println!("╚════════════════════════════════════════╝\n");

    // Create a symbol definition representing: (A + B) * C
    let mut symbol = SymbolDef::new("MathExpression")
        .with_description("Computes (A + B) * C")
        .with_category("Math");

    // Add inputs
    symbol.add_input(InputDef::float("A", 5.0));
    symbol.add_input(InputDef::float("B", 3.0));
    symbol.add_input(InputDef::float("C", 2.0));

    // Add output
    symbol.add_output(OutputDef::float("Result"));

    // Add child operators
    let add_child = ChildDef::builtin("Add")
        .with_name("Add")
        .at_position(100.0, 100.0);
    let add_id = add_child.id;
    symbol.add_child(add_child);

    let mult_child = ChildDef::builtin("Multiply")
        .with_name("Multiply")
        .at_position(300.0, 100.0);
    let mult_id = mult_child.id;
    symbol.add_child(mult_child);

    // Add connections (simplified - using indices)
    // Note: In a real implementation, you'd connect symbol inputs to child inputs
    // and child outputs to symbol outputs. This example shows the structure.
    symbol.add_connection(ConnectionDef::new(add_id, 0, mult_id, 0));

    println!("Created symbol: {}", symbol.name);
    println!("  ID: {}", symbol.id);
    println!("  Inputs: {}", symbol.inputs.len());
    println!("  Outputs: {}", symbol.outputs.len());
    println!("  Children: {}", symbol.children.len());
    println!("  Connections: {}", symbol.connections.len());

    // Wrap in SymbolFile for versioned serialization
    let file = SymbolFile::from_def(symbol.clone());

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&file).expect("serialize");
    println!("\nSerialized symbol file:\n{}\n", json);

    // Demonstrate round-trip
    let restored: SymbolFile = serde_json::from_str(&json).expect("deserialize");
    println!("Restored symbol file:");
    println!("  Version: {}", restored.version);
    println!("  Symbol name: {}", restored.symbol.name);
    println!("  Inputs: {}", restored.symbol.inputs.len());
    println!("  Outputs: {}", restored.symbol.outputs.len());
    println!("  Children: {}", restored.symbol.children.len());
    println!("  Connections: {}", restored.symbol.connections.len());

    println!("\n✓ Round-trip serialization successful!");
}
