# Flux Documentation

Welcome to the Flux documentation. This guide helps you understand and use Flux, a reactive dataflow graph system for creative coding in Rust.

## Quick Start

| Resource | Description |
|----------|-------------|
| [Main README](../README.md) | Overview, installation, and quick start code |
| [Examples](../examples/README.md) | 29 hands-on examples with learning paths |
| `cargo doc --open` | API reference (generated from source) |

## Architecture & Concepts

Understanding how Flux works:

| Document | Description |
|----------|-------------|
| [Architecture](ARCHITECTURE.md) | System overview, crate structure, execution models |
| [Graph Evaluation](GRAPH_EVALUATION.md) | Evaluation lifecycle, caching, lazy evaluation, compiled execution |
| [Type System](TYPE_SYSTEM.md) | Value types, categories, coercion rules, polymorphism |

## Learning Path

**New to Flux?** Follow this sequence:

1. **[Main README](../README.md)** - Get Flux running with a simple example
2. **[Architecture](ARCHITECTURE.md)** - Understand the big picture
3. **[Examples 01-06](../examples/README.md#tier-1-foundation-01-06)** - Core concepts hands-on
4. **[Graph Evaluation](GRAPH_EVALUATION.md)** - Deep dive into execution
5. **[Type System](TYPE_SYSTEM.md)** - Master the type system

## Operator Reference

Flux includes 150+ operators across categories:

| Category | Count | Examples |
|----------|-------|----------|
| Math | 30 | Add, Multiply, Lerp, Sin, PerlinNoise |
| Time | 9 | Time, SineWave, Spring, Accumulator |
| Vector | 17 | Vec3Compose, Normalize, Dot, Cross |
| Color | 8 | RgbaColor, HsvToRgb, BlendColors |
| Flow | 14 | Switch, Select, Gate, Loop, ForEach |
| Logic | 13 | And, Or, Not, IntAdd, IntCompare |
| String | 8 | StringConcat, Format, Split |
| List | 40 | FloatList, ListGet, ListMap, ArrayIterator |
| Utility | 6 | Print, Passthrough, Comment |

Run `cargo run --example 29_operator_showcase` for a complete catalog.

## Contributing

Flux is organized as a Cargo workspace:

```
flux/
├── flux-core       # Foundation: Value, Operator, Port, Context
├── flux-operators  # 150+ operator implementations
├── flux-graph      # Graph execution, serialization, animation
└── flux-macros     # Derive macros for operators
```

See [Architecture](ARCHITECTURE.md) for how these crates interact.
