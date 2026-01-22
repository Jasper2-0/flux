//! Demo 16: Enhanced Serialization (Modern Flux Format)
//!
//! This example demonstrates the modern Flux serialization format:
//! - SymbolFile creation with versioning and metadata
//! - Input/output definitions with types
//! - Child operator instances (symbol children)
//! - Connection definitions
//! - Animation curves with keyframes and interpolation
//! - JSON roundtrip serialization
//!
//! Run with: `cargo run --example 16_enhanced_serialization`

use std::f64::consts::TAU;

use flux_graph::serialization::{
    AnimationDef, ChildDef, ConnectionDef, InputDef, InterpolationMode, KeyframeDef, OutputDef,
    SymbolDef, SymbolFile,
};

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 16: Enhanced Serialization        ║");
    println!("╚════════════════════════════════════════╝\n");

    // Create a complete symbol definition
    let mut symbol = SymbolDef::new("WaveGenerator")
        .with_category("Audio.Generators")
        .with_description("Generates various waveforms for audio synthesis")
        .with_tag("audio")
        .with_tag("synthesis");

    println!("Creating symbol: {}", symbol.name);
    println!("  Category: {:?}", symbol.category);
    println!("  Description: {:?}", symbol.description);
    println!("  ID: {}", symbol.id);

    // Add input definitions
    symbol.add_input(
        InputDef::float("Frequency", 440.0)
            .with_description("Oscillator frequency in Hz")
            .with_range(20.0, 20000.0),
    );
    symbol.add_input(
        InputDef::float("Amplitude", 1.0)
            .with_description("Output amplitude")
            .with_range(0.0, 2.0),
    );
    symbol.add_input(
        InputDef::float("Phase", 0.0)
            .with_description("Phase offset in radians")
            .with_range(0.0, TAU), // 2π
    );

    println!("\nInput definitions: {}", symbol.inputs.len());
    for input in &symbol.inputs {
        println!("  - {} ({}): {:?}", input.name, input.value_type, input.default);
    }

    // Add output definitions
    symbol.add_output(OutputDef::float("Waveform"));
    symbol.add_output(OutputDef::float("RMS"));

    println!("\nOutput definitions: {}", symbol.outputs.len());
    for output in &symbol.outputs {
        println!("  - {} ({})", output.name, output.value_type);
    }

    // Add children (operator instances)
    let sine_child = ChildDef::builtin("SineOscillator")
        .with_name("SineOscillator")
        .at_position(100.0, 100.0);
    let sine_id = sine_child.id;

    let mult_child = ChildDef::builtin("Multiply")
        .with_name("AmplitudeScale")
        .at_position(300.0, 100.0);
    let mult_id = mult_child.id;

    println!("\nChildren:");
    println!("  - {} at ({}, {})", sine_child.name.as_ref().unwrap(), 100.0, 100.0);
    println!("  - {} at ({}, {})", mult_child.name.as_ref().unwrap(), 300.0, 100.0);

    symbol.add_child(sine_child);
    symbol.add_child(mult_child);

    // Add connections
    symbol.add_connection(ConnectionDef::new(sine_id, 0, mult_id, 0));

    println!("\nConnections: {}", symbol.connections.len());
    println!("  {} (out 0) -> {} (in 0)", sine_id, mult_id);

    // Add animations with keyframes
    let mut freq_anim = AnimationDef::new(sine_id, 0);

    freq_anim.add_keyframe(KeyframeDef::new(0.0, 220.0));
    freq_anim.add_keyframe(
        KeyframeDef::new(1.0, 440.0)
            .with_interpolation(InterpolationMode::Smooth)
    );
    freq_anim.add_keyframe(
        KeyframeDef::new(2.0, 880.0)
            .with_interpolation(InterpolationMode::Linear)
    );
    freq_anim.add_keyframe(
        KeyframeDef::new(3.0, 440.0)
            .with_interpolation(InterpolationMode::Bezier)
            .with_tangents(-100.0, 100.0)
    );

    symbol.animations.push(freq_anim);

    println!("\nAnimations:");
    println!("  Curves: {}", symbol.animations.len());
    for anim in &symbol.animations {
        println!(
            "    - Child {} input {}: {} keyframes",
            anim.target_child,
            anim.target_input,
            anim.curve.keyframes.len()
        );
    }

    // Wrap in SymbolFile for versioned serialization
    let file = SymbolFile::from_def(symbol.clone());

    // Serialize to JSON
    let json_output = serde_json::to_string_pretty(&file).expect("Failed to serialize");
    println!("\nJSON output length: {} characters", json_output.len());

    // Show a snippet of the JSON
    let lines: Vec<&str> = json_output.lines().take(25).collect();
    println!("\nJSON preview (first 25 lines):");
    for line in lines {
        println!("  {}", line);
    }
    println!("  ...");

    // Roundtrip test
    let restored: SymbolFile = serde_json::from_str(&json_output).expect("Failed to deserialize");
    println!("\nRoundtrip verification:");
    println!("  Version: {}", restored.version);
    println!("  Name matches: {}", restored.symbol.name == symbol.name);
    println!("  Inputs match: {}", restored.symbol.inputs.len() == symbol.inputs.len());
    println!("  Outputs match: {}", restored.symbol.outputs.len() == symbol.outputs.len());
    println!("  Children match: {}", restored.symbol.children.len() == symbol.children.len());
    println!("  Connections match: {}", restored.symbol.connections.len() == symbol.connections.len());
    println!("  Animations present: {}", !restored.symbol.animations.is_empty());

    if let Some(anim) = restored.symbol.animations.first() {
        println!("  First animation keyframes: {}", anim.curve.keyframes.len());
        println!("  Curve interpolation modes:");
        for (i, kf) in anim.curve.keyframes.iter().enumerate() {
            println!("    [{i}] t={:.1}s, v={:.1}, mode={:?}", kf.time, kf.value, kf.interpolation);
        }
    }

    println!("\n✓ Modern format serialization successful!");
    println!("  Schema version: {}", file.version);
    println!("  Tags: {:?}", symbol.tags);
}
