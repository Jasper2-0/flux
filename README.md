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
- **Rich type system** - Float, Vec3, Color, Gradient, Matrix4, and more
- **110+ operators** - Math, time, vector, color, flow control, and utilities
- **Animation system** - Keyframe curves with multiple interpolation modes
- **Serialization** - Save and load graphs as JSON

## Crate Structure

```
flux/
├── flux-core          # Foundation: Value, Operator, Context, Port
├── flux-operators     # 110+ operator implementations
├── flux-graph         # Graph execution, Symbol system, Animation
└── flux-macros        # Derive macros for operators
```

## Quick Start

```rust
use flux_core::EvalContext;
use flux_graph::Graph;
use flux_operators::{AddOp, ConstantOp, SineWaveOp};

fn main() {
    // Create a graph
    let mut graph = Graph::new();

    // Add operators
    let a = graph.add(ConstantOp::new(5.0));
    let b = graph.add(ConstantOp::new(3.0));
    let add = graph.add(AddOp::new());
    let sine = graph.add(SineWaveOp::new());

    // Connect: (5 + 3) * sin(time)
    graph.connect(a, 0, add, 0).unwrap();
    graph.connect(b, 0, add, 1).unwrap();

    // Create evaluation context
    let mut ctx = EvalContext::new();

    // Evaluate
    let result = graph.evaluate(add, 0, &ctx).unwrap();
    println!("Result: {}", result);  // "Result: 8"

    // Advance time and re-evaluate
    ctx.advance(1.0);
    let animated = graph.evaluate(sine, 0, &ctx).unwrap();
    println!("Sine at t=1: {}", animated);
}
```

## Documentation

Comprehensive documentation is available in the [`docs/`](docs/) directory:

### Concepts
- [Overview](docs/01-concepts/overview.md) - What is Flux and mental model
- [Dataflow Model](docs/01-concepts/dataflow-model.md) - Operators, Values, Ports
- [Lazy Evaluation](docs/01-concepts/lazy-evaluation.md) - Dirty flags and optimization
- [Type System](docs/01-concepts/type-system.md) - Value types and coercion

### Architecture
- [Crate Structure](docs/02-architecture/crate-structure.md) - How the crates fit together
- [Evaluation Flow](docs/02-architecture/evaluation-flow.md) - How data moves through graphs
- [Symbol/Instance](docs/02-architecture/symbol-instance.md) - Definition vs runtime state
- [Composition](docs/02-architecture/composition.md) - Hierarchical graph nesting

### Creative Coding
- [Getting Started](docs/03-creative-coding/getting-started.md) - First steps
- [Real-Time Graphics](docs/03-creative-coding/real-time-graphics.md) - Camera, materials, lights
- [Audio Sync](docs/03-creative-coding/audio-sync.md) - BPM, beat quantization
- [Animation](docs/03-creative-coding/animation.md) - Keyframes and curves
- [Resource Management](docs/03-creative-coding/resource-management.md) - Asset organization

### Integration
- [Application Loop](docs/04-integration/application-loop.md) - Integrating into apps
- [Custom Operators](docs/04-integration/custom-operators.md) - Building your own
- [Serialization](docs/04-integration/serialization.md) - Saving and loading
- [Extending Flux](docs/04-integration/extending-flux.md) - Adding new features

### Reference
- [Operator Catalog](docs/05-reference/operator-catalog.md) - All 110+ operators
- [Value Types](docs/05-reference/value-types.md) - Complete Value enum reference
- [EvalContext](docs/05-reference/eval-context.md) - Full context API
- [Examples Guide](docs/05-reference/examples-guide.md) - Learning path through examples

## Examples

18 examples demonstrate Flux concepts:

```bash
# Fundamentals
cargo run --example 01_basic_arithmetic
cargo run --example 02_sine_wave
cargo run --example 05_vec3_composition

# Animation
cargo run --example 10_animation_system
cargo run --example 15_playback_settings

# Advanced
cargo run --example 08_composite_operators
cargo run --example 11_symbol_instance
cargo run --example 18_phase3_operators
```

See [Examples Guide](docs/05-reference/examples-guide.md) for detailed descriptions.

## Operator Categories

| Category | Count | Key Operators |
|----------|-------|---------------|
| Math | 35 | Add, Multiply, Lerp, Sin, Clamp, PerlinNoise |
| Time | 10 | Time, DeltaTime, SineWave, Spring, Accumulator |
| Vector | 15 | Vec3Compose, Normalize, Dot, Cross, Distance |
| Color | 8 | RgbaColor, HsvToRgb, BlendColors, SampleGradient |
| Flow | 12 | Switch, Gate, Delay, Counter, GetFloatVar |
| Logic | 12 | And, Or, Not, IntAdd, IntClamp |
| String | 8 | StringConcat, Format, Split, Contains |
| List | 8 | FloatList, ListGet, ListSum, ListMap |
| Utility | 6 | Print, Passthrough, TypeOf |

## Design Philosophy

1. **Creative-first** - Optimized for real-time visual applications
2. **Lazy evaluation** - Only recompute what changed
3. **Type safety** - Strong typing with automatic coercion
4. **Composable** - Build complex behaviors from simple operators
5. **Inspectable** - Easy to debug and visualize

## License

MIT License
