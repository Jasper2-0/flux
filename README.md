# Flux

A reactive dataflow graph system for creative coding in Rust.

```
┌─────────────────────────────────────────────────────────────────┐
│                         FLUX                                     │
│                                                                  │
│    ┌──────────┐     ┌──────────┐     ┌──────────┐              │
│    │ Constant │────▶│          │     │          │              │
│    │   5.0    │     │   Add    │────▶│ Multiply │────▶ Result  │
│    └──────────┘     │          │     │          │              │
│    ┌──────────┐     └──────────┘     └──────────┘              │
│    │ Constant │────────▲                   ▲                    │
│    │   3.0    │                            │                    │
│    └──────────┘     ┌──────────┐           │                    │
│                     │ SineWave │───────────┘                    │
│                     │  ~time~  │                                │
│                     └──────────┘                                │
│                                                                  │
│    Result: (5 + 3) * sin(time)                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Overview

Flux is a dataflow graph library designed for real-time creative applications. It provides:

- **Reactive evaluation** - Values flow through connected operators automatically
- **Lazy computation** - Dirty flags ensure only changed nodes recompute
- **Rich type system** - Float, Int, Bool, Vec2, Vec3, Vec4, Color, Gradient, Matrix4, and more
- **120+ operators** - Math, time, vector, color, flow control, and utilities
- **Animation system** - Keyframe curves with multiple interpolation modes
- **Serialization** - Save and load graphs as JSON

## Crate Structure

```
flux/
├── flux-core          # Foundation: Value, Operator, Context, Port
├── flux-operators     # 120+ operator implementations
├── flux-graph         # Graph execution, Symbol system, Animation
└── flux-macros        # Derive macros for operators
```

## Quick Start

```rust
use flux_core::EvalContext;
use flux_graph::Graph;
use flux_operators::{AddOp, ConstantOp, MultiplyOp};

fn main() {
    // Create a graph
    let mut graph = Graph::new();

    // Add operators
    let a = graph.add(ConstantOp::new(5.0));
    let b = graph.add(ConstantOp::new(3.0));
    let c = graph.add(ConstantOp::new(2.0));
    let add = graph.add(AddOp::new());
    let multiply = graph.add(MultiplyOp::new());

    // Connect: (5 + 3) * 2
    graph.connect(a, 0, add, 0).unwrap();
    graph.connect(b, 0, add, 1).unwrap();
    graph.connect(add, 0, multiply, 0).unwrap();
    graph.connect(c, 0, multiply, 1).unwrap();

    // Create evaluation context
    let ctx = EvalContext::new();

    // Evaluate
    let result = graph.evaluate(multiply, 0, &ctx).unwrap();
    println!("Result: {}", result);  // "Result: 16"
}
```

## Documentation

Documentation is provided through:

- **Rustdoc** - Run `cargo doc --open` for API documentation
- **Examples** - 28 annotated examples covering all major features (see below)
- **Source comments** - Extensive doc comments throughout the codebase

### Key Concepts

- **Operators** - Computational nodes with inputs and outputs
- **Graph** - Container that connects operators and manages evaluation
- **EvalContext** - Provides time, frame count, and variables during evaluation
- **Values** - Typed data flowing between operators (Float, Vec3, Color, etc.)
- **Lazy Evaluation** - Dirty flags ensure only changed nodes recompute

## Examples

28 examples demonstrate Flux concepts:

```bash
# Fundamentals
cargo run --example 01_basic_arithmetic    # Core graph evaluation
cargo run --example 02_sine_wave           # Time-based oscillators
cargo run --example 03_multi_input_sum     # Variable input operators
cargo run --example 04_compare_operator    # Comparison and logic
cargo run --example 05_vec3_composition    # Vector operations
cargo run --example 06_type_validation     # Type system and coercion

# Serialization & Registry
cargo run --example 07_json_serialization  # Save/load graphs
cargo run --example 09_operator_registry   # Dynamic operator creation

# Graph Features
cargo run --example 08_composite_operators # Nested graphs
cargo run --example 10_animation_system    # Keyframe animation
cargo run --example 11_symbol_instance     # Symbol/instance pattern
cargo run --example 12_dirty_flag_system   # Lazy evaluation
cargo run --example 13_bypass_system       # Bypassing nodes

# Advanced Features
cargo run --example 15_playback_settings   # Timeline playback
cargo run --example 16_enhanced_serialization
cargo run --example 18_phase3_operators    # Advanced operator patterns
cargo run --example 19_auto_conversion     # Automatic type conversion
cargo run --example 20_trigger_system      # Event triggers
cargo run --example 21_compiled_execution  # Compiled graph execution
cargo run --example 22_undo_redo           # Command pattern undo/redo
cargo run --example 23_flow_control        # Loops and conditionals

# Showcase Examples
cargo run --example 24_procedural_terrain  # FBM noise terrain generation
cargo run --example 25_spring_physics      # Chained spring physics simulation
cargo run --example 26_color_wheel         # HSV color pipeline & harmony
cargo run --example 27_diamond_dependency  # Fan-out/fan-in caching patterns
cargo run --example 28_state_machine       # Trigger-based finite state machine
cargo run --example 29_performance_benchmark --release  # Graph scaling benchmarks
cargo run --example 30_list_processing        # List operators in graph context
```

## Operator Categories

| Category | Count | Key Operators |
|----------|-------|---------------|
| Math | 30 | Add, Multiply, Lerp, Sin, Clamp, PerlinNoise, Pow, Sqrt |
| Time | 9 | Time, DeltaTime, SineWave, SawWave, Spring, Accumulator |
| Vector | 17 | Vec2/Vec3/Vec4 Compose, Normalize, Dot, Cross, Distance |
| Color | 8 | RgbaColor, HsvToRgb, BlendColors, SampleGradient |
| Flow | 14 | Switch, Select, Gate, Loop, ForEach, Delay, Counter, Trigger |
| Logic | 13 | And, Or, Not, Compare, IntAdd, IntClamp, IntToFloat |
| String | 8 | StringConcat, Format, Split, Contains, FloatToString |
| List | 11 | FloatList, ListGet, ListSum, ListAverage, ListMap, ListFilter, ListSlice, ListConcat |
| Utility | 6 | Print, Passthrough, Comment, TypeOf, IsConnected |

## Design Philosophy

1. **Creative-first** - Optimized for real-time visual applications
2. **Lazy evaluation** - Only recompute what changed
3. **Type safety** - Strong typing with automatic coercion
4. **Composable** - Build complex behaviors from simple operators
5. **Inspectable** - Easy to debug and visualize

## License

MIT License
